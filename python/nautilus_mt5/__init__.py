"""
nautilus_mt5 - MetaTrader 5 adapter for NautilusTrader.

This package provides Rust bindings for communicating with MT5 via a REST API middleware.
"""

# Import everything from the Rust extension
from nautilus_mt5.nautilus_mt5 import *  # noqa: F401, F403

# Re-export version and docstring
__doc__ = nautilus_mt5.__doc__  # type: ignore  # noqa: F405
if hasattr(nautilus_mt5, "__all__"):  # type: ignore  # noqa: F405
    __all__ = nautilus_mt5.__all__  # type: ignore  # noqa: F405
