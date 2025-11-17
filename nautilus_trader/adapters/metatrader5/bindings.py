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
from pathlib import Path

# Auto-import the PyO3 module from the bindings directory
try:
    # First try to import the PyO3 module directly (after compilation)
    import nautilus_adapters_mt5
    
    # If successful, we're done
    print(f"✅ Successfully loaded PyO3 bindings from compiled module")
    
except ImportError as e:
    print(f"⚠️  Failed to load PyO3 bindings: {e}")
    print("💡 Make sure to build the adapter with PyO3:")
    print("   cargo build -p nautilus-adapters-mt5 --features python-bindings --release")
    
    # Try to add the target directory to path where the compiled module would be
    import os
    from pathlib import Path
    
    # Look for the compiled module in common locations after compilation
    possible_paths = [
        Path(__file__).parent / ".." / ".." / ".." / ".." / "target" / "release",  # After cargo build
        Path(__file__).parent / ".." / ".." / ".." / ".." / "target" / "debug",     # After cargo build --debug
        Path(__file__).parent / "bindings",  # Alternative location
    ]
    
    for path in possible_paths:
        if path.exists():
            sys.path.insert(0, str(path.resolve()))
            try:
                import nautilus_adapters_mt5
                print(f"✅ Successfully loaded PyO3 bindings from {path}")
                break
            except ImportError:
                continue
    else:
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