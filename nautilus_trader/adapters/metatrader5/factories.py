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

from nautilus_trader.adapters.metatrader5.config import Mt5DataClientConfig, Mt5ExecClientConfig
from nautilus_trader.adapters.metatrader5.data import Mt5DataClient
from nautilus_trader.adapters.metatrader5.execution import Mt5ExecutionClient
from nautilus_trader.adapters.metatrader5.providers import Mt5InstrumentProvider, Mt5InstrumentProviderConfig
from nautilus_trader.cache.cache import Cache
from nautilus_trader.common.component import LiveClock, MessageBus
from nautilus_trader.core.correctness import PyCondition


class Mt5Factories:
    """
    Factory helpers for constructing MT5 adapter components.
    """

    @staticmethod
    def create_data_client(
        loop,
        http_client,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
        config: Mt5DataClientConfig,
    ) -> Mt5DataClient:
        """
        Create a new MetaTrader 5 market data client.

        The provided http_client should be the Python binding over the Rust MT5
        HTTP/WebSocket implementation.
        """
        PyCondition.not_none(http_client, "http_client")
        PyCondition.not_none(msgbus, "msgbus")
        PyCondition.not_none(cache, "cache")
        PyCondition.not_none(clock, "clock")
        PyCondition.not_none(config, "config")

        return Mt5DataClient(
            loop=loop,
            client=http_client,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
        )

    @staticmethod
    def create_exec_client(
        loop,
        http_client,
        msgbus: MessageBus,
        cache: Cache,
        clock: LiveClock,
        config: Mt5ExecClientConfig,
    ) -> Mt5ExecutionClient:
        """
        Create a new MetaTrader 5 execution client.
        """
        PyCondition.not_none(http_client, "http_client")
        PyCondition.not_none(msgbus, "msgbus")
        PyCondition.not_none(cache, "cache")
        PyCondition.not_none(clock, "clock")
        PyCondition.not_none(config, "config")

        return Mt5ExecutionClient(
            loop=loop,
            client=http_client,
            msgbus=msgbus,
            cache=cache,
            clock=clock,
        )

    @staticmethod
    def create_instrument_provider(
        http_client,
        config: Mt5InstrumentProviderConfig | None = None,
    ) -> Mt5InstrumentProvider:
        """
        Create a new MetaTrader 5 instrument provider.
        """
        if config is None:
            config = Mt5InstrumentProviderConfig()

        PyCondition.not_none(http_client, "http_client")

        return Mt5InstrumentProvider(client=http_client, config=config)