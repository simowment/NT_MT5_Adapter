#!/usr/bin/env python3
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
Test script for the MetaTrader 5 adapter.

This script demonstrates how to use the MT5 adapter with NautilusTrader.
"""

import asyncio
import sys
from decimal import Decimal

try:
    # Import NautilusTrader components
    from nautilus_trader.backtest.data_loaders import CSVBarDataLoader
    from nautilus_trader.backtest.data_loaders import CSVTickDataLoader
    from nautilus_trader.backtest.engine import BacktestEngine
    from nautilus_trader.backtest.engine import BacktestEngineConfig
    from nautilus_trader.config import LiveDataEngineConfig
    from nautilus_trader.config import LoggingConfig
    from nautilus_trader.config import RiskEngineConfig
    from nautilus_trader.examples.strategies.ema_cross import EMACross
    from nautilus_trader.examples.strategies.ema_cross import EMACrossConfig
    from nautilus_trader.model.data import BarType
    from nautilus_trader.model.identifiers import InstrumentId
    from nautilus_trader.model.identifiers import Symbol
    from nautilus_trader.model.identifiers import Venue
    from nautilus_trader.model.objects import Price
    from nautilus_trader.model.objects import Quantity
    from nautilus_trader.model.objects import Money
    from nautilus_trader.model.currency import USD
    from nautilus_trader.test_kit.providers import TestInstrumentProvider
    
    # Import your MT5 adapter components
    from nautilus_trader.adapters.metatrader5 import Mt5DataClient
    from nautilus_trader.adapters.metatrader5 import Mt5ExecutionClient
    from nautilus_trader.adapters.metatrader5 import Mt5InstrumentProvider
    from nautilus_trader.adapters.metatrader5.config import Mt5DataClientConfig
    from nautilus_trader.adapters.metatrader5.config import Mt5ExecClientConfig

    print("Successfully imported MT5 adapter components")
    
except ImportError as e:
    print(f"Import error: {e}")
    print("Make sure your MT5 adapter is properly installed and the Rust extension is compiled.")
    sys.exit(1)


async def test_mt5_adapter():
    """
    Test the MT5 adapter functionality.
    """
    print("Testing MT5 adapter...")
    
    # Configuration for MT5
    mt5_config = Mt5DataClientConfig(
        username="your_username",
        password="your_password", 
        server="your_server",
        base_url="http://localhost:8000",
        ws_url="ws://localhost:8000"
    )
    
    # Create backtest engine
    config = BacktestEngineConfig(
        log_level="INFO",
        bypass_logging=True,
        run_analysis=False,
    )
    
    engine = BacktestEngine(config=config)
    
    try:
        # Add a venue (for testing purposes)
        engine.add_venue(
            venue=Venue("SIM"),
            oms_type="HEDGING",
            account_type="MARGIN",
            base_currency=None,
            starting_balances=[Money(1_000_000, USD)],
        )
        
        # Load instruments (for testing)
        instrument = TestInstrumentProvider.default()
        engine.add_instrument(instrument)
        
        # Add your strategy
        strategy_config = EMACrossConfig(
            instrument_id=instrument.id.value,
            bar_type=f"{instrument.id.value}-1-MINUTE-LAST-INTERNAL",
            trade_size=Decimal("1000"),
        )
        strategy = EMACross(config=strategy_config)
        engine.add_strategy(strategy)
        
        print("MT5 adapter test setup completed successfully")
        print("Note: This test only verifies import and basic setup.")
        print("For full functionality, you need a running MT5 bridge service.")
        
    finally:
        await engine.dispose()


def test_rust_bindings():
    """
    Test the Rust bindings for MT5 adapter.
    """
    print("Testing Rust bindings...")
    
    try:
        # Try to import the Rust extension
        import nautilus_adapters_mt5
        
        print(f"Successfully imported Rust extension: {nautilus_adapters_mt5.__version__}")
        
        # List available classes in the Rust extension
        available_classes = [attr for attr in dir(nautilus_adapters_mt5) if not attr.startswith('_')]
        print(f"Available classes in Rust extension: {available_classes}")
        
        return True
        
    except ImportError as e:
        print(f"Could not import Rust bindings: {e}")
        print("This indicates the Rust extension has not been compiled yet.")
        return False


def compile_rust_extension():
    """
    Compile the Rust extension with Python bindings.
    """
    print("To compile the Rust extension with Python bindings, run:")
    print("cargo build --release --features python-bindings")
    print("or for development:")
    print("maturin develop --features python-bindings")


def main():
    """
    Main entry point for testing the MT5 adapter.
    """
    print("=" * 60)
    print("MT5 Adapter Test Script")
    print("=" * 60)
    
    # Test Rust bindings first
    bindings_ok = test_rust_bindings()
    
    if not bindings_ok:
        print("\nRust bindings not available. Compiling instructions:")
        compile_rust_extension()
        print("\nAfter compilation, run this script again.")
        return
    
    print("\nStarting async test...")
    try:
        asyncio.run(test_mt5_adapter())
    except Exception as e:
        print(f"Error during async test: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
