//! Configuration for MT5 Data Client.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5DataClientConfig {
    /// Base URL of the MT5 bridge (ex: http://localhost:8000)
    pub base_url: String,
    /// WebSocket URL if used (optional for MVP)
    pub ws_url: Option<String>,
    /// HTTP timeout in seconds
    pub http_timeout: u64,
    /// MT5 credentials (login/password/server)
    pub credential: crate::common::credential::Mt5Credential,
    /// Enable client-side logging output
    pub enable_logging: bool,
}

impl Default for Mt5DataClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8000".to_string(),
            ws_url: None,
            http_timeout: 30,
            credential: crate::common::credential::Mt5Credential::builder()
                .login("demo")
                .password("demo")
                .server("mt5-demo")
                .build()
                .unwrap(),
            enable_logging: true,
        }
    }
}

impl Mt5DataClientConfig {
    pub fn with_credentials(login: String, password: String, server: String) -> Self {
        let mut config = Self::default();
        config.credential = crate::common::credential::Mt5Credential::builder()
            .login(login)
            .password(password)
            .server(server)
            .build()
            .unwrap();
        config
    }
}