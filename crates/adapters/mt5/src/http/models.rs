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

//! Data models for MetaTrader 5 REST API responses.
//!
//! The MT5 REST API returns responses in the format:
//! - Success: `{"result": <data>}`
//! - Error: `{"error": "error message"}`
//!
//! Most responses are handled as raw `serde_json::Value` to maintain
//! flexibility with the MT5 Python API format.

use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

/// MT5 Symbol information from REST API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python-bindings", pyclass(get_all, set_all))]
pub struct Mt5Symbol {
    pub symbol: String,
    pub digits: u32,
    pub point_size: f64,
    pub volume_min: f64,
    pub volume_max: f64,
    pub volume_step: f64,
    pub contract_size: f64,
    pub margin_initial: Option<f64>,
    pub margin_maintenance: Option<f64>,
    #[serde(rename = "type")]
    pub symbol_type: String,
}

/// Standard MT5 REST API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Mt5Response<T> {
    Success { result: T },
    Error { error: String },
}

impl<T> Mt5Response<T> {
    pub fn into_result(self) -> Result<T, String> {
        match self {
            Mt5Response::Success { result } => Ok(result),
            Mt5Response::Error { error } => Err(error),
        }
    }
}
