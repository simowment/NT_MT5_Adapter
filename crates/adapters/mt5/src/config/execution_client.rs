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

//! Configuration for MT5 Execution Client.

use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python-bindings", pyclass)]
pub struct Mt5ExecutionClientConfig {
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    /// Base URL of the MT5 REST API
    pub base_url: String,
    /// HTTP timeout in seconds
    pub http_timeout: u64,
    /// MT5 credentials
    pub credential: crate::common::credential::Mt5Credential,
    /// Maximum number of concurrent orders
    pub max_concurrent_orders: u32,
    /// Enable logging
    pub enable_logging: bool,
    /// Simulation mode for backtesting
    pub simulate_orders: bool,
}

impl Default for Mt5ExecutionClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:5000".to_string(),
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