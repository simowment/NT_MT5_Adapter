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
High-level MT5 Client for scripts and research.

This client abstracts the raw REST API calls allowing easy data fetching.
"""

from __future__ import annotations

import asyncio
import json
from datetime import datetime, timezone
from datetime import timedelta
from typing import Any

from nautilus_trader.model.data import Bar, BarType
from nautilus_trader.model.instruments import Instrument
from nautilus_trader.model.objects import Price, Quantity

from .bindings import Mt5Config, Mt5HttpClient
from .constants import Mt5Timeframe


class Mt5Client:
    """
    High-level client for interacting with MetaTrader 5 REST API.

    Provides abstraction over the raw JSON requests/responses.
    """

    # Timeframe mapping
    TIMEFRAMES = {
        "M1": 1,
        "M5": 5,
        "M15": 15,
        "M30": 30,
        "H1": 16385,
        "H4": 16388,
        "D1": 16408,
        "W1": 32769,
        "MN1": 49153,
    }

    def __init__(self, base_url: str = "http://localhost:5000", **kwargs):
        self._config = Mt5Config(base_url=base_url)
        self._client = Mt5HttpClient(config=self._config, base_url=base_url)

    async def initialize(self) -> bool:
        """Initialize connection to MT5 terminal."""
        res = await self._client.initialize()
        return self._parse_bool(res)

    async def shutdown(self) -> bool:
        """Shutdown connection to MT5 terminal."""
        res = await self._client.shutdown()
        return self._parse_bool(res)

    async def request_bars(
        self,
        bar_type: BarType,
        instrument: Instrument,
        start: datetime | None = None,
        end: datetime | None = None,
        count: int | None = None,
    ) -> list[Bar]:
        """
        Request historical bars from MT5 as Nautilus Bar objects.

        This follows the NautilusTrader adapter pattern where the HTTP client
        returns domain objects directly instead of raw dictionaries.

        Parameters
        ----------
        bar_type : BarType
            The bar type specification including instrument, timeframe, and aggregation.
        instrument : Instrument
            The instrument for precision information (price_precision, size_precision).
        start : datetime, optional
            Start time for fetching bars.
        end : datetime, optional
            End time for fetching bars.
        count : int, optional
            Number of bars to fetch (used if start/end not specified).

        Returns
        -------
        list[Bar]
            List of Nautilus Bar objects.
        """
        symbol = bar_type.instrument_id.symbol.value

        # Map BarType timeframe to MT5 timeframe value
        tf_seconds = self._bar_type_to_seconds(bar_type)
        tf_val = self._seconds_to_mt5_timeframe(tf_seconds)

        bars: list[Bar] = []

        if start and end:
            # RANGE REQUEST (Paginated)
            start_ts = int(start.timestamp())
            end_ts = int(end.timestamp())
            current_start = start_ts
            chunk_size = int(timedelta(days=30).total_seconds())

            while current_start < end_ts:
                current_end = min(current_start + chunk_size, end_ts)
                params = [symbol, tf_val, current_start, current_end]
                response_json = await self._client.copy_rates_range(json.dumps(params))
                chunk_data = self._parse_response(response_json)

                if isinstance(chunk_data, list):
                    for row in chunk_data:
                        if isinstance(row, list) and len(row) >= 6:
                            bar = self._row_to_bar(
                                row, bar_type, instrument, tf_seconds
                            )
                            bars.append(bar)

                current_start = current_end
                await asyncio.sleep(0.01)

        else:
            # COUNT REQUEST (Default)
            ts = int(datetime.now(timezone.utc).timestamp())
            request_count = count if count is not None else 1000
            params = [symbol, tf_val, ts, request_count]
            response_json = await self._client.copy_rates_from(json.dumps(params))
            data = self._parse_response(response_json)

            if isinstance(data, list):
                for row in data:
                    if isinstance(row, list) and len(row) >= 6:
                        bar = self._row_to_bar(row, bar_type, instrument, tf_seconds)
                        bars.append(bar)

        return bars

    def _row_to_bar(
        self, row: list, bar_type: BarType, instrument: Instrument, tf_seconds: int
    ) -> Bar:
        """Convert a raw MT5 rate row to a Nautilus Bar."""
        ts_open = int(row[0])
        ts_close = ts_open + tf_seconds
        ts_ns = ts_close * 1_000_000_000

        return Bar(
            bar_type=bar_type,
            open=Price(float(row[1]), instrument.price_precision),
            high=Price(float(row[2]), instrument.price_precision),
            low=Price(float(row[3]), instrument.price_precision),
            close=Price(float(row[4]), instrument.price_precision),
            volume=Quantity(float(row[5]), instrument.size_precision),
            ts_event=ts_ns,
            ts_init=ts_ns,
        )

    def _bar_type_to_seconds(self, bar_type: BarType) -> int:
        """Extract timeframe in seconds from BarType."""
        spec = bar_type.spec
        step = spec.step

        # BarAggregation enum values
        from nautilus_trader.model.enums import BarAggregation

        if spec.aggregation == BarAggregation.SECOND:
            return step
        elif spec.aggregation == BarAggregation.MINUTE:
            return step * 60
        elif spec.aggregation == BarAggregation.HOUR:
            return step * 3600
        elif spec.aggregation == BarAggregation.DAY:
            return step * 86400
        else:
            return 60  # Default to 1 minute

    def _seconds_to_mt5_timeframe(self, seconds: int) -> int:
        """Convert seconds to MT5 timeframe value."""
        mapping = {
            60: Mt5Timeframe.M1,
            120: Mt5Timeframe.M2,
            180: Mt5Timeframe.M3,
            240: Mt5Timeframe.M4,
            300: Mt5Timeframe.M5,
            360: Mt5Timeframe.M6,
            600: Mt5Timeframe.M10,
            720: Mt5Timeframe.M12,
            900: Mt5Timeframe.M15,
            1200: Mt5Timeframe.M20,
            1800: Mt5Timeframe.M30,
            3600: Mt5Timeframe.H1,
            7200: Mt5Timeframe.H2,
            10800: Mt5Timeframe.H3,
            14400: Mt5Timeframe.H4,
            21600: Mt5Timeframe.H6,
            28800: Mt5Timeframe.H8,
            43200: Mt5Timeframe.H12,
            86400: Mt5Timeframe.D1,
            604800: Mt5Timeframe.W1,
        }
        return mapping.get(seconds, Mt5Timeframe.M1)

    async def fetch_bars(
        self,
        symbol: str,
        timeframe: str | int,
        start_time: int | datetime | None = None,
        end_time: int | datetime | None = None,
        count: int | None = None,
    ) -> list[dict[str, Any]]:
        """
        Fetch historical bars from MT5.

        Parameters
        ----------
        symbol : str
            Instrument symbol (e.g. "EURUSD").
        timeframe : str | int
            Timeframe string (e.g. "M1") or MT5 integer value.
        start_time : int | datetime | None
            Start time for fetching bars. Defaults to current time if None.
        count : int
            Number of bars to fetch.

        Returns
        -------
        list[dict]
            List of bars as dictionaries.
        """
        # Resolve timeframe
        tf_val = self._resolve_timeframe(timeframe)

        # Resolve start_time
        if start_time is None:
            ts = int(datetime.now().timestamp())
        elif isinstance(start_time, datetime):
            ts = int(start_time.timestamp())
        else:
            ts = int(start_time)

        # Prepare params list for copy_rates_from: [symbol, timeframe, from, count]
        # or copy_rates_range: [symbol, timeframe, start, end]

        bars = []

        if start_time and end_time:
            # RANGE REQUEST (Paginated)
            if isinstance(end_time, datetime):
                end_ts = int(end_time.timestamp())
            else:
                end_ts = int(end_time)

            current_start = ts
            chunk_size = timedelta(days=30).total_seconds()

            while current_start < end_ts:
                # Calculate chunk end
                current_end = min(current_start + chunk_size, end_ts)

                params = [symbol, tf_val, int(current_start), int(current_end)]
                response_json = await self._client.copy_rates_range(json.dumps(params))
                chunk_data = self._parse_response(response_json)

                if isinstance(chunk_data, list):
                    for row in chunk_data:
                        if isinstance(row, list) and len(row) >= 6:
                            bars.append(self._format_bar(row))

                current_start = current_end
                await asyncio.sleep(0.01)  # Yield

        else:
            # COUNT REQUEST (Default)
            request_count = count if count is not None else 1000
            params = [symbol, tf_val, ts, request_count]
            response_json = await self._client.copy_rates_from(json.dumps(params))
            data = self._parse_response(response_json)

            if isinstance(data, list):
                for row in data:
                    if isinstance(row, list) and len(row) >= 6:
                        bars.append(self._format_bar(row))

        return bars

    def _format_bar(self, row: list) -> dict[str, Any]:
        """Format raw MT5 bar list to dict."""
        return {
            "time": row[0],
            "open": row[1],
            "high": row[2],
            "low": row[3],
            "close": row[4],
            "tick_volume": row[5],
            "spread": row[6] if len(row) > 6 else 0,
            "real_volume": row[7] if len(row) > 7 else 0,
        }

    async def fetch_ticks(
        self, symbol: str, start_time: int | datetime | None = None, count: int = 1000
    ) -> list[dict[str, Any]]:
        """Fetch historical ticks."""
        if start_time is None:
            ts = int(datetime.now().timestamp())
        elif isinstance(start_time, datetime):
            ts = int(start_time.timestamp())
        else:
            ts = int(start_time)

        # copy_ticks_from: [symbol, from, count, flags]
        # flags usually 1 (INFO) | 2 (TRADE) | 4 (ORDER) | 8 (STOCKS) | ...
        # Default to COPY_TICKS_ALL (-1) or similar?
        # Documentation uses 1 (COPY_TICKS_ALL in some contexts or INFO)
        params = [symbol, ts, count, 1]

        response_json = await self._client.copy_ticks_from(json.dumps(params))
        data = self._parse_response(response_json)

        # Format: [time, bid, ask, last, volume, time_msc, flags, volume_real]
        ticks = []
        if isinstance(data, list):
            for row in data:
                if isinstance(row, list) and len(row) >= 3:
                    ticks.append(
                        {
                            "time": row[0],
                            "bid": row[1],
                            "ask": row[2],
                            "last": row[3] if len(row) > 3 else 0.0,
                            "volume": row[4] if len(row) > 4 else 0,
                            "time_msc": row[5] if len(row) > 5 else 0,
                            "flags": row[6] if len(row) > 6 else 0,
                            "volume_real": row[7] if len(row) > 7 else 0.0,
                        }
                    )
        return ticks

    def _resolve_timeframe(self, timeframe: str | int) -> int:
        if isinstance(timeframe, int):
            return timeframe
        return self.TIMEFRAMES.get(str(timeframe).upper(), 1)  # Default M1

    def _parse_response(self, response_json: str) -> Any:
        if not response_json:
            return None
        try:
            resp = json.loads(response_json)
            if "error" in resp:
                raise RuntimeError(f"MT5 API Error: {resp['error']}")
            return resp.get("result")
        except json.JSONDecodeError:
            return None

    def _parse_bool(self, response_json: str) -> bool:
        res = self._parse_response(response_json)
        return bool(res)
