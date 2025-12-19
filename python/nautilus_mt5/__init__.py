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
MT5 Adapter for NautilusTrader.

This module provides the MetaTrader 5 adapter for the NautilusTrader framework.
"""

from .common import MT5, MT5_VENUE
from .config import (
    Mt5DataClientConfig,
    Mt5ExecClientConfig,
    Mt5InstrumentProviderConfig,
)
from .constants import (
    Mt5CopyTicks,
    Mt5DealType,
    Mt5OrderFilling,
    Mt5OrderState,
    Mt5OrderTime,
    Mt5OrderType,
    Mt5PositionType,
    Mt5RetCode,
    Mt5Timeframe,
    Mt5TradeAction,
)
from .data import Mt5DataClient
from .execution import Mt5ExecutionClient
from .factories import (
    Mt5LiveDataClientFactory,
    Mt5LiveExecClientFactory,
)
from .providers import Mt5InstrumentProvider
from .client import Mt5Client

# PyO3 bindings - will raise ImportError if not built
try:
    from .bindings import (
        Mt5Config,
        Mt5HttpClient,
        Mt5Credential,
        Mt5Symbol,
    )
except ImportError:
    # Attempt to import directly from the extension if rust_bindings.py fails
    try:
        from .nautilus_mt5 import (
            Mt5Config,
            Mt5HttpClient,
            Mt5Credential,
            Mt5Symbol,
        )
    except ImportError:
        # We'll allow it to fail later when these are accessed if not found
        pass


__version__ = "0.1.0"

__all__ = [
    # Version
    "__version__",
    # Client key constant (for TradingNodeConfig dictionaries)
    "MT5",
    # Venue identifier
    "MT5_VENUE",
    # Configuration
    "Mt5DataClientConfig",
    "Mt5ExecClientConfig",
    "Mt5InstrumentProviderConfig",
    # Factories (required for TradingNode integration)
    "Mt5LiveDataClientFactory",
    "Mt5LiveExecClientFactory",
    # Clients
    "Mt5DataClient",
    "Mt5ExecutionClient",
    "Mt5Client",
    # Providers
    "Mt5InstrumentProvider",
    # Rust bindings
    "Mt5Config",
    "Mt5HttpClient",
    "Mt5Credential",
    "Mt5Symbol",
    # Constants
    "Mt5CopyTicks",
    "Mt5DealType",
    "Mt5OrderFilling",
    "Mt5OrderState",
    "Mt5OrderTime",
    "Mt5OrderType",
    "Mt5PositionType",
    "Mt5RetCode",
    "Mt5Timeframe",
    "Mt5TradeAction",
]
