#!/usr/bin/env python3
"""
MT5 Adapter Backtest Test

This script demonstrates how to:
1. Fetch historical data from MT5 via the middleware
2. Convert it to a format suitable for backtesting
3. Run a simple backtest with that data

NOTE: NautilusTrader backtesting uses pre-loaded historical data, not live connections.
This script fetches data from MT5 and saves it, then you can use it for backtesting.

Usage:
    1. Start your MT5 middleware server
    2. Run: python Adapter_Backtest_Test.py
"""

import asyncio
import json
from datetime import datetime, timedelta
from pathlib import Path


print("\n" + "=" * 60)
print("STEP 1: Testing Rust Bindings")
print("=" * 60)

try:
    import nautilus_mt5

    version = getattr(nautilus_mt5, "__version__", "Unknown")
    print(f"✅ Rust bindings loaded: v{version}")

    available = [attr for attr in dir(nautilus_mt5) if not attr.startswith("_")]
    print(f"   Available: {', '.join(available)}")

except ImportError as e:
    print(f"❌ Could not import Rust bindings: {e}")
    print("\n   To compile, run:")
    print("   maturin develop --features python-bindings")


BASE_URL = "http://localhost:5000"


async def fetch_mt5_data(
    symbol: str = "EURUSD", timeframe: str = "M1", bars: int = 1000
):
    """
    Fetch historical bar data from MT5 middleware.

    Args:
        symbol: The symbol to fetch (e.g., "EURUSD")
        timeframe: MT5 timeframe (M1, M5, M15, M30, H1, H4, D1, W1, MN1)
        bars: Number of bars to fetch

    Returns:
        List of bar dictionaries or None on error
    """
    print("\n" + "=" * 60)
    print("STEP 2: Fetching Historical Data from MT5")
    print("=" * 60)

    from nautilus_mt5 import Mt5HttpClient, Mt5Config

    # Create client
    config = Mt5Config()
    client = Mt5HttpClient(config, BASE_URL)

    # Import and use Mt5Timeframe constants
    from nautilus_mt5_adapter.constants import Mt5Timeframe

    # Map timeframe string to MT5 timeframe value
    timeframe_map = {
        "M1": Mt5Timeframe.M1,
        "M2": Mt5Timeframe.M2,
        "M3": Mt5Timeframe.M3,
        "M4": Mt5Timeframe.M4,
        "M5": Mt5Timeframe.M5,
        "M6": Mt5Timeframe.M6,
        "M10": Mt5Timeframe.M10,
        "M12": Mt5Timeframe.M12,
        "M15": Mt5Timeframe.M15,
        "M20": Mt5Timeframe.M20,
        "M30": Mt5Timeframe.M30,
        "H1": Mt5Timeframe.H1,
        "H2": Mt5Timeframe.H2,
        "H3": Mt5Timeframe.H3,
        "H4": Mt5Timeframe.H4,
        "H6": Mt5Timeframe.H6,
        "H8": Mt5Timeframe.H8,
        "H12": Mt5Timeframe.H12,
        "D1": Mt5Timeframe.D1,
        "W1": Mt5Timeframe.W1,
        "MN1": Mt5Timeframe.MN1,
    }

    tf_value = timeframe_map.get(timeframe, Mt5Timeframe.M1)

    print(f"   Symbol: {symbol}")
    print(f"   Timeframe: {timeframe} (MT5 value: {tf_value})")
    print(f"   Bars requested: {bars}")

    try:
        # Fetch bars using copy_rates_from
        # API format: [symbol, timeframe, from_timestamp, count]
        # Using current time minus some buffer to get recent bars
        from_time = int((datetime.now() - timedelta(days=1)).timestamp())
        request = [symbol, tf_value, from_time, bars]

        print("\n   Fetching data...")
        result_json = await client.copy_rates_from(json.dumps(request))
        result = json.loads(result_json)

        if "error" in result:
            print(f"❌ Error fetching data: {result['error']}")
            return None

        if "result" not in result:
            print(f"❌ Unexpected response format: {result}")
            return None

        bars_raw = result["result"]

        if bars_raw is None:
            print("❌ No data returned (result is null)")
            return None

        print(f"✅ Received {len(bars_raw)} bars")

        # Convert array format to dict format
        # MT5 format: [time, open, high, low, close, tick_volume, spread, real_volume]
        bars_data = []
        for bar in bars_raw:
            bars_data.append(
                {
                    "time": bar[0],
                    "open": bar[1],
                    "high": bar[2],
                    "low": bar[3],
                    "close": bar[4],
                    "tick_volume": bar[5],
                    "spread": bar[6],
                    "real_volume": bar[7],
                }
            )

        if bars_data:
            # Show sample of data
            print("\n   Sample (first 3 bars):")
            for i, bar in enumerate(bars_data[:3]):
                dt = datetime.fromtimestamp(bar["time"])
                print(
                    f"   {i + 1}. {dt} | O:{bar['open']:.5f} H:{bar['high']:.5f} L:{bar['low']:.5f} C:{bar['close']:.5f} V:{bar['tick_volume']}"
                )

        return bars_data

    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback

        traceback.print_exc()
        return None


def save_bars_to_csv(
    bars_data: list, symbol: str, timeframe: str, output_dir: str = "data"
):
    """
    Save bar data to CSV format for backtesting.

    Args:
        bars_data: List of bar dictionaries from MT5
        symbol: Symbol name
        timeframe: Timeframe string
        output_dir: Directory to save CSV files
    """
    print("\n" + "=" * 60)
    print("STEP 3: Saving Data for Backtesting")
    print("=" * 60)

    if not bars_data:
        print("❌ No data to save")
        return None

    # Create output directory
    output_path = Path(output_dir)
    output_path.mkdir(exist_ok=True)

    # Generate filename
    filename = f"{symbol}_{timeframe}_bars.csv"
    filepath = output_path / filename

    # Write CSV
    with open(filepath, "w") as f:
        # Header (NautilusTrader format)
        f.write("timestamp,open,high,low,close,volume\n")

        for bar in bars_data:
            # Convert MT5 timestamp to ISO format
            dt = datetime.fromtimestamp(bar["time"])
            timestamp = dt.strftime("%Y-%m-%dT%H:%M:%S.000Z")

            f.write(
                f"{timestamp},{bar['open']},{bar['high']},{bar['low']},{bar['close']},{bar['tick_volume']}\n"
            )

    print(f"✅ Saved {len(bars_data)} bars to: {filepath}")
    print(f"   File size: {filepath.stat().st_size / 1024:.1f} KB")

    return filepath


async def fetch_tick_data(symbol: str = "EURUSD", count: int = 1000):
    """
    Fetch recent tick data from MT5.

    Args:
        symbol: The symbol to fetch
        count: Number of ticks to fetch

    Returns:
        List of tick dictionaries or None on error
    """
    print("\n" + "=" * 60)
    print("STEP 4: Fetching Tick Data from MT5")
    print("=" * 60)

    from nautilus_mt5 import Mt5HttpClient, Mt5Config

    config = Mt5Config()
    client = Mt5HttpClient(config, BASE_URL)

    print(f"   Symbol: {symbol}")
    print(f"   Ticks requested: {count}")

    try:
        # Use copy_ticks_from with current time
        # API format: [symbol, from_timestamp, count, flags]
        # flags: 1 = COPY_TICKS_INFO (bid/ask), 2 = COPY_TICKS_TRADE (last), 3 = COPY_TICKS_ALL
        now = int(datetime.now().timestamp())
        from_time = now - (count * 60)  # Rough estimate for tick frequency

        request = [symbol, from_time, count, 1]  # 1 = COPY_TICKS_INFO (bid/ask)

        print("\n   Fetching ticks...")
        result_json = await client.copy_ticks_from(json.dumps(request))
        result = json.loads(result_json)

        if "error" in result:
            print(f"❌ Error fetching ticks: {result['error']}")
            return None

        if "result" not in result:
            print(f"❌ Unexpected response format: {result}")
            return None

        ticks_raw = result["result"]

        if ticks_raw is None:
            print("❌ No tick data returned (result is null)")
            return None

        print(f"✅ Received {len(ticks_raw)} ticks")

        # Convert array format to dict format
        # MT5 format: [time, bid, ask, last, volume, time_msc, flags, volume_real]
        ticks_data = []
        for tick in ticks_raw:
            ticks_data.append(
                {
                    "time": tick[0],
                    "bid": tick[1],
                    "ask": tick[2],
                    "last": tick[3],
                    "volume": tick[4],
                    "time_msc": tick[5],
                    "flags": tick[6],
                    "volume_real": tick[7],
                }
            )

        if ticks_data:
            print("\n   Sample (first 3 ticks):")
            for i, tick in enumerate(ticks_data[:3]):
                dt = datetime.fromtimestamp(tick["time"])
                print(f"   {i + 1}. {dt} | Bid:{tick['bid']:.5f} Ask:{tick['ask']:.5f}")

        return ticks_data

    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback

        traceback.print_exc()
        return None


def print_backtest_instructions(csv_path: Path):
    """Print instructions for using the data with NautilusTrader backtesting."""
    print("\n" + "=" * 60)
    print("STEP 5: Backtesting Instructions")
    print("=" * 60)

    print("""
To use this data with NautilusTrader backtesting, you'll need:

1. Install nautilus_trader:
   pip install nautilus_trader

2. Create a backtest script that loads this CSV data:

```python
from decimal import Decimal
from nautilus_trader.backtest.engine import BacktestEngine, BacktestEngineConfig
from nautilus_trader.model.currencies import USD
from nautilus_trader.model.identifiers import Venue
from nautilus_trader.model.objects import Money
from nautilus_trader.persistence.loaders import CSVBarDataLoader
from nautilus_trader.test_kit.providers import TestInstrumentProvider

# Create engine
engine = BacktestEngine(config=BacktestEngineConfig(
    trader_id="BACKTESTER-001",
))

# Add venue (MT5)
MT5_VENUE = Venue("MT5")
engine.add_venue(
    venue=MT5_VENUE,
    oms_type="HEDGING",
    account_type="MARGIN",
    base_currency=USD,
    starting_balances=[Money(100_000, USD)],
)

# Load your CSV data
# (You'll need to create an instrument and data loader)

# Add strategy and run
# engine.add_strategy(your_strategy)
# engine.run()
```

NOTE: The full NautilusTrader backtesting requires:
- Proper instrument definitions
- Data loaders for your specific format
- Strategy implementation

For more info, see: https://nautilustrader.io/docs/latest/user_guide/backtest.html
""")


async def main():
    """Main entry point."""
    print("=" * 60)
    print("MT5 Adapter - Backtest Data Fetcher")
    print("=" * 60)
    print("\nThis script fetches historical data from MT5 for backtesting.")

    # Fetch bar data
    symbol = "EURUSD"
    timeframe = "M1"  # 1-minute bars
    bars_count = 500  # Last 500 bars

    bars_data = await fetch_mt5_data(symbol, timeframe, bars_count)

    if not bars_data:
        print("\n❌ Failed to fetch bar data. Is your middleware running?")
        return

    # Save to CSV
    csv_path = save_bars_to_csv(bars_data, symbol, timeframe)

    # Fetch tick data (optional)
    tick_data = await fetch_tick_data(symbol, count=100)
    if tick_data:
        print("Tick data fetched successfully", tick_data)

    # Print instructions
    if csv_path:
        print_backtest_instructions(csv_path)

    print("\n" + "=" * 60)
    print("✅ Data fetching complete!")
    print("=" * 60)
    print("\nYour historical data is saved in the 'data/' directory.")
    print("You can now use this data for backtesting with NautilusTrader.")


if __name__ == "__main__":
    asyncio.run(main())
