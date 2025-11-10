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

//! HTTP client implementation for MetaTrader 5 REST API.
//!
//! This module implements the inner/outer client pattern with Arc wrapping
//! for efficient cloning in Python bindings while keeping HTTP logic centralized.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tracing::{debug, error, info};

use nautilus_network::http::HttpClient as NautilusHttpClient;

use std::collections::HashMap;

use crate::common::credential::Mt5Credential;
use crate::common::urls::Mt5Url;
use crate::config::Mt5Config;
use crate::http::models::{
    Mt5AccountInfo, Mt5Symbol, Mt5Rate, Mt5LoginRequest, Mt5LoginResponse,
    Mt5OrderRequest, Mt5OrderResponse, Mt5Position, Mt5Trade,
};
use crate::http::parse::{parse_account_info, parse_symbols, parse_rates, parse_single_symbol};
use thiserror::Error;

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;
#[cfg(feature = "python-bindings")]
use pyo3_async_runtimes::tokio::future_into_py;

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Request error: {0}")]
    RequestError(String),

    #[error("HTTP error: {0} - {1}")]
    HttpError(u16, String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Invalid request: {0}")]
    InvalidRequestError(String),

    #[error("Resource not found: {0}")]
    NotFoundError(String),

    #[error("JSON decode error: {0}")]
    JsonDecodeError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

impl HttpClientError {
    /// Determines if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            HttpClientError::ConnectionError(_)
                | HttpClientError::RequestError(_)
                | HttpClientError::ServerError(_)
                | HttpClientError::TimeoutError(_)
                | HttpClientError::RateLimitError(_)
                | HttpClientError::NetworkError(_)
        )
    }

    /// Determines if the error is non-retryable (should fail immediately)
    pub fn is_non_retryable(&self) -> bool {
        matches!(
            self,
            HttpClientError::AuthenticationError(_)
                | HttpClientError::AuthorizationError(_)
                | HttpClientError::InvalidRequestError(_)
                | HttpClientError::NotFoundError(_)
                | HttpClientError::JsonDecodeError(_)
                | HttpClientError::ParseError(_)
        )
    }

    /// Determines if the error is fatal (should terminate the connection)
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            HttpClientError::AuthenticationError(_)
                | HttpClientError::AuthorizationError(_)
        )
    }

    /// Maps HTTP status codes to appropriate error variants
    pub fn from_http_status(status: u16, message: String) -> Self {
        match status {
            400 => HttpClientError::InvalidRequestError(message),
            401 => HttpClientError::AuthenticationError(message),
            403 => HttpClientError::AuthorizationError(message),
            404 => HttpClientError::NotFoundError(message),
            429 => HttpClientError::RateLimitError(message),
            500..=599 => HttpClientError::ServerError(message),
            _ => HttpClientError::HttpError(status, message),
        }
    }
}

// Inner client - contains all HTTP logic and state
pub struct Mt5HttpInnerClient {
    config: Mt5Config,
    credential: Arc<Mutex<Mt5Credential>>,
    url: Mt5Url,
    http_client: NautilusHttpClient,
}

// Outer client - wraps inner with Arc for efficient cloning (needed for Python)
#[cfg_attr(feature = "python-bindings", pyclass)]
pub struct Mt5HttpClient {
    pub(crate) inner: Arc<Mt5HttpInnerClient>,
}

impl Mt5HttpInnerClient {
    pub fn new(config: Mt5Config, credential: Mt5Credential, url: Mt5Url) -> Result<Self, HttpClientError> {
        let timeout = Duration::from_secs(config.http_timeout);
        let http_client = NautilusHttpClient::new(
            HashMap::new(), // headers
            Vec::new(),     // user agents
            Vec::new(),     // quotas
            None,           // default quota
            timeout.as_secs(),  // timeout (convert Duration to u64 seconds for the network crate)
        );

        Ok(Self {
            config,
            credential: Arc::new(Mutex::new(credential)),
            url,
            http_client,
        })
    }

    async fn get_auth_header(&self) -> Result<String, HttpClientError> {
        let cred = self.credential.lock().await;
        
        if let Some(token) = &cred.token {
            Ok(format!("Bearer {}", token))
        } else {
            Err(HttpClientError::AuthenticationError("No authentication token available".to_string()))
        }
    }

    pub async fn login(&self) -> Result<(), HttpClientError> {
        let cred = self.credential.lock().await.clone();
        
        let login_request = Mt5LoginRequest {
            login: cred.login.clone(),
            password: cred.password.clone(),
            server: cred.server.clone(),
        };

        let response = self.http_client
            .post_json(&self.url.login_url(), &login_request)
            .await
            .map_err(|e| HttpClientError::NetworkError(e.to_string()))?;

        let status = response.status().as_u16();
        if status != 200 {
            let error_text = response.text().await.unwrap_or_default();
            return Err(HttpClientError::from_http_status(status, error_text));
        }

        let login_response: Mt5LoginResponse = response
            .json()
            .await
            .map_err(|e| HttpClientError::JsonDecodeError(format!("Failed to parse login response: {}", e)))?;

        let mut cred_mut = self.credential.lock().await;
        cred_mut.token = Some(login_response.token);

        Ok(())
    }

    async fn http_get(&self, url: &str) -> Result<String, HttpClientError> {
        let auth = self.get_auth_header().await?;

        let response = self.http_client
            .get_json(url, Some(&auth))
            .await
            .map_err(|e| HttpClientError::NetworkError(e.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|e| HttpClientError::RequestError(format!("read body failed: {e}")))?;

        if status != 200 {
            return Err(HttpClientError::from_http_status(status, body));
        }

        Ok(body)
    }

    async fn http_post<T: serde::Serialize>(&self, url: &str, body: &T) -> Result<String, HttpClientError> {
        let auth = self.get_auth_header().await?;

        let response = self.http_client
            .post_json(url, body)
            .await
            .map_err(|e| HttpClientError::NetworkError(e.to_string()))?;

        let status = response.status().as_u16();
        let body_text = response
            .text()
            .await
            .map_err(|e| HttpClientError::RequestError(format!("read body failed: {e}")))?;

        if status != 200 {
            return Err(HttpClientError::from_http_status(status, body_text));
        }

        Ok(body_text)
    }

    async fn http_delete(&self, url: &str) -> Result<String, HttpClientError> {
        let auth = self.get_auth_header().await?;

        let response = self.http_client
            .delete_json(url)
            .await
            .map_err(|e| HttpClientError::NetworkError(e.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|e| HttpClientError::RequestError(format!("read body failed: {e}")))?;

        if status != 200 {
            return Err(HttpClientError::from_http_status(status, body));
        }

        Ok(body)
    }

    // High-level domain methods
    
    // HTTP low-level API calls (prefixed with http_)
    
    pub async fn http_get_account_info(&self) -> Result<String, HttpClientError> {
        let response = self.http_get(&self.url.account_info_url()).await?;
        Ok(response)
    }

    pub async fn http_get_symbols(&self) -> Result<String, HttpClientError> {
        let response = self.http_get(&self.url.symbols_url()).await?;
        Ok(response)
    }

    pub async fn http_get_symbol_info(&self, symbol: &str) -> Result<String, HttpClientError> {
        let response = self.http_get(&self.url.symbol_info_url(symbol)).await?;
        Ok(response)
    }

    pub async fn http_get_rates(&self, symbol: &str) -> Result<String, HttpClientError> {
        let url = format!("{}?symbol={}", self.url.rates_url(), symbol);
        let response = self.http_get(&url).await?;
        Ok(response)
    }

    pub async fn http_get_positions(&self) -> Result<String, HttpClientError> {
        let response = self.http_get(&self.url.positions_url()).await?;
        Ok(response)
    }

    pub async fn http_get_trades(&self) -> Result<String, HttpClientError> {
        let response = self.http_get(&self.url.trades_url()).await?;
        Ok(response)
    }

    pub async fn http_get_orders(&self) -> Result<String, HttpClientError> {
        let response = self.http_get(&self.url.orders_url()).await?;
        Ok(response)
    }

    pub async fn http_submit_order(&self, order: &Mt5OrderRequest) -> Result<String, HttpClientError> {
        let response = self.http_post(&self.url.orders_url(), order).await?;
        Ok(response)
    }

    pub async fn http_cancel_order(&self, order_id: u64) -> Result<String, HttpClientError> {
        let response = self.http_delete(&self.url.orders_by_id_url(order_id)).await?;
        Ok(response)
    }

    pub async fn http_get_history(&self, symbol: Option<&str>, from: Option<u64>, to: Option<u64>) -> Result<String, HttpClientError> {
        let mut url = self.url.history_url();
        let mut params = Vec::new();
        
        if let Some(symbol) = symbol {
            params.push(format!("symbol={}", symbol));
        }
        if let Some(from) = from {
            params.push(format!("from={}", from));
        }
        if let Some(to) = to {
            params.push(format!("to={}", to));
        }
        
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        
        let response = self.http_get(&url).await?;
        Ok(response)
    }

    pub async fn http_modify_order(&self, order_id: u64, order: &Mt5OrderRequest) -> Result<String, HttpClientError> {
        let url = self.url.orders_by_id_url(order_id);
        let response = self.http_post(&url, order).await?;
        Ok(response)
    }

    // High-level domain methods (no prefix)
    
    pub async fn get_account_info(&self) -> Result<Mt5AccountInfo, HttpClientError> {
        let response = self.http_get_account_info().await?;
        parse_account_info(&response).map_err(|e| HttpClientError::ParseError(e.to_string()))
    }

    pub async fn get_symbols(&self) -> Result<Vec<Mt5Symbol>, HttpClientError> {
        let response = self.http_get_symbols().await?;
        parse_symbols(&response).map_err(|e| HttpClientError::ParseError(e.to_string()))
    }

    pub async fn get_symbol_info(&self, symbol: &str) -> Result<Mt5Symbol, HttpClientError> {
        let response = self.http_get_symbol_info(symbol).await?;
        // For single symbol info, we parse it as a single object rather than array
        let value: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| HttpClientError::JsonDecodeError(format!("Failed to parse symbol info: {}", e)))?;
        
        // Parse the single symbol object using the same logic as parse_symbols
        parse_single_symbol(&value).map_err(|e| HttpClientError::ParseError(e.to_string()))
    }

    pub async fn get_rates(&self, symbol: &str) -> Result<Vec<Mt5Rate>, HttpClientError> {
        let response = self.http_get_rates(symbol).await?;
        parse_rates(&response).map_err(|e| HttpClientError::ParseError(e.to_string()))
    }

    pub async fn get_positions(&self) -> Result<Vec<Mt5Position>, HttpClientError> {
        let response = self.http_get_positions().await?;
        serde_json::from_str::<Vec<Mt5Position>>(&response)
            .map_err(|e| HttpClientError::JsonDecodeError(format!("Failed to parse positions: {}", e)))
    }

    pub async fn get_trades(&self) -> Result<Vec<Mt5Trade>, HttpClientError> {
        let response = self.http_get_trades().await?;
        serde_json::from_str::<Vec<Mt5Trade>>(&response)
            .map_err(|e| HttpClientError::JsonDecodeError(format!("Failed to parse trades: {}", e)))
    }

    pub async fn get_orders(&self) -> Result<Vec<Mt5OrderResponse>, HttpClientError> {
        let response = self.http_get_orders().await?;
        serde_json::from_str::<Vec<Mt5OrderResponse>>(&response)
            .map_err(|e| HttpClientError::JsonDecodeError(format!("Failed to parse orders: {}", e)))
    }

    pub async fn submit_order(&self, order: Mt5OrderRequest) -> Result<Mt5OrderResponse, HttpClientError> {
        let response = self.http_submit_order(&order).await?;
        serde_json::from_str::<Mt5OrderResponse>(&response)
            .map_err(|e| HttpClientError::JsonDecodeError(format!("Failed to parse order response: {}", e)))
    }

    pub async fn cancel_order(&self, order_id: u64) -> Result<(), HttpClientError> {
        let _response = self.http_cancel_order(order_id).await?;
        Ok(())
    }

    pub async fn modify_order(&self, order_id: u64, order: Mt5OrderRequest) -> Result<Mt5OrderResponse, HttpClientError> {
        let response = self.http_modify_order(order_id, &order).await?;
        serde_json::from_str::<Mt5OrderResponse>(&response)
            .map_err(|e| HttpClientError::JsonDecodeError(format!("Failed to parse modify order response: {}", e)))
    }

    pub async fn get_history(&self, symbol: Option<&str>, from: Option<u64>, to: Option<u64>) -> Result<Vec<Mt5Trade>, HttpClientError> {
        let response = self.http_get_history(symbol, from, to).await?;
        serde_json::from_str::<Vec<Mt5Trade>>(&response)
            .map_err(|e| HttpClientError::JsonDecodeError(format!("Failed to parse history: {}", e)))
    }
}

impl Mt5HttpClient {
    pub fn new(config: Mt5Config, credential: Mt5Credential, url: Mt5Url) -> Result<Self, HttpClientError> {
        let inner = Mt5HttpInnerClient::new(config, credential, url)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub async fn login(&self) -> Result<(), HttpClientError> {
        self.inner.login().await
    }

    // Delegate all methods to the inner client
    pub async fn get_account_info(&self) -> Result<Mt5AccountInfo, HttpClientError> {
        self.inner.get_account_info().await
    }

    pub async fn get_symbols(&self) -> Result<Vec<Mt5Symbol>, HttpClientError> {
        self.inner.get_symbols().await
    }

    pub async fn get_symbol_info(&self, symbol: &str) -> Result<Mt5Symbol, HttpClientError> {
        self.inner.get_symbol_info(symbol).await
    }

    pub async fn get_rates(&self, symbol: &str) -> Result<Vec<Mt5Rate>, HttpClientError> {
        self.inner.get_rates(symbol).await
    }

    pub async fn get_positions(&self) -> Result<Vec<Mt5Position>, HttpClientError> {
        self.inner.get_positions().await
    }

    pub async fn get_trades(&self) -> Result<Vec<Mt5Trade>, HttpClientError> {
        self.inner.get_trades().await
    }

    pub async fn get_orders(&self) -> Result<Vec<Mt5OrderResponse>, HttpClientError> {
        self.inner.get_orders().await
    }

    pub async fn submit_order(&self, order: Mt5OrderRequest) -> Result<Mt5OrderResponse, HttpClientError> {
        self.inner.submit_order(order).await
    }

    pub async fn cancel_order(&self, order_id: u64) -> Result<(), HttpClientError> {
        self.inner.cancel_order(order_id).await
    }

    pub async fn modify_order(&self, order_id: u64, order: Mt5OrderRequest) -> Result<Mt5OrderResponse, HttpClientError> {
        self.inner.modify_order(order_id, order).await
    }

    pub async fn get_history(&self, symbol: Option<&str>, from: Option<u64>, to: Option<u64>) -> Result<Vec<Mt5Trade>, HttpClientError> {
        self.inner.get_history(symbol, from, to).await
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5HttpClient {
    #[new]
    fn new_py(config: Mt5Config, credential: Mt5Credential, url: Mt5Url) -> Result<Self, HttpClientError> {
        Self::new(config, credential, url)
    }
    
    #[pyo3(name = "login")]
    fn py_login(&self) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.login().await
        })
    }
    
    #[pyo3(name = "get_account_info")]
    fn py_get_account_info(&self) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.get_account_info().await
        })
    }
    
    #[pyo3(name = "get_symbols")]
    fn py_get_symbols(&self) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.get_symbols().await
        })
    }
    
    #[pyo3(name = "get_rates")]
    fn py_get_rates(&self, symbol: &str) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.get_rates(symbol).await
        })
    }
    
    #[pyo3(name = "get_symbol_info")]
    fn py_get_symbol_info(&self, symbol: &str) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.get_symbol_info(symbol).await
        })
    }
    
    #[pyo3(name = "get_positions")]
    fn py_get_positions(&self) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.get_positions().await
        })
    }
    
    #[pyo3(name = "get_trades")]
    fn py_get_trades(&self) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.get_trades().await
        })
    }
    
    #[pyo3(name = "get_orders")]
    fn py_get_orders(&self) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.get_orders().await
        })
    }
    
    #[pyo3(name = "submit_order")]
    fn py_submit_order(&self, order: Mt5OrderRequest) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.submit_order(order).await
        })
    }
    
    #[pyo3(name = "cancel_order")]
    fn py_cancel_order(&self, order_id: u64) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.cancel_order(order_id).await
        })
    }
    
    #[pyo3(name = "modify_order")]
    fn py_modify_order(&self, order_id: u64, order: Mt5OrderRequest) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.modify_order(order_id, order).await
        })
    }
    
    #[pyo3(name = "get_history")]
    fn py_get_history(&self, symbol: Option<String>, from: Option<u64>, to: Option<u64>) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.get_history(symbol.as_deref(), from, to).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_client() -> Result<Mt5HttpClient, HttpClientError> {
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
        let result = client.get_account_info().await;
        assert!(result.is_err());
        match result {
            Err(HttpClientError::AuthenticationError(_)) => (),
            _ => panic!("Expected AuthenticationError"),
        }
    }

    #[cfg(test)]
    mod wiremock_tests {
        use super::*;
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        #[tokio::test]
        async fn test_login_success() {
            let mock_server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/api/login"))
                .respond_with(ResponseTemplate::new(200).set_body_json(Mt5LoginResponse {
                    token: "test_token_123".to_string(),
                }))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: None,
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = client.login().await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_login_failure_invalid_credentials() {
            let mock_server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/api/login"))
                .respond_with(ResponseTemplate::new(401).set_body_string("Invalid credentials"))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "wrongpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: None,
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = client.login().await;
            assert!(result.is_err());
            match result {
                Err(HttpClientError::AuthenticationError(_)) => (),
                _ => panic!("Expected AuthenticationError for status 401"),
            }
        }

        #[tokio::test]
        async fn test_get_account_info_with_token() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/api/account"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "login": "123456789",
                    "balance": 10000.00,
                    "equity": 10000.00,
                    "margin": 0,
                    "marginFree": 10000.00,
                    "marginLevel": 0
                })))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("test_token_123".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = client.get_account_info().await;
            assert!(result.is_ok());

            let account = result.unwrap();
            assert_eq!(account.login, "123456789");
            assert_eq!(account.balance, 10000.00);
            assert_eq!(account.equity, 10000.00);
        }

        #[tokio::test]
        async fn test_get_symbols_with_token() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/api/symbols"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                    {
                        "symbol": "EURUSD",
                        "digits": 5,
                        "pointSize": 0.00001,
                        "volumeMin": 0.01,
                        "volumeMax": 100.0,
                        "volumeStep": 0.01,
                        "contractSize": 10000.0,
                        "marginInitial": 0.03,
                        "marginMaintenance": 0.03
                    }
                ])))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("test_token_123".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = client.get_symbols().await;
            assert!(result.is_ok());

            let symbols = result.unwrap();
            assert_eq!(symbols.len(), 1);
            assert_eq!(symbols[0].symbol, "EURUSD");
            assert_eq!(symbols[0].digits, 5);
        }

        #[tokio::test]
        async fn test_get_account_info_unauthorized() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/api/account"))
                .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("invalid_token".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = client.get_account_info().await;
            assert!(result.is_err());
            match result {
                Err(HttpClientError::AuthenticationError(_)) => (),
                _ => panic!("Expected AuthenticationError for status 401"),
            }
        }

        #[tokio::test]
        async fn test_submit_order_with_token() {
            let mock_server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/api/orders"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "order_id": 12345,
                    "symbol": "EURUSD",
                    "volume": 1.0,
                    "price": 1.08,
                    "type": "BUY",
                    "status": "EXECUTED"
                })))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("test_token_123".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let order = Mt5OrderRequest {
                symbol: "EURUSD".to_string(),
                volume: 1.0,
                price: 1.08,
                order_type: "BUY".to_string(),
                comment: None,
            };

            let result = client.submit_order(order).await;
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.order_id, 12345);
            assert_eq!(response.symbol, "EURUSD");
            assert_eq!(response.status, "EXECUTED");
        }

        #[tokio::test]
        async fn test_cancel_order_with_token() {
            let mock_server = MockServer::start().await;

            Mock::given(method("DELETE"))
                .and(path("/api/orders/12345"))
                .respond_with(ResponseTemplate::new(204))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("test_token_123".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = client.cancel_order(12345).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_get_positions_with_token() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/api/positions"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                    {
                        "ticket": 1,
                        "symbol": "EURUSD",
                        "volume": 1.0,
                        "open_price": 1.08,
                        "current_price": 1.085,
                        "profit": 50.0
                    }
                ])))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("test_token_123".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = client.get_positions().await;
            assert!(result.is_ok());

            let positions = result.unwrap();
            assert_eq!(positions.len(), 1);
            assert_eq!(positions[0].symbol, "EURUSD");
            assert_eq!(positions[0].profit, 50.0);
        }

        #[tokio::test]
        async fn test_get_trades_with_token() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/api/trades"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                    {
                        "ticket": 1,
                        "symbol": "EURUSD",
                        "volume": 1.0,
                        "open_price": 1.08,
                        "close_price": 1.10,
                        "open_time": 1678886400,
                        "close_time": 1678972800
                    }
                ])))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("test_token_123".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = client.get_trades().await;
            assert!(result.is_ok());

            let trades = result.unwrap();
            assert_eq!(trades.len(), 1);
            assert_eq!(trades[0].symbol, "EURUSD");
            assert_eq!(trades[0].close_price, Some(1.10));
        }

        #[tokio::test]
        async fn test_timeout_configuration() {
            let config = Mt5Config {
                base_url: "http://localhost:9999".to_string(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 1,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("test_token".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = tokio::time::timeout(
                Duration::from_secs(2),
                client.get_account_info(),
            ).await;

            assert!(result.is_ok());
            assert!(result.unwrap().is_err());
        }

        #[tokio::test]
        async fn test_get_symbol_info_with_token() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/api/symbols/EURUSD"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "symbol": "EURUSD",
                    "digits": 5,
                    "pointSize": 0.00001,
                    "volumeMin": 0.01,
                    "volumeMax": 100.0,
                    "volumeStep": 0.01,
                    "contractSize": 10000.0,
                    "marginInitial": 0.03,
                    "marginMaintenance": 0.03
                })))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("test_token_123".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = client.get_symbol_info("EURUSD").await;
            assert!(result.is_ok());

            let symbol = result.unwrap();
            assert_eq!(symbol.symbol, "EURUSD");
            assert_eq!(symbol.digits, 5);
        }

        #[tokio::test]
        async fn test_modify_order_with_token() {
            let mock_server = MockServer::start().await;

            Mock::given(method("POST"))
                .and(path("/api/orders/12345"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "order_id": 12345,
                    "symbol": "EURUSD",
                    "volume": 2.0,
                    "price": 1.09,
                    "type": "BUY",
                    "status": "MODIFIED"
                })))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("test_token_123".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let order = Mt5OrderRequest {
                symbol: "EURUSD".to_string(),
                volume: 2.0,
                price: 1.09,
                order_type: "BUY".to_string(),
                comment: None,
            };

            let result = client.modify_order(12345, order).await;
            assert!(result.is_ok());

            let response = result.unwrap();
            assert_eq!(response.order_id, 12345);
            assert_eq!(response.volume, 2.0);
            assert_eq!(response.status, "MODIFIED");
        }

        #[tokio::test]
        async fn test_get_history_with_token() {
            let mock_server = MockServer::start().await;

            Mock::given(method("GET"))
                .and(path("/api/history"))
                .and(query_param("symbol", "EURUSD"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                    {
                        "ticket": 1,
                        "symbol": "EURUSD",
                        "volume": 1.0,
                        "open_price": 1.08,
                        "close_price": 1.10,
                        "open_time": 1678886400,
                        "close_time": 1678972800
                    }
                ])))
                .mount(&mock_server)
                .await;

            let config = Mt5Config {
                base_url: mock_server.uri(),
                ws_url: "ws://localhost".to_string(),
                http_timeout: 30,
                ws_timeout: 30,
                proxy: None,
            };
            let cred = Mt5Credential {
                login: "testuser".to_string(),
                password: "testpass".to_string(),
                server: "testserver".to_string(),
                proxy: None,
                token: Some("test_token_123".to_string()),
            };
            let url = Mt5Url::new(&config.base_url);

            let client = Mt5HttpClient::new(config, cred, url).unwrap();
            let result = client.get_history(Some("EURUSD"), None, None).await;
            assert!(result.is_ok());

            let history = result.unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].symbol, "EURUSD");
            assert_eq!(history[0].close_price, Some(1.10));
        }
    }
}