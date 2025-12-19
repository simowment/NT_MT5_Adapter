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
The instrument provider for the MetaTrader 5 integration.
"""

from __future__ import annotations

import json
from decimal import Decimal
from typing import TYPE_CHECKING

from nautilus_mt5.common import MT5_VENUE
from nautilus_mt5.config import Mt5InstrumentProviderConfig
from nautilus_trader.common.providers import InstrumentProvider
from nautilus_trader.core.correctness import PyCondition
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import Symbol
from nautilus_trader.model.instruments import Cfd
from nautilus_trader.model.instruments import CurrencyPair
from nautilus_trader.model.objects import Currency
from nautilus_trader.model.objects import Price
from nautilus_trader.model.objects import Quantity

if TYPE_CHECKING:
    from nautilus_trader.common.component import LiveClock


class Mt5InstrumentProvider(InstrumentProvider):
    """
    Provides a MetaTrader 5 instrument definition lookup and creation facility.

    This provider fetches symbol information from the MT5 REST API and converts
    them to Nautilus instrument definitions.

    Parameters
    ----------
    client : Mt5HttpClient
        The MetaTrader 5 HTTP client.
    clock : LiveClock
        The clock for the provider.
    config : Mt5InstrumentProviderConfig
        The configuration for the provider.

    """

    def __new__(cls, *args, **kwargs):
        return super().__new__(cls)

    def __init__(
        self,
        client,
        clock: LiveClock,
        config: Mt5InstrumentProviderConfig,
    ) -> None:
        super().__init__(config=config)

        PyCondition.not_none(client, "client")
        PyCondition.not_none(clock, "clock")

        self._client = client
        self._clock = clock
        self._config = config

    async def load_all_async(self, filters: dict | None = None) -> None:
        """
        Load all instruments from MT5.

        Parameters
        ----------
        filters : dict, optional
            Filters to apply (e.g., {"category": "forex"}).

        """
        if filters is None:
            filters = {}

        self._log.info("Loading all instruments from MT5...")

        # Get total symbol count first
        response_json = await self._client.symbols_total()
        response = json.loads(response_json) if response_json else {}
        total = response.get("result", 0) if response else 0
        self._log.info(f"MT5 reports {total} available symbols")

        # Load symbols in batches
        batch_size = 50
        for start in range(0, total, batch_size):
            request = {"start": start, "count": batch_size}
            response_json = await self._client.symbols_get(json.dumps(request))
            response = json.loads(response_json) if response_json else {}
            symbols = response.get("result", []) if response else []
            if symbols:
                for symbol_data in symbols:
                    instrument = self._parse_instrument(symbol_data)
                    self.add(instrument)

        self._log.info(f"Loaded {self.count} instruments from MT5")

    async def load_ids_async(
        self,
        instrument_ids: list[InstrumentId],
        filters: dict | None = None,
    ) -> None:
        """Load specific instruments by ID."""
        if filters is None:
            filters = {}

        self._log.info(f"Loading {len(instrument_ids)} instruments...")

        for instrument_id in instrument_ids:
            await self.load_async(instrument_id, filters)

    async def load_async(
        self,
        instrument_id: InstrumentId,
        filters: dict | None = None,
    ) -> None:
        """Load a specific instrument by ID."""
        if filters is None:
            filters = {}

        symbol = instrument_id.symbol.value
        self._log.info(f"Loading instrument {symbol}...")

        # Get specific symbol info from MT5
        # API expects a list of symbols
        response_json = await self._client.symbol_info(json.dumps([symbol]))
        response = json.loads(response_json) if response_json else {}
        symbol_data = response.get("result", {}) if response else {}

        # Result might be a list containing the dict
        if isinstance(symbol_data, list) and symbol_data:
            symbol_data = symbol_data[0]

        if not symbol_data:
            raise ValueError(f"Symbol not found in MT5: {symbol}")

        if not isinstance(symbol_data, dict):
            raise TypeError(f"Expected dict for symbol data, got {type(symbol_data)}")

        instrument = self._parse_instrument(symbol_data)
        self.add(instrument)
        self._log.info(f"Loaded instrument: {instrument_id}")

    def _parse_instrument(self, symbol_data: dict) -> CurrencyPair:
        """
        Parse MT5 symbol data into a Nautilus instrument.

        Based on the actual MT5 REST API response format from symbols_get/symbol_info.

        Raises
        ------
        ValueError
            If required fields are missing or invalid.
        """
        symbol_name = symbol_data.get("name") or symbol_data.get("symbol")
        if not symbol_name:
            raise ValueError(f"Symbol data missing 'name' field: {symbol_data}")

        ts_now = self._clock.timestamp_ns()

        # Extract currency information - these are required
        base_currency_str = symbol_data.get("currency_base")
        quote_currency_str = symbol_data.get("currency_profit")

        # Fallback to parsing from symbol name if not provided
        if not base_currency_str:
            base_currency_str = symbol_name[:3]
        if not quote_currency_str:
            quote_currency_str = symbol_name[3:6]

        # Handle special cases (e.g., XAUUSD, XAGUSD)
        if base_currency_str in ("XAU", "XAG", "XPT", "XPD"):
            # Precious metals - use USD as quote if not specified
            if not quote_currency_str or len(quote_currency_str) < 3:
                quote_currency_str = "USD"

        # Create currency objects - will raise if invalid
        base_currency = Currency.from_str(base_currency_str)
        quote_currency = Currency.from_str(quote_currency_str)

        # Extract required fields
        digits = symbol_data.get("digits")
        if digits is None:
            raise ValueError(f"Symbol {symbol_name} missing 'digits' field")

        point = symbol_data.get("point")
        if point is None:
            raise ValueError(f"Symbol {symbol_name} missing 'point' field")

        # Size constraints
        volume_min = symbol_data.get("volume_low") or symbol_data.get("volume_min")
        volume_max = symbol_data.get("volume_high") or symbol_data.get("volume_max")
        volume_step = symbol_data.get("volume_step")

        if volume_min is None:
            raise ValueError(f"Symbol {symbol_name} missing volume_min field")
        if volume_max is None:
            raise ValueError(f"Symbol {symbol_name} missing volume_max field")
        if volume_step is None:
            raise ValueError(f"Symbol {symbol_name} missing volume_step field")

        # Calculate size precision from volume_step
        size_precision = (
            len(str(volume_step).split(".")[-1]) if "." in str(volume_step) else 0
        )

        instrument_id = InstrumentId(
            symbol=Symbol(symbol_name),
            venue=MT5_VENUE,
        )

        # Determine instrument type based on path or category
        path = symbol_data.get("path", "").lower()
        category = symbol_data.get("category", "").lower()

        is_forex = "forex" in path or "fx" in category

        if is_forex:
            return CurrencyPair(
                instrument_id=instrument_id,
                raw_symbol=Symbol(symbol_name),
                base_currency=base_currency,
                quote_currency=quote_currency,
                price_precision=digits,
                size_precision=size_precision,
                price_increment=Price.from_str(str(point)),
                size_increment=Quantity.from_str(str(volume_step)),
                lot_size=Quantity.from_str(str(volume_step)),
                max_quantity=Quantity.from_str(str(volume_max)),
                min_quantity=Quantity.from_str(str(volume_min)),
                max_notional=None,
                min_notional=None,
                max_price=None,
                min_price=None,
                margin_init=Decimal(str(symbol_data.get("margin_initial", 0))),
                margin_maint=Decimal(str(symbol_data.get("margin_maintenance", 0))),
                maker_fee=Decimal("0"),
                taker_fee=Decimal("0"),
                ts_event=ts_now,
                ts_init=ts_now,
                info=symbol_data,
            )
        else:
            # Default to CFD for everything else (Indices, Crypto, Stocks)
            # as most MT5 brokers are CFD based.
            return Cfd(
                instrument_id=instrument_id,
                raw_symbol=Symbol(symbol_name),
                base_currency=base_currency,
                quote_currency=quote_currency,
                price_precision=digits,
                size_precision=size_precision,
                price_increment=Price.from_str(str(point)),
                size_increment=Quantity.from_str(str(volume_step)),
                lot_size=Quantity.from_str(str(volume_step)),
                max_quantity=Quantity.from_str(str(volume_max)),
                min_quantity=Quantity.from_str(str(volume_min)),
                max_notional=None,
                min_notional=None,
                max_price=None,
                min_price=None,
                margin_init=Decimal(str(symbol_data.get("margin_initial", 0))),
                margin_maint=Decimal(str(symbol_data.get("margin_maintenance", 0))),
                maker_fee=Decimal("0"),
                taker_fee=Decimal("0"),
                ts_event=ts_now,
                ts_init=ts_now,
                info=symbol_data,
            )
