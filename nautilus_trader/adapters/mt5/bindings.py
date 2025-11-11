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
PyO3 module loader for MT5 adapter.

This module automatically loads the compiled PyO3 bindings from the bindings directory.
"""

import sys
import os
from pathlib import Path

# Auto-import the PyO3 module from the bindings directory
try:
    # Add bindings directory to Python path
    bindings_dir = Path(__file__).parent / "bindings"
    
    if bindings_dir.exists():
        sys.path.insert(0, str(bindings_dir))
        
        # Try to import the PyO3 module
        from nautilus_adapters_mt5 import *
        
        print(f"✅ Successfully loaded PyO3 bindings from {bindings_dir}")
        
    else:
        raise ImportError(f"Bindings directory not found: {bindings_dir}")
        
except ImportError as e:
    print(f"⚠️  Failed to load PyO3 bindings: {e}")
    print("💡 Make sure to build the adapter with PyO3:")
    print("   cargo build -p nautilus-adapters-mt5 --features python-bindings --release")
    
    # Create a mock module for development
    class MockMt5Module:
        """Mock module for development when PyO3 bindings are not available."""
        
        @classmethod
        def __getattr__(cls, name):
            """Mock any attribute access."""
            def mock_method(*args, **kwargs):
                print(f"🔧 Mock method {name} called with args={args}, kwargs={kwargs}")
                return None
            return mock_method
    
    # Create and expose mock module
    sys.modules['nautilus_adapters_mt5'] = MockMt5Module()
    nautilus_adapters_mt5 = sys.modules['nautilus_adapters_mt5']
    
    print("📝 Using mock module for development")