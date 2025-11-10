//! Configuration for MT5 Data Client.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5DataClientConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub ws_url: String,
    pub http_timeout: u64,
    pub ws_timeout: u64,
    pub credentials: super::instrument_provider::Mt5Credentials,
    pub subscribe_quotes: bool,
    pub subscribe_trades: bool,
    pub subscribe_order_book: bool,
    pub subscribe_instrument_status: bool,
    pub max_subscriptions: u32,
    pub connection_retry_attempts: u32,
    pub connection_retry_delay: u64, // seconds
    pub heartbeat_interval: u64, // seconds
    pub enable_ping_pong: bool,
    pub reconnection_enabled: bool,
    pub snapshot_frequency: u32, // seconds
    pub enable_logging: bool,
}

impl Default for Mt5DataClientConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            base_url: "http://localhost:8080".to_string(),
            ws_url: "ws://localhost:8080".to_string(),
            http_timeout: 30,
            ws_timeout: 30,
            credentials: super::instrument_provider::Mt5Credentials::default(),
            subscribe_quotes: true,
            subscribe_trades: true,
            subscribe_order_book: false,
            subscribe_instrument_status: true,
            max_subscriptions: 1000,
            connection_retry_attempts: 3,
            connection_retry_delay: 5,
            heartbeat_interval: 30,
            enable_ping_pong: true,
            reconnection_enabled: true,
            snapshot_frequency: 60,
            enable_logging: true,
        }
    }
}

impl Mt5DataClientConfig {
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