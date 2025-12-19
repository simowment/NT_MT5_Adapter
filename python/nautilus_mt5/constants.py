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
MetaTrader 5 constants and enumerations.

This module centralizes all MT5 magic numbers for better maintainability.
"""

from enum import IntEnum


class Mt5TradeAction(IntEnum):
    """MT5 trade action types for order_send requests."""

    DEAL = 1  # Place a trade order for an immediate execution
    PENDING = 5  # Place a pending order
    MODIFY = 3  # Modify parameters of the previously placed trading order
    REMOVE = 8  # Delete the pending order placed previously
    CLOSE_BY = 10  # Close a position by an opposite one
    SLTP = 6  # Modify Stop Loss and Take Profit values


class Mt5OrderType(IntEnum):
    """MT5 order types."""

    BUY = 0  # Market buy order
    SELL = 1  # Market sell order
    BUY_LIMIT = 2  # Buy Limit pending order
    SELL_LIMIT = 3  # Sell Limit pending order
    BUY_STOP = 4  # Buy Stop pending order
    SELL_STOP = 5  # Sell Stop pending order
    BUY_STOP_LIMIT = 6  # Upon reaching the order price, a pending Buy Limit is placed
    SELL_STOP_LIMIT = 7  # Upon reaching the order price, a pending Sell Limit is placed
    CLOSE_BY = 8  # Order to close a position by an opposite one


class Mt5OrderState(IntEnum):
    """MT5 order states."""

    STARTED = 0  # Order checked, but not yet accepted by broker
    PLACED = 1  # Order accepted
    CANCELED = 2  # Order canceled by client
    PARTIAL = 3  # Order partially executed
    FILLED = 4  # Order fully executed
    REJECTED = 5  # Order rejected
    EXPIRED = 6  # Order expired
    REQUEST_ADD = 7  # Order is being registered
    REQUEST_MODIFY = 8  # Order is being modified
    REQUEST_CANCEL = 9  # Order is being deleted


class Mt5RetCode(IntEnum):
    """MT5 trade server return codes."""

    DONE = 10009  # Request completed
    PLACED = 10008  # Order placed
    REQUOTE = 10004  # Requote
    REJECT = 10006  # Request rejected
    CANCEL = 10007  # Request canceled by trader
    ERROR = 10011  # Request processing error
    TIMEOUT = 10012  # Request processing timeout
    INVALID = 10013  # Invalid request
    INVALID_VOLUME = 10014  # Invalid volume
    INVALID_PRICE = 10015  # Invalid price
    INVALID_STOPS = 10016  # Invalid stops
    TRADE_DISABLED = 10017  # Trade is disabled
    MARKET_CLOSED = 10018  # Market is closed
    NO_MONEY = 10019  # Not enough money
    PRICE_CHANGED = 10020  # Price changed
    PRICE_OFF = 10021  # No quotes to process request
    INVALID_EXPIRATION = 10022  # Invalid order expiration
    ORDER_CHANGED = 10023  # Order state changed
    TOO_MANY_REQUESTS = 10024  # Too many requests
    NO_CHANGES = 10025  # No changes in request
    SERVER_DISABLES_AT = 10026  # Autotrading disabled by server
    CLIENT_DISABLES_AT = 10027  # Autotrading disabled by client
    LOCKED = 10028  # Request locked for processing
    FROZEN = 10029  # Order or position frozen
    INVALID_FILL = 10030  # Invalid order filling type
    CONNECTION = 10031  # No connection


class Mt5PositionType(IntEnum):
    """MT5 position types."""

    BUY = 0  # Buy position
    SELL = 1  # Sell position


class Mt5DealType(IntEnum):
    """MT5 deal types."""

    BUY = 0  # Buy
    SELL = 1  # Sell
    BALANCE = 2  # Balance
    CREDIT = 3  # Credit
    CHARGE = 4  # Commission
    CORRECTION = 5  # Correction
    BONUS = 6  # Bonus
    COMMISSION = 7  # Commission
    COMMISSION_DAILY = 8  # Daily commission
    COMMISSION_MONTHLY = 9  # Monthly commission
    COMMISSION_AGENT_DAILY = 10  # Agent daily commission
    COMMISSION_AGENT_MONTHLY = 11  # Agent monthly commission
    INTEREST = 12  # Interest


class Mt5Timeframe(IntEnum):
    """MT5 timeframe constants."""

    M1 = 1  # 1 minute
    M2 = 2  # 2 minutes
    M3 = 3  # 3 minutes
    M4 = 4  # 4 minutes
    M5 = 5  # 5 minutes
    M6 = 6  # 6 minutes
    M10 = 10  # 10 minutes
    M12 = 12  # 12 minutes
    M15 = 15  # 15 minutes
    M20 = 20  # 20 minutes
    M30 = 30  # 30 minutes
    H1 = 16385  # 1 hour
    H2 = 16386  # 2 hours
    H3 = 16387  # 3 hours
    H4 = 16388  # 4 hours
    H6 = 16390  # 6 hours
    H8 = 16392  # 8 hours
    H12 = 16396  # 12 hours
    D1 = 16408  # 1 day
    W1 = 32769  # 1 week
    MN1 = 49153  # 1 month


class Mt5CopyTicks(IntEnum):
    """MT5 copy ticks flags."""

    ALL = 0  # All ticks
    INFO = 1  # Ticks with bid and/or ask changes
    TRADE = 2  # Ticks with last and volume changes


class Mt5OrderFilling(IntEnum):
    """MT5 order filling types."""

    FOK = 0  # Fill or Kill
    IOC = 1  # Immediate or Cancel
    RETURN = 2  # Return remaining volume


class Mt5OrderTime(IntEnum):
    """MT5 order time types."""

    GTC = 0  # Good Till Cancelled
    DAY = 1  # Day order
    SPECIFIED = 2  # Order valid till specified time
    SPECIFIED_DAY = 3  # Order valid till specified day
