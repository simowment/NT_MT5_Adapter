//! MT5 Instrument Provider implementation.

use crate::config::Mt5InstrumentProviderConfig;
use crate::http::client::Mt5HttpClient;
use crate::common::parse::InstrumentMetadata;
use nautilus_trader_core::clock::Clock;
use nautilus_trader_model::data::Data;
use nautilus_trader_model::instruments::Instrument;
use nautilus_trader_model::identifiers::InstrumentId;
use nautilus_trader_model::types::Price;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
}

pub struct Mt5InstrumentProvider {
    config: Mt5InstrumentProviderConfig,
    http_client: Arc<Mt5HttpClient>,
    cache: Arc<RwLock<HashMap<InstrumentId, Instrument>>>,
    clock: Clock,
}

impl Mt5InstrumentProvider {
    pub fn new(config: Mt5InstrumentProviderConfig) -> Result<Self, InstrumentProviderError> {
        let http_client = Arc::new(Mt5HttpClient::new(config.base_url.clone(), config.credentials.clone())?);
        
        Ok(Self {
            config,
            http_client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            clock: Clock::new(),
        })
    }

    pub async fn discover_instruments(&self) -> Result<Vec<Instrument>, InstrumentProviderError> {
        // Get all symbols from MT5
        let symbols = self.http_client.request_symbols().await
            .map_err(|e| InstrumentProviderError::ConnectionError(e.to_string()))?;

        // Filter instruments based on configuration
        let mut instruments = Vec::new();
        
        for symbol in symbols {
            if self.should_include_instrument(&symbol.symbol) {
                let metadata = self.parse_symbol_metadata(&symbol)?;
                let instrument = self.create_instrument(&metadata)?;
                instruments.push(instrument);
            }
        }

        // Cache the instruments
        {
            let mut cache = self.cache.write().await;
            for instrument in &instruments {
                cache.insert(instrument.instrument_id().clone(), instrument.clone());
            }
        }

        Ok(instruments)
    }

    pub async fn get_instrument(&self, instrument_id: &InstrumentId) -> Option<Instrument> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(instrument) = cache.get(instrument_id) {
                return Some(instrument.clone());
            }
        }

        // If not in cache, get from MT5
        match self.http_client.request_symbol_info(instrument_id.to_string()).await {
            Ok(symbol) => {
                let metadata = self.parse_symbol_metadata(&symbol)?;
                let instrument = self.create_instrument(&metadata)?;
                
                // Add to cache
                {
                    let mut cache = self.cache.write().await;
                    cache.insert(instrument_id.clone(), instrument.clone());
                }
                
                Some(instrument)
            }
            Err(_) => None,
        }
    }

    pub async fn load_instruments(&self) -> Result<(), InstrumentProviderError> {
        let instruments = self.discover_instruments().await?;
        
        for instrument in instruments {
            tracing::info!("Loaded instrument: {}", instrument.instrument_id());
        }
        
        Ok(())
    }

    pub fn should_include_instrument(&self, symbol: &str) -> bool {
        // Check currency filters
        if symbol.len() == 6 {
            let base = &symbol[..3];
            let quote = &symbol[3..];
            
            if self.config.filter_currencies.contains(&base.to_string()) || 
               self.config.filter_currencies.contains(&quote.to_string()) {
                return true;
            }
        }
        
        // Check index filters
        if self.config.filter_indices.contains(&symbol.to_string()) {
            return true;
        }
        
        // Check CFDs
        if self.config.filter_cfds && symbol.chars().all(|c| c.is_alphanumeric()) {
            return true;
        }
        
        // Check futures
        if self.config.filter_futures && symbol.len() >= 5 {
            let letters_part: String = symbol.chars().take(symbol.len() - 4).collect();
            let numbers_part: String = symbol.chars().skip(symbol.len() - 4).collect();
            
            if letters_part.chars().all(|c| c.is_alphabetic()) && 
               numbers_part.chars().all(|c| c.is_numeric()) {
                return true;
            }
        }
        
        false
    }

    fn parse_symbol_metadata(&self, symbol: &crate::http::models::Mt5Symbol) -> Result<InstrumentMetadata, InstrumentProviderError> {
        // Use the parsing logic from common/parse.rs
        let instrument_type = crate::common::parse::parse_instrument_symbol(&symbol.symbol)
            .map_err(|e| InstrumentProviderError::ParseError(e.to_string()))?;

        Ok(InstrumentMetadata {
            symbol: symbol.symbol.clone(),
            digits: symbol.digits,
            point_size: symbol.point_size,
            volume_min: symbol.volume_min,
            volume_max: symbol.volume_max,
            volume_step: symbol.volume_step,
            contract_size: symbol.contract_size,
            instrument_type,
        })
    }

    fn create_instrument(&self, metadata: &InstrumentMetadata) -> Result<Instrument, InstrumentProviderError> {
        // Create the appropriate Nautilus instrument based on type
        match &metadata.instrument_type {
            crate::common::parse::InstrumentType::CurrencyPair { base_currency, quote_currency } => {
                // Create FX currency pair
                let instrument_id = InstrumentId::from_str(&metadata.symbol).map_err(|e| {
                    InstrumentProviderError::ParseError(format!("Invalid instrument ID: {}", e))
                })?;

                // For now, use generic instrument - this would need to use actual Nautilus FX types
                let instrument = Instrument::new(
                    instrument_id,
                    Price::new(metadata.point_size, metadata.digits as u8),
                    metadata.volume_min,
                    metadata.volume_max,
                    metadata.volume_step,
                );

                Ok(instrument)
            }
            _ => {
                // Create generic instrument for CFDs, Futures, etc.
                let instrument_id = InstrumentId::from_str(&metadata.symbol).map_err(|e| {
                    InstrumentProviderError::ParseError(format!("Invalid instrument ID: {}", e))
                })?;

                let instrument = Instrument::new(
                    instrument_id,
                    Price::new(metadata.point_size, metadata.digits as u8),
                    metadata.volume_min,
                    metadata.volume_max,
                    metadata.volume_step,
                );

                Ok(instrument)
            }
        }
    }
}

// Mock implementation for Price::new - this would need the actual Nautilus types
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