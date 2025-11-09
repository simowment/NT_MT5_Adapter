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

    pub fn account_info_url(&self) -> String {
        format!("{}/account/info", self.base_url)
    }

    pub fn symbols_url(&self) -> String {
        format!("{}/symbols", self.base_url)
    }

    pub fn rates_url(&self) -> String {
        format!("{}/rates", self.base_url)
    }
}

impl fmt::Display for Mt5Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base_url)
    }
}
