//! Configuration structures for the MetaTrader 5 adapter.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5Config {
    pub api_key: String,
    pub api_secret: String,
    pub base_url: String,
    pub ws_url: String,
    pub http_timeout: u64,
    pub ws_timeout: u64,
}

impl Default for Mt5Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_secret: String::new(),
            base_url: "https://mt5.example.com".to_string(),
            ws_url: "wss://mt5.example.com".to_string(),
            http_timeout: 30,
            ws_timeout: 30,
        }
    }
}