//! WebSocket message types for MetaTrader 5 adapter.

use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python-bindings", pyclass)]
#[serde(tag = "type")]
pub enum Mt5WsMessage {
    /// Trade data message
    Trade {
        symbol: String,
        price: f64,
        volume: f64,
        timestamp: u64,
    },
    /// Quote data message
    Quote {
        symbol: String,
        bid: f64,
        ask: f64,
        timestamp: u64,
    },
    /// Order book update
    OrderBook {
        symbol: String,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        timestamp: u64,
    },
    /// Connection status
    Connection {
        status: String,
        message: Option<String>,
    },
}

/// Subscription message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python-bindings", pyclass)]
pub struct Mt5Subscription {
    pub channel: String,
    pub symbol: String,
    pub params: Option<serde_json::Value>,
}