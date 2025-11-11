# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# -------------------------------------------------------------------------------------------------

"""
MT5 Adapter for NautilusTrader.

This module provides the MetaTrader 5 adapter for the NautilusTrader framework.
"""

import sys
import os
from pathlib import Path

# Auto-load the PyO3 module if available
try:
    # Try to import the compiled PyO3 module
    from .bindings import nautilus_adapters_mt5
    
    # Import all classes from the PyO3 module
    from nautilus_adapters_mt5 import (
        Mt5Credential,
        Mt5Config,
        Mt5DataClientConfig,
        Mt5ExecutionClientConfig, 
        Mt5InstrumentProviderConfig,
        Mt5HttpClient,
        Mt5AccountInfo,
        Mt5Symbol,
        Mt5Rate,
        Mt5OrderRequest,
        Mt5OrderResponse,
        Mt5Position,
        Mt5Trade,
        Mt5WebSocketClient,
        AccountInfoParams,
        SymbolsInfoParams,
        RatesInfoParams,
    )
    
    print("✅ MT5 adapter loaded with PyO3 bindings")
    
except ImportError as e:
    print(f"⚠️  PyO3 bindings not available: {e}")
    print("Please build the adapter with: cargo build -p nautilus-adapters-mt5 --features python-bindings --release")

# Version information
__version__ = "0.1.0"

# Export the classes
__all__ = [
    "Mt5Credential",
    "Mt5Config", 
    "Mt5DataClientConfig",
    "Mt5ExecutionClientConfig",
    "Mt5InstrumentProviderConfig",
    "Mt5HttpClient",
    "Mt5AccountInfo",
    "Mt5Symbol",
    "Mt5Rate", 
    "Mt5OrderRequest",
    "Mt5OrderResponse",
    "Mt5Position",
    "Mt5Trade",
    "Mt5WebSocketClient",
    "AccountInfoParams",
    "SymbolsInfoParams",
    "RatesInfoParams",
]