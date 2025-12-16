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

//! Configuration for MT5 Data Client.

use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python-bindings", pyclass(get_all, set_all))]
pub struct Mt5DataClientConfig {
    /// Base URL of the MT5 REST API
    pub base_url: String,
    /// HTTP timeout in seconds
    pub http_timeout: u64,
    /// MT5 credentials
    pub credential: crate::common::credential::Mt5Credential,
    /// Enable client-side logging
    pub enable_logging: bool,
}

impl Default for Mt5DataClientConfig {
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

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5DataClientConfig {
    #[new]
    #[pyo3(signature = (mt5_base_url="http://localhost:5000".to_string(), http_timeout=30, enable_logging=true))]
    fn new(mt5_base_url: String, http_timeout: u64, enable_logging: bool) -> Self {
        let mut config = Self::default();
        config.base_url = mt5_base_url;
        config.http_timeout = http_timeout;
        config.enable_logging = enable_logging;
        config
    }
}