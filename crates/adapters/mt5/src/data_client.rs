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
//
//! MT5 Data Client implementation.
//!
//! This module implements the data client for the MetaTrader 5 adapter,
//! providing market data functionality including subscriptions and requests.

use crate::config::{Mt5Config, Mt5DataClientConfig};
use crate::http::client::Mt5HttpClient;
use crate::http::error::Mt5HttpError as HttpClientError;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] HttpClientError),
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl From<String> for DataClientError {
    fn from(s: String) -> Self {
        DataClientError::ParseError(s)
    }
}

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
#[derive(Clone, Debug)]
#[pyclass]
pub struct Mt5DataClient {
    #[pyo3(get)]
    config: Mt5DataClientConfig,
    http_client: Arc<Mt5HttpClient>,
}

#[cfg(not(feature = "python-bindings"))]
pub struct Mt5DataClient {
    pub config: Mt5DataClientConfig,
    http_client: Arc<Mt5HttpClient>,
}

impl Mt5DataClient {
    /// Creates a new instance of the MT5 data client.
    pub fn new(config: Mt5DataClientConfig) -> Result<Self, DataClientError> {
        let base_url = config.base_url.clone();
        let http_config = Mt5Config {
            base_url: base_url.clone(),
            http_timeout: config.http_timeout,
            proxy: None,
        };

        let http_client = Arc::new(Mt5HttpClient::new(http_config, base_url).map_err(|e| DataClientError::ConnectionError(e.to_string()))?);

        Ok(Self { config, http_client })
    }

    /// Performs a login to validate connectivity with the MT5 bridge.
    pub async fn connect(&self) -> Result<(), DataClientError> {
        self.http_client.login().await.map_err(|e| DataClientError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    /// Fetches all symbols from the MT5 bridge.
    pub async fn get_symbols(&self) -> Result<Vec<crate::http::models::Mt5Symbol>, DataClientError> {
        let body = serde_json::json!({});
        let response = self.http_client.symbols_get(&body).await.map_err(|e| DataClientError::ConnectionError(e.to_string()))?;
        let symbols: Vec<crate::http::models::Mt5Symbol> = serde_json::from_value(response)
            .map_err(|e| DataClientError::ParseError(e.to_string()))?;
        Ok(symbols)
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5DataClient {
    #[new]
    pub fn py_new(config: Mt5DataClientConfig) -> Result<Self, PyErr> {
        Self::new(config).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    #[pyo3(name = "connect")]
    pub fn py_connect<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client.connect().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    #[pyo3(name = "get_symbols")]
    pub fn py_get_symbols<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let symbols = client.get_symbols().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            // Return as JSON string
            serde_json::to_string(&symbols)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
}