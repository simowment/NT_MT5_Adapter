//! HTTP client implementation for MetaTrader 5 REST API.
//! 
//! This module implements the inner/outer client pattern with Arc wrapping
//! for efficient cloning in Python bindings while keeping HTTP logic centralized.

use std::sync::Arc;
use crate::common::credential::Mt5Credential;
use crate::common::urls::Mt5Url;
use crate::config::Mt5Config;
use crate::http::query::{AccountInfoParams, SymbolsInfoParams, RatesInfoParams};
use crate::http::models::{Mt5AccountInfo, Mt5Symbol, Mt5Rate};
use crate::http::parse::{parse_account_info, parse_symbols, parse_rates};
use thiserror::Error;

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Request error: {0}")]
    RequestError(String),
    #[error("Response error: {0}")]
    ResponseError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Inner client - contains actual HTTP logic
pub struct Mt5HttpInnerClient {
    config: Mt5Config,
    credential: Mt5Credential,
    url: Mt5Url,
    // HttpClient would be added here when using nautilus_network::http::HttpClient
}

// Outer client - wraps inner with Arc for cheap cloning (needed for Python)
#[cfg_attr(feature = "python-bindings", pyclass)]
pub struct Mt5HttpClient {
    pub(crate) inner: Arc<Mt5HttpInnerClient>,
}

impl Mt5HttpInnerClient {
    pub fn new(config: Mt5Config, credential: Mt5Credential, url: Mt5Url) -> Self {
        Self {
            config,
            credential,
            url,
        }
    }

    // HTTP low-level API calls (prefixed with http_)
    
    pub async fn http_get_account_info(&self, params: AccountInfoParams) -> Result<String, HttpClientError> {
        // This would make actual HTTP request in real implementation
        // For now, returning mock data
        Ok(format!(r#"{{"login": "{}", "balance": 10000, "equity": 10000, "margin": 0, "marginFree": 10000, "marginLevel": 0}}"#, 
            self.credential.login))
    }

    pub async fn http_get_symbols(&self, params: SymbolsInfoParams) -> Result<String, HttpClientError> {
        // This would make actual HTTP request in real implementation
        // For now, returning mock data
        Ok(r#"[{"symbol": "EURUSD", "digits": 5, "pointSize": 0.00001, "volumeMin": 0.01, "volumeMax": 10, "volumeStep": 0.01, "contractSize": 10000, "marginInitial": 0.03, "marginMaintenance": 0.03}]"#.to_string())
    }

    pub async fn http_get_rates(&self, params: RatesInfoParams) -> Result<String, HttpClientError> {
        // This would make actual HTTP request in real implementation
        // For now, returning mock data
        Ok(r#"[{"symbol": "EURUSD", "time": 1678886400, "open": 1.08, "high": 1.09, "low": 1.07, "close": 1.085, "tickVolume": 10}]"#.to_string())
    }

    // High-level domain methods (no prefix)
    
    pub async fn get_account_info(&self) -> Result<Mt5AccountInfo, HttpClientError> {
        let params = AccountInfoParams::default();
        let response = self.http_get_account_info(params).await?;
        parse_account_info(&response).map_err(|e| HttpClientError::ParseError(e.to_string()))
    }

    pub async fn get_symbols(&self) -> Result<Vec<Mt5Symbol>, HttpClientError> {
        let params = SymbolsInfoParams::default();
        let response = self.http_get_symbols(params).await?;
        parse_symbols(&response).map_err(|e| HttpClientError::ParseError(e.to_string()))
    }

    pub async fn get_rates(&self, symbol: &str) -> Result<Vec<Mt5Rate>, HttpClientError> {
        let mut params = RatesInfoParams::default();
        params.symbol = symbol.to_string();
        let response = self.http_get_rates(params).await?;
        parse_rates(&response).map_err(|e| HttpClientError::ParseError(e.to_string()))
    }
}

impl Mt5HttpClient {
    pub fn new(config: Mt5Config, credential: Mt5Credential, url: Mt5Url) -> Self {
        Self {
            inner: Arc::new(Mt5HttpInnerClient::new(config, credential, url)),
        }
    }

    // Delegate all methods to the inner client
    pub async fn get_account_info(&self) -> Result<Mt5AccountInfo, HttpClientError> {
        self.inner.get_account_info().await
    }

    pub async fn get_symbols(&self) -> Result<Vec<Mt5Symbol>, HttpClientError> {
        self.inner.get_symbols().await
    }

    pub async fn get_rates(&self, symbol: &str) -> Result<Vec<Mt5Rate>, HttpClientError> {
        self.inner.get_rates(symbol).await
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5HttpClient {
    #[new]
    fn new_py(config: Mt5Config, credential: Mt5Credential, url: Mt5Url) -> Self {
        Self::new(config, credential, url)
    }
    
    async fn py_get_account_info(&self) -> Result<Mt5AccountInfo, HttpClientError> {
        self.get_account_info().await
    }
    
    async fn py_get_symbols(&self) -> Result<Vec<Mt5Symbol>, HttpClientError> {
        self.get_symbols().await
    }
    
    async fn py_get_rates(&self, symbol: &str) -> Result<Vec<Mt5Rate>, HttpClientError> {
        self.get_rates(symbol).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_client_creation() {
        let config = Mt5Config::default();
        let cred = Mt5Credential::builder()
            .login("user")
            .password("pass")
            .server("server")
            .build()
            .unwrap();
        let url = Mt5Url::new("http://localhost");

        let client = Mt5HttpClient::new(config, cred, url);

        // Test that the client was created successfully
        assert!(client.get_account_info().await.is_ok());
    }
}