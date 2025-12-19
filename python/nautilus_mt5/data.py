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
import json
from datetime import timedelta
from typing import TYPE_CHECKING

from nautilus_mt5.common import MT5_VENUE
from nautilus_trader.core.correctness import PyCondition
from nautilus_trader.data.messages import RequestBars
from nautilus_trader.data.messages import RequestData
from nautilus_trader.data.messages import RequestInstrument
from nautilus_trader.data.messages import RequestInstruments
from nautilus_trader.data.messages import RequestOrderBookSnapshot
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
    from nautilus_mt5.config import Mt5DataClientConfig
    from nautilus_mt5.providers import Mt5InstrumentProvider
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
        await self._client.market_book_add(json.dumps(symbol))

    async def _unsubscribe_order_book_snapshots(
        self, command: UnsubscribeOrderBook
    ) -> None:
        """Unsubscribe from order book snapshots."""
        self._log.info(
            f"Unsubscribing from order book snapshots for {command.instrument_id}"
        )
        symbol = command.instrument_id.symbol.value
        await self._client.market_book_release(json.dumps(symbol))

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
            self._handle_instrument(instrument, request.id, None, None, request.params)
        else:
            raise ValueError(f"Instrument not found: {instrument_id}")

    async def _request_instruments(self, request: RequestInstruments) -> None:
        """Request all instruments."""
        self._log.info("Requesting all instruments")
        await self._instrument_provider.load_all_async()
        instruments = self._instrument_provider.list_all()
        for instrument in instruments:
            self._handle_instrument(instrument, request.id, None, None, request.params)

    async def _request_order_book_snapshot(
        self, request: RequestOrderBookSnapshot
    ) -> None:
        """Request order book snapshot (limited support in MT5 REST API)."""
        instrument_id = request.instrument_id
        # MT5 market book requires explicit add/release session
        # We can try to get it if we wrap it in add/remove calls, but polling is better usage
        # This is a one-off request
        symbol = instrument_id.symbol.value

        # 1. Add to market book
        await self._client.market_book_add(json.dumps([symbol]))

        # 2. Get data
        response_json = await self._client.market_book_get(json.dumps([symbol]))
        response = json.loads(response_json) if response_json else {}
        book_entries = (
            response.get("result", []) if isinstance(response, dict) else response
        )

        # 3. Release
        await self._client.market_book_release(json.dumps([symbol]))

        if not book_entries:
            self._log.warning(f"No order book data received for {symbol}")
            return

        # TODO: Parse into OrderBookSnapshot and publish
        # Currently just logging as warning that full parsing is not implemented
        self._log.warning("Order book snapshot parsing not fully implemented yet")

    async def _request_quote_ticks(self, request: RequestQuoteTicks) -> None:
        """Request historical quote ticks."""
        instrument_id = request.instrument_id
        self._log.info(f"Requesting quote ticks for {instrument_id}")
        symbol = instrument_id.symbol.value

        # Get instrument for precision info
        instrument = self._cache.instrument(instrument_id)
        if not instrument:
            self._log.error(f"Instrument {instrument_id} not found in cache")
            return

        # Decide strategy: Range (paginated) or Count
        if request.start and request.end:
            # Range request with pagination
            await self._fetch_and_publish_ticks_range(
                instrument_id=instrument_id,
                request_id=request.id,
                start=request.start,
                end=request.end,
                tick_type="QUOTE",
                params=request.params,
            )
        else:
            # Count request (from start)
            from_ts = int(request.start.timestamp()) if request.start else 0
            limit = request.limit or 1000

            # Request tick data from MT5: [symbol, from_time, count, flags]
            params = [symbol, from_ts, limit, 1]  # flags=1 for INFO ticks
            response_json = await self._client.copy_ticks_from(json.dumps(params))
            response = json.loads(response_json) if response_json else {}
            ticks = (
                response.get("result", []) if isinstance(response, dict) else response
            )

            if not ticks:
                self._log.warning(f"No quote ticks received for {symbol}")
                return

            self._log.info(f"Received {len(ticks)} ticks for {symbol}")
            self._process_quote_ticks(
                instrument_id,
                ticks,
                request.id,
                request.start,
                request.end,
                request.params,
            )

    async def _request_trade_ticks(self, request: RequestTradeTicks) -> None:
        """Request historical trade ticks."""
        instrument_id = request.instrument_id
        self._log.info(f"Requesting trade ticks for {instrument_id}")
        symbol = instrument_id.symbol.value

        # Get instrument for precision info
        instrument = self._cache.instrument(instrument_id)
        if not instrument:
            self._log.error(f"Instrument {instrument_id} not found in cache")
            return

        if request.start and request.end:
            # Range request with pagination
            await self._fetch_and_publish_ticks_range(
                instrument_id=instrument_id,
                request_id=request.id,
                start=request.start,
                end=request.end,
                tick_type="TRADE",
                params=request.params,
            )
        else:
            # Count request
            from_ts = int(request.start.timestamp()) if request.start else 0
            limit = request.limit or 1000

            # Request tick data from MT5: [symbol, from_time, count, flags]
            params = [symbol, from_ts, limit, 2]  # flags=2 for TRADE ticks
            response_json = await self._client.copy_ticks_from(json.dumps(params))
            response = json.loads(response_json) if response_json else {}
            ticks = (
                response.get("result", []) if isinstance(response, dict) else response
            )

            if not ticks:
                self._log.warning(f"No trade ticks received for {symbol}")
                return

            self._log.info(f"Received {len(ticks)} trade ticks for {symbol}")
            self._process_trade_ticks(
                instrument_id,
                ticks,
                request.id,
                request.start,
                request.end,
                request.params,
            )

    async def _request_bars(self, request: RequestBars) -> None:
        """Request historical bars."""
        bar_type = request.bar_type
        self._log.info(f"Requesting bars for {bar_type}")
        symbol = bar_type.instrument_id.symbol.value
        instrument_id = bar_type.instrument_id

        # Get instrument for precision info
        instrument = self._cache.instrument(instrument_id)
        if not instrument:
            self._log.error(f"Instrument {instrument_id} not found in cache")
            return

        # Map bar_type spec to MT5 timeframe value
        from nautilus_mt5.constants import Mt5Timeframe

        # Get timeframe from bar_type (e.g., "1-MINUTE" -> M1)
        step = bar_type.spec.step
        aggregation = str(bar_type.spec.aggregation)

        tf_map = {
            ("1", "MINUTE"): Mt5Timeframe.M1,
            ("5", "MINUTE"): Mt5Timeframe.M5,
            ("15", "MINUTE"): Mt5Timeframe.M15,
            ("30", "MINUTE"): Mt5Timeframe.M30,
            ("1", "HOUR"): Mt5Timeframe.H1,
            ("4", "HOUR"): Mt5Timeframe.H4,
            ("1", "DAY"): Mt5Timeframe.D1,
        }
        tf_value = tf_map.get((str(step), aggregation), Mt5Timeframe.M1)

        # Calculate bar period in seconds for close time calculation
        tf_seconds = {
            Mt5Timeframe.M1: 60,
            Mt5Timeframe.M5: 300,
            Mt5Timeframe.M15: 900,
            Mt5Timeframe.M30: 1800,
            Mt5Timeframe.H1: 3600,
            Mt5Timeframe.H4: 14400,
            Mt5Timeframe.D1: 86400,
        }.get(tf_value, 60)

        # Decide strategy: Range (paginated) or Count
        if request.start and request.end:
            # Range request with pagination
            # Chunk size: 30 days for M1, larger for others? 30 days is safe default
            chunk_size = timedelta(days=30)

            # Ensure start is timezone-aware if end is
            start_cursor = request.start
            end_limit = request.end

            total_bars = 0

            self._log.info(
                f"Starting paginated bars request: {start_cursor} to {end_limit}"
            )

            while start_cursor < end_limit:
                chunk_end = min(start_cursor + chunk_size, end_limit)

                req_start_ts = int(start_cursor.timestamp())
                req_end_ts = int(chunk_end.timestamp())

                # copy_rates_range: [symbol, timeframe, start_time, end_time]
                params = [symbol, tf_value, req_start_ts, req_end_ts]

                # self._log.debug(f"Fetching chunk: {start_cursor} -> {chunk_end}")

                try:
                    response_json = await self._client.copy_rates_range(
                        json.dumps(params)
                    )
                    response = json.loads(response_json) if response_json else {}
                    rates = (
                        response.get("result", [])
                        if isinstance(response, dict)
                        else response
                    )

                    if rates:
                        total_bars += len(rates)
                        self._process_bars(
                            bar_type,
                            rates,
                            instrument,
                            tf_seconds,
                            request.id,
                            request.start,
                            request.end,
                            request.params,
                        )
                    else:
                        # Empty response, might be a gap, just continue
                        pass
                except Exception as e:
                    self._log.error(
                        f"Error fetching bars chunk {start_cursor}-{chunk_end}: {e}"
                    )

                # Advance cursor
                start_cursor = chunk_end

                # Yield to event loop to avoid blocking too long
                await asyncio.sleep(0.01)

            self._log.info(f"Finished paginated bars request. Total bars: {total_bars}")

        else:
            # Count request
            from_ts = int(request.start.timestamp()) if request.start else 0
            limit = request.limit or 1000

            params = [symbol, tf_value, from_ts, limit]
            response_json = await self._client.copy_rates_from(json.dumps(params))
            response = json.loads(response_json) if response_json else {}
            rates = (
                response.get("result", []) if isinstance(response, dict) else response
            )

            if not rates:
                self._log.warning(f"No bars received for {symbol}")
                return

            self._log.info(f"Received {len(rates)} bars for {symbol}")
            self._process_bars(
                bar_type,
                rates,
                instrument,
                tf_seconds,
                request.id,
                request.start,
                request.end,
                request.params,
            )

    # -- PROCESSING HELPERS -----------------------------------------------------------------------

    def _process_bars(
        self, bar_type, rates, instrument, tf_seconds, request_id, start, end, params
    ):
        """Convert MT5 rates to Bars and publish."""
        from nautilus_trader.model.data import Bar
        from nautilus_trader.model.objects import Price, Quantity

        bars = []
        for row in rates:
            if not isinstance(row, list) or len(row) < 6:
                continue

            ts_open = int(row[0])
            ts_close = ts_open + tf_seconds
            ts_ns = ts_close * 1_000_000_000

            # Skip if out of strict range (MT5 might return slightly outside)
            if start and ts_open < start.timestamp():
                continue
            if end and ts_open >= end.timestamp():
                continue

            bar = Bar(
                bar_type=bar_type,
                open=Price(float(row[1]), instrument.price_precision),
                high=Price(float(row[2]), instrument.price_precision),
                low=Price(float(row[3]), instrument.price_precision),
                close=Price(float(row[4]), instrument.price_precision),
                volume=Quantity(float(row[5]), instrument.size_precision),
                ts_event=ts_ns,
                ts_init=ts_ns,
            )
            bars.append(bar)

        if bars:
            self._handle_bars(
                bar_type,
                bars,
                request_id,
                start,
                end,
                params,
            )

    async def _fetch_and_publish_ticks_range(
        self, instrument_id, request_id, start, end, tick_type, params
    ):
        """Fetch ticks (QUOTE or TRADE) in chunks and publish."""
        symbol = instrument_id.symbol.value
        chunk_size = timedelta(hours=6)  # 6 hours for ticks
        start_cursor = start

        while start_cursor < end:
            chunk_end = min(start_cursor + chunk_size, end)

            req_start_ts = int(start_cursor.timestamp())
            req_end_ts = int(chunk_end.timestamp())

            # copy_ticks_range: [symbol, start, end, flags]
            flags = 1 if tick_type == "QUOTE" else 2
            api_params = [symbol, req_start_ts, req_end_ts, flags]

            try:
                response_json = await self._client.copy_ticks_range(
                    json.dumps(api_params)
                )
                response = json.loads(response_json) if response_json else {}
                ticks = (
                    response.get("result", [])
                    if isinstance(response, dict)
                    else response
                )

                if ticks:
                    if tick_type == "QUOTE":
                        self._process_quote_ticks(
                            instrument_id, ticks, request_id, start, end, params
                        )
                    else:
                        self._process_trade_ticks(
                            instrument_id, ticks, request_id, start, end, params
                        )

            except Exception as e:
                self._log.error(
                    f"Error fetching {tick_type} ticks chunk {start_cursor}-{chunk_end}: {e}"
                )

            start_cursor = chunk_end
            await asyncio.sleep(0.01)

    def _process_quote_ticks(
        self, instrument_id, ticks, request_id, start, end, params
    ):
        """Convert and publish quote ticks."""
        from nautilus_trader.model.data import QuoteTick
        from nautilus_trader.model.objects import Price, Quantity

        instrument = self._cache.instrument(instrument_id)
        if not instrument:
            return

        quote_ticks = []
        for row in ticks:
            # Format: [time, bid, ask, last, volume, time_msc, ... ]
            if not isinstance(row, list) or len(row) < 3:
                continue

            ts_ns = (
                int(row[5]) * 1_000_000 if len(row) > 5 else int(row[0]) * 1_000_000_000
            )

            quote_tick = QuoteTick(
                instrument_id=instrument_id,
                bid_price=Price(float(row[1]), instrument.price_precision),
                ask_price=Price(float(row[2]), instrument.price_precision),
                bid_size=Quantity(1.0, instrument.size_precision),
                ask_size=Quantity(1.0, instrument.size_precision),
                ts_event=ts_ns,
                ts_init=ts_ns,
            )
            quote_ticks.append(quote_tick)

        if quote_ticks:
            self._handle_quote_ticks(
                instrument_id, quote_ticks, request_id, start, end, params
            )

    def _process_trade_ticks(
        self, instrument_id, ticks, request_id, start, end, params
    ):
        """Convert and publish trade ticks."""
        from nautilus_trader.model.data import TradeTick
        from nautilus_trader.model.enums import AggressorSide
        from nautilus_trader.model.identifiers import TradeId
        from nautilus_trader.model.objects import Price, Quantity

        instrument = self._cache.instrument(instrument_id)
        if not instrument:
            return

        trade_ticks = []
        for i, row in enumerate(ticks):
            # Format: [time, bid, ask, last, volume, time_msc, ... ]
            if not isinstance(row, list) or len(row) < 5:
                continue

            # Need to handle trade tick filtering if deduplication is needed
            # But here we just map stream

            ts_ns = (
                int(row[5]) * 1_000_000 if len(row) > 5 else int(row[0]) * 1_000_000_000
            )
            val_last = float(row[3]) if row[3] else float(row[1])
            val_vol = float(row[7]) if len(row) > 7 and row[7] else float(row[4])

            trade_tick = TradeTick(
                instrument_id=instrument_id,
                price=Price(val_last, instrument.price_precision),
                size=Quantity(val_vol, instrument.size_precision)
                if val_vol > 0
                else Quantity(1.0, instrument.size_precision),
                aggressor_side=AggressorSide.NO_AGGRESSOR,
                trade_id=TradeId(f"{ts_ns}_{i}"),  # Generate a pseudo-unique ID
                ts_event=ts_ns,
                ts_init=ts_ns,
            )
            trade_ticks.append(trade_tick)

        if trade_ticks:
            self._handle_trade_ticks(
                instrument_id, trade_ticks, request_id, start, end, params
            )

    # -- LONG POLLING MECHANISM --------------------------------------------------------------------
    # Long polling: holds connection open, only processes when data changes.
    # More efficient than fixed-interval polling.

    def _start_polling_if_needed(self) -> None:
        """Start the polling task if subscriptions exist and polling is not running."""
        has_subscriptions = (
            self._subscribed_quote_ticks
            or self._subscribed_trade_ticks
            or self._subscribed_bars
        )
        if has_subscriptions and self._poll_task is None:
            self._poll_task = self.create_task(self._poll_loop())
            self._log.info("Started long-polling loop")

    def _stop_polling_if_idle(self) -> None:
        """Stop polling if no subscriptions remain."""
        has_subscriptions = (
            self._subscribed_quote_ticks
            or self._subscribed_trade_ticks
            or self._subscribed_bars
        )
        if not has_subscriptions and self._poll_task is not None:
            self._poll_task.cancel()
            self._poll_task = None
            self._log.info("Stopped long-polling loop (no subscriptions)")

    async def _poll_loop(self) -> None:
        """
        Long-polling loop for REST-based data updates.

        Uses longer intervals and only processes changed data.
        """
        base_interval = self._config.poll_interval_ms / 1000.0
        min_interval = 0.1  # 100ms minimum
        max_interval = 5.0  # 5s maximum when idle
        current_interval = base_interval

        self._log.info(f"Long-polling loop started (base interval: {base_interval}s)")

        while True:
            try:
                # Check if we still have subscriptions
                if not (
                    self._subscribed_quote_ticks
                    or self._subscribed_trade_ticks
                    or self._subscribed_bars
                ):
                    self._log.debug("No subscriptions, waiting...")
                    await asyncio.sleep(1.0)
                    continue

                # Poll all subscribed data types
                data_received = False

                if self._subscribed_quote_ticks:
                    quotes_received = await self._poll_quotes()
                    data_received = data_received or quotes_received

                if self._subscribed_trade_ticks:
                    trades_received = await self._poll_trades()
                    data_received = data_received or trades_received

                # Adaptive interval: speed up when data flows, slow down when quiet
                if data_received:
                    current_interval = max(min_interval, current_interval * 0.8)
                else:
                    current_interval = min(max_interval, current_interval * 1.2)

                await asyncio.sleep(current_interval)

            except asyncio.CancelledError:
                self._log.info("Long-polling loop cancelled")
                break
            except Exception as e:
                self._log.error(f"Error in polling loop: {e}")
                await asyncio.sleep(base_interval)

    async def _poll_quotes(self) -> bool:
        """
        Poll for quote updates for subscribed symbols.

        Returns True if any new data was received.
        """
        from nautilus_trader.model.data import QuoteTick
        from nautilus_trader.model.objects import Price, Quantity

        data_received = False

        for symbol in list(self._subscribed_quote_ticks):
            try:
                response_json = await self._client.symbol_info_tick(
                    json.dumps([symbol])
                )
                response = json.loads(response_json) if response_json else {}
                tick = (
                    response.get("result") if isinstance(response, dict) else response
                )

                if not tick:
                    continue

                # Get instrument for precision
                from nautilus_trader.model.identifiers import InstrumentId, Symbol

                instrument_id = InstrumentId(Symbol(symbol), MT5_VENUE)
                instrument = self._cache.instrument(instrument_id)

                if not instrument:
                    continue

                # Check if tick has changed (dedupe)
                bid = float(tick.get("bid", 0))
                ask = float(tick.get("ask", 0))
                time_msc = tick.get("time_msc", 0)

                last_key = f"{symbol}_quote"
                last_time = getattr(self, "_last_tick_times", {}).get(last_key, 0)

                if time_msc <= last_time:
                    continue  # Already processed this tick

                # Update last seen time
                if not hasattr(self, "_last_tick_times"):
                    self._last_tick_times = {}
                self._last_tick_times[last_key] = time_msc

                # Convert to QuoteTick
                ts_ns = int(time_msc) * 1_000_000  # msc to ns

                quote_tick = QuoteTick(
                    instrument_id=instrument_id,
                    bid_price=Price(bid, instrument.price_precision),
                    ask_price=Price(ask, instrument.price_precision),
                    bid_size=Quantity(1.0, instrument.size_precision),
                    ask_size=Quantity(1.0, instrument.size_precision),
                    ts_event=ts_ns,
                    ts_init=ts_ns,
                )

                self._handle_data(quote_tick)
                data_received = True

            except Exception as e:
                self._log.warning(f"Failed to poll quote for {symbol}: {e}")

        return data_received

    async def _poll_trades(self) -> bool:
        """
        Poll for trade tick updates for subscribed symbols.

        Returns True if any new data was received.
        """
        # Trade ticks require copy_ticks_from which is heavier.
        # For live trading, quote ticks (bid/ask) are usually sufficient.
        # This is a placeholder - full implementation would track last tick time per symbol.
        return False
