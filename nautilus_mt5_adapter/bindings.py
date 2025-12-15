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

This module loads the compiled Rust PyO3 bindings and re-exports them.
If the bindings are not available, it raises an ImportError immediately.
"""

from __future__ import annotations

import sys
from pathlib import Path

# Try to import the PyO3 module from standard location first
try:
    import nautilus_mt5 as _nautilus_mt5
except ImportError:
    # Look for the compiled module in common build locations
    possible_paths = [
        Path(__file__).parent.parent / "target" / "release",
        Path(__file__).parent.parent / "target" / "debug",
    ]

    _nautilus_mt5 = None
    for path in possible_paths:
        if path.exists():
            sys.path.insert(0, str(path.resolve()))
            try:
                import nautilus_mt5 as _nautilus_mt5

                break
            except ImportError:
                continue

    if _nautilus_mt5 is None:
        raise ImportError(
            "MT5 PyO3 bindings not available. "
            "Build with: maturin develop --features python-bindings"
        )


# Re-export classes from nautilus_mt5
Mt5Config = _nautilus_mt5.Mt5Config
Mt5HttpClient = _nautilus_mt5.Mt5HttpClient
Mt5Credential = _nautilus_mt5.Mt5Credential
Mt5Symbol = _nautilus_mt5.Mt5Symbol


__all__ = [
    "Mt5Config",
    "Mt5HttpClient",
    "Mt5Credential",
    "Mt5Symbol",
]
