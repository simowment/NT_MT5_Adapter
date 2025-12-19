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

import importlib.util
from pathlib import Path


def _load_extension(path: Path):
    """Load extension from file path."""
    try:
        spec = importlib.util.spec_from_file_location("nautilus_mt5", path)
        if spec and spec.loader:
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            return module
    except Exception:
        pass
    return None


_nautilus_mt5 = None

# 1. Try importing as standard module (if installed correctly)
try:
    # Try relative import first (within the package)
    from . import nautilus_mt5 as _nautilus_mt5

    # Check if we got the extension, not the package
    if not hasattr(_nautilus_mt5, "Mt5Config"):
        _nautilus_mt5 = None
except (ImportError, ValueError):
    _nautilus_mt5 = None

# 2. If not found, look for extension files in likely locations
if _nautilus_mt5 is None:
    root = Path(__file__).parent.parent
    possible_dirs = [
        root / "python" / "nautilus_mt5",
        root / "target" / "release",
        root / "target" / "debug",
    ]

    # Platform specific extension suffixes
    import importlib.machinery

    suffixes = importlib.machinery.EXTENSION_SUFFIXES

    found_path = None
    for d in possible_dirs:
        if not d.exists():
            continue
        for suffix in suffixes:
            p = d / f"nautilus_mt5{suffix}"
            if p.exists():
                found_path = p
                break
        if found_path:
            break

    if found_path:
        _nautilus_mt5 = _load_extension(found_path)

if _nautilus_mt5 is None:
    raise ImportError(
        "MT5 PyO3 bindings not available. "
        "Build with: maturin develop --features python-bindings. "
        "Ensure the compiled extension is found."
    )


# Re-export classes from _nautilus_mt5
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
