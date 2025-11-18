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

//! Credential configuration for MetaTrader 5 connections.
//!
//! Note: The MT5 REST API uses MT5's existing terminal login session.
//! No additional authentication is required for the REST API itself.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

/// MT5 connection credentials.
///
/// These credentials are used to configure the MT5 terminal connection
/// but are not required for REST API authentication.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[cfg_attr(feature = "python-bindings", pyclass)]
#[builder(setter(into))]
pub struct Mt5Credential {
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub login: String,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub password: String,
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub server: String,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub proxy: Option<String>,
}

impl Mt5Credential {
    pub fn builder() -> Mt5CredentialBuilder {
        Mt5CredentialBuilder::default()
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5Credential {
    #[new]
    fn new(login: String, password: String, server: String) -> Self {
        Mt5Credential {
            login,
            password,
            server,
            proxy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_builder() {
        let cred = Mt5Credential::builder()
            .login("user123")
            .password("pass123")
            .server("mt5.example.com")
            .build()
            .unwrap();

        assert_eq!(cred.login, "user123");
        assert_eq!(cred.password, "pass123");
        assert_eq!(cred.server, "mt5.example.com");
        assert_eq!(cred.proxy, None);
    }
}
