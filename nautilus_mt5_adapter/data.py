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
The data client for the MetaTrader 5 integration.
"""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING

from nautilus_mt5_adapter.common import MT5_VENUE
from nautilus_trader.core.correctness import PyCondition
from nautilus_trader.data.messages import RequestBars
from nautilus_trader.data.messages import RequestData
from nautilus_trader.data.messages import RequestInstrument
from nautilus_trader.data.messages import RequestInstruments
from nautilus_trader.data.messages import RequestQuoteTicks
from nautilus_trader.data.messages import RequestTradeTicks
from nautilus_trader.data.messages import SubscribeBars
from nautilus_trader.data.messages import SubscribeData
from nautilus_trader.data.messages import SubscribeInstrument
from nautilus_trader.data.messages import SubscribeInstruments
from nautilus_trader.data.messages import SubscribeInstrumentStatus
from nautilus_trader.data.messages import SubscribeOrderBook
from nautilus_trader.data.messages import SubscribeQuoteTicks
from nautilus_trader.data.messages import SubscribeTradeTicks
from nautilus_trader.data.messages import UnsubscribeBars
from nautilus_trader.data.messages import UnsubscribeData
from nautilus_trader.data.messages import UnsubscribeInstrument
from nautilus_trader.data.messages import UnsubscribeInstruments
from nautilus_trader.data.messages import UnsubscribeInstrumentStatus
from nautilus_trader.data.messages import UnsubscribeOrderBook
from nautilus_trader.data.messages import UnsubscribeQuoteTicks
from nautilus_trader.data.messages import UnsubscribeTradeTicks
from nautilus_trader.live.data_client import LiveMarketDataClient
from nautilus_trader.model.identifiers import ClientId

if TYPE_CHECKING:
    from nautilus_mt5_adapter.config import Mt5DataClientConfig
    from nautilus_mt5_adapter.providers import Mt5InstrumentProvider
    from nautilus_trader.cache.cache import Cache
    from nautilus_trader.common.component import LiveClock
    from nautilus_trader.common.component import MessageBus


class Mt5DataClient(LiveMarketDataClient):
    """
    MetaTrader 5 market data client.

    This client implements the NautilusTrader LiveMarketDataClient interface
    and delegates to the Rust MT5 HTTP client exposed via PyO3.

    Since MT5 middleware only supports REST (no WebSocket), data subscriptions
    are implemented using polling at configured intervals.
    """

    def __init__(
        self,
        loop: asyncio.AbstractEventLoop,
        client,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
        instrument_provider: Mt5InstrumentProvider,
        config: Mt5DataClientConfig,
    ) -> None:
        super().__init__(
            loop=loop,
            client_id=ClientId("MT5"),
            venue=MT5_VENUE,
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

        # Track subscriptions for polling
        self._subscribed_quote_ticks: set[str] = set()
        self._subscribed_trade_ticks: set[str] = set()
        self._subscribed_bars: set[str] = set()

        # Polling task handle
        self._poll_task: asyncio.Task | None = None

    # -- CONNECTION HANDLERS ----------------------------------------------------------------------

    async def _connect(self) -> None:
        """Connect to the MT5 middleware."""
        self._log.info("Connecting to MT5 middleware...")
        try:
            # Initialize HTTP client connection (login if needed)
            await self._client.initialize()

            # Load instruments if configured
            await self._instrument_provider.initialize()

            self._log.info("Connected to MT5 middleware successfully.")
        except Exception as e:
            self._log.error(f"Failed to connect to MT5: {e}")
            raise

    async def _disconnect(self) -> None:
        """Disconnect from the MT5 middleware."""
        self._log.info("Disconnecting from MT5 middleware...")
        # Stop polling task if running
        if self._poll_task is not None:
            self._poll_task.cancel()
            self._poll_task = None

        # Shutdown HTTP client
        await self._client.shutdown()

        self._log.info("Disconnected from MT5 middleware.")

    # -- SUBSCRIPTION HANDLERS -------------------------------------------------------------------

    async def _subscribe(self, command: SubscribeData) -> None:
        """Handle generic data subscription."""
        self._log.info(f"Subscribing to custom data: {command.data_type}")
        # MT5 adapter doesn't support generic custom data subscriptions
        self._log.warning(
            f"Custom data subscription not supported: {command.data_type}"
        )

    async def _unsubscribe(self, command: UnsubscribeData) -> None:
        """Handle generic data unsubscription."""
        self._log.info(f"Unsubscribing from custom data: {command.data_type}")

    async def _subscribe_instruments(self, command: SubscribeInstruments) -> None:
        """Subscribe to all instrument updates."""
        self._log.info("Subscribing to all instruments")
        # Load all instruments from provider
        await self._instrument_provider.load_all_async()
        instruments = self._instrument_provider.list_all()
        for instrument in instruments:
            self._handle_data(instrument)

    async def _unsubscribe_instruments(self, command: UnsubscribeInstruments) -> None:
        """Unsubscribe from all instrument updates."""
        self._log.info("Unsubscribing from all instruments")

    async def _subscribe_instrument(self, command: SubscribeInstrument) -> None:
        """Subscribe to a specific instrument."""
        self._log.info(f"Subscribing to instrument {command.instrument_id}")
        await self._instrument_provider.load_async(command.instrument_id)
        instrument = self._instrument_provider.find(command.instrument_id)
        if instrument:
            self._handle_data(instrument)

    async def _unsubscribe_instrument(self, command: UnsubscribeInstrument) -> None:
        """Unsubscribe from a specific instrument."""
        self._log.info(f"Unsubscribing from instrument {command.instrument_id}")

    async def _subscribe_order_book_deltas(self, command: SubscribeOrderBook) -> None:
        """Subscribe to order book deltas (not supported by MT5 REST API)."""
        self._log.warning(
            f"Order book deltas not supported for {command.instrument_id} "
            "(MT5 REST API limitation)"
        )

    async def _unsubscribe_order_book_deltas(
        self, command: UnsubscribeOrderBook
    ) -> None:
        """Unsubscribe from order book deltas."""
        self._log.info(
            f"Unsubscribing from order book deltas for {command.instrument_id}"
        )

    async def _subscribe_order_book_snapshots(
        self, command: SubscribeOrderBook
    ) -> None:
        """Subscribe to order book snapshots via polling."""
        self._log.info(
            f"Subscribing to order book snapshots for {command.instrument_id}"
        )
        # MT5 market book requires explicit add/release
        symbol = command.instrument_id.symbol.value
        await self._client.market_book_add(symbol)

    async def _unsubscribe_order_book_snapshots(
        self, command: UnsubscribeOrderBook
    ) -> None:
        """Unsubscribe from order book snapshots."""
        self._log.info(
            f"Unsubscribing from order book snapshots for {command.instrument_id}"
        )
        symbol = command.instrument_id.symbol.value
        await self._client.market_book_release(symbol)

    async def _subscribe_quote_ticks(self, command: SubscribeQuoteTicks) -> None:
        """Subscribe to quote ticks via polling."""
        instrument_id = command.instrument_id
        self._log.info(f"Subscribing to quote ticks for {instrument_id}")
        symbol = instrument_id.symbol.value
        self._subscribed_quote_ticks.add(symbol)
        self._start_polling_if_needed()

    async def _unsubscribe_quote_ticks(self, command: UnsubscribeQuoteTicks) -> None:
        """Unsubscribe from quote ticks."""
        instrument_id = command.instrument_id
        self._log.info(f"Unsubscribing from quote ticks for {instrument_id}")
        symbol = instrument_id.symbol.value
        self._subscribed_quote_ticks.discard(symbol)

    async def _subscribe_trade_ticks(self, command: SubscribeTradeTicks) -> None:
        """Subscribe to trade ticks via polling."""
        instrument_id = command.instrument_id
        self._log.info(f"Subscribing to trade ticks for {instrument_id}")
        symbol = instrument_id.symbol.value
        self._subscribed_trade_ticks.add(symbol)
        self._start_polling_if_needed()

    async def _unsubscribe_trade_ticks(self, command: UnsubscribeTradeTicks) -> None:
        """Unsubscribe from trade ticks."""
        instrument_id = command.instrument_id
        self._log.info(f"Unsubscribing from trade ticks for {instrument_id}")
        symbol = instrument_id.symbol.value
        self._subscribed_trade_ticks.discard(symbol)

    async def _subscribe_bars(self, command: SubscribeBars) -> None:
        """Subscribe to bars via polling."""
        bar_type = command.bar_type
        self._log.info(f"Subscribing to bars for {bar_type}")
        self._subscribed_bars.add(str(bar_type))
        self._start_polling_if_needed()

    async def _unsubscribe_bars(self, command: UnsubscribeBars) -> None:
        """Unsubscribe from bars."""
        bar_type = command.bar_type
        self._log.info(f"Unsubscribing from bars for {bar_type}")
        self._subscribed_bars.discard(str(bar_type))

    async def _subscribe_instrument_status(
        self, command: SubscribeInstrumentStatus
    ) -> None:
        """Subscribe to instrument status updates."""
        self._log.info(f"Subscribing to instrument status for {command.instrument_id}")
        # MT5 doesn't have a dedicated instrument status stream

    async def _unsubscribe_instrument_status(
        self, command: UnsubscribeInstrumentStatus
    ) -> None:
        """Unsubscribe from instrument status updates."""
        self._log.info(
            f"Unsubscribing from instrument status for {command.instrument_id}"
        )

    # -- REQUEST HANDLERS -------------------------------------------------------------------------

    async def _request(self, request: RequestData) -> None:
        """Handle generic data request."""
        self._log.info(f"Requesting data: {request.data_type}")
        self._log.warning(f"Generic data request not implemented: {request.data_type}")

    async def _request_instrument(self, request: RequestInstrument) -> None:
        """Request a specific instrument."""
        instrument_id = request.instrument_id
        self._log.info(f"Requesting instrument: {instrument_id}")
        await self._instrument_provider.load_async(instrument_id)
        instrument = self._instrument_provider.find(instrument_id)
        if instrument:
            self._handle_instrument(instrument, request.id, request.params)
        else:
            raise ValueError(f"Instrument not found: {instrument_id}")

    async def _request_instruments(self, request: RequestInstruments) -> None:
        """Request all instruments."""
        self._log.info("Requesting all instruments")
        await self._instrument_provider.load_all_async()
        instruments = self._instrument_provider.list_all()
        for instrument in instruments:
            self._handle_instrument(instrument, request.id, request.params)

    async def _request_quote_ticks(self, request: RequestQuoteTicks) -> None:
        """Request historical quote ticks."""
        instrument_id = request.instrument_id
        self._log.info(f"Requesting quote ticks for {instrument_id}")
        symbol = instrument_id.symbol.value
        # Request tick data from MT5
        ticks = await self._client.copy_ticks_from(
            symbol=symbol,
            count=request.limit or 1000,
        )
        self._log.info(f"Received {len(ticks) if ticks else 0} ticks for {symbol}")
        # TODO: Convert MT5 ticks to QuoteTick and publish

    async def _request_trade_ticks(self, request: RequestTradeTicks) -> None:
        """Request historical trade ticks."""
        instrument_id = request.instrument_id
        self._log.info(f"Requesting trade ticks for {instrument_id}")
        symbol = instrument_id.symbol.value
        # Request tick data from MT5
        ticks = await self._client.copy_ticks_from(
            symbol=symbol,
            count=request.limit or 1000,
        )
        self._log.info(
            f"Received {len(ticks) if ticks else 0} trade ticks for {symbol}"
        )
        # TODO: Convert MT5 ticks to TradeTick and publish

    async def _request_bars(self, request: RequestBars) -> None:
        """Request historical bars."""
        bar_type = request.bar_type
        self._log.info(f"Requesting bars for {bar_type}")
        symbol = bar_type.instrument_id.symbol.value
        # Request OHLC data from MT5
        rates = await self._client.copy_rates_from(
            symbol=symbol,
            count=request.limit or 1000,
        )
        self._log.info(f"Received {len(rates) if rates else 0} bars for {symbol}")
        # TODO: Convert MT5 rates to Bar and publish

    # -- POLLING MECHANISM ------------------------------------------------------------------------

    def _start_polling_if_needed(self) -> None:
        """Start the polling task if subscriptions exist and polling is not running."""
        has_subscriptions = (
            self._subscribed_quote_ticks
            or self._subscribed_trade_ticks
            or self._subscribed_bars
        )
        if has_subscriptions and self._poll_task is None:
            self._poll_task = self.create_task(self._poll_loop())

    async def _poll_loop(self) -> None:
        """Main polling loop for REST-based data updates."""
        poll_interval_secs = self._config.poll_interval_ms / 1000.0
        self._log.info(f"Starting polling loop with interval {poll_interval_secs}s")

        while True:
            try:
                await self._poll_quotes()
                await asyncio.sleep(poll_interval_secs)
            except asyncio.CancelledError:
                self._log.info("Polling loop cancelled")
                break
            except Exception as e:
                self._log.error(f"Error in polling loop: {e}")
                await asyncio.sleep(poll_interval_secs)

    async def _poll_quotes(self) -> None:
        """Poll for quote updates for subscribed symbols."""
        for symbol in list(self._subscribed_quote_ticks):
            try:
                tick = await self._client.symbol_info_tick(symbol)
                if tick:
                    # TODO: Convert MT5 tick to QuoteTick and publish via _handle_data
                    pass
            except Exception as e:
                self._log.warning(f"Failed to poll quote for {symbol}: {e}")
