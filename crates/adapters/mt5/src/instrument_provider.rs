//! MT5 Instrument Provider implementation.

use crate::config::{Mt5Config, Mt5InstrumentProviderConfig};
use crate::http::client::Mt5HttpClient;
use crate::http::error::{Mt5HttpError};
use crate::common::parse::InstrumentMetadata;
use crate::common::credential::Mt5Credential;
use crate::common::urls::Mt5Url;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

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

// Remove the problematic conversion implementation
// We'll handle the error conversion differently

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

    pub async fn discover_instruments(&self) -> Result<Vec<Instrument>, InstrumentProviderError> {
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
                instrument_type: crate::common::parse::InstrumentType::CurrencyPair {
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

    pub async fn load_instruments(&self) -> Result<(), InstrumentProviderError> {
        let instruments = self.discover_instruments().await?;
        
        for instrument in instruments {
            tracing::info!("Loaded instrument: {}", instrument.instrument_id());
        }
        
        Ok(())
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5InstrumentProvider {
    #[new]
    pub fn new_py(config: Mt5InstrumentProviderConfig) -> Result<Self, PyErr> {
        Self::new(config).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    pub async fn discover_instruments(&self) -> Result<Vec<Instrument>, PyErr> {
        self.discover_instruments().await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }

    pub async fn load_instruments(&self) -> Result<(), PyErr> {
        self.load_instruments().await.map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
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