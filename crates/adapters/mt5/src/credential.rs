//! Credential management for MetaTrader 5 connections.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(setter(into))]
pub struct Mt5Credential {
    pub login: String,
    pub password: String,
    pub server: String,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
}

impl Mt5Credential {
    pub fn builder() -> Mt5CredentialBuilder {
        Mt5CredentialBuilder::default()
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
