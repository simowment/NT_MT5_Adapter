//! Configuration for MT5 Execution Client.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5ExecutionClientConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub ws_url: String,
    pub http_timeout: u64,
    pub ws_timeout: u64,
    pub credentials: super::instrument_provider::Mt5Credentials,
    pub max_concurrent_orders: u32,
    pub order_timeout: u64, // seconds
    pub connection_retry_attempts: u32,
    pub connection_retry_delay: u64, // seconds
    pub enable_partial_fills: bool,
    pub enable_market_data: bool,
    pub risk_management_enabled: bool,
    pub position_sizing_enabled: bool,
    pub enable_logging: bool,
    pub simulate_orders: bool, // For backtesting
}

impl Default for Mt5ExecutionClientConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            base_url: "http://localhost:8080".to_string(),
            ws_url: "ws://localhost:8080".to_string(),
            http_timeout: 30,
            ws_timeout: 30,
            credentials: super::instrument_provider::Mt5Credentials::default(),
            max_concurrent_orders: 50,
            order_timeout: 30,
            connection_retry_attempts: 3,
            connection_retry_delay: 5,
            enable_partial_fills: true,
            enable_market_data: true,
            risk_management_enabled: true,
            position_sizing_enabled: true,
            enable_logging: true,
            simulate_orders: false,
        }
    }
}

impl Mt5ExecutionClientConfig {
    pub fn with_credentials(
        login: String,
        password: String,
        server: String,
    ) -> Self {
        let mut config = Self::default();
        config.credentials = super::instrument_provider::Mt5Credentials::new(login, password, server);
        config
    }
}