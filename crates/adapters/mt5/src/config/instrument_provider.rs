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
//! Configuration for MT5 Instrument Provider.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5InstrumentProviderConfig {
    pub base_url: String,
    pub http_timeout: Option<u64>,
    pub credential: crate::common::credential::Mt5Credential,
    pub filter_currencies: Vec<String>,
    pub filter_indices: Vec<String>,
    pub filter_futures: bool,
    pub filter_cfds: bool,
    pub auto_discover_instruments: bool,
    pub cache_expiry: u32, // Cache expiry in seconds
    pub enable_logging: bool,
}

impl Default for Mt5InstrumentProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            http_timeout: Some(30),
            credential: crate::common::credential::Mt5Credential::builder()
                .login("demo")
                .password("demo")
                .server("mt5-demo")
                .build()
                .unwrap(),
            filter_currencies: vec![
                "USD".to_string(),
                "EUR".to_string(),
                "GBP".to_string(),
                "JPY".to_string(),
                "CHF".to_string(),
                "CAD".to_string(),
                "AUD".to_string(),
                "NZD".to_string(),
            ],
            filter_indices: vec![
                "US30".to_string(),
                "SPX500".to_string(),
                "NAS100".to_string(),
                "UK100".to_string(),
                "GER30".to_string(),
                "FRA40".to_string(),
                "JPN225".to_string(),
                "AUS200".to_string(),
            ],
            filter_futures: false,
            filter_cfds: true,
            auto_discover_instruments: true,
            cache_expiry: 300, // 5 minutes
            enable_logging: true,
        }
    }
}
