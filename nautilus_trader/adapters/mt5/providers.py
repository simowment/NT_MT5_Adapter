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

from typing import Optional

from nautilus_trader.adapters.mt5.common import MT5_VENUE
from nautilus_trader.common.providers import InstrumentProvider
from nautilus_trader.core.correctness import PyCondition
from nautilus_trader.model.identifiers import InstrumentId


class Mt5InstrumentProviderConfig:
    """
    Placeholder configuration for Mt5InstrumentProvider.

    Extend with MT5-specific settings if needed.
    """
    ...


class Mt5InstrumentProvider(InstrumentProvider):
    """
    Provides a MetaTrader 5 instrument definition lookup and creation facility.

    Parameters
    ----------
    client : Mt5HttpClient
        The MetaTrader 5 HTTP client (Python bindings over the Rust MT5 adapter).
    config : Mt5InstrumentProviderConfig, optional
        The configuration for the provider instance.
    """

    def __init__(self, client, config=None):
        if config is None:
            config = Mt5InstrumentProviderConfig()
        super().__init__(config=config)

        PyCondition.not_none(client, "client")
        self._client = client
        self._config = config

    async def load_all_async(self, filters: Optional[dict] = None) -> None:
        if filters is None:
            filters = {}

        self._log.info("Loading instruments...")

        # Load all symbols from MT5
        symbols = await self._client.get_symbols()

        for symbol_data in symbols:
            # Convert MT5 symbol data to Nautilus instrument
            instrument = self._create_instrument_from_mt5_symbol(symbol_data)
            self.add(instrument)

        self._log.info(f"Loaded {len(self.get_all())} instruments")

    async def load_ids_async(self, instrument_ids: list[InstrumentId], filters: Optional[dict] = None) -> None:
        if filters is None:
            filters = {}

        self._log.info(f"Loading instruments {instrument_ids}...")

        for instrument_id in instrument_ids:
            await self.load_async(instrument_id, filters)

    async def load_async(self, instrument_id: InstrumentId, filters: Optional[dict] = None) -> None:
        if filters is None:
            filters = {}

        self._log.info(f"Loading instrument {instrument_id}...")

        # Get specific symbol from MT5
        symbols = await self._client.get_symbols()
        symbol_str = instrument_id.symbol.value
        
        # Find the specific symbol
        for symbol_data in symbols:
            if symbol_data.symbol == symbol_str:
                instrument = self._create_instrument_from_mt5_symbol(symbol_data)
                self.add(instrument)
                break

    def _create_instrument_from_mt5_symbol(self, symbol_data):
        """
        Create a Nautilus instrument from MT5 symbol data.

        This is intentionally simplified and assumes FX-style symbols like 'EURUSD'.
        Extend this to map full MT5 symbol metadata into the correct Nautilus
        instrument type (FX, CFD, futures, etc.).
        """
        from nautilus_trader.model.instruments import CurrencyPair
        from nautilus_trader.model.objects import Currency
        from nautilus_trader.model.enums import AssetClass
        from nautilus_trader.model.identifiers import InstrumentId, Symbol, Venue

        # Create currency pair instrument from MT5 symbol data
        # Note: This is a simplified implementation - in reality, you'd need to handle
        # different instrument types (FX, CFD, futures, etc.)
        
        base_currency_code = symbol_data.symbol[:3]  # First 3 chars as base currency
        quote_currency_code = symbol_data.symbol[3:]  # Remaining chars as quote currency
        
        base_currency = Currency.from_str(base_currency_code)
        quote_currency = Currency.from_str(quote_currency_code)
        
        instrument_id = InstrumentId(
            symbol=Symbol(symbol_data.symbol),
            venue=MT5_VENUE,
        )

        # Calculate price accuracy based on digits
        price_accuracy = symbol_data.digits
        
        # Convert point size to tick value
        tick_size = symbol_data.point_size

        instrument = CurrencyPair(
            instrument_id=instrument_id,
            raw_symbol=Symbol(symbol_data.symbol),
            base_currency=base_currency,
            quote_currency=quote_currency,
            price_precision=price_accuracy,
            size_precision=2,  # Using standard size precision
            price_increment=tick_size,
            size_increment=0.01,  # Standard size increment for FX
            multiplier=symbol_data.contract_size,
            lot_size=None,
            max_quantity=None,
            min_quantity=None,
            max_notional=None,
            min_notional=None,
            max_price=None,
            min_price=None,
            margin_init=symbol_data.margin_initial,
            margin_maint=symbol_data.margin_maintenance,
            maker_fee=symbol_data.maker_fee,
            taker_fee=symbol_data.taker_fee,
            ts_event=0,  # Will be set to current time
            ts_init=0,   # Will be set to current time
            info={},     # Additional instrument info from MT5
        )
        
        return instrument
