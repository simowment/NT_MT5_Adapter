// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Configuration structures for the MetaTrader 5 adapter.
//!
//! This module defines the configuration structures for the MT5 adapter,
//! including base configuration and specialized configurations for different
//! adapter components.

use serde::{Deserialize, Serialize};

pub mod instrument_provider;
pub mod data_client;
pub mod execution_client;

/// Main configuration for the MT5 adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5Config {
    /// The API key for authentication.
    pub api_key: String,
    /// The API secret for authentication.
    pub api_secret: String,
    /// The base URL for the MT5 REST API.
    pub base_url: String,
    /// The WebSocket URL for MT5 streaming.
    pub ws_url: String,
    /// HTTP timeout in seconds.
    pub http_timeout: u64,
    /// WebSocket timeout in seconds.
    pub ws_timeout: u64,
    /// Optional proxy URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
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
            proxy: None,
        }
    }
}

impl Mt5Config {
    /// Creates a new configuration with the specified credentials.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authentication
    /// * `api_secret` - The API secret for authentication
    ///
    /// # Returns
    ///
    /// A new configuration instance with the specified credentials.
    pub fn with_credentials(api_key: String, api_secret: String) -> Self {
        let mut config = Self::default();
        config.api_key = api_key;
        config.api_secret = api_secret;
        config
    }
}

// Re-exports for convenience
pub use instrument_provider::{Mt5InstrumentProviderConfig, Mt5Credentials};
pub use data_client::Mt5DataClientConfig;
pub use execution_client::Mt5ExecutionClientConfig;