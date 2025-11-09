//! Enumerations for the MetaTrader 5 adapter.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
