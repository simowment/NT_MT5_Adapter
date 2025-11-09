//! Python bindings for the MetaTrader 5 adapter.

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
#[pymodule]
pub fn nautilus_adapters_mt5(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", crate::consts::MT5_VERSION)?;
    m.add("__name__", crate::consts::MT5_NAME)?;
    Ok(())
}
