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

//! Credential management for MetaTrader 5 connections.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac};
use sha2::Sha256;

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

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
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "python-bindings", pyo3(get, set))]
    pub token: Option<String>,
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
            token: None,
        }
    }
}

/// MT5 API credentials for signing requests (similar to BitMEX).
///
/// This structure provides HMAC SHA256 signing capabilities similar to the BitMEX API.
#[derive(Debug, Clone)]
pub struct Mt5SignedCredential {
    pub api_key: String,
    pub api_secret: String, // Store as String for now, can be converted to bytes when needed
}

impl Mt5SignedCredential {
    /// Creates a new [`Mt5SignedCredential`] instance.
    #[must_use]
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
        }
    }

    /// Signs a request message according to the BitMEX authentication scheme using HMAC SHA256.
    ///
    /// # Arguments
    ///
    /// * `verb` - HTTP verb (GET, POST, etc.)
    /// * `endpoint` - API endpoint path (e.g., /api/login)
    /// * `expires` - Expiration timestamp (Unix timestamp)
    /// * `data` - Request body data (JSON string or empty)
    ///
    /// # Returns
    ///
    /// Base64-encoded HMAC SHA256 signature
    #[must_use]
    pub fn sign(&self, verb: &str, endpoint: &str, expires: i64, data: &str) -> String {
        use base64::{Engine as _, engine::general_purpose};
        
        let message = format!("{}{}{}{}", verb, endpoint, expires, data);
        
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        
        general_purpose::STANDARD.encode(mac.finalize().into_bytes())
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
        assert_eq!(cred.token, None);
    }
}
