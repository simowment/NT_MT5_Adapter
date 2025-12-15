# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------

"""
The execution client for the MetaTrader 5 integration.
"""

from __future__ import annotations

import asyncio
import json
from decimal import Decimal
from typing import TYPE_CHECKING

from nautilus_mt5_adapter.common import MT5_VENUE
from nautilus_mt5_adapter.constants import (
    Mt5OrderState,
    Mt5OrderType,
    Mt5RetCode,
    Mt5TradeAction,
)
from nautilus_trader.core.correctness import PyCondition
from nautilus_trader.core.uuid import UUID4
from nautilus_trader.execution.messages import BatchCancelOrders
from nautilus_trader.execution.messages import CancelAllOrders
from nautilus_trader.execution.messages import CancelOrder
from nautilus_trader.execution.messages import ModifyOrder
from nautilus_trader.execution.messages import SubmitOrder
from nautilus_trader.execution.messages import SubmitOrderList
from nautilus_trader.execution.reports import FillReport
from nautilus_trader.execution.reports import OrderStatusReport
from nautilus_trader.execution.reports import PositionStatusReport
from nautilus_trader.live.execution_client import LiveExecutionClient
from nautilus_trader.model.enums import AccountType
from nautilus_trader.model.enums import LiquiditySide
from nautilus_trader.model.enums import OmsType
from nautilus_trader.model.enums import OrderSide
from nautilus_trader.model.enums import OrderStatus
from nautilus_trader.model.enums import OrderType
from nautilus_trader.model.enums import PositionSide
from nautilus_trader.model.enums import TimeInForce
from nautilus_trader.model.identifiers import AccountId
from nautilus_trader.model.identifiers import ClientId
from nautilus_trader.model.identifiers import ClientOrderId
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import Symbol
from nautilus_trader.model.identifiers import TradeId
from nautilus_trader.model.identifiers import VenueOrderId
from nautilus_trader.model.objects import Currency
from nautilus_trader.model.objects import Money
from nautilus_trader.model.objects import Price
from nautilus_trader.model.objects import Quantity

if TYPE_CHECKING:
    from nautilus_mt5_adapter.config import Mt5ExecClientConfig
    from nautilus_mt5_adapter.providers import Mt5InstrumentProvider
    from nautilus_trader.cache.cache import Cache
    from nautilus_trader.common.component import LiveClock
    from nautilus_trader.common.component import MessageBus


class Mt5ExecutionClient(LiveExecutionClient):
    """
    MetaTrader 5 execution client.

    This client implements the NautilusTrader LiveExecutionClient interface
    and delegates to the Rust MT5 HTTP client exposed via PyO3.
    """

    def __init__(
        self,
        loop: asyncio.AbstractEventLoop,
        client,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
        instrument_provider: Mt5InstrumentProvider,
        config: Mt5ExecClientConfig,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId("MT5"),
            venue=MT5_VENUE,
            oms_type=OmsType.NETTING,  # MT5 uses netting by default
            account_type=AccountType.MARGIN,
            base_currency=None,  # Will be set from account info
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=instrument_provider,
            config=config,
        )

        PyCondition.not_none(client, "client")

        self._client = client
        self._instrument_provider = instrument_provider
        self._config = config
        self._account_id: AccountId | None = None

    # -- CONNECTION HANDLERS ----------------------------------------------------------------------

    async def _connect(self) -> None:
        """Connect to the MT5 middleware."""
        self._log.info("Connecting to MT5 middleware...")
        try:
            # Initialize HTTP client
            await self._client.initialize()

            # Get account info to set account ID
            response_json = await self._client.account_info()
            response = json.loads(response_json) if response_json else {}
            account_info = response.get("result", {}) if response else {}

            if account_info and "login" in account_info:
                self._account_id = AccountId(f"MT5-{account_info['login']}")
                self._log.info(f"Connected to account: {self._account_id}")

            # Load instruments
            await self._instrument_provider.initialize()

            self._log.info("Connected to MT5 middleware successfully.")
        except Exception as e:
            self._log.error(f"Failed to connect to MT5: {e}")
            raise

    async def _disconnect(self) -> None:
        """Disconnect from the MT5 middleware."""
        self._log.info("Disconnecting from MT5 middleware...")
        await self._client.shutdown()
        self._log.info("Disconnected from MT5 middleware.")

    # -- ORDER HANDLERS --------------------------------------------------------------------------

    async def _submit_order(self, command: SubmitOrder) -> None:
        """Submit an order to MT5."""
        order = command.order
        self._log.info(f"Submitting order: {order.client_order_id}")
        try:
            # Convert Nautilus order to MT5 format
            symbol = order.instrument_id.symbol.value
            volume = float(order.quantity)

            # Determine MT5 order type (0=Buy, 1=Sell, 2=BuyLimit, etc.)
            mt5_order_type = self._convert_order_type(
                order.order_side, order.order_type
            )

            # Build order request matching MT5 REST API format
            request = {
                "action": Mt5TradeAction.DEAL,
                "symbol": symbol,
                "volume": volume,
                "type": mt5_order_type,
                "comment": str(order.client_order_id),
            }

            # Add price for limit/stop orders
            if order.order_type in (OrderType.LIMIT, OrderType.STOP_LIMIT):
                if order.price:
                    request["price"] = float(order.price)
            if order.order_type in (OrderType.STOP_MARKET, OrderType.STOP_LIMIT):
                if order.trigger_price:
                    request["stoplimit"] = float(order.trigger_price)

            # Add SL/TP if configured
            # TODO: Extract from order tags or linked orders

            # Submit via HTTP
            response_json = await self._client.order_send(json.dumps(request))
            response = json.loads(response_json) if response_json else {}
            result = response.get("result", {}) if response else {}

            if result and result.get("retcode") == Mt5RetCode.DONE:
                self._log.info(f"Order submitted successfully: {result}")
                # Generate accepted event
                self._generate_order_accepted(order, result)
            else:
                error_msg = (
                    result.get("comment", "Unknown error") if result else "No response"
                )
                self._log.error(f"Order rejected by MT5: {error_msg}")
                self._generate_order_rejected(order, error_msg)

        except Exception as e:
            self._log.error(f"Failed to submit order: {e}")
            self._generate_order_rejected(order, str(e))

    async def _submit_order_list(self, command: SubmitOrderList) -> None:
        """Submit a list of orders (bracket orders not directly supported by MT5)."""
        self._log.info(
            f"Submitting order list with {len(command.order_list.orders)} orders"
        )
        # MT5 doesn't support atomic bracket orders, submit individually
        for order in command.order_list.orders:
            await self._submit_order(
                SubmitOrder(
                    trader_id=command.trader_id,
                    strategy_id=command.strategy_id,
                    order=order,
                    position_id=command.position_id,
                    command_id=UUID4(),
                    ts_init=self._clock.timestamp_ns(),
                )
            )

    async def _modify_order(self, command: ModifyOrder) -> None:
        """Modify an existing order."""
        self._log.info(f"Modifying order: {command.client_order_id}")
        try:
            # Build modification request
            request = {
                "action": Mt5TradeAction.MODIFY,
                "order": int(command.venue_order_id.value),
            }

            if command.price:
                request["price"] = float(command.price)
            if command.trigger_price:
                request["stoplimit"] = float(command.trigger_price)

            response_json = await self._client.order_send(json.dumps(request))
            response = json.loads(response_json) if response_json else {}
            result = response.get("result", {}) if response else {}

            if result and result.get("retcode") == Mt5RetCode.DONE:
                self._log.info("Order modified successfully")
                self._generate_order_updated(command, result)
            else:
                error_msg = (
                    result.get("comment", "Unknown error") if result else "No response"
                )
                self._log.error(f"Order modification failed: {error_msg}")

        except Exception as e:
            self._log.error(f"Failed to modify order: {e}")
            raise

    async def _cancel_order(self, command: CancelOrder) -> None:
        """Cancel an existing order."""
        self._log.info(f"Canceling order: {command.client_order_id}")
        try:
            # Build cancellation request
            request = {
                "action": Mt5TradeAction.REMOVE,
                "order": int(command.venue_order_id.value),
            }

            response_json = await self._client.order_send(json.dumps(request))
            response = json.loads(response_json) if response_json else {}
            result = response.get("result", {}) if response else {}

            if result and result.get("retcode") == Mt5RetCode.DONE:
                self._log.info("Order canceled successfully")
                self._generate_order_canceled(command)
            else:
                error_msg = (
                    result.get("comment", "Unknown error") if result else "No response"
                )
                self._log.error(f"Order cancellation failed: {error_msg}")

        except Exception as e:
            self._log.error(f"Failed to cancel order: {e}")
            raise

    async def _cancel_all_orders(self, command: CancelAllOrders) -> None:
        """Cancel all orders for a specific instrument."""
        self._log.info(f"Canceling all orders for {command.instrument_id}")
        try:
            # Get all pending orders
            response_json = await self._client.orders_get()
            response = json.loads(response_json) if response_json else {}
            orders = response.get("result", []) if response else []
            if not orders:
                return

            symbol = (
                command.instrument_id.symbol.value if command.instrument_id else None
            )

            for order in orders:
                if symbol and order.get("symbol") != symbol:
                    continue
                # Cancel each matching order
                request = {
                    "action": Mt5TradeAction.REMOVE,
                    "order": order.get("ticket"),
                }
                await self._client.order_send(json.dumps(request))

            self._log.info(f"All orders canceled for {symbol or 'all symbols'}")

        except Exception as e:
            self._log.error(f"Failed to cancel all orders: {e}")
            raise

    async def _batch_cancel_orders(self, command: BatchCancelOrders) -> None:
        """Cancel a batch of orders."""
        self._log.info(f"Batch canceling {len(command.cancels)} orders")
        for cancel in command.cancels:
            await self._cancel_order(cancel)

    # -- REPORT GENERATION ------------------------------------------------------------------------

    async def generate_order_status_reports(
        self,
        instrument_id: InstrumentId | None = None,
        start: int | None = None,
        end: int | None = None,
        open_only: bool = False,
    ) -> list[OrderStatusReport]:
        """Generate order status reports from MT5."""
        self._log.info("Generating order status reports")
        reports = []
        try:
            # Get pending orders
            response_json = await self._client.orders_get()
            response = json.loads(response_json) if response_json else {}
            orders = response.get("result", []) if response else []
            if orders:
                for order in orders:
                    report = self._parse_order_status_report(order)
                    reports.append(report)

            # Get historical orders if not open_only
            if not open_only and start and end:
                # history_orders_get expects a dict body with start/end
                request = {"from": start, "to": end}
                response_json = await self._client.history_orders_get(
                    json.dumps(request)
                )
                response = json.loads(response_json) if response_json else {}
                history = response.get("result", []) if response else []
                if history:
                    for order in history:
                        report = self._parse_order_status_report(order)
                        reports.append(report)

        except Exception as e:
            self._log.error(f"Failed to generate order status reports: {e}")
            raise

        return reports

    async def generate_fill_reports(
        self,
        instrument_id: InstrumentId | None = None,
        venue_order_id: VenueOrderId | None = None,
        start: int | None = None,
        end: int | None = None,
    ) -> list[FillReport]:
        """Generate fill reports from MT5 deal history."""
        self._log.info("Generating fill reports")
        reports = []
        try:
            if not start or not end:
                # Default to last 24 hours
                import time

                end = int(time.time())
                start = end - 86400

            # history_deals_get expects a dict body with start/end
            request = {"from": start, "to": end}
            response_json = await self._client.history_deals_get(json.dumps(request))
            response = json.loads(response_json) if response_json else {}
            deals = response.get("result", []) if response else []
            if deals:
                for deal in deals:
                    report = self._parse_fill_report(deal)
                    reports.append(report)

        except Exception as e:
            self._log.error(f"Failed to generate fill reports: {e}")
            raise

        return reports

    async def generate_position_status_reports(
        self,
        instrument_id: InstrumentId | None = None,
        start: int | None = None,
        end: int | None = None,
    ) -> list[PositionStatusReport]:
        """Generate position status reports from MT5."""
        self._log.info("Generating position status reports")
        reports = []
        try:
            response_json = await self._client.positions_get()
            response = json.loads(response_json) if response_json else {}
            positions = response.get("result", []) if response else []
            if positions:
                for position in positions:
                    report = self._parse_position_status_report(position)
                    reports.append(report)

        except Exception as e:
            self._log.error(f"Failed to generate position status reports: {e}")
            raise

        return reports

    # -- CONVERSION HELPERS -----------------------------------------------------------------------

    def _convert_order_type(self, side: OrderSide, order_type: OrderType) -> int:
        """Convert Nautilus order side/type to MT5 order type."""
        if order_type == OrderType.MARKET:
            return Mt5OrderType.BUY if side == OrderSide.BUY else Mt5OrderType.SELL
        elif order_type == OrderType.LIMIT:
            return (
                Mt5OrderType.BUY_LIMIT
                if side == OrderSide.BUY
                else Mt5OrderType.SELL_LIMIT
            )
        elif order_type == OrderType.STOP_MARKET:
            return (
                Mt5OrderType.BUY_STOP
                if side == OrderSide.BUY
                else Mt5OrderType.SELL_STOP
            )
        else:
            return Mt5OrderType.BUY if side == OrderSide.BUY else Mt5OrderType.SELL

    def _parse_order_status_report(self, mt5_order: dict) -> OrderStatusReport:
        """Parse MT5 order dict to OrderStatusReport."""
        # Map MT5 order status to Nautilus OrderStatus
        status_map = {
            Mt5OrderState.STARTED: OrderStatus.SUBMITTED,
            Mt5OrderState.PLACED: OrderStatus.ACCEPTED,
            Mt5OrderState.CANCELED: OrderStatus.CANCELED,
            Mt5OrderState.PARTIAL: OrderStatus.PARTIALLY_FILLED,
            Mt5OrderState.FILLED: OrderStatus.FILLED,
            Mt5OrderState.REJECTED: OrderStatus.REJECTED,
        }

        mt5_status = mt5_order.get("status")
        if mt5_status is None:
            raise ValueError(f"Order missing 'status' field: {mt5_order}")

        order_status = status_map.get(mt5_status)
        if order_status is None:
            raise ValueError(f"Unknown MT5 order status: {mt5_status}")

        order_side = (
            OrderSide.BUY if mt5_order.get("type", 0) % 2 == 0 else OrderSide.SELL
        )

        symbol = mt5_order.get("symbol")
        if not symbol:
            raise ValueError(f"Order missing 'symbol' field: {mt5_order}")

        ticket = mt5_order.get("ticket")
        if ticket is None:
            raise ValueError(f"Order missing 'ticket' field: {mt5_order}")

        return OrderStatusReport(
            account_id=self._account_id or AccountId("MT5-UNKNOWN"),
            instrument_id=InstrumentId(Symbol(symbol), MT5_VENUE),
            client_order_id=ClientOrderId(str(ticket)),
            venue_order_id=VenueOrderId(str(ticket)),
            order_side=order_side,
            order_type=OrderType.LIMIT,  # Simplification
            time_in_force=TimeInForce.GTC,
            order_status=order_status,
            quantity=Quantity.from_str(str(mt5_order.get("volume", 0))),
            filled_qty=Quantity.from_str(str(mt5_order.get("volume_done", 0))),
            avg_px=Price.from_str(str(mt5_order.get("price_open", 0)))
            if mt5_order.get("price_open")
            else None,
            report_id=UUID4(),
            ts_accepted=mt5_order.get("time_setup", 0) * 10**9,
            ts_last=mt5_order.get("time_done", mt5_order.get("time_setup", 0)) * 10**9,
            ts_init=self._clock.timestamp_ns(),
        )

    def _parse_fill_report(self, mt5_deal: dict) -> FillReport:
        """Parse MT5 deal dict to FillReport."""
        symbol = mt5_deal.get("symbol")
        if not symbol:
            raise ValueError(f"Deal missing 'symbol' field: {mt5_deal}")

        order_id = mt5_deal.get("order")
        if order_id is None:
            raise ValueError(f"Deal missing 'order' field: {mt5_deal}")

        ticket = mt5_deal.get("ticket")
        if ticket is None:
            raise ValueError(f"Deal missing 'ticket' field: {mt5_deal}")

        order_side = OrderSide.BUY if mt5_deal.get("type", 0) == 0 else OrderSide.SELL

        return FillReport(
            account_id=self._account_id or AccountId("MT5-UNKNOWN"),
            instrument_id=InstrumentId(Symbol(symbol), MT5_VENUE),
            client_order_id=ClientOrderId(str(order_id)),
            venue_order_id=VenueOrderId(str(order_id)),
            trade_id=TradeId(str(ticket)),
            order_side=order_side,
            last_qty=Quantity.from_str(str(mt5_deal.get("volume", 0))),
            last_px=Price.from_str(str(mt5_deal.get("price", 0))),
            commission=Money(
                Decimal(str(mt5_deal.get("commission", 0))),
                self._base_currency or Currency.from_str("USD"),
            ),
            liquidity_side=LiquiditySide.TAKER,
            report_id=UUID4(),
            ts_event=mt5_deal.get("time", 0) * 10**9,
            ts_init=self._clock.timestamp_ns(),
        )

    def _parse_position_status_report(self, mt5_position: dict) -> PositionStatusReport:
        """Parse MT5 position dict to PositionStatusReport."""
        symbol = mt5_position.get("symbol")
        if not symbol:
            raise ValueError(f"Position missing 'symbol' field: {mt5_position}")

        # MT5 position type: 0=Buy, 1=Sell
        position_side = (
            PositionSide.LONG
            if mt5_position.get("type", 0) == 0
            else PositionSide.SHORT
        )

        return PositionStatusReport(
            account_id=self._account_id or AccountId("MT5-UNKNOWN"),
            instrument_id=InstrumentId(Symbol(symbol), MT5_VENUE),
            position_side=position_side,
            quantity=Quantity.from_str(str(abs(mt5_position.get("volume", 0)))),
            report_id=UUID4(),
            ts_last=mt5_position.get("time", 0) * 10**9,
            ts_init=self._clock.timestamp_ns(),
        )

    # -- EVENT GENERATION -------------------------------------------------------------------------

    def _generate_order_accepted(self, order, result: dict) -> None:
        """Generate order accepted event."""
        # TODO: Emit OrderAccepted event via msgbus
        pass

    def _generate_order_rejected(self, order, reason: str) -> None:
        """Generate order rejected event."""
        # TODO: Emit OrderRejected event via msgbus
        pass

    def _generate_order_updated(self, command: ModifyOrder, result: dict) -> None:
        """Generate order updated event."""
        # TODO: Emit OrderUpdated event via msgbus
        pass

    def _generate_order_canceled(self, command: CancelOrder) -> None:
        """Generate order canceled event."""
        # TODO: Emit OrderCanceled event via msgbus
        pass
