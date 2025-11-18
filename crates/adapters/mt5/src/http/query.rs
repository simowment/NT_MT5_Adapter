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
//! Structs for HTTP query parameters in the MetaTrader 5 adapter.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(setter(into, strip_option), default)]
pub struct AccountInfoParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
}

impl Default for AccountInfoParams {
    fn default() -> Self {
        Self {
            login: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(setter(into, strip_option), default)]
pub struct SymbolsInfoParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl Default for SymbolsInfoParams {
    fn default() -> Self {
        Self {
            symbol: None,
            group: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(setter(into, strip_option), default)]
pub struct RatesInfoParams {
    pub symbol: String,
    pub timeframe: String,
    pub from: Option<u64>,
    pub to: Option<u64>,
    pub count: Option<u32>,
}

impl Default for RatesInfoParams {
    fn default() -> Self {
        Self {
            symbol: "EURUSD".to_string(),
            timeframe: "M1".to_string(),
            from: None,
            to: None,
            count: Some(100),
        }
    }
}