//! Configuration for MT5 Instrument Provider.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5InstrumentProviderConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub ws_url: String,
    pub http_timeout: u64,
    pub ws_timeout: u64,
    pub credentials: Mt5Credentials,
    pub filter_currencies: Vec<String>,
    pub filter_indices: Vec<String>,
    pub filter_futures: bool,
    pub filter_cfds: bool,
    pub auto_discover_instruments: bool,
    pub cache_expiry: u32, // Cache expiry in seconds
    pub enable_logging: bool,
}

impl Default for Mt5InstrumentProviderConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            base_url: "http://localhost:8080".to_string(),
            ws_url: "ws://localhost:8080".to_string(),
            http_timeout: 30,
            ws_timeout: 30,
            credentials: Mt5Credentials::default(),
            filter_currencies: vec![
                "USD".to_string(),
                "EUR".to_string(),
                "GBP".to_string(),
                "JPY".to_string(),
                "CHF".to_string(),
                "CAD".to_string(),
                "AUD".to_string(),
                "NZD".to_string(),
            ],
            filter_indices: vec![
                "US30".to_string(),
                "SPX500".to_string(),
                "NAS100".to_string(),
                "UK100".to_string(),
                "GER30".to_string(),
                "FRA40".to_string(),
                "JPN225".to_string(),
                "AUS200".to_string(),
            ],
            filter_futures: false,
            filter_cfds: true,
            auto_discover_instruments: true,
            cache_expiry: 300, // 5 minutes
            enable_logging: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5Credentials {
    pub login: String,
    pub password: String,
    pub server: String,
}

impl Default for Mt5Credentials {
    fn default() -> Self {
        Self {
            login: "".to_string(),
            password: "".to_string(),
            server: "".to_string(),
        }
    }
}

impl Mt5Credentials {
    pub fn new(login: String, password: String, server: String) -> Self {
        Self {
            login,
            password,
            server,
        }
    }
}