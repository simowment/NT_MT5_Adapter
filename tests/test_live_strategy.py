#!/usr/bin/env python3
"""
MT5 Adapter Live Strategy Test

Tests the Rust HTTP client with order placement functionality.
This tests the adapter's ability to send orders to MT5.

WARNING: This will place REAL trades if connected to a real account!
         Use a demo account for testing.

Usage:
    1. Start your MT5 middleware server
    2. Run: python test_live_strategy.py
"""

import asyncio
import json


async def test_order_placement(base_url: str):
    """Test order placement through the Rust HTTP client."""
    print("=" * 60)
    print("MT5 Order Placement Test")
    print("=" * 60)
    print("\n⚠️  WARNING: This will place REAL trades!")
    print("    Make sure you're using a DEMO account!\n")

    from nautilus_mt5 import Mt5HttpClient, Mt5Config

    # Create client
    config = Mt5Config()
    client = Mt5HttpClient(config, base_url)

    # Step 1: Get account info
    print("\n📊 Step 1: Getting account info...")
    account_json = await client.account_info()
    account = json.loads(account_json)
    if "result" in account:
        acc = account["result"]
        print(f"   Balance: ${acc.get('balance', 'N/A')}")
        print(f"   Equity: ${acc.get('equity', 'N/A')}")
        print(f"   Leverage: 1:{acc.get('leverage', 'N/A')}")
    else:
        print(f"   Error: {account}")
        return

    # Step 2: Get EURUSD tick
    print("\n📈 Step 2: Getting EURUSD current price...")
    tick_json = await client.symbol_info_tick(json.dumps(["EURUSD"]))
    tick = json.loads(tick_json)
    if "result" in tick:
        t = tick["result"]
        print(f"   Bid: {t.get('bid', 'N/A')}")
        print(f"   Ask: {t.get('ask', 'N/A')}")
        current_price = t.get("ask", 0)
    else:
        print(f"   Error: {tick}")
        return

    # Step 3: Check current positions
    print("\n📋 Step 3: Checking current positions...")
    positions_json = await client.positions_get()
    positions = json.loads(positions_json)
    if "result" in positions:
        pos_list = positions["result"]
        print(f"   Open positions: {len(pos_list)}")
        for p in pos_list[:3]:  # Show first 3
            print(
                f"   - {p.get('symbol', 'N/A')}: {p.get('volume', 'N/A')} lots @ {p.get('price_open', 'N/A')}"
            )

    # Step 4: Place a test order
    print("\n🛒 Step 4: Placing test BUY order...")
    print("   Symbol: EURUSD")
    print("   Volume: 0.01 lots")
    print("   Type: Market BUY")

    # MT5 order_send request format
    order_request = {
        "symbol": "EURUSD",
        "volume": 0.01,
        "type": 0,  # ORDER_TYPE_BUY = 0
        "action": 1,  # TRADE_ACTION_DEAL = 1
        "price": current_price,
        "deviation": 20,
        "magic": 123456,
        "comment": "Nautilus MT5 Test",
        "type_filling": 0,  # ORDER_FILLING_FOK = 0
        "type_time": 0,  # ORDER_TIME_GTC = 0
    }

    confirm = input("\n   ⚠️  Place order? (yes/no): ")
    if confirm.lower() != "yes":
        print("   Order cancelled by user.")
        return

    try:
        result_json = await client.order_send(json.dumps(order_request))
        result = json.loads(result_json)
        print(f"\n   Order result: {json.dumps(result, indent=2)}")

        if "result" in result:
            r = result["result"]
            if r.get("retcode") == 10009:  # TRADE_RETCODE_DONE
                print("\n   ✅ Order placed successfully!")
                print(f"   Order ID: {r.get('order', 'N/A')}")
                print(f"   Deal ID: {r.get('deal', 'N/A')}")
                print(f"   Volume: {r.get('volume', 'N/A')}")
                print(f"   Price: {r.get('price', 'N/A')}")
            else:
                print(f"\n   ❌ Order failed: {r.get('comment', 'Unknown error')}")
                print(f"   Return code: {r.get('retcode', 'N/A')}")
        else:
            print(f"\n   ❌ Error: {result}")

    except Exception as e:
        print(f"\n   ❌ Order failed: {e}")

    # Step 5: Check positions again
    print("\n📋 Step 5: Checking positions after order...")
    await asyncio.sleep(1)  # Wait for order to process
    positions_json = await client.positions_get()
    positions = json.loads(positions_json)
    if "result" in positions:
        pos_list = positions["result"]
        print(f"   Open positions: {len(pos_list)}")
        for p in pos_list[:5]:
            print(
                f"   - {p.get('symbol', 'N/A')}: {p.get('volume', 'N/A')} lots @ {p.get('price_open', 'N/A')}"
            )

    print("\n" + "=" * 60)
    print("✅ Test complete!")
    print("=" * 60)


def main(base_url: str):
    asyncio.run(test_order_placement(base_url))


if __name__ == "__main__":
    base_url = "http://localhost:5000"
    main(base_url)
