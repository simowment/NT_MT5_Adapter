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
Configuration for the MetaTrader 5 integration.

This module defines configuration classes that align with both the Rust config
structures and NautilusTrader's configuration patterns.

The adapter supports two approaches:
1. Use Python-native configs (inheriting from NautilusTrader base configs)
2. Use Rust configs directly via the bindings module

For live trading, the Rust configs are used internally via the factories.
"""

from __future__ import annotations

from nautilus_trader.config import InstrumentProviderConfig
from nautilus_trader.config import LiveDataClientConfig
from nautilus_trader.config import LiveExecClientConfig


# =============================================================================
# INSTRUMENT PROVIDER CONFIG
# =============================================================================


class Mt5InstrumentProviderConfig(InstrumentProviderConfig, frozen=True):
    """
    Configuration for ``Mt5InstrumentProvider`` instances.

    This config aligns with the Rust `Mt5InstrumentProviderConfig` structure.

    Parameters
    ----------
    base_url : str, default "http://localhost:5000"
        The base URL for the MT5 REST API middleware.
    http_timeout : int | None, default 30
        HTTP timeout in seconds.
    login : str | None
        The MetaTrader 5 account login.
    password : str | None
        The MetaTrader 5 account password.
    server : str | None
        The MetaTrader 5 account server.
    filter_currencies : list[str] | None
        Currency filters (e.g., ["USD", "EUR", "GBP"]).
    filter_indices : list[str] | None
        Index symbol filters.
    filter_futures : bool, default False
        Whether to include futures instruments.
    filter_cfds : bool, default True
        Whether to include CFD instruments.
    auto_discover_instruments : bool, default True
        Whether to auto-discover available instruments.
    cache_expiry : int, default 300
        Cache expiry time in seconds.
    enable_logging : bool, default True
        Whether to enable client-side logging.
    """

    base_url: str = "http://localhost:5000"
    http_timeout: int | None = 30
    login: str | None = None
    password: str | None = None
    server: str | None = None
    filter_currencies: tuple[str, ...] | None = None
    filter_indices: tuple[str, ...] | None = None
    filter_futures: bool = False
    filter_cfds: bool = True
    auto_discover_instruments: bool = True
    cache_expiry: int = 300
    enable_logging: bool = True


# =============================================================================
# DATA CLIENT CONFIG
# =============================================================================


class Mt5DataClientConfig(LiveDataClientConfig, frozen=True):
    """
    Configuration for ``Mt5DataClient`` instances.

    This config aligns with the Rust `Mt5DataClientConfig` structure.

    Parameters
    ----------
    base_url : str, default "http://localhost:5000"
        The base URL for the MT5 REST API middleware.
    http_timeout : int, default 30
        HTTP timeout in seconds.
    login : str | None
        The MetaTrader 5 account login.
    password : str | None
        The MetaTrader 5 account password.
    server : str | None
        The MetaTrader 5 account server.
    enable_logging : bool, default True
        Whether to enable client-side logging.
    poll_interval_ms : int, default 1000
        Polling interval in milliseconds for REST-based data updates.
    instrument_provider : Mt5InstrumentProviderConfig
        The instrument provider configuration.
    """

    base_url: str = "http://localhost:5000"
    http_timeout: int = 30
    login: str | None = None
    password: str | None = None
    server: str | None = None
    enable_logging: bool = True
    poll_interval_ms: int = 1000
    instrument_provider: Mt5InstrumentProviderConfig = Mt5InstrumentProviderConfig()


# =============================================================================
# EXECUTION CLIENT CONFIG
# =============================================================================


class Mt5ExecClientConfig(LiveExecClientConfig, frozen=True):
    """
    Configuration for ``Mt5ExecutionClient`` instances.

    This config aligns with the Rust `Mt5ExecutionClientConfig` structure.

    Parameters
    ----------
    base_url : str, default "http://localhost:5000"
        The base URL for the MT5 REST API middleware.
    http_timeout : int, default 30
        HTTP timeout in seconds.
    login : str | None
        The MetaTrader 5 account login.
    password : str | None
        The MetaTrader 5 account password.
    server : str | None
        The MetaTrader 5 account server.
    max_concurrent_orders : int, default 50
        Maximum number of concurrent orders.
    enable_logging : bool, default True
        Whether to enable client-side logging.
    simulate_orders : bool, default False
        Whether to simulate orders (for backtesting).
    instrument_provider : Mt5InstrumentProviderConfig
        The instrument provider configuration.
    """

    base_url: str = "http://localhost:5000"
    http_timeout: int = 30
    login: str | None = None
    password: str | None = None
    server: str | None = None
    max_concurrent_orders: int = 50
    enable_logging: bool = True
    simulate_orders: bool = False
    instrument_provider: Mt5InstrumentProviderConfig = Mt5InstrumentProviderConfig()
