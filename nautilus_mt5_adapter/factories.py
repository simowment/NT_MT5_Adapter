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
Factories for the MetaTrader 5 adapter.

These factories wire together the MT5-specific clients with NautilusTrader
in a way consistent with other adapters (OKX, Bybit, etc.).
"""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING

from nautilus_mt5_adapter.config import (
    Mt5DataClientConfig,
    Mt5ExecClientConfig,
    Mt5InstrumentProviderConfig,
)
from nautilus_mt5_adapter.data import Mt5DataClient
from nautilus_mt5_adapter.execution import Mt5ExecutionClient
from nautilus_mt5_adapter.providers import Mt5InstrumentProvider
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock
from nautilus_trader.common.component import MessageBus
from nautilus_trader.live.factories import LiveDataClientFactory
from nautilus_trader.live.factories import LiveExecClientFactory

if TYPE_CHECKING:
    from nautilus_trader.live.data_client import LiveMarketDataClient
    from nautilus_trader.live.execution_client import LiveExecutionClient


def get_mt5_http_client(
    base_url: str,
    http_timeout: int = 30,
):
    """
    Create and return an MT5 HTTP client instance.

    Parameters
    ----------
    base_url : str
        The base URL for the MT5 REST API middleware.
    http_timeout : int, default 30
        HTTP timeout in seconds.

    Returns
    -------
    Mt5HttpClient
        The configured HTTP client instance.

    Note
    ----
    The Rust HTTP client constructor takes (Mt5Config, base_url).
    Mt5Config contains http_timeout and other settings.
    Credentials are not passed here - they are used by the middleware/terminal.

    """
    try:
        from nautilus_mt5_adapter.bindings import (
            Mt5Config,
            Mt5HttpClient,
        )

        config = Mt5Config(
            base_url=base_url,
            http_timeout=http_timeout,
        )
        return Mt5HttpClient(config, base_url)
    except ImportError:
        raise RuntimeError(
            "MT5 PyO3 bindings not available. "
            "Please build with: maturin develop --features python-bindings"
        )


def get_mt5_instrument_provider(
    client,
    clock: LiveClock,
    config: Mt5InstrumentProviderConfig,
) -> Mt5InstrumentProvider:
    """
    Create and return an MT5 instrument provider.

    Parameters
    ----------
    client : Mt5HttpClient
        The HTTP client for MT5 API calls.
    clock : LiveClock
        The clock for the provider.
    config : Mt5InstrumentProviderConfig
        The configuration for the provider.

    Returns
    -------
    Mt5InstrumentProvider

    """
    return Mt5InstrumentProvider(
        client=client,
        clock=clock,
        config=config,
    )


class Mt5LiveDataClientFactory(LiveDataClientFactory):
    """
    Factory for creating MT5 live data clients.

    Provides a ``create`` method for instantiating a ``Mt5DataClient``.
    """

    @staticmethod
    def create(
        loop: asyncio.AbstractEventLoop,
        name: str,
        config: Mt5DataClientConfig,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
    ) -> LiveMarketDataClient:
        """
        Create a new MT5 data client.

        Parameters
        ----------
        loop : asyncio.AbstractEventLoop
            The event loop for the client.
        name : str
            The client name (typically the venue name).
        config : Mt5DataClientConfig
            The client configuration.
        msgbus : MessageBus
            The message bus for the client.
        cache : Cache
            The cache for the client.
        clock : LiveClock
            The clock for the client.

        Returns
        -------
        Mt5DataClient

        """
        # Create HTTP client
        http_client = get_mt5_http_client(
            base_url=config.base_url,
            http_timeout=config.http_timeout,
        )

        # Create instrument provider
        instrument_provider = get_mt5_instrument_provider(
            client=http_client,
            clock=clock,
            config=config.instrument_provider,
        )

        return Mt5DataClient(
            loop=loop,
            client=http_client,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=instrument_provider,
            config=config,
        )


class Mt5LiveExecClientFactory(LiveExecClientFactory):
    """
    Factory for creating MT5 live execution clients.

    Provides a ``create`` method for instantiating a ``Mt5ExecutionClient``.
    """

    @staticmethod
    def create(
        loop: asyncio.AbstractEventLoop,
        name: str,
        config: Mt5ExecClientConfig,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
    ) -> LiveExecutionClient:
        """
        Create a new MT5 execution client.

        Parameters
        ----------
        loop : asyncio.AbstractEventLoop
            The event loop for the client.
        name : str
            The client name (typically the venue name).
        config : Mt5ExecClientConfig
            The client configuration.
        msgbus : MessageBus
            The message bus for the client.
        cache : Cache
            The cache for the client.
        clock : LiveClock
            The clock for the client.

        Returns
        -------
        Mt5ExecutionClient

        """
        # Create HTTP client
        http_client = get_mt5_http_client(
            base_url=config.base_url,
            http_timeout=config.http_timeout,
        )

        # Create instrument provider
        instrument_provider = get_mt5_instrument_provider(
            client=http_client,
            clock=clock,
            config=config.instrument_provider,
        )

        return Mt5ExecutionClient(
            loop=loop,
            client=http_client,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
            instrument_provider=instrument_provider,
            config=config,
        )
