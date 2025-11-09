//! Credential management for MetaTrader 5 connections.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

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
