//! MT5 Instrument Provider implementation.
//! 
//! This module implements the instrument provider for the MT5 adapter,
//! following the specifications in the adapter documentation.

use crate::config::{Mt5Config, Mt5InstrumentProviderConfig};
use crate::http::client::Mt5HttpClient;
use crate::http::error::{Mt5HttpError};
use crate::common::parse::InstrumentMetadata;
use crate::common::credential::Mt5Credential;
use crate::common::urls::Mt5Url;
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
    config: Mt5InstrumentProviderConfig,
    http_client: Arc<Mt5HttpClient>,
    cache: Arc<RwLock<Vec<InstrumentMetadata>>>,
}

#[cfg(not(feature = "python-bindings"))]
pub struct Mt5InstrumentProvider {
    config: Mt5InstrumentProviderConfig,
    http_client: Arc<Mt5HttpClient>,
    cache: Arc<RwLock<Vec<InstrumentMetadata>>>,
}

impl Mt5InstrumentProvider {
    pub fn new(config: Mt5InstrumentProviderConfig) -> Result<Self, InstrumentProviderError> {
        // Construire la config HTTP globale pour le client.
        let http_config = Mt5Config {
            base_url: config.base_url.clone(),
            ws_url: config.ws_url.clone().unwrap_or_else(|| "ws://localhost:8080".to_string()),
            http_timeout: config.http_timeout.unwrap_or(30),
            ws_timeout: 30, // Default value since config doesn't have ws_timeout
            proxy: None,
        };

        let cred = Mt5Credential {
            login: config.credential.login.clone(),
            password: config.credential.password.clone(),
            server: config.credential.server.clone(),
            proxy: None,
            token: None,
        };

        let url = Mt5Url::new(&http_config.base_url);
        let http_client_result = Mt5HttpClient::new(http_config, cred, url);
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
    pub async fn load_all_async(&self, filters: Option<Vec<InstrumentFilter>>) -> Result<Vec<Instrument>, InstrumentProviderError> {
        let instruments = self.discover_instruments().await?;
        
        // Apply filters if provided
        let filtered_instruments = if let Some(filters) = filters {
            instruments.into_iter()
                .filter(|instrument| self.matches_filters(instrument, &filters))
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
    pub async fn load_ids_async(&self, instrument_ids: Vec<InstrumentId>, filters: Option<Vec<InstrumentFilter>>) -> Result<Vec<Instrument>, InstrumentProviderError> {
        let all_instruments = self.discover_instruments().await?;
        
        // Filter by instrument IDs
        let mut filtered_instruments: Vec<Instrument> = all_instruments.into_iter()
            .filter(|instrument| instrument_ids.contains(instrument.instrument_id()))
            .collect();
        
        // Apply additional filters if provided
        if let Some(filters) = filters {
            filtered_instruments.retain(|instrument| self.matches_filters(instrument, &filters));
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
    pub async fn load_async(&self, instrument_id: InstrumentId, filters: Option<Vec<InstrumentFilter>>) -> Result<Instrument, InstrumentProviderError> {
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
    fn matches_filters(&self, instrument: &Instrument, filters: &[InstrumentFilter]) -> bool {
        for filter in filters {
            match filter {
                InstrumentFilter::Symbol(symbol) => {
                    if instrument.instrument_id().symbol != *symbol {
                        return false;
                    }
                },
                InstrumentFilter::Venue(venue) => {
                    if instrument.instrument_id().venue != *venue {
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
    async fn discover_instruments(&self) -> Result<Vec<Instrument>, InstrumentProviderError> {
        // Get all symbols from MT5
        let body = serde_json::json!({});
        let response = self.http_client.symbols_get(&body).await
            .map_err(|e| InstrumentProviderError::ConnectionError(e.to_string()))?;
        let symbols: Vec<crate::http::models::Mt5Symbol> = serde_json::from_value(response)
            .map_err(|e| InstrumentProviderError::ParseError(e.to_string()))?;

        let mut instruments = Vec::new();

        for symbol in symbols {
            // Here we simply map Mt5Symbol -> InstrumentMetadata (MVP).
            // To be refined according to the actual bridge schema.
            let metadata = InstrumentMetadata {
                symbol: symbol.symbol.clone(),
                digits: symbol.digits as u8,
                point_size: symbol.point_size,
                volume_min: symbol.volume_min,
                volume_max: symbol.volume_max,
                volume_step: symbol.volume_step,
                contract_size: symbol.contract_size,
                instrument_type: InstrumentType::CurrencyPair {
                    base_currency: "BASE".to_string(),
                    quote_currency: "QUOTE".to_string(),
                },
            };
            instruments.push(metadata);
        }

        // Cache the instruments
        {
            let mut cache = self.cache.write().await;
            *cache = instruments.clone();
        }

        // Convert Vec<InstrumentMetadata> to Vec<Instrument>
        let instruments_converted: Vec<Instrument> = instruments
            .into_iter()
            .map(|metadata| self.create_instrument(&metadata))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(instruments_converted)
    }

    /// Loads all instruments (legacy method for backward compatibility).
    ///
    /// # Returns
    /// A `Result` containing a unit type or an `InstrumentProviderError`
    pub async fn load_instruments(&self) -> Result<(), InstrumentProviderError> {
        let instruments = self.discover_instruments().await?;
        
        for instrument in instruments {
            tracing::info!("Loaded instrument: {}", instrument.instrument_id());
        }
        
        Ok(())
    }

    pub fn create_instrument(&self, metadata: &InstrumentMetadata) -> Result<Instrument, InstrumentProviderError> {
        let instrument_id = InstrumentId::from_str(&metadata.symbol)
            .map_err(|e| InstrumentProviderError::ParseError(e.to_string()))?;
        
        let price = Price::new(0.0, metadata.digits); // Using 0.0 as placeholder price
        
        Ok(Instrument::new(
            instrument_id,
            price,
            metadata.volume_min,
            metadata.volume_max,
            metadata.volume_step,
        ))
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5InstrumentProvider {
    #[new]
    pub fn new_py(config: Mt5InstrumentProviderConfig) -> Result<Self, PyErr> {
        Self::new(config).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    /// Loads all instruments asynchronously, optionally applying filters.
    /// 
    /// # Arguments
    /// * `filters` - Optional filters to apply to the instrument list
    /// 
    /// # Returns
    /// A list of loaded instruments
    #[pyo3(name = "load_all_async")]
    pub fn load_all_async_py(&self, filters: Option<Vec<InstrumentFilter>>) -> PyResult<PyObject> {
        use pyo3_async_runtimes::tokio::future_into_py;
        future_into_py(self.py().unwrap(), async move {
            self.load_all_async(filters).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Loads specific instruments by their IDs asynchronously.
    /// 
    /// # Arguments
    /// * `instrument_ids` - List of instrument IDs to load
    /// * `filters` - Optional filters to apply to the instrument list
    /// 
    /// # Returns
    /// A list of loaded instruments
    #[pyo3(name = "load_ids_async")]
    pub fn load_ids_async_py(&self, instrument_ids: Vec<InstrumentId>, filters: Option<Vec<InstrumentFilter>>) -> PyResult<PyObject> {
        use pyo3_async_runtimes::tokio::future_into_py;
        future_into_py(self.py().unwrap(), async move {
            self.load_ids_async(instrument_ids, filters).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Loads a single instrument by its ID asynchronously.
    /// 
    /// # Arguments
    /// * `instrument_id` - The ID of the instrument to load
    /// * `filters` - Optional filters to apply to the instrument
    /// 
    /// # Returns
    /// The loaded instrument
    #[pyo3(name = "load_async")]
    pub fn load_async_py(&self, instrument_id: InstrumentId, filters: Option<Vec<InstrumentFilter>>) -> PyResult<PyObject> {
        use pyo3_async_runtimes::tokio::future_into_py;
        future_into_py(self.py().unwrap(), async move {
            self.load_async(instrument_id, filters).await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Discovers all instruments from the MT5 server (legacy method).
    /// 
    /// # Returns
    /// A list of discovered instruments
    #[pyo3(name = "discover_instruments")]
    pub fn discover_instruments_py(&self) -> PyResult<PyObject> {
        use pyo3_async_runtimes::tokio::future_into_py;
        future_into_py(self.py().unwrap(), async move {
            self.discover_instruments().await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }

    /// Loads all instruments (legacy method for backward compatibility).
    /// 
    /// # Returns
    /// A unit type indicating successful loading
    #[pyo3(name = "load_instruments")]
    pub fn load_instruments_py(&self) -> PyResult<PyObject> {
        use pyo3_async_runtimes::tokio::future_into_py;
        future_into_py(self.py().unwrap(), async move {
            self.load_instruments().await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        })
    }
}

// Mock implementation for Price::new - this would need the actual Nautilus types
#[derive(Debug, Clone, PartialEq)]
pub struct Price {
    pub value: f64,
    pub precision: u8,
}

impl Price {
    pub fn new(value: f64, precision: u8) -> Self {
        Self { value, precision }
    }
}

// Mock implementation for Instrument - this would use the actual Nautilus Instrument
#[derive(Debug, Clone, PartialEq)]
pub struct Instrument {
    pub instrument_id: InstrumentId,
    pub price: Price,
    pub volume_min: f64,
    pub volume_max: f64,
    pub volume_step: f64,
}

impl Instrument {
    pub fn new(
        instrument_id: InstrumentId,
        price: Price,
        volume_min: f64,
        volume_max: f64,
        volume_step: f64,
    ) -> Self {
        Self {
            instrument_id,
            price,
            volume_min,
            volume_max,
            volume_step,
        }
    }

    pub fn instrument_id(&self) -> &InstrumentId {
        &self.instrument_id
    }
}

// Mock implementation for InstrumentId
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct InstrumentId {
    pub symbol: String,
    pub venue: String,
}

impl InstrumentId {
    pub fn from_str(s: &str) -> Result<Self, String> {
        // Simple split for venue.symbol format
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() == 2 {
            Ok(Self {
                symbol: parts[0].to_string(),
                venue: parts[1].to_string(),
            })
        } else {
            Ok(Self {
                symbol: s.to_string(),
                venue: "MT5".to_string(),
            })
        }
    }
}

impl std::fmt::Display for InstrumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.symbol, self.venue)
    }
}