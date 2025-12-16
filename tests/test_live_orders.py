#!/usr/bin/env python3
"""
MT5 Adapter - Live Execution Test (Advanced)
============================================

This script tests the advanced ORDER PLACEMENT capabilities of the adapter:
1. Place PENDING Order (Buy Limit) with SL/TP
2. MODIFY Order (Change Price and SL/TP)
3. CANCEL Order
4. Place MARKET Order with SL/TP

⚠️ WARNING: This places REAL TRADES. Use on DEMO accounts only.
"""

import asyncio
import sys
import json
from pathlib import Path

# Add project root to sys.path
sys.path.insert(0, str(Path(__file__).parent.parent))

from nautilus_mt5 import Mt5HttpClient, Mt5Config
from nautilus_mt5.constants import Mt5TradeAction, Mt5OrderType, Mt5RetCode

BASE_URL = "http://localhost:5000"
SYMBOL = "EURUSD"
VOLUME = 0.01


async def test_advanced_execution():
    print("=" * 60)
    print("MT5 ADAPTER - ADVANCED EXECUTION TEST")
    print("=" * 60)
    print(f"Target: {SYMBOL} | Volume: {VOLUME}")
    print("⚠️  WARNING: Using DEMO account is highly recommended.\n")

    config = Mt5Config()
    client = Mt5HttpClient(config, BASE_URL)

    # 0. Connection Check
    try:
        await client.version()
    except Exception:
        print("❌ Cannot connect to middleware.")
        return

    # 1. Get Current Market Data
    print("[1] Fetching Market Data...")
    tick_json = await client.symbol_info_tick(json.dumps([SYMBOL]))
    tick = json.loads(tick_json).get("result", {})
    ask = tick.get("ask")
    bid = tick.get("bid")
    print(f"    Bid: {bid} | Ask: {ask}")

    if not ask:
        print("❌ Failed to get price.")
        return

    # Define prices for Limit Order (far below market to avoid fill)
    limit_price = round(ask - 0.0050, 5)  # 50 pips below
    sl_price = round(limit_price - 0.0020, 5)  # 20 pips SL
    tp_price = round(limit_price + 0.0050, 5)  # 50 pips TP

    print(f"    Plan: Buy Limit @ {limit_price} (SL: {sl_price}, TP: {tp_price})")

    # 2. Place Pending Order (Buy Limit)
    confirm = input("\n[2] Place Buy Limit with SL/TP? (y/N): ")
    if confirm.lower() != "y":
        return

    req_limit = {
        "action": Mt5TradeAction.PENDING,
        "symbol": SYMBOL,
        "volume": VOLUME,
        "type": Mt5OrderType.BUY_LIMIT,
        "price": limit_price,
        "sl": sl_price,
        "tp": tp_price,
        "magic": 1001,
        "comment": "NT_Limit_Test",
    }

    res_limit = await _send_order(client, req_limit)
    if not res_limit:
        return

    order_ticket = res_limit.get("order")
    print(f"    ✅ Pending Order Placed! Ticket: {order_ticket}")

    # 2.1 Verify Order Status
    print("    [2.1] Verifying Order Status...")
    orders_json = await client.orders_get()
    orders = json.loads(orders_json).get("result", [])

    pending_order = next((o for o in orders if o.get("ticket") == order_ticket), None)
    if not pending_order:
        print("    ❌ Order not found in Pending Orders! Was it filled immediately?")
        # Check positions
        pos_json = await client.positions_get()
        positions = json.loads(pos_json).get("result", [])
        filled_pos = next(
            (p for p in positions if p.get("ticket") == order_ticket), None
        )
        if filled_pos:
            print(
                "    ⚠️  Order was FILLED immediately (became a position). Cannot MODIFY as Pending."
            )
            return
        return
    else:
        print(
            f"    ✅ Order confirmed pending. Current Price: {pending_order.get('price_open')}"
        )

    # 3. Modify Order
    new_price = round(limit_price + 0.0010, 5)  # Move up 10 pips
    new_sl = round(new_price - 0.0020, 5)

    confirm = input(f"\n[3] Modify Order {order_ticket} -> Price: {new_price}? (y/N): ")
    if confirm.lower() == "y":
        req_mod = {
            "action": Mt5TradeAction.MODIFY,
            "order": order_ticket,
            "symbol": SYMBOL,  # Explicitly add symbol
            "volume": VOLUME,  # Required by some APIs even for modify
            "price": new_price,
            "sl": new_sl,
            "tp": tp_price,  # Keep TP same
            "type_filling": 0,  # FOK seems standard even for modify
        }
        res_mod = await _send_order(client, req_mod)
        if res_mod:
            print(f"    ✅ Order Modified! Result Code: {res_mod.get('retcode')}")

            # 3.1 Verify Actual State
            print("    [3.1] Verifying Actual Order State...")
            pending_order = await _get_order(client, order_ticket)
            if pending_order:
                actual_price = pending_order.get("price_open")
                print(
                    f"       Current Pending Price: {actual_price} (Target: {new_price})"
                )
                if abs(actual_price - new_price) < 0.00001:
                    print(f"       ✅ Modification Confirmed! Price updated correctly.")
                else:
                    print(f"       ❌ Price mismatch! It didn't update.")
            else:
                print("       ⚠️  Order not found in pending! checking positions...")
                # Check if it became a position
                pos = await _get_position(client, order_ticket)
                if pos:
                    print(
                        f"       ⚠️  Order executed! Open Price: {pos.get('price_open')}"
                    )
                else:
                    print("       ❌ Order disappeared!")

    # 4. Cancel Order
    confirm = input(f"\n[4] Cancel Order {order_ticket}? (y/N): ")
    if confirm.lower() == "y":
        req_cancel = {
            "action": Mt5TradeAction.REMOVE,
            "order": order_ticket,
            "symbol": SYMBOL,
            "volume": VOLUME,
        }
        res_cancel = await _send_order(client, req_cancel)
        if res_cancel:
            print("    ✅ Order Cancelled!")

    # 5. Market Order with SL/TP
    confirm = input(f"\n[5] Place MARKET Buy with SL/TP? (y/N): ")
    if confirm.lower() == "y":
        market_sl = round(ask - 0.0020, 5)
        market_tp = round(ask + 0.0020, 5)

        req_market = {
            "action": Mt5TradeAction.DEAL,
            "symbol": SYMBOL,
            "volume": VOLUME,
            "type": Mt5OrderType.BUY,
            "price": ask,
            "sl": market_sl,
            "tp": market_tp,
            "magic": 1002,
            "comment": "NT_Market_SLTP",
        }

        res_market = await _send_order(client, req_market)
        if res_market:
            print(f"    ✅ Market Order Filled! Deal: {res_market.get('deal')}")
            print(
                "    ⚠️  Remember to close this position manually or runs tests validation."
            )


async def _get_order(client, ticket):
    """Helper to find a pending order by ticket."""
    orders_json = await client.orders_get()
    orders = json.loads(orders_json).get("result", [])
    return next((o for o in orders if o.get("ticket") == ticket), None)


async def _get_position(client, ticket):
    """Helper to find a position by ticket (deal/order)."""
    pos_json = await client.positions_get()
    positions = json.loads(pos_json).get("result", [])
    return next((p for p in positions if p.get("ticket") == ticket), None)


async def _send_order(client, request):
    """Helper to send order and parse response."""
    try:
        res_json = await client.order_send(json.dumps(request))
        res = json.loads(res_json).get("result", {})

        if (
            res.get("retcode") == Mt5RetCode.DONE
            or res.get("retcode") == Mt5RetCode.PLACED
        ):
            return res
        else:
            print(f"    ❌ Failed: {res.get('comment')} (Code: {res.get('retcode')})")
            return None
    except Exception as e:
        print(f"    ❌ Error: {e}")
        return None


if __name__ == "__main__":
    asyncio.run(test_advanced_execution())
