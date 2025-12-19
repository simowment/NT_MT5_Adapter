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
Common constants and utilities for the MetaTrader 5 integration.
"""

from nautilus_trader.model.identifiers import Venue


# Venue identifier for MT5
MT5_VENUE = Venue("MT5")

# String constant for use as client key in TradingNodeConfig dictionaries
# Usage: data_clients={MT5: Mt5DataClientConfig(...)}
MT5 = "MT5"
