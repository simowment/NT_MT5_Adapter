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

//! Python bindings for the MetaTrader 5 adapter using PyO3.
//!
//! This module re-exports Rust functionality to Python through PyO3,
//! making it available to the Python layer of the adapter.

#![allow(clippy::needless_pass_by_value)]

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
#[pymodule]
pub fn nautilus_mt5(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    // Add configuration classes
    m.add_class::<crate::config::Mt5Config>()?;
    m.add_class::<crate::config::instrument_provider::Mt5InstrumentProviderConfig>()?;
    m.add_class::<crate::config::data_client::Mt5DataClientConfig>()?;
    m.add_class::<crate::config::execution_client::Mt5ExecutionClientConfig>()?;

    // Add common types
    m.add_class::<crate::common::credential::Mt5Credential>()?;

    // Add HTTP-related types
    m.add_class::<crate::http::client::Mt5HttpClient>()?;
    m.add_class::<crate::http::models::Mt5Symbol>()?;

    // Add the main client classes
    m.add_class::<crate::data_client::Mt5DataClient>()?;
    m.add_class::<crate::execution_client::Mt5ExecutionClient>()?;
    m.add_class::<crate::instrument_provider::Mt5InstrumentProvider>()?;

    Ok(())
}
