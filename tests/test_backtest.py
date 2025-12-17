#!/usr/bin/env python3
"""
MT5 Adapter Backtest Test

This script fetches data from MT5 and runs a NautilusTrader backtest using the built-in EMACross strategy.
It uses the proper object-oriented adapter components.
"""

import asyncio
import sys
from decimal import Decimal
from pathlib import Path

# Add project root to sys.path
sys.path.insert(0, str(Path(__file__).parent.parent))

from nautilus_trader.backtest.engine import BacktestEngine, BacktestEngineConfig  # type: ignore
from nautilus_trader.common.component import LiveClock
from nautilus_trader.config import LoggingConfig, RiskEngineConfig
from datetime import datetime, timedelta, timezone
from nautilus_trader.model.currencies import USD
from nautilus_trader.model.data import BarType
from nautilus_trader.model.enums import AccountType, OmsType
from nautilus_trader.model.identifiers import InstrumentId, Symbol
from nautilus_trader.model.objects import Money

from nautilus_mt5 import Mt5Config, Mt5HttpClient
from nautilus_mt5.client import Mt5Client
from nautilus_mt5.providers import Mt5InstrumentProvider
from nautilus_mt5.config import Mt5InstrumentProviderConfig
from nautilus_mt5.common import MT5_VENUE

from nautilus_trader.examples.strategies.ema_cross import EMACross, EMACrossConfig


BASE_URL = "http://localhost:5000"


async def fetch_and_run():
    print("MT5 Adapter - Backtest Refactored (Clean + Mt5Client)")
    print("=" * 60)

    # 1. Setup Client and Provider
    print("1. Initializing Client and Provider...")

    # We need the low-level client for the provider (as provider uses it internally)
    # But we can share the config
    client_config = Mt5Config(base_url=BASE_URL)
    http_client = Mt5HttpClient(config=client_config, base_url=BASE_URL)

    # Provider for Instrument Loading
    clock = LiveClock()
    provider_config = Mt5InstrumentProviderConfig(base_url=BASE_URL)

    try:
        # Pass args positionally
        provider = Mt5InstrumentProvider(http_client, clock, provider_config)
    except TypeError as e:
        print(f"DEBUG: Provider instantiation error: {e}")
        # Fallback if needed
        return

    # High-level client for data fetching
    # Use the same base URL
    mt5_client = Mt5Client(base_url=BASE_URL)

    print("   Clients initialized.")

    # 2. Load Instrument
    symbol_str = "EURUSD"
    print(f"2. Loading Instrument {symbol_str}...")

    instrument_id = InstrumentId(Symbol(symbol_str), MT5_VENUE)

    try:
        await provider.load_async(instrument_id)
        instrument = provider.find(instrument_id)
        if not instrument:
            print(f"FAILED to load instrument {symbol_str}")
            return
        print(f"Loaded instrument: {instrument}")
    except Exception as e:
        print(f"Error loading instrument: {e}")
        import traceback

        traceback.print_exc()
        return

    # 3. Fetch Data using request_bars (returns Nautilus Bar objects directly)
    print("3. Fetching Historical Data...")

    # Define bar type first
    bar_type = BarType.from_str(f"{instrument_id}-1-MINUTE-MID-EXTERNAL")

    try:
        print("   Requesting bars via Mt5Client.request_bars (Nautilus pattern)...")

        end_time = datetime.now(timezone.utc)
        start_time = end_time - timedelta(days=30)  # 30 days of data

        # Use request_bars which returns list[Bar] directly
        nautilus_bars = await mt5_client.request_bars(
            bar_type=bar_type,
            instrument=instrument,
            start=start_time,
            end=end_time,
        )

        if not nautilus_bars:
            print("No bars received.")
            return

        print(f"Received {len(nautilus_bars)} Nautilus Bar objects")
        if len(nautilus_bars) > 0:
            print(f"First bar: {nautilus_bars[0]}")
            print(f"Last bar: {nautilus_bars[-1]}")

    except Exception as e:
        print(f"Error fetching bars: {e}")
        import traceback

        traceback.print_exc()
        return

    print(f"Created {len(nautilus_bars)} Bar objects.")

    # 5. Run Backtest
    print("5. Running NautilusTrader Backtest...")

    engine_config = BacktestEngineConfig(
        trader_id="BACKTESTER-001",
        logging=LoggingConfig(log_level="INFO"),
        risk_engine=RiskEngineConfig(bypass=True),
    )
    engine = BacktestEngine(config=engine_config)

    engine.add_venue(
        venue=MT5_VENUE,
        oms_type=OmsType.HEDGING,
        account_type=AccountType.MARGIN,
        base_currency=USD,
        starting_balances=[Money(100_000, USD)],
    )

    engine.add_instrument(instrument)
    engine.add_data(nautilus_bars)  # type: ignore[arg-type]  # list[Bar] is safely assignable to list[Data] here

    config = EMACrossConfig(
        instrument_id=instrument.id,
        bar_type=bar_type,
        fast_ema_period=10,
        slow_ema_period=20,
        trade_size=Decimal("10000"),
    )
    strategy = EMACross(config=config)
    engine.add_strategy(strategy)

    engine.run()

    print("\n" + "=" * 30)
    print("   ACCOUNT REPORT")
    print("=" * 30)
    print(engine.trader.generate_account_report(MT5_VENUE))

    print("\n" + "=" * 30)
    print("   FILLS")
    print("=" * 30)
    print(engine.trader.generate_order_fills_report())

    print(engine.portfolio.analyzer.get_performance_stats_general())  # type: ignore


if __name__ == "__main__":
    asyncio.run(fetch_and_run())
