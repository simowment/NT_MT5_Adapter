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

Usage:

    from nautilus_mt5_adapter import (
        MT5_VENUE,
        Mt5DataClientConfig,
        Mt5ExecClientConfig,
        Mt5LiveDataClientFactory,
        Mt5LiveExecClientFactory,
    )

    # Add factories to the trading node
    node.add_data_client_factory("MT5", Mt5LiveDataClientFactory)
    node.add_exec_client_factory("MT5", Mt5LiveExecClientFactory)

"""

# Re-export from submodules using relative imports
from nautilus_mt5_adapter.common import MT5_VENUE
from nautilus_mt5_adapter.config import (
    Mt5DataClientConfig,
    Mt5ExecClientConfig,
    Mt5InstrumentProviderConfig,
)
from nautilus_mt5_adapter.constants import (
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
from nautilus_mt5_adapter.data import Mt5DataClient
from nautilus_mt5_adapter.execution import Mt5ExecutionClient
from nautilus_mt5_adapter.factories import (
    Mt5LiveDataClientFactory,
    Mt5LiveExecClientFactory,
)
from nautilus_mt5_adapter.providers import Mt5InstrumentProvider

# PyO3 bindings - will raise ImportError if not built
from nautilus_mt5_adapter.bindings import (
    Mt5Config,
    Mt5HttpClient,
    Mt5Credential,
    Mt5Symbol,
)


__version__ = "0.1.0"

__all__ = [
    # Version
    "__version__",
    # Venue constant
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
    # Providers
    "Mt5InstrumentProvider",
    # Rust bindings (if available)
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
