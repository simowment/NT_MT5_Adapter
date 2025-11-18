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
 //! This module defines the configuration structures for the MT5 adapter.
 //! Mt5Config describes the HTTP endpoints and timeouts of the MT5 bridge.
 //! The credentials (login/password/server) are carried by `Mt5Credential` (common/credential.rs).
//! Configuration for MT5 Data Client.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5DataClientConfig {
    /// Base URL of the MT5 bridge (ex: http://localhost:8000)
    pub base_url: String,
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