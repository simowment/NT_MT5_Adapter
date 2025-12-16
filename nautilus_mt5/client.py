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
from datetime import datetime
from datetime import timedelta
from typing import Any

from .bindings import Mt5Config, Mt5HttpClient


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
