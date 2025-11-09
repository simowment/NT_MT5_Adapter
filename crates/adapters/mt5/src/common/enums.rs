//! Enumerations for the MetaTrader 5 adapter.

use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "python-bindings", pyclass)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Connected => write!(f, "CONNECTED"),
            ConnectionStatus::Disconnected => write!(f, "DISCONNECTED"),
            ConnectionStatus::Connecting => write!(f, "CONNECTING"),
            ConnectionStatus::Error => write!(f, "ERROR"),
        }
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl ConnectionStatus {
    fn __str__(&self) -> String {
        self.to_string()
    }
    
    fn __repr__(&self) -> String {
        format!("ConnectionStatus.{}", self.to_string())
    }
}
