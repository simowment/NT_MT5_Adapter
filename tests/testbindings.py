#!/usr/bin/env python3
"""
MT5 Adapter Live Test Script

This script tests the MT5 adapter with your middleware.
Make sure your MT5 middleware is running at base_url before executing.

Usage:
    1. Start your MT5 middleware server
    2. Run: python testbindings.py
"""

import asyncio
import sys

# =============================================================================
# STEP 1: Test Rust Bindings
# =============================================================================


def test_rust_bindings():
    """Test if Rust bindings are compiled and available."""
    print("=" * 60)
    print("STEP 1: Testing Rust Bindings")
    print("=" * 60)

    try:
        import nautilus_mt5

        print("✅ Rust bindings loaded successfully!")
        print(f"   Version: {nautilus_mt5.__version__}")

        # List available classes
        available = [a for a in dir(nautilus_mt5) if not a.startswith("_")]
        print(f"   Available: {', '.join(available)}")
        return True

    except ImportError as e:
        print(f"❌ Rust bindings NOT available: {e}")
        print("\n   To compile, run:")
        print("   maturin develop --features python-bindings")
        print("   or:")
        print("   pip install maturin && maturin develop")
        return False


# =============================================================================
# STEP 2: Test HTTP Client Connection
# =============================================================================


async def test_http_client(base_url: str):
    """Test the HTTP client connection to middleware."""
    print("\n" + "=" * 60)
    print("STEP 2: Testing HTTP Client Connection")
    print("=" * 60)

    try:
        from nautilus_mt5 import Mt5HttpClient, Mt5Config

        # Create config and client
        config = Mt5Config()
        client = Mt5HttpClient(config, base_url)

        # Test account info first (most reliable endpoint)
        print("Testing /api/account_info...")
        account = await client.account_info()
        print(
            f"✅ Account: {account[:200]}..."
            if len(account) > 200
            else f"✅ Account: {account}"
        )

        # Test version endpoint
        print("Testing /api/version...")
        result = await client.version()
        print(f"✅ Version: {result}")

        # Test symbols
        print("Testing /api/symbols_total...")
        total = await client.symbols_total()
        print(f"✅ Total symbols: {total}")

        return True, client

    except Exception as e:
        print(f"❌ HTTP client test failed: {e}")
        import traceback

        traceback.print_exc()
        print("\n   Make sure your MT5 middleware is running at base_url")
        return False, None


# =============================================================================
# STEP 3: Test Data Retrieval
# =============================================================================


async def test_data_retrieval(client):
    """Test fetching market data."""
    print("\n" + "=" * 60)
    print("STEP 3: Testing Data Retrieval")
    print("=" * 60)

    import json

    try:
        # Get symbol info for EURUSD (API expects array format: ["EURUSD"])
        print("Fetching EURUSD symbol info...")
        symbol_info = await client.symbol_info(json.dumps(["EURUSD"]))
        print(
            f"✅ EURUSD info: {symbol_info[:200]}..."
            if len(symbol_info) > 200
            else f"✅ EURUSD info: {symbol_info}"
        )

        # Get current tick (API expects array format: ["EURUSD"])
        print("Fetching EURUSD tick...")
        tick = await client.symbol_info_tick(json.dumps(["EURUSD"]))
        print(f"✅ EURUSD tick: {tick}")

        # Get positions
        print("Fetching positions...")
        positions = await client.positions_get()
        print(f"✅ Positions: {positions}")

        # Get orders
        print("Fetching orders...")
        orders = await client.orders_get()
        print(f"✅ Orders: {orders}")

        return True

    except Exception as e:
        print(f"❌ Data retrieval failed: {e}")
        import traceback

        traceback.print_exc()
        return False


# =============================================================================
# MAIN
# =============================================================================


async def main(base_url: str):
    print("\n🚀 MT5 Adapter Live Test")
    print("Make sure your MT5 middleware is running at base_url\n")

    # Step 1: Test Rust bindings
    if not test_rust_bindings():
        print("\n⚠️  Please compile Rust bindings first!")
        sys.exit(1)

    # Step 2: Test HTTP client
    success, client = await test_http_client(base_url)
    if not success:
        print("\n⚠️  Cannot connect to middleware!")
        sys.exit(1)

    # Step 3: Test data retrieval
    if not await test_data_retrieval(client):
        print("\n⚠️  Data retrieval failed!")
        sys.exit(1)

    print("\n" + "=" * 60)
    print("✅ ALL TESTS PASSED!")
    print("=" * 60)
    print("\nYour MT5 adapter is working correctly.")
    print("Next step: Run test_live_strategy.py to test a live strategy.")


if __name__ == "__main__":
    base_url = "http://localhost:5000"
    asyncio.run(main(base_url))
