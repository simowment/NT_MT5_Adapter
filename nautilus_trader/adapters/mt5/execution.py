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

from nautilus_trader.core.correctness import PyCondition
from nautilus_trader.execution.messages import (
    SubmitOrder,
    SubmitOrderList,
    CancelOrder,
    CancelAllOrders,
    BatchCancelOrders,
    ModifyOrder,
    GenerateOrderStatusReport,
    GenerateOrderStatusReports,
    GenerateFillReports,
    GeneratePositionStatusReports,
)
from nautilus_trader.execution.reports import FillReport, OrderStatusReport, PositionStatusReport
from nautilus_trader.live.execution_client import LiveExecutionClient
from nautilus_trader.model.identifiers import ClientId, Venue


class Mt5ExecutionError(Exception):
    """Base exception for MT5 execution client errors."""
    pass


class Mt5OrderSubmissionError(Mt5ExecutionError):
    """Raised when order submission fails."""
    pass


class Mt5OrderModificationError(Mt5ExecutionError):
    """Raised when order modification fails."""
    pass


class Mt5OrderCancellationError(Mt5ExecutionError):
    """Raised when order cancellation fails."""
    pass


class Mt5AccountError(Mt5ExecutionError):
    """Raised when account operations fail."""
    pass


class Mt5ExecutionClient(LiveExecutionClient):
    """
    MetaTrader 5 execution client.

    This class is wired to fit the NautilusTrader LiveExecutionClient interface
    and should delegate to the Rust MT5 HTTP/WS clients exposed via PyO3.
    """

    def __init__(self, loop, http_client, ws_client, msgbus, cache, clock):
        super().__init__(
            client_id=ClientId("MT5"),
            venue=Venue("MT5"),
            loop=loop,
            clock=clock,
            msgbus=msgbus,
            cache=cache,
        )

        PyCondition.not_none(http_client, "http_client")
        PyCondition.not_none(ws_client, "ws_client")

        self._http_client = http_client
        self._ws_client = ws_client
        self._connected = False

    def connect(self):
        self._log.info("Connecting to MT5...")
        try:
            # Authentification HTTP
            import asyncio
            loop = asyncio.get_event_loop()
            loop.run_until_complete(self._http_client.login())
            
            # Connexion WebSocket
            loop.run_until_complete(self._ws_client.connect())
            loop.run_until_complete(self._ws_client.authenticate())
            
            self._connected = True
            self._log.info("Connected to MT5 successfully.")
        except Exception as e:
            self._log.error(f"Failed to connect to MT5: {e}")
            raise

    def disconnect(self):
        self._log.info("Disconnecting from MT5...")
        if self._connected:
            try:
                import asyncio
                loop = asyncio.get_event_loop()
                if self._ws_client:
                    loop.run_until_complete(self._ws_client.disconnect())
                self._connected = False
                self._log.info("Disconnected from MT5.")
            except Exception as e:
                self._log.error(f"Error during disconnection: {e}")
        else:
            self._log.info("Already disconnected.")

    async def _submit_order(self, command: SubmitOrder):
        self._log.info(f"Submitting order: {command}")
        try:
            if not self._connected:
                raise RuntimeError("Not connected to MT5")
                
            # Convert Nautilus order to MT5 format
            symbol = str(command.instrument_id)
            volume = float(command.quantity)
            price = float(command.limit_price) if command.limit_price else None
            order_type = command.order_type
            
            # Create MT5 order request
            mt5_order = {
                'symbol': symbol,
                'volume': volume,
                'order_type': order_type.value,
                'comment': command.client_order_id.value if command.client_order_id else None
            }
            
            if price:
                mt5_order['price'] = price
                
            # Submit via HTTP
            result = await self._http_client.submit_order(mt5_order)
            
            # Update order status
            self._log.info(f"Order submitted successfully: {result}")
            
            # Publish order status report
            await self.generate_order_status_report(command)
            
        except Exception as e:
            self._log.error(f"Failed to submit order: {e}")
            # Generate rejection report
            await self._generate_rejection_report(command, str(e))
            raise

    async def _submit_order_list(self, command: SubmitOrderList):
        self._log.info(f"Submitting order list: {command}")
        try:
            if not self._connected:
                raise RuntimeError("Not connected to MT5")
            
            # Submit each order in the list
            for order in command.orders:
                # Create a SubmitOrder command for each order in the list
                submit_order = SubmitOrder(
                    instrument_id=order.instrument_id,
                    client_order_id=order.client_order_id,
                    order_side=order.order_side,
                    order_type=order.order_type,
                    quantity=order.quantity,
                    time_in_force=order.time_in_force,
                    limit_price=order.limit_price,
                    stop_price=order.stop_price,
                    expire_time=order.expire_time,
                    flags=order.flags,
                    init_id=command.id,
                )
                
                # Submit the individual order
                await self._submit_order(submit_order)
            
            self._log.info(f"Order list submitted successfully with {len(command.orders)} orders")
            
        except Exception as e:
            self._log.error(f"Failed to submit order list: {e}")
            raise

    async def _modify_order(self, command: ModifyOrder):
        self._log.info(f"Modifying order: {command}")
        try:
            if not self._connected:
                raise RuntimeError("Not connected to MT5")
                
            # Get order ID from command
            order_id = command.order_id.value if command.order_id else None
            if not order_id:
                raise ValueError("Order ID required for modification")
                
            # Convert modification to MT5 format
            symbol = str(command.instrument_id)
            volume = command.quantity if command.quantity else None
            price = float(command.limit_price) if command.limit_price else None
            
            mt5_order = {
                'symbol': symbol,
                'order_type': 'MODIFY',  # MT5 modify type
                'comment': command.client_order_id.value if command.client_order_id else None
            }
            
            if volume:
                mt5_order['volume'] = volume
            if price:
                mt5_order['price'] = price
                
            # Submit modification via HTTP
            result = await self._http_client.modify_order(int(order_id), mt5_order)
            
            self._log.info(f"Order modified successfully: {result}")
            
            # Publish updated order status report
            await self.generate_order_status_report(command)
            
        except Exception as e:
            self._log.error(f"Failed to modify order: {e}")
            raise

    async def _cancel_order(self, command: CancelOrder):
        self._log.info(f"Canceling order: {command}")
        try:
            if not self._connected:
                raise RuntimeError("Not connected to MT5")
                
            # Get order ID from command
            order_id = command.order_id.value if command.order_id else None
            if not order_id:
                raise ValueError("Order ID required for cancellation")
                
            # Cancel via HTTP
            await self._http_client.cancel_order(int(order_id))
            
            self._log.info(f"Order canceled successfully: {order_id}")
            
            # Publish order status report with canceled status
            await self.generate_order_status_report(command)
            
        except Exception as e:
            self._log.error(f"Failed to cancel order: {e}")
            raise

    async def _cancel_all_orders(self, command: CancelAllOrders):
        self._log.info(f"Canceling all orders for {command.instrument_id}")
        try:
            if not self._connected:
                raise RuntimeError("Not connected to MT5")
            
            # Cancel all orders via HTTP
            symbol = str(command.instrument_id)
            await self._http_client.cancel_all_orders(symbol)
            
            self._log.info(f"All orders canceled for {symbol}")
            
        except Exception as e:
            self._log.error(f"Failed to cancel all orders: {e}")
            raise

    async def _batch_cancel_orders(self, command: BatchCancelOrders):
        self._log.info(f"Batch canceling {len(command.order_ids)} orders")
        try:
            if not self._connected:
                raise RuntimeError("Not connected to MT5")
            
            # Cancel each order in the batch
            for order_id in command.order_ids:
                cancel_command = CancelOrder(
                    instrument_id=command.instrument_id,
                    client_order_id=order_id,
                    id=command.id,
                )
                await self._cancel_order(cancel_command)
            
            self._log.info(f"Batch canceled {len(command.order_ids)} orders")
            
        except Exception as e:
            self._log.error(f"Failed to batch cancel orders: {e}")
            raise

    async def generate_order_status_report(
        self,
        command: GenerateOrderStatusReport,
    ) -> OrderStatusReport | None:
        self._log.info(f"Generating order status report: {command}")
        try:
            if not self._connected:
                self._log.warning("Not connected to MT5, cannot generate order status report")
                return None
                
            # Get current order status from MT5
            order_id = command.order_id.value if command.order_id else None
            if order_id:
                # Query specific order status
                orders = await self._http_client.get_orders()
                mt5_order = next((o for o in orders if o.order_id == int(order_id)), None)
                
                if mt5_order:
                    # Convert to Nautilus order status report
                    report = self._convert_mt5_order_to_status_report(mt5_order, command)
                    return report
                else:
                    self._log.warning(f"Order {order_id} not found in MT5")
            else:
                self._log.warning("No order ID provided for status report")
                return None
                
        except Exception as e:
            self._log.error(f"Failed to generate order status report: {e}")
            return None

    async def generate_order_status_reports(
        self,
        command: GenerateOrderStatusReports,
    ) -> list[OrderStatusReport]:
        self._log.info(f"Generating order status reports: {command}")
        try:
            if not self._connected:
                self._log.warning("Not connected to MT5, cannot generate order status reports")
                return []
                
            # Get all orders
            orders = await self._http_client.get_orders()
            reports = []
            
            for mt5_order in orders:
                # Create a synthetic command for each order
                report = self._convert_mt5_order_to_status_report(mt5_order, command)
                reports.append(report)
                
            return reports
                
        except Exception as e:
            self._log.error(f"Failed to generate order status reports: {e}")
            return []

    async def generate_fill_reports(
        self,
        command: GenerateFillReports,
    ) -> list[FillReport]:
        self._log.info(f"Generating fill reports: {command}")
        try:
            if not self._connected:
                self._log.warning("Not connected to MT5, cannot generate fill reports")
                return []
                
            order_id = command.order_id.value if command.order_id else None
            if not order_id:
                self._log.warning("No order ID provided for fill report")
                return []
                
            # Get trade history for this order
            trades = await self._http_client.get_trades()
            order_trades = [t for t in trades if t.ticket == int(order_id)]
            reports = []
            
            for trade in order_trades:
                # Convert to Nautilus fill report
                report = self._convert_mt5_trade_to_fill_report(trade, command)
                reports.append(report)
                
            return reports
                
        except Exception as e:
            self._log.error(f"Failed to generate fill reports: {e}")
            return []

    async def generate_position_status_reports(
        self,
        command: GeneratePositionStatusReports,
    ) -> list[PositionStatusReport]:
        self._log.info(f"Generating position status reports: {command}")
        try:
            if not self._connected:
                self._log.warning("Not connected to MT5, cannot generate position status reports")
                return []
                
            # Get current positions
            positions = await self._http_client.get_positions()
            reports = []
            
            for position in positions:
                # Convert to Nautilus position status report
                report = self._convert_mt5_position_to_status_report(position)
                reports.append(report)
                
            return reports
                
        except Exception as e:
            self._log.error(f"Failed to generate position status reports: {e}")
            return []

    def _convert_mt5_order_to_status_report(self, mt5_order, command):
        """Convert MT5 order to Nautilus OrderStatusReport"""
        # Import Nautilus types
        from nautilus_trader.execution.reports import OrderStatusReport
        from nautilus_trader.model.enums import OrderStatus
        from nautilus_trader.model.identifiers import ClientOrderId
        
        # Map MT5 status to Nautilus status
        status_map = {
            "PENDING": OrderStatus.PENDING_NEW,
            "FILLED": OrderStatus.FILLED,
            "PARTIALLY_FILLED": OrderStatus.PARTIALLY_FILLED,
            "CANCELLED": OrderStatus.CANCELED,
            "REJECTED": OrderStatus.REJECTED,
        }
        
        status = status_map.get(mt5_order.status, OrderStatus.UNKNOWN)
        
        return OrderStatus(
            client_order_id=ClientOrderId(mt5_order.order_id),
            venue_order_id=str(mt5_order.order_id),
            status=status,
            filled_quantity=mt5_order.filled_volume,
            average_price=mt5_order.price,
            submitted_time=mt5_order.time,
            filled_time=mt5_order.time if status == OrderStatus.FILLED else None,
        )

    def _convert_mt5_trade_to_fill_report(self, mt5_trade, command):
        """Convert MT5 trade to Nautilus FillReport"""
        # Import Nautilus types
        from nautilus_trader.execution.reports import FillReport
        from nautilus_trader.model.identifiers import ClientOrderId
        
        return FillReport(
            order_id=ClientOrderId(mt5_trade.ticket),
            client_order_id=ClientOrderId(mt5_trade.order_id),
            trade_id=mt5_trade.ticket,
            venue_order_id=mt5_trade.ticket,
            position_id=mt5_trade.position_id,
            order_side=mt5_trade.side,
            quantity=mt5_trade.volume,
            price=mt5_trade.price,
            commission=mt5_trade.commission,
            liquidity_flags=mt5_trade.liquidity_flag,
            fee=mt5_trade.swap,
            ts_event=mt5_trade.time,
            ts_init=mt5_trade.time,
        )

    def _convert_mt5_position_to_status_report(self, mt5_position):
        """Convert MT5 position to Nautilus PositionStatusReport"""
        # Import Nautilus types
        from nautilus_trader.execution.reports import PositionStatusReport
        from nautilus_trader.model.enums import PositionSide
        
        # Map volume sign to position side
        side = PositionSide.LONG if mt5_position.volume > 0 else PositionSide.SHORT
        
        return PositionStatusReport(
            instrument_id=mt5_position.symbol,
            position_id=mt5_position.ticket,
            position_side=side,
            quantity=abs(mt5_position.volume),
            avg_price=mt5_position.price,
            realized_pnl=mt5_position.profit,
            unrealized_pnl=mt5_position.profit,
            ts_event=mt5_position.time,
            ts_init=mt5_position.time,
        )

    async def _generate_rejection_report(self, command, reason):
        """Generate order rejection report"""
        # Import Nautilus types
        from nautilus_trader.execution.reports import OrderStatusReport
        from nautilus_trader.model.enums import OrderStatus
        from nautilus_trader.model.identifiers import ClientOrderId
        import time
        
        report = OrderStatusReport(
            client_order_id=command.client_order_id,
            venue_order_id="",
            status=OrderStatus.REJECTED,
            filled_quantity=0.0,
            average_price=0.0,
            submitted_time=int(time.time() * 1_000_000_000),
            filled_time=None,
        )
        
        # Publish rejection report
        self._msgbus.publish(
            f"order.status.{command.client_id}",
            report
        )
