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
"""

from nautilus_trader.config import LiveDataClientConfig
from nautilus_trader.config import LiveExecClientConfig


class Mt5DataClientConfig(LiveDataClientConfig):
    """
    Configuration for ``Mt5DataClient`` instances.

    Parameters
    ----------
    mt5_host : str
        The MetaTrader 5 host address for the REST API.
    mt5_port : int
        The MetaTrader 5 port for the REST API.
    mt5_login : str
        The MetaTrader 5 account login.
    mt5_password : str
        The MetaTrader 5 account password.
    mt5_server : str
        The MetaTrader 5 account server.
    """

    def __init__(
        self,
        mt5_host: str = "127.0.0.1",
        mt5_port: int = 8080,
        mt5_login: str = None,
        mt5_password: str = None,
        mt5_server: str = None,
        **kwargs,
    ):
        super().__init__(**kwargs)
        self.mt5_host = mt5_host
        self.mt5_port = mt5_port
        self.mt5_login = mt5_login
        self.mt5_password = mt5_password
        self.mt5_server = mt5_server


class Mt5ExecClientConfig(LiveExecClientConfig):
    """
    Configuration for ``Mt5ExecClient`` instances.

    Parameters
    ----------
    mt5_host : str
        The MetaTrader 5 host address for the REST API.
    mt5_port : int
        The MetaTrader 5 port for the REST API.
    mt5_login : str
        The MetaTrader 5 account login.
    mt5_password : str
        The MetaTrader 5 account password.
    mt5_server : str
        The MetaTrader 5 account server.
    """

    def __init__(
        self,
        mt5_host: str = "127.0.0.1",
        mt5_port: int = 8080,
        mt5_login: str = None,
        mt5_password: str = None,
        mt5_server: str = None,
        **kwargs,
    ):
        super().__init__(**kwargs)
        self.mt5_host = mt5_host
        self.mt5_port = mt5_port
        self.mt5_login = mt5_login
        self.mt5_password = mt5_password
        self.mt5_server = mt5_server