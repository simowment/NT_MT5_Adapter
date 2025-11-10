//! Configuration for MT5 Execution Client.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5ExecutionClientConfig {
    /// Base URL du bridge MT5
    pub base_url: String,
    /// Timeout HTTP en secondes
    pub http_timeout: u64,
    /// Identifiants MT5 (login/password/server)
    pub credential: crate::common::credential::Mt5Credential,
    /// Nombre max d'ordres concurrents (basique)
    pub max_concurrent_orders: u32,
    /// Activer les logs
    pub enable_logging: bool,
    /// Mode simulation (pour backtests via bridge)
    pub simulate_orders: bool,
}

impl Default for Mt5ExecutionClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8000".to_string(),
            http_timeout: 30,
            credential: crate::common::credential::Mt5Credential::builder()
                .login("demo")
                .password("demo")
                .server("mt5-demo")
                .build()
                .unwrap(),
            max_concurrent_orders: 50,
            enable_logging: true,
            simulate_orders: true,
        }
    }
}

impl Mt5ExecutionClientConfig {
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