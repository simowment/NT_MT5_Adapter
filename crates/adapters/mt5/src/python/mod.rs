pub mod bindings;

/// Python bindings for the MetaTrader 5 adapter using PyO3.
///
/// This module re-exports Rust functionality to Python through PyO3,
/// making it available to the Python layer of the adapter.

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
#[pymodule]
pub fn nautilus_adapters_mt5(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    // Re-export all types from the bindings module
    m.add_class::<crate::config::Mt5Config>()?;
    m.add_class::<crate::common::credential::Mt5Credential>()?;
    m.add_class::<crate::http::client::Mt5HttpClient>()?;
    m.add_class::<crate::http::models::Mt5AccountInfo>()?;
    m.add_class::<crate::http::models::Mt5Symbol>()?;
    m.add_class::<crate::http::models::Mt5Rate>()?;
    m.add_class::<crate::http::models::Mt5OrderRequest>()?;
    m.add_class::<crate::http::models::Mt5OrderResponse>()?;
    m.add_class::<crate::http::models::Mt5Position>()?;
    m.add_class::<crate::http::models::Mt5Trade>()?;
    m.add_class::<crate::websocket::client::Mt5WebSocketClient>()?;

    Ok(())
}