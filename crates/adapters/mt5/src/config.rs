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
//! This module defines the configuration for the MT5 REST API adapter.
//! The credentials (login/password/server) are carried by `Mt5Credential` (common/credential.rs).

use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

pub mod instrument_provider;
pub mod data_client;
pub mod execution_client;

/// Main configuration for the MT5 adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python-bindings", pyclass)]
pub struct Mt5Config {
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    /// The base URL for the MT5 REST API (e.g., "http://localhost:5000").
    pub base_url: String,
    /// HTTP timeout in seconds.
    pub http_timeout: u64,
    /// Optional proxy URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
}

impl Default for Mt5Config {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:5000".to_string(),
            http_timeout: 30,
            proxy: None,
        }
    }
}

impl Mt5Config {
    /// Creates a new configuration with the specified base URL.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL for the MT5 REST API
    ///
    /// # Returns
    ///
    /// A new configuration instance with the specified URL.
    pub fn with_base_url(base_url: String) -> Self {
        let mut config = Self::default();
        config.base_url = base_url;
        config
    }
}

// Re-exports for convenience
pub use instrument_provider::Mt5InstrumentProviderConfig;
pub use data_client::Mt5DataClientConfig;
pub use execution_client::Mt5ExecutionClientConfig;