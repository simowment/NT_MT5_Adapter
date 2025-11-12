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

//! URL management for MetaTrader 5 API endpoints.

use std::fmt;

#[derive(Debug, Clone)]
pub struct Mt5Url {
    base_url: String,
}

impl Mt5Url {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn login_url(&self) -> String {
        format!("{}/api/login", self.base_url)
    }

    pub fn account_info_url(&self) -> String {
        format!("{}/api/account", self.base_url)
    }

    pub fn symbols_url(&self) -> String {
        format!("{}/api/symbols", self.base_url)
    }

    pub fn symbol_info_url(&self, symbol: &str) -> String {
        format!("{}/api/symbols/{}", self.base_url, symbol)
    }

    pub fn rates_url(&self) -> String {
        format!("{}/api/rates", self.base_url)
    }

    pub fn orders_url(&self) -> String {
        format!("{}/api/orders", self.base_url)
    }

    pub fn orders_by_id_url(&self, order_id: u64) -> String {
        format!("{}/api/orders/{}", self.base_url, order_id)
    }

    pub fn trades_url(&self) -> String {
        format!("{}/api/trades", self.base_url)
    }

    pub fn positions_url(&self) -> String {
        format!("{}/api/positions", self.base_url)
    }

    pub fn history_url(&self) -> String {
        format!("{}/api/history", self.base_url)
    }
}

impl fmt::Display for Mt5Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base_url)
    }
}
