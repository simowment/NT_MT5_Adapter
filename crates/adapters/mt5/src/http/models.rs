//! Structs for REST payloads in the MetaTrader 5 adapter.

use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

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