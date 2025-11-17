// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
// Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------
//
//! Generic HTTP client for the MetaTrader 5 HTTP proxy.
//!
//! Purpose (confirmed):
//! - Consume ONLY the POST JSON routes exposed by the MT5 proxy (ex: /api/login, /api/account_info, /api/order_send, etc.).
//! - Transmit `json_body` as-is (serde_json::Value) without Nautilus business logic.
//! - Let a higher layer adapt responses to Nautilus / backtest types.
//!
//! This module therefore provides:
//! - Mt5HttpInnerClient: low-level client, finely configurable, that speaks in raw JSON.
//! - Mt5HttpClient: clonable wrapper, exposing an ergonomic API for each proxy endpoint.

use std::{collections::HashMap, sync::Arc, sync::atomic::{AtomicBool, Ordering}};

use nautilus_network::http::HttpClient;
use tokio::sync::Mutex;

use crate::common::credential::Mt5Credential;
use crate::common::urls::Mt5Url;
use crate::config::Mt5Config;
use crate::http::error::Mt5HttpError;

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;
#[cfg(feature = "python-bindings")]
use pyo3_async_runtimes::tokio::future_into_py;

// Low-level MT5 HTTP client (inner)
pub struct Mt5HttpInnerClient {
    base_url: String,
    client: HttpClient,
    credential: Arc<Mutex<Mt5Credential>>,
    is_connected: Arc<AtomicBool>,
}

#[cfg_attr(feature = "python-bindings", pyclass)]
pub struct Mt5HttpClient {
    inner: Arc<Mt5HttpInnerClient>,
}

impl Mt5HttpInnerClient {
    pub fn new(config: Mt5Config, credential: Mt5Credential, url: Mt5Url) -> Result<Self, Mt5HttpError> {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "nautilus-mt5-adapter".to_string());

        // HttpClient::new signature (0.51.0):
        // new(default_headers, header_keys, keyed_quotas, default_quota, timeout_secs)
        let client = HttpClient::new(
            headers,
            Vec::new(),
            Vec::new(),
            None,
            Some(config.http_timeout as u64),
        );

        Ok(Self {
            base_url: url.base_url().to_string(),
            client,
            credential: Arc::new(Mutex::new(credential)),
            is_connected: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    fn set_connected(&self, connected: bool) {
        self.is_connected.store(connected, Ordering::SeqCst);
    }

    async fn get_auth_header(&self) -> Result<String, Mt5HttpError> {
        let cred = self.credential.lock().await;

        if let Some(token) = &cred.token {
            Ok(format!("Bearer {}", token))
        } else {
            Err(Mt5HttpError::AuthenticationError(
                "No authentication token available".to_string(),
            ))
        }
    }

    async fn post_json(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        let auth = self.get_auth_header().await?;

        let url = format!("{}{}", self.base_url, path);
        let body_bytes =
            serde_json::to_vec(body).map_err(|e| Mt5HttpError::JsonDecodeError(e.to_string()))?;

        let resp = self
            .client
            .request(
                reqwest::Method::POST,
                url,
                Some(HashMap::from([
                    ("Authorization".to_string(), auth),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ])),
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

    // Login via /api/login (without prior token)
    pub async fn http_login(
        &self,
        login: &str,
        password: &str,
        server: &str,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        let body = serde_json::json!({
            "login": login,
            "password": password,
            "server": server,
        });

        let resp = self.post_json("/api/login", &body).await?;

        // If the proxy returns a token, store it in the credentials and mark as connected
        if let Some(token) = resp
            .get("token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
        {
            let mut cred = self.credential.lock().await;
            cred.token = Some(token);
            drop(cred);
            self.set_connected(true);
        }

        Ok(resp)
    }

    // Generic endpoints: each method takes a `serde_json::Value` (json_body)
    // and returns raw `serde_json::Value` as returned by the MT5 proxy.

    pub async fn http_account_info(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/account_info", body).await
    }

    pub async fn http_copy_rates_from(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/copy_rates_from", body).await
    }

    pub async fn http_copy_rates_from_pos(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/copy_rates_from_pos", body).await
    }

    pub async fn http_copy_rates_range(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/copy_rates_range", body).await
    }

    pub async fn http_copy_ticks_from(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/copy_ticks_from", body).await
    }

    pub async fn http_copy_ticks_range(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/copy_ticks_range", body).await
    }

    pub async fn http_history_deals_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/history_deals_get", body).await
    }

    pub async fn http_history_deals_total(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/history_deals_total", body).await
    }

    pub async fn http_history_orders_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/history_orders_get", body).await
    }

    pub async fn http_history_orders_total(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/history_orders_total", body).await
    }

    pub async fn http_initialize(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/initialize", body).await
    }

    pub async fn http_last_error(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/last_error", body).await
    }

    pub async fn http_market_book_add(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/market_book_add", body).await
    }

    pub async fn http_market_book_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/market_book_get", body).await
    }

    pub async fn http_market_book_release(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/market_book_release", body).await
    }

    pub async fn http_order_calc_margin(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/order_calc_margin", body).await
    }

    pub async fn http_order_calc_profit(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/order_calc_profit", body).await
    }

    pub async fn http_order_check(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/order_check", body).await
    }

    pub async fn http_order_send(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/order_send", body).await
    }

    pub async fn http_orders_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/orders_get", body).await
    }

    pub async fn http_orders_total(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/orders_total", body).await
    }

    pub async fn http_positions_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/positions_get", body).await
    }

    pub async fn http_positions_total(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/positions_total", body).await
    }

    pub async fn http_shutdown(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/shutdown", body).await
    }

    pub async fn http_symbol_info(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/symbol_info", body).await
    }

    pub async fn http_symbol_info_tick(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/symbol_info_tick", body).await
    }

    pub async fn http_symbol_select(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/symbol_select", body).await
    }

    pub async fn http_symbols_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/symbols_get", body).await
    }

    pub async fn http_symbols_total(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/symbols_total", body).await
    }

    pub async fn http_terminal_info(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/terminal_info", body).await
    }

    pub async fn http_version(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.post_json("/api/version", body).await
    }
}

impl Mt5HttpClient {
    pub fn new(
        config: Mt5Config,
        credential: Mt5Credential,
        url: Mt5Url,
    ) -> Result<Self, Mt5HttpError> {
        let inner = Mt5HttpInnerClient::new(config, credential, url)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    // Login via proxy
    pub async fn login(&self) -> Result<serde_json::Value, Mt5HttpError> {
        let cred = { self.inner.credential.lock().await.clone() };
        self.inner
            .http_login(&cred.login, &cred.password, &cred.server)
            .await
    }

    // Expose all proxy methods delegating to Mt5HttpInnerClient.
    pub async fn account_info(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_account_info(body).await
    }

    pub async fn copy_rates_from(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_copy_rates_from(body).await
    }

    pub async fn copy_rates_from_pos(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_copy_rates_from_pos(body).await
    }

    pub async fn copy_rates_range(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_copy_rates_range(body).await
    }

    pub async fn copy_ticks_from(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_copy_ticks_from(body).await
    }

    pub async fn copy_ticks_range(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_copy_ticks_range(body).await
    }

    pub async fn history_deals_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_history_deals_get(body).await
    }

    pub async fn history_deals_total(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_history_deals_total(body).await
    }

    pub async fn history_orders_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_history_orders_get(body).await
    }

    pub async fn history_orders_total(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_history_orders_total(body).await
    }

    pub async fn initialize(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_initialize(body).await
    }

    pub async fn last_error(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_last_error(body).await
    }

    pub async fn market_book_add(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_market_book_add(body).await
    }

    pub async fn market_book_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_market_book_get(body).await
    }

    pub async fn market_book_release(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_market_book_release(body).await
    }

    pub async fn order_calc_margin(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_order_calc_margin(body).await
    }

    pub async fn order_calc_profit(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_order_calc_profit(body).await
    }

    pub async fn order_check(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_order_check(body).await
    }

    pub async fn order_send(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_order_send(body).await
    }

    pub async fn orders_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_orders_get(body).await
    }

    pub async fn orders_total(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_orders_total(body).await
    }

    pub async fn positions_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_positions_get(body).await
    }

    pub async fn positions_total(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_positions_total(body).await
    }

    pub async fn shutdown(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_shutdown(body).await
    }

    pub async fn symbol_info(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_symbol_info(body).await
    }

    pub async fn symbol_info_tick(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_symbol_info_tick(body).await
    }

    pub async fn symbol_select(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_symbol_select(body).await
    }

    pub async fn symbols_get(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_symbols_get(body).await
    }

    pub async fn symbols_total(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_symbols_total(body).await
    }

    pub async fn terminal_info(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_terminal_info(body).await
    }

    pub async fn version(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, Mt5HttpError> {
        self.inner.http_version(body).await
    }

    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5HttpClient {
    #[new]
    fn new_py(config: Mt5Config, credential: Mt5Credential, url: Mt5Url) -> Result<Self, Mt5HttpError> {
        Self::new(config, credential, url)
    }
    
    #[pyo3(name = "login")]
    fn py_login(&self) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.login().await
        })
    }
    
    #[pyo3(name = "account_info")]
    fn py_account_info(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.account_info(&body).await
        })
    }
    
    #[pyo3(name = "copy_rates_from")]
    fn py_copy_rates_from(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.copy_rates_from(&body).await
        })
    }
    
    #[pyo3(name = "copy_rates_from_pos")]
    fn py_copy_rates_from_pos(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.copy_rates_from_pos(&body).await
        })
    }
    
    #[pyo3(name = "copy_rates_range")]
    fn py_copy_rates_range(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.copy_rates_range(&body).await
        })
    }
    
    #[pyo3(name = "copy_ticks_from")]
    fn py_copy_ticks_from(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.copy_ticks_from(&body).await
        })
    }
    
    #[pyo3(name = "copy_ticks_range")]
    fn py_copy_ticks_range(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.copy_ticks_range(&body).await
        })
    }
    
    #[pyo3(name = "history_deals_get")]
    fn py_history_deals_get(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.history_deals_get(&body).await
        })
    }
    
    #[pyo3(name = "history_deals_total")]
    fn py_history_deals_total(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.history_deals_total(&body).await
        })
    }
    
    #[pyo3(name = "history_orders_get")]
    fn py_history_orders_get(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.history_orders_get(&body).await
        })
    }
    
    #[pyo3(name = "history_orders_total")]
    fn py_history_orders_total(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.history_orders_total(&body).await
        })
    }
    
    #[pyo3(name = "initialize")]
    fn py_initialize(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.initialize(&body).await
        })
    }
    
    #[pyo3(name = "last_error")]
    fn py_last_error(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.last_error(&body).await
        })
    }
    
    #[pyo3(name = "market_book_add")]
    fn py_market_book_add(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.market_book_add(&body).await
        })
    }
    
    #[pyo3(name = "market_book_get")]
    fn py_market_book_get(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.market_book_get(&body).await
        })
    }
    
    #[pyo3(name = "market_book_release")]
    fn py_market_book_release(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.market_book_release(&body).await
        })
    }
    
    #[pyo3(name = "order_calc_margin")]
    fn py_order_calc_margin(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.order_calc_margin(&body).await
        })
    }
    
    #[pyo3(name = "order_calc_profit")]
    fn py_order_calc_profit(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.order_calc_profit(&body).await
        })
    }
    
    #[pyo3(name = "order_check")]
    fn py_order_check(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.order_check(&body).await
        })
    }
    
    #[pyo3(name = "order_send")]
    fn py_order_send(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.order_send(&body).await
        })
    }
    
    #[pyo3(name = "orders_get")]
    fn py_orders_get(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.orders_get(&body).await
        })
    }
    
    #[pyo3(name = "orders_total")]
    fn py_orders_total(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.orders_total(&body).await
        })
    }
    
    #[pyo3(name = "positions_get")]
    fn py_positions_get(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.positions_get(&body).await
        })
    }
    
    #[pyo3(name = "positions_total")]
    fn py_positions_total(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.positions_total(&body).await
        })
    }
    
    #[pyo3(name = "shutdown")]
    fn py_shutdown(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.shutdown(&body).await
        })
    }
    
    #[pyo3(name = "symbol_info")]
    fn py_symbol_info(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.symbol_info(&body).await
        })
    }
    
    #[pyo3(name = "symbol_info_tick")]
    fn py_symbol_info_tick(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.symbol_info_tick(&body).await
        })
    }
    
    #[pyo3(name = "symbol_select")]
    fn py_symbol_select(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.symbol_select(&body).await
        })
    }
    
    #[pyo3(name = "symbols_get")]
    fn py_symbols_get(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.symbols_get(&body).await
        })
    }
    
    #[pyo3(name = "symbols_total")]
    fn py_symbols_total(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.symbols_total(&body).await
        })
    }
    
    #[pyo3(name = "terminal_info")]
    fn py_terminal_info(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.terminal_info(&body).await
        })
    }
    
    #[pyo3(name = "version")]
    fn py_version(&self, body: serde_json::Value) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.version(&body).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client() -> Result<Mt5HttpClient, Mt5HttpError> {
        let config = Mt5Config {
            base_url: "http://localhost:8080".to_string(),
            ws_url: "ws://localhost:8080".to_string(),
            http_timeout: 30,
            ws_timeout: 30,
            proxy: None,
        };
        let cred = Mt5Credential {
            login: "user".to_string(),
            password: "pass".to_string(),
            server: "server".to_string(),
            proxy: None,
            token: None,
        };
        let url = Mt5Url::new("http://localhost:8080");

        Mt5HttpClient::new(config, cred, url)
    }

    #[test]
    fn test_http_client_creation() {
        let result = create_test_client();
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_client_with_proxy() {
        let config = Mt5Config {
            base_url: "http://localhost:8080".to_string(),
            ws_url: "ws://localhost:8080".to_string(),
            http_timeout: 30,
            ws_timeout: 30,
            proxy: Some("http://proxy.example.com:8080".to_string()),
        };
        let cred = Mt5Credential {
            login: "user".to_string(),
            password: "pass".to_string(),
            server: "server".to_string(),
            proxy: None,
            token: None,
        };
        let url = Mt5Url::new("http://localhost:8080");

        let result = Mt5HttpClient::new(config, cred, url);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_login_without_token_fails() {
        let client = create_test_client().unwrap();
        let result = client.account_info(&serde_json::json!({})).await;
        assert!(result.is_err());
        match result {
            Err(Mt5HttpError::AuthenticationError(_)) => (),
            _ => panic!("Expected AuthenticationError"),
        }
    }
}