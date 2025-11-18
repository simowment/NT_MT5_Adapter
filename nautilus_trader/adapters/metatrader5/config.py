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

This module defines the configuration classes for the MT5 adapter components,
including instrument provider, data client, and execution client configurations.
"""

from typing import Optional, List
from nautilus_trader.config import LiveDataClientConfig
from nautilus_trader.config import LiveExecClientConfig

#TODO : actually use correct config for rest server
class BaseMt5Config:
    """
    Base configuration class for MT5 components.

    Contains common parameters shared across all MT5 configuration classes.
    """
    
    def __init__(
        self,
        mt5_host: str = "127.0.0.1",
        mt5_port: int = 8080,
        mt5_base_url: Optional[str] = None,
        mt5_login: Optional[str] = None,
        mt5_password: Optional[str] = None,
        mt5_server: Optional[str] = None,
        enable_logging: bool = True,
    ):
        """
        Initialize the base MT5 configuration.

        Parameters
        ----------
        mt5_host : str
            The MT5 host address.
        mt5_port : int
            The MT5 port number.
        mt5_base_url : str, optional
            The base URL for MT5 API (overrides host/port).
        mt5_login : str, optional
            The MT5 account login.
        mt5_password : str, optional
            The MT5 account password.
        mt5_server : str, optional
            The MT5 account server.
        enable_logging : bool
            Whether to enable detailed logging.
        """
        self.mt5_host = mt5_host
        self.mt5_port = mt5_port
        self.mt5_base_url = mt5_base_url or f"http://{mt5_host}:{mt5_port}"
        self.mt5_login = mt5_login
        self.mt5_password = mt5_password
        self.mt5_server = mt5_server
        self.enable_logging = enable_logging


class Mt5InstrumentProviderConfig(BaseMt5Config):
    """
    Configuration for ``Mt5InstrumentProvider`` instances.

    Parameters
    ----------
    mt5_host : str
        The MetaTrader 5 host address for the REST API.
    mt5_port : int
        The MetaTrader 5 port for the REST API.
    mt5_base_url : str
        The full base URL for the MT5 API (overrides host/port).
    mt5_login : str
        The MetaTrader 5 account login.
    mt5_password : str
        The MetaTrader 5 account password.
    mt5_server : str
        The MetaTrader 5 account server.
    filter_currencies : list
        List of currency codes to include (e.g., ["USD", "EUR"]).
    filter_indices : list
        List of index symbols to include (e.g., ["US30", "SPX500"]).
    filter_cfds : bool
        Whether to include CFD instruments.
    filter_futures : bool
        Whether to include futures contracts.
    auto_discover_instruments : bool
        Whether to automatically discover instruments.
    cache_expiry : int
        Cache expiry time in seconds.
    enable_logging : bool
        Whether to enable detailed logging.
    """

    def __init__(
        self,
        mt5_host: str = "127.0.0.1",
        mt5_port: int = 8080,
        mt5_base_url: Optional[str] = None,
        mt5_login: Optional[str] = None,
        mt5_password: Optional[str] = None,
        mt5_server: Optional[str] = None,
        filter_currencies: Optional[List[str]] = None,
        filter_indices: Optional[List[str]] = None,
        filter_cfds: bool = True,
        filter_futures: bool = False,
        auto_discover_instruments: bool = True,
        cache_expiry: int = 300,
        enable_logging: bool = True,
        **kwargs,
    ):
        """
        Initialize the MT5 instrument provider configuration.
        """
        super().__init__(
            mt5_host=mt5_host,
            mt5_port=mt5_port,
            mt5_base_url=mt5_base_url,
            mt5_login=mt5_login,
            mt5_password=mt5_password,
            mt5_server=mt5_server,
            enable_logging=enable_logging,
        )
        self.filter_currencies = filter_currencies or []
        self.filter_indices = filter_indices or []
        self.filter_cfds = filter_cfds
        self.filter_futures = filter_futures
        self.auto_discover_instruments = auto_discover_instruments
        self.cache_expiry = cache_expiry


class Mt5DataClientConfig(BaseMt5Config, LiveDataClientConfig):
    """
    Configuration for ``Mt5DataClient`` instances.

    Parameters
    ----------
    mt5_host : str
        The MetaTrader 5 host address for the REST API.
    mt5_port : int
        The MetaTrader 5 port for the REST API.
    mt5_base_url : str
        The full base URL for the MT5 API (overrides host/port).
    mt5_login : str
        The MetaTrader 5 account login.
    mt5_password : str
        The MetaTrader 5 account password.
    mt5_server : str
        The MetaTrader 5 account server.
    subscribe_quotes : bool
        Whether to subscribe to quote ticks by default.
    subscribe_trades : bool
        Whether to subscribe to trade ticks by default.
    subscribe_order_book : bool
        Whether to subscribe to order book data by default.
    max_subscriptions : int
        Maximum number of concurrent subscriptions.
    connection_retry_attempts : int
        Number of retry attempts for connection.
    connection_retry_delay : int
        Delay between connection retries (seconds).
    heartbeat_interval : int
        WebSocket heartbeat interval (seconds).
    reconnection_enabled : bool
        Whether to enable automatic reconnection.
    enable_logging : bool
        Whether to enable detailed logging.
    """

    def __init__(
        self,
        mt5_host: str = "127.0.0.1",
        mt5_port: int = 8080,
        mt5_base_url: Optional[str] = None,
        mt5_login: Optional[str] = None,
        mt5_password: Optional[str] = None,
        mt5_server: Optional[str] = None,
        subscribe_quotes: bool = True,
        subscribe_trades: bool = True,
        subscribe_order_book: bool = False,
        max_subscriptions: int = 1000,
        connection_retry_attempts: int = 3,
        connection_retry_delay: int = 5,
        heartbeat_interval: int = 30,  # This parameter is now unused since WebSocket is removed
        reconnection_enabled: bool = True,  # This parameter is now unused since WebSocket is removed
        enable_logging: bool = True,
        **kwargs,
    ):
        """
        Initialize the MT5 data client configuration.
        """
        BaseMt5Config.__init__(
            self,
            mt5_host=mt5_host,
            mt5_port=mt5_port,
            mt5_base_url=mt5_base_url,
            mt5_login=mt5_login,
            mt5_password=mt5_password,
            mt5_server=mt5_server,
            enable_logging=enable_logging,
        )
        LiveDataClientConfig.__init__(self, **kwargs)
        
        self.subscribe_quotes = subscribe_quotes
        self.subscribe_trades = subscribe_trades
        self.subscribe_order_book = subscribe_order_book
        self.max_subscriptions = max_subscriptions
        self.connection_retry_attempts = connection_retry_attempts
        self.connection_retry_delay = connection_retry_delay
        self.heartbeat_interval = heartbeat_interval
        self.reconnection_enabled = reconnection_enabled


class Mt5ExecClientConfig(BaseMt5Config, LiveExecClientConfig):
    """
    Configuration for ``Mt5ExecutionClient`` instances.

    Parameters
    ----------
    mt5_host : str
        The MetaTrader 5 host address for the REST API.
    mt5_port : int
        The MetaTrader 5 port for the REST API.
    mt5_base_url : str
        The full base URL for the MT5 API (overrides host/port).
    mt5_login : str
        The MetaTrader 5 account login.
    mt5_password : str
        The MetaTrader 5 account password.
    mt5_server : str
        The MetaTrader 5 account server.
    max_concurrent_orders : int
        Maximum number of concurrent orders.
    order_timeout : int
        Order timeout in seconds.
    connection_retry_attempts : int
        Number of retry attempts for connection.
    connection_retry_delay : int
        Delay between connection retries (seconds).
    enable_partial_fills : bool
        Whether to enable partial fills.
    enable_market_data : bool
        Whether to enable market data streaming for execution.
    risk_management_enabled : bool
        Whether to enable risk management checks.
    position_sizing_enabled : bool
        Whether to enable automatic position sizing.
    simulate_orders : bool
        Whether to simulate orders without sending to MT5.
    enable_logging : bool
        Whether to enable detailed logging.
    """

    def __init__(
        self,
        mt5_host: str = "127.0.0.1",
        mt5_port: int = 8080,
        mt5_base_url: Optional[str] = None,
        mt5_login: Optional[str] = None,
        mt5_password: Optional[str] = None,
        mt5_server: Optional[str] = None,
        max_concurrent_orders: int = 50,
        order_timeout: int = 30,
        connection_retry_attempts: int = 3,
        connection_retry_delay: int = 5,
        enable_partial_fills: bool = True,
        enable_market_data: bool = True,
        risk_management_enabled: bool = True,
        position_sizing_enabled: bool = True,
        simulate_orders: bool = False,
        enable_logging: bool = True,
        **kwargs,
    ):
        """
        Initialize the MT5 execution client configuration.
        """
        BaseMt5Config.__init__(
            self,
            mt5_host=mt5_host,
            mt5_port=mt5_port,
            mt5_base_url=mt5_base_url,
            mt5_login=mt5_login,
            mt5_password=mt5_password,
            mt5_server=mt5_server,
            enable_logging=enable_logging,
        )
        LiveExecClientConfig.__init__(self, **kwargs)
        
        self.max_concurrent_orders = max_concurrent_orders
        self.order_timeout = order_timeout
        self.connection_retry_attempts = connection_retry_attempts
        self.connection_retry_delay = connection_retry_delay
        self.enable_partial_fills = enable_partial_fills
        self.enable_market_data = enable_market_data
        self.risk_management_enabled = risk_management_enabled
        self.position_sizing_enabled = position_sizing_enabled
        self.simulate_orders = simulate_orders