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

//! MT5 Instrument Provider implementation.
//! 
//! This module implements the instrument provider for the MT5 adapter,
//! following the specifications in the adapter documentation.

use crate::config::{Mt5Config, Mt5InstrumentProviderConfig};
use crate::http::client::Mt5HttpClient;
use crate::http::error::{Mt5HttpError};
use crate::common::parse::InstrumentMetadata;
use crate::common::parse::InstrumentType;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

// Filter types for instrument loading
#[derive(Debug, Clone, PartialEq)]
pub enum InstrumentFilter {
    Symbol(String),
    Venue(String),
    Type(InstrumentType),
}

#[derive(Debug, Error)]
pub enum InstrumentProviderError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] Mt5HttpError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<String> for InstrumentProviderError {
    fn from(s: String) -> Self {
        InstrumentProviderError::ParseError(s)
    }
}

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
#[derive(Clone, Debug)]
#[pyclass]
pub struct Mt5InstrumentProvider {
    #[pyo3(get)]
    config: Mt5InstrumentProviderConfig,
    http_client: Arc<Mt5HttpClient>,
    cache: Arc<RwLock<Vec<InstrumentMetadata>>>,
}

#[cfg(not(feature = "python-bindings"))]
pub struct Mt5InstrumentProvider {
    pub config: Mt5InstrumentProviderConfig,
    http_client: Arc<Mt5HttpClient>,
    cache: Arc<RwLock<Vec<InstrumentMetadata>>>,
}

impl Mt5InstrumentProvider {
    pub fn new(config: Mt5InstrumentProviderConfig) -> Result<Self, InstrumentProviderError> {
        // Build the HTTP config for the client
        let base_url = config.base_url.clone();
        let http_config = Mt5Config {
            base_url: base_url.clone(),
            http_timeout: config.http_timeout.unwrap_or(30),
            proxy: None,
        };

        let http_client_result = Mt5HttpClient::new(http_config, base_url);
        let http_client = match http_client_result {
            Ok(client) => client,
            Err(e) => return Err(InstrumentProviderError::ConnectionError(e.to_string())),
        };
        let http_client = Arc::new(http_client);

        Ok(Self {
            config,
            http_client,
            cache: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Loads all instruments asynchronously, optionally applying filters.
    ///
    /// # Arguments
    /// * `filters` - Optional filters to apply to the instrument list
    ///
    /// # Returns
    /// A `Result` containing a `Vec` of loaded instruments or an `InstrumentProviderError`
    pub async fn load_all_async(&self, _filters: Option<Vec<InstrumentFilter>>) -> Result<Vec<InstrumentMetadata>, InstrumentProviderError> {
        let instruments = self.discover_instruments_metadata().await?;
        
        // Apply filters if provided
        let filtered_instruments = if let Some(filters) = _filters {
            instruments.into_iter()
                .filter(|instrument| self.matches_filters_metadata(instrument, &filters))
                .collect()
        } else {
            instruments
        };
        
        Ok(filtered_instruments)
    }

    /// Loads specific instruments by their IDs asynchronously.
    ///
    /// # Arguments
    /// * `instrument_ids` - List of instrument IDs to load
    /// * `filters` - Optional filters to apply to the instrument list
    ///
    /// # Returns
    /// A `Result` containing a `Vec` of loaded instruments or an `InstrumentProviderError`
    pub async fn load_ids_async(&self, _instrument_ids: Vec<String>, _filters: Option<Vec<InstrumentFilter>>) -> Result<Vec<InstrumentMetadata>, InstrumentProviderError> {
        let all_instruments = self.discover_instruments_metadata().await?;
        
        // Filter by instrument IDs
        let mut filtered_instruments: Vec<InstrumentMetadata> = all_instruments.into_iter()
            .filter(|instrument| _instrument_ids.contains(&instrument.symbol))
            .collect();
        
        // Apply additional filters if provided
        if let Some(filters) = _filters {
            filtered_instruments.retain(|instrument| self.matches_filters_metadata(instrument, &filters));
        }
        
        Ok(filtered_instruments)
    }

    /// Loads a single instrument by its ID asynchronously.
    ///
    /// # Arguments
    /// * `instrument_id` - The ID of the instrument to load
    /// * `filters` - Optional filters to apply to the instrument
    ///
    /// # Returns
    /// A `Result` containing the loaded instrument or an `InstrumentProviderError`
    pub async fn load_async(&self, instrument_id: String, filters: Option<Vec<InstrumentFilter>>) -> Result<InstrumentMetadata, InstrumentProviderError> {
        let instruments = self.load_ids_async(vec![instrument_id], filters).await?;
        
        instruments.into_iter()
            .next()
            .ok_or_else(|| InstrumentProviderError::ParseError("Instrument not found".to_string()))
    }

    /// Helper method to check if an instrument matches the provided filters.
    ///
    /// # Arguments
    /// * `instrument` - The instrument to check
    /// * `filters` - List of filters to apply
    ///
    /// # Returns
    /// `true` if the instrument matches all filters, `false` otherwise
    fn matches_filters_metadata(&self, instrument: &InstrumentMetadata, filters: &[InstrumentFilter]) -> bool {
        for filter in filters {
            match filter {
                InstrumentFilter::Symbol(symbol) => {
                    if instrument.symbol != *symbol {
                        return false;
                    }
                },
                InstrumentFilter::Venue(venue) => {
                    // For MT5, we can consider all instruments as belonging to MT5 venue
                    if "MT5" != *venue {
                        return false;
                    }
                },
                InstrumentFilter::Type(_instrument_type) => {
                    // This would need to be implemented based on the actual instrument type
                    // For now, we'll assume all instruments match
                },
            }
        }
        true
    }

    /// Discovers all instruments from the MT5 server.
    ///
    /// # Returns
    /// A `Result` containing a `Vec` of discovered instruments or an `InstrumentProviderError`
    async fn discover_instruments_metadata(&self) -> Result<Vec<InstrumentMetadata>, InstrumentProviderError> {
        // Get all symbols from MT5
        let body = serde_json::json!({});
        let response = self.http_client.symbols_get(&body).await
            .map_err(|e| InstrumentProviderError::ConnectionError(e.to_string()))?;
        let symbols: Vec<crate::http::models::Mt5Symbol> = serde_json::from_value(response)
            .map_err(|e| InstrumentProviderError::ParseError(e.to_string()))?;

        let mut instruments = Vec::new();

        for symbol in symbols {
            // Parse instrument type from symbol name
            let instrument_type = crate::common::parse::parse_instrument_symbol(&symbol.symbol)
                .unwrap_or_else(|_| InstrumentType::Cfd { symbol: symbol.symbol.clone() });
            
            let metadata = InstrumentMetadata {
                symbol: symbol.symbol.clone(),
                digits: symbol.digits as u8,
                point_size: symbol.point_size,
                volume_min: symbol.volume_min,
                volume_max: symbol.volume_max,
                volume_step: symbol.volume_step,
                contract_size: symbol.contract_size,
                instrument_type,
            };
            instruments.push(metadata);
        }

        // Cache the instruments
        {
            let mut cache = self.cache.write().await;
            *cache = instruments.clone();
        }

        Ok(instruments)
    }

    /// Loads all instruments (legacy method for backward compatibility).
    ///
    /// # Returns
    /// A `Result` containing a unit type or an `InstrumentProviderError`
    pub async fn load_instruments(&self) -> Result<(), InstrumentProviderError> {
        let instruments = self.discover_instruments_metadata().await?;
        
        for instrument in instruments {
            tracing::info!("Loaded instrument: {}", instrument.symbol);
        }
        
        Ok(())
    }

    // Remove the create_instrument method as it's not needed with the simplified approach
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5InstrumentProvider {
    #[new]
    pub fn py_new(config: Mt5InstrumentProviderConfig) -> Result<Self, PyErr> {
        Self::new(config).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    /// Loads all instruments asynchronously (returns JSON string).
    #[pyo3(name = "load_all_async")]
    pub fn py_load_all_async<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let provider = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = provider.load_all_async(None).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            // Serialize to JSON string
            serde_json::to_string(&result)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Loads specific instruments by their IDs asynchronously (returns JSON string).
    #[pyo3(name = "load_ids_async")]
    pub fn py_load_ids_async<'py>(&self, py: Python<'py>, instrument_ids: Vec<String>) -> PyResult<Bound<'py, PyAny>> {
        let provider = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = provider.load_ids_async(instrument_ids, None).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            serde_json::to_string(&result)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Loads a single instrument by its ID asynchronously (returns JSON string).
    #[pyo3(name = "load_async")]
    pub fn py_load_async<'py>(&self, py: Python<'py>, instrument_id: String) -> PyResult<Bound<'py, PyAny>> {
        let provider = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = provider.load_async(instrument_id, None).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            serde_json::to_string(&result)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Discovers all instruments from the MT5 server (returns JSON string).
    #[pyo3(name = "discover_instruments")]
    pub fn py_discover_instruments<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let provider = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = provider.discover_instruments_metadata().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            serde_json::to_string(&result)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Loads all instruments (returns empty string on success).
    #[pyo3(name = "load_instruments")]
    pub fn py_load_instruments<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let provider = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            provider.load_instruments().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok::<String, PyErr>("".to_string())
        })
    }
}