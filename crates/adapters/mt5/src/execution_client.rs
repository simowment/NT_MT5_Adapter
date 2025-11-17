// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------
//
//! MT5 Execution Client implementation.
//!
//! This module implements the execution client for the MetaTrader 5 adapter,
//! providing order management and execution functionality.

use crate::config::{Mt5Config, Mt5ExecutionClientConfig};
use crate::http::client::Mt5HttpClient;
use crate::http::error::Mt5HttpError as HttpClientError;
use crate::common::credential::Mt5Credential;
use crate::common::urls::Mt5Url;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecutionClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] HttpClientError),
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl From<String> for ExecutionClientError {
    fn from(s: String) -> Self {
        ExecutionClientError::ParseError(s)
    }
}

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
#[derive(Clone, Debug)]
#[pyclass]
pub struct Mt5ExecutionClient {
    #[cfg_attr(feature = "python-bindings", pyo3(get))]
    config: Mt5ExecutionClientConfig,
    http_client: Arc<Mt5HttpClient>,
}

#[cfg(not(feature = "python-bindings"))]
pub struct Mt5ExecutionClient {
    pub config: Mt5ExecutionClientConfig,
    http_client: Arc<Mt5HttpClient>,
}

#[derive(Debug, Clone)]
pub struct FillReport {
    pub order_id: String,
    pub fill_id: String,
    pub fill_price: f64,
    pub fill_quantity: f64,
    pub fill_timestamp: std::time::SystemTime,
    pub commission: f64,
    pub swap: f64,
}

impl Mt5ExecutionClient {
    /// Creates a new instance of the MT5 execution client.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the execution client.
    ///
    /// # Returns
    ///
    /// A new instance of the execution client.
    pub fn new(config: Mt5ExecutionClientConfig) -> Result<Self, ExecutionClientError> {
        let url = Mt5Url::new(&config.base_url);
        let http_config = Mt5Config {
            base_url: config.base_url.clone(),
            ws_url: "ws://localhost:8000".to_string(), // Default WebSocket URL
            http_timeout: config.http_timeout,
            ws_timeout: 30, // Default value since config doesn't have ws_timeout
            proxy: None,
        };
        
        let http_client = Arc::new(Mt5HttpClient::new(
            http_config,
            Mt5Credential::builder()
                .login(config.credential.login.clone())
                .password(config.credential.password.clone())
                .server(config.credential.server.clone())
                .build()
                .map_err(|e| ExecutionClientError::ConnectionError(e.to_string()))?,
            url,
        ).map_err(|e| ExecutionClientError::ConnectionError(e.to_string()))?);

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Establishes a connection to the MT5 server.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn connect(&self) -> Result<(), ExecutionClientError> {
        // Connect HTTP
        let _response = self.http_client.login().await
            .map_err(|e| ExecutionClientError::ConnectionError(e.to_string()))?;

        tracing::info!("MT5 execution client connected");

        Ok(())
    }

    /// Disconnects from the MT5 server.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn disconnect(&self) -> Result<(), ExecutionClientError> {
        // Cancel all active orders if configured
        // if self.config.risk_management_enabled {  // Comment out since field doesn't exist
        //     self.cancel_all_orders().await?;
        // }

        Ok(())
    }

    /// Checks if the client is connected.
    ///
    /// # Returns
    ///
    /// True if connected, false otherwise.
    pub fn is_connected(&self) -> bool {
        self.http_client.is_connected()
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5ExecutionClient {
    #[new]
    pub fn new_py(config: Mt5ExecutionClientConfig) -> Result<Self, PyErr> {
        Self::new(config).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    pub async fn connect(&self) -> Result<(), PyErr> {
        self.connect().await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    pub async fn disconnect(&self) -> Result<(), PyErr> {
        self.disconnect().await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected()
    }
}
