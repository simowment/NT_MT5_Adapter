//! Structs for REST payloads in the MetaTrader 5 adapter.

use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5LoginRequest {
    pub login: String,
    pub password: String,
    pub server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5LoginResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python-bindings", pyclass)]
#[serde(rename_all = "camelCase")]
pub struct Mt5AccountInfo {
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub login: String,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub balance: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub equity: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub margin: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub margin_free: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub margin_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python-bindings", pyclass)]
#[serde(rename_all = "camelCase")]
pub struct Mt5Symbol {
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub symbol: String,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub digits: u32,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub point_size: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub volume_min: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub volume_max: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub volume_step: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub contract_size: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub margin_initial: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub margin_maintenance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python-bindings", pyclass)]
#[serde(rename_all = "camelCase")]
pub struct Mt5Rate {
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub symbol: String,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub time: u64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub open: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub high: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub low: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub close: f64,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub tick_volume: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5OrderRequest {
    pub symbol: String,
    pub volume: f64,
    pub price: f64,
    #[serde(rename = "type")]
    pub order_type: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5OrderResponse {
    pub order_id: u64,
    pub symbol: String,
    pub volume: f64,
    pub price: f64,
    #[serde(rename = "type")]
    pub order_type: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5Position {
    pub ticket: u64,
    pub symbol: String,
    pub volume: f64,
    pub open_price: f64,
    pub current_price: f64,
    pub profit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5Trade {
    pub ticket: u64,
    pub symbol: String,
    pub volume: f64,
    pub open_price: f64,
    pub close_price: Option<f64>,
    pub open_time: u64,
    pub close_time: Option<u64>,
}