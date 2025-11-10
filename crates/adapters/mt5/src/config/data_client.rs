//! Configuration for MT5 Data Client.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5DataClientConfig {
    /// Base URL du bridge MT5 (ex: http://localhost:8000)
    pub base_url: String,
    /// URL WebSocket si utilisée (optionnelle pour le MVP)
    pub ws_url: Option<String>,
    /// Timeout HTTP en secondes
    pub http_timeout: u64,
    /// Identifiants MT5 (login/password/server)
    pub credential: crate::common::credential::Mt5Credential,
    /// Active la sortie de logs côté client
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