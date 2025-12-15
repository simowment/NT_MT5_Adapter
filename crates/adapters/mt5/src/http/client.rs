// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! HTTP client for MetaTrader 5 REST API.
//!
//! This module provides a simple HTTP client that interfaces with the MT5 REST server.
//! The API exposes all MT5 Python API functions via HTTP endpoints.
//!
//! All responses follow the format:
//! - Success: `{"result": <data>}`
//! - Error: `{"error": "error message"}`

use std::collections::HashMap;
use std::sync::Arc;

use nautilus_network::http::HttpClient;
use serde_json::Value;

use crate::config::Mt5Config;
use crate::http::error::Mt5HttpError;

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

/// Inner MT5 HTTP client implementation
#[derive(Debug)]
pub struct Mt5HttpInnerClient {
    base_url: String,
    #[allow(dead_code)]
    client: HttpClient,
}

/// MT5 HTTP client (clonable wrapper)
#[derive(Debug)]
#[cfg_attr(feature = "python-bindings", pyclass)]
pub struct Mt5HttpClient {
    inner: Arc<Mt5HttpInnerClient>,
}

impl Mt5HttpInnerClient {
    pub fn new(config: Mt5Config, base_url: String) -> Result<Self, Mt5HttpError> {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "nautilus-mt5-adapter".to_string());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let client = HttpClient::new(
            headers,
            Vec::new(),
            Vec::new(),
            None,
            Some(config.http_timeout as u64),
        );

        Ok(Self { base_url, client })
    }

    #[allow(dead_code)]
    async fn get_request(&self, path: &str) -> Result<Value, Mt5HttpError> {
        let url = format!("{}{}", self.base_url, path);

        let resp = self
            .client
            .request(
                reqwest::Method::GET,
                url,
                None,
                None,
                None,
                None,
            )
            .await
            .map_err(|e| Mt5HttpError::NetworkError(e.to_string()))?;

        let status = resp.status.as_u16();
        let text = String::from_utf8_lossy(&resp.body).to_string();

        if status != 200 {
            return Err(Mt5HttpError::from_http_status(status, text));
        }

        serde_json::from_str(&text)
            .map_err(|e| Mt5HttpError::JsonDecodeError(format!("Invalid JSON response: {}", e)))
    }

    async fn post_request(&self, path: &str, body: &Value) -> Result<Value, Mt5HttpError> {
        let url = format!("{}{}", self.base_url, path);
        let body_bytes =
            serde_json::to_vec(body).map_err(|e| Mt5HttpError::JsonDecodeError(e.to_string()))?;

        let resp = self
            .client
            .request(
                reqwest::Method::POST,
                url,
                None,
                Some(body_bytes),
                None,
                None,
            )
            .await
            .map_err(|e| Mt5HttpError::NetworkError(e.to_string()))?;

        let status = resp.status.as_u16();
        let text = String::from_utf8_lossy(&resp.body).to_string();

        if status != 200 {
            return Err(Mt5HttpError::from_http_status(status, text));
        }

        serde_json::from_str(&text)
            .map_err(|e| Mt5HttpError::JsonDecodeError(format!("Invalid JSON response: {}", e)))
    }

    // ========================================================================
    // BASIC INFORMATION (POST - middleware uses POST for all endpoints)
    // ========================================================================

    pub async fn http_version(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/version", &serde_json::json!({})).await
    }

    pub async fn http_terminal_info(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/terminal_info", &serde_json::json!({})).await
    }

    pub async fn http_account_info(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/account_info", &serde_json::json!({})).await
    }

    // ========================================================================
    // SYMBOLS MANAGEMENT
    // ========================================================================

    pub async fn http_symbols_total(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/symbols_total", &serde_json::json!({})).await
    }

    pub async fn http_symbols_get(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/symbols_get", body).await
    }

    pub async fn http_symbol_info(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/symbol_info", body).await
    }

    pub async fn http_symbol_info_tick(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/symbol_info_tick", body).await
    }

    pub async fn http_symbol_select(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/symbol_select", body).await
    }

    // ========================================================================
    // MARKET DATA
    // ========================================================================

    pub async fn http_copy_ticks_from(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/copy_ticks_from", body).await
    }

    pub async fn http_copy_ticks_range(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/copy_ticks_range", body).await
    }

    pub async fn http_copy_rates_from(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/copy_rates_from", body).await
    }

    pub async fn http_copy_rates_range(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/copy_rates_range", body).await
    }

    // ========================================================================
    // ORDERS AND POSITIONS (POST - middleware uses POST for all endpoints)
    // ========================================================================

    pub async fn http_orders_total(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/orders_total", &serde_json::json!({})).await
    }

    pub async fn http_orders_get(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/orders_get", &serde_json::json!({})).await
    }

    pub async fn http_positions_total(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/positions_total", &serde_json::json!({})).await
    }

    pub async fn http_positions_get(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/positions_get", &serde_json::json!({})).await
    }

    // ========================================================================
    // HISTORY DATA
    // ========================================================================

    pub async fn http_history_orders_total(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/history_orders_total", body).await
    }

    pub async fn http_history_orders_get(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/history_orders_get", body).await
    }

    pub async fn http_history_deals_total(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/history_deals_total", body).await
    }

    pub async fn http_history_deals_get(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/history_deals_get", body).await
    }

    // ========================================================================
    // CALCULATIONS
    // ========================================================================

    pub async fn http_order_calc_margin(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/order_calc_margin", body).await
    }

    pub async fn http_order_calc_profit(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/order_calc_profit", body).await
    }

    pub async fn http_order_check(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/order_check", body).await
    }

    // ========================================================================
    // ORDER PLACEMENT
    // ========================================================================

    pub async fn http_order_send(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/order_send", body).await
    }

    // ========================================================================
    // MARKET BOOK (MICROSTRUCTURE)
    // ========================================================================

    pub async fn http_market_book_add(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/market_book_add", body).await
    }

    pub async fn http_market_book_get(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/market_book_get", body).await
    }

    pub async fn http_market_book_release(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/market_book_release", body).await
    }

    // ========================================================================
    // SYSTEM (POST - middleware uses POST for all endpoints)
    // ========================================================================

    pub async fn http_last_error(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/last_error", &serde_json::json!({})).await
    }

    // ========================================================================
    // SESSION MANAGEMENT (POST - middleware uses POST for all endpoints)
    // ========================================================================

    pub async fn http_initialize(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/initialize", &serde_json::json!({})).await
    }

    pub async fn http_login(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/login", &serde_json::json!({})).await
    }

    pub async fn http_shutdown(&self) -> Result<Value, Mt5HttpError> {
        self.post_request("/api/shutdown", &serde_json::json!({})).await
    }
}

impl Mt5HttpClient {
    pub fn new(config: Mt5Config, base_url: String) -> Result<Self, Mt5HttpError> {
        let inner = Mt5HttpInnerClient::new(config, base_url)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    // ========================================================================
    // PUBLIC API - Delegates to inner client
    // ========================================================================

    // Basic Information
    pub async fn version(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_version().await
    }

    pub async fn terminal_info(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_terminal_info().await
    }

    pub async fn account_info(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_account_info().await
    }

    // Symbols Management
    pub async fn symbols_total(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_symbols_total().await
    }

    pub async fn symbols_get(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_symbols_get(body).await
    }

    pub async fn symbol_info(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_symbol_info(body).await
    }

    pub async fn symbol_info_tick(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_symbol_info_tick(body).await
    }

    pub async fn symbol_select(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_symbol_select(body).await
    }

    // Market Data
    pub async fn copy_ticks_from(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_copy_ticks_from(body).await
    }

    pub async fn copy_ticks_range(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_copy_ticks_range(body).await
    }

    pub async fn copy_rates_from(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_copy_rates_from(body).await
    }

    pub async fn copy_rates_range(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_copy_rates_range(body).await
    }

    // Orders and Positions
    pub async fn orders_total(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_orders_total().await
    }

    pub async fn orders_get(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_orders_get().await
    }

    pub async fn positions_total(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_positions_total().await
    }

    pub async fn positions_get(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_positions_get().await
    }

    // History Data
    pub async fn history_orders_total(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_history_orders_total(body).await
    }

    pub async fn history_orders_get(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_history_orders_get(body).await
    }

    pub async fn history_deals_total(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_history_deals_total(body).await
    }

    pub async fn history_deals_get(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_history_deals_get(body).await
    }

    // Calculations
    pub async fn order_calc_margin(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_order_calc_margin(body).await
    }

    pub async fn order_calc_profit(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_order_calc_profit(body).await
    }

    pub async fn order_check(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_order_check(body).await
    }

    // Order Placement
    pub async fn order_send(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_order_send(body).await
    }

    // Market Book
    pub async fn market_book_add(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_market_book_add(body).await
    }

    pub async fn market_book_get(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_market_book_get(body).await
    }

    pub async fn market_book_release(&self, body: &Value) -> Result<Value, Mt5HttpError> {
        self.inner.http_market_book_release(body).await
    }

    // System
    pub async fn last_error(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_last_error().await
    }

    // Session Management
    pub async fn initialize(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_initialize().await
    }

    pub async fn login(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_login().await
    }

    pub async fn shutdown(&self) -> Result<Value, Mt5HttpError> {
        self.inner.http_shutdown().await
    }
}

impl Clone for Mt5HttpClient {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(feature = "python-bindings")]
mod py_helpers {
    use super::*;

    /// Helper function to convert serde_json::Value to JSON string
    pub fn value_to_json_string(v: Value) -> Result<String, PyErr> {
        serde_json::to_string(&v)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Helper function to parse JSON string to serde_json::Value
    pub fn json_string_to_value(s: &str) -> Result<Value, PyErr> {
        serde_json::from_str(s)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

#[cfg(feature = "python-bindings")]
use py_helpers::{value_to_json_string, json_string_to_value};

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5HttpClient {
    /// Create a new Mt5HttpClient from Python
    #[new]
    fn py_new(config: Mt5Config, base_url: String) -> PyResult<Self> {
        Self::new(config, base_url)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    #[pyo3(name = "version")]
    fn py_version<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.version().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "terminal_info")]
    fn py_terminal_info<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.terminal_info().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "account_info")]
    fn py_account_info<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.account_info().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "symbols_total")]
    fn py_symbols_total<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.symbols_total().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "symbols_get")]
    fn py_symbols_get<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.symbols_get(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "symbol_info")]
    fn py_symbol_info<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.symbol_info(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "symbol_info_tick")]
    fn py_symbol_info_tick<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.symbol_info_tick(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "symbol_select")]
    fn py_symbol_select<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.symbol_select(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "copy_ticks_from")]
    fn py_copy_ticks_from<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.copy_ticks_from(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "copy_ticks_range")]
    fn py_copy_ticks_range<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.copy_ticks_range(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "copy_rates_from")]
    fn py_copy_rates_from<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.copy_rates_from(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "copy_rates_range")]
    fn py_copy_rates_range<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.copy_rates_range(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "orders_total")]
    fn py_orders_total<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.orders_total().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "orders_get")]
    fn py_orders_get<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.orders_get().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "positions_total")]
    fn py_positions_total<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.positions_total().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "positions_get")]
    fn py_positions_get<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.positions_get().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "history_orders_total")]
    fn py_history_orders_total<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.history_orders_total(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "history_orders_get")]
    fn py_history_orders_get<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.history_orders_get(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "history_deals_total")]
    fn py_history_deals_total<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.history_deals_total(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "history_deals_get")]
    fn py_history_deals_get<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.history_deals_get(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "order_calc_margin")]
    fn py_order_calc_margin<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.order_calc_margin(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "order_calc_profit")]
    fn py_order_calc_profit<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.order_calc_profit(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "order_check")]
    fn py_order_check<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.order_check(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "order_send")]
    fn py_order_send<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.order_send(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "market_book_add")]
    fn py_market_book_add<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.market_book_add(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "market_book_get")]
    fn py_market_book_get<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.market_book_get(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "market_book_release")]
    fn py_market_book_release<'py>(&self, py: Python<'py>, body: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let body_value = json_string_to_value(&body)?;
            let result = client.market_book_release(&body_value).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "last_error")]
    fn py_last_error<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.last_error().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "initialize")]
    fn py_initialize<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.initialize().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "login")]
    fn py_login<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.login().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }

    #[pyo3(name = "shutdown")]
    fn py_shutdown<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.shutdown().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            value_to_json_string(result)
        })
    }
}
