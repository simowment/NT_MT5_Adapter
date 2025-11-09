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
The NautilusTrader MetaTrader 5 integration package.
"""

from nautilus_trader.adapters.mt5.config import Mt5DataClientConfig
from nautilus_trader.adapters.mt5.config import Mt5ExecClientConfig
from nautilus_trader.adapters.mt5.providers import Mt5InstrumentProvider, Mt5InstrumentProviderConfig
from nautilus_trader.adapters.mt5.data import Mt5DataClient
from nautilus_trader.adapters.mt5.execution import Mt5ExecutionClient
from nautilus_trader.adapters.mt5.factories import Mt5Factories

__all__ = [
    "Mt5DataClientConfig",
    "Mt5ExecClientConfig",
    "Mt5InstrumentProviderConfig",
    "Mt5InstrumentProvider",
    "Mt5DataClient",
    "Mt5ExecutionClient",
    "Mt5Factories",
]