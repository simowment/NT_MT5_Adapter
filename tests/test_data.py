#!/usr/bin/env python3
"""
MT5 Adapter - Data & Connectivity Test
======================================

This script provides a comprehensive creation of the Data capabilities of the MT5 Adapter.
It tests:
1. Rust Bindings Availability
2. Middleware Connectivity
3. Real-time Market Data (Ticks, Symbol Info)
4. Historical Market Data (Bars, Pagination)
5. Account Data (Read-only)

Usage:
    python test_data.py
"""

import asyncio
import sys
import json
from datetime import datetime, timedelta, timezone

# Add project root to sys.path
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from nautilus_mt5 import Mt5HttpClient, Mt5Config
from nautilus_mt5.client import Mt5Client

BASE_URL = "http://localhost:5000"


async def test_connectivity():
    """Test basic connectivity to the middleware."""
    print("\n[1/5] Testing Connectivity...")

    try:
        config = Mt5Config()
        client = Mt5HttpClient(config, BASE_URL)

        # Version Check
        version_json = await client.version()
        version = json.loads(version_json)
        print(f"   ✅ Connected to MT5 Version: {version.get('result')}")

        # Terminal Info
        info_json = await client.terminal_info()
        info = json.loads(info_json)
        print(f"   ✅ Terminal Company: {info.get('result', {}).get('company')}")

        return client
    except Exception as e:
        print(f"   ❌ Connection Failed: {e}")
        return None


async def test_market_data_realtime(client: Mt5HttpClient):
    """Test real-time data fetching (ticks, symbols)."""
    print("\n[2/5] Testing Real-time Market Data...")

    symbol = "EURUSD"

    try:
        # Symbol Info
        info_json = await client.symbol_info(json.dumps([symbol]))
        info = json.loads(info_json).get("result", [])
        if not info:
            print(f"   ❌ Symbol {symbol} not found")
            return

        s = info[0]
        print(
            f"   ✅ Symbol Info ({symbol}): Digits={s.get('digits')}, Point={s.get('point')}"
        )

        # Current Tick
        tick_json = await client.symbol_info_tick(json.dumps([symbol]))
        tick = json.loads(tick_json).get("result", {})
        print(
            f"   ✅ Current Tick: Bid={tick.get('bid')} Ask={tick.get('ask')} Last={tick.get('last')}"
        )

    except Exception as e:
        print(f"   ❌ Real-time Data Failed: {e}")


async def test_market_data_historical():
    """Test historical data fetching using the high-level Mt5Client."""
    print("\n[3/5] Testing Historical Market Data (Mt5Client)...")

    client = Mt5Client(base_url=BASE_URL)
    symbol = "EURUSD"

    try:
        # 1. Fetch recent bars (Count)
        print("   -> Fetching last 10 M1 bars...")
        bars_count = await client.fetch_bars(symbol, "M1", count=10)
        print(f"   ✅ Received {len(bars_count)} bars")
        if bars_count:
            print(
                f"      Last: {bars_count[-1]['time']} | Close: {bars_count[-1]['close']}"
            )

        # 2. Fetch bars range (Pagination)
        print("   -> Fetching 2 days of M1 bars (Pagination Test)...")
        end_time = datetime.now(timezone.utc)
        start_time = end_time - timedelta(days=2)

        bars_range = await client.fetch_bars(
            symbol, "M1", start_time=start_time, end_time=end_time
        )
        print(f"   ✅ Received {len(bars_range)} bars over 48h")

        # 3. Fetch ticks
        print("   -> Fetching last 50 ticks...")
        ticks = await client.fetch_ticks(symbol, count=50)
        print(f"   ✅ Received {len(ticks)} ticks")

    except Exception as e:
        print(f"   ❌ Historical Data Failed: {e}")


async def test_account_data(client: Mt5HttpClient):
    """Test reading account information."""
    print("\n[4/5] Testing Account Data (Read-only)...")

    try:
        # Account Info
        acc_json = await client.account_info()
        acc = json.loads(acc_json).get("result", {})
        print(f"   ✅ Account: {acc.get('name')} | ID: {acc.get('login')}")
        print(f"      Balance: {acc.get('balance')} {acc.get('currency')}")
        print(f"      Equity: {acc.get('equity')}")

        # Orders
        orders_json = await client.orders_get()
        orders = json.loads(orders_json).get("result", [])
        print(f"   ✅ Active Orders: {len(orders)}")

        # Positions
        positions_json = await client.positions_get()
        positions = json.loads(positions_json).get("result", [])
        print(f"   ✅ Open Positions: {len(positions)}")

    except Exception as e:
        print(f"   ❌ Account Data Failed: {e}")


async def main():
    print("=" * 60)
    print("MT5 ADAPTER - DATA & CONNECTIVITY TEST")
    print("=" * 60)

    # 1. Low-level Client for Raw API
    ll_client = await test_connectivity()

    if ll_client:
        await test_market_data_realtime(ll_client)
        await test_account_data(ll_client)

        # 2. High-level Client for Bars/Ticks
        await test_market_data_historical()

    print("\n" + "=" * 60)
    print("TEST COMPLETE")
    print("=" * 60)


if __name__ == "__main__":
    asyncio.run(main())
