// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
// Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Python bindings for the MetaTrader 5 adapter using PyO3.
//!
//! This module re-exports Rust functionality to Python through PyO3,
//! making it available to the Python layer of the adapter.

#![allow(clippy::needless_pass_by_value)]

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
#[pymodule]
pub fn nautilus_adapters_mt5(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    // Add configuration classes
    m.add_class::<crate::config::Mt5Config>()?;

    // Add common types
    m.add_class::<crate::common::credential::Mt5Credential>()?;

    // Add HTTP-related types
    m.add_class::<crate::http::client::Mt5HttpClient>()?;
    m.add_class::<crate::http::models::Mt5AccountInfo>()?;
    m.add_class::<crate::http::models::Mt5Symbol>()?;
    m.add_class::<crate::http::models::Mt5Rate>()?;
    m.add_class::<crate::http::models::Mt5OrderRequest>()?;
    m.add_class::<crate::http::models::Mt5OrderResponse>()?;
    m.add_class::<crate::http::models::Mt5Position>()?;
    m.add_class::<crate::http::models::Mt5Trade>()?;

    // Add WebSocket-related types
    m.add_class::<crate::websocket::client::Mt5WebSocketClient>()?;

    // Add query parameter builders
    m.add_class::<crate::http::query::AccountInfoParamsBuilder>()?;
    m.add_class::<crate::http::query::SymbolsInfoParamsBuilder>()?;
    m.add_class::<crate::http::query::RatesInfoParamsBuilder>()?;

    Ok(())
}
