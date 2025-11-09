//! HTTP client for MetaTrader 5 REST API.

use crate::credential::Mt5Credential;
use crate::urls::Mt5Url;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Request error: {0}")]
    RequestError(String),
    #[error("Response error: {0}")]
    ResponseError(String),
}

pub struct HttpClient {
    credential: Mt5Credential,
    url: Mt5Url,
}

impl HttpClient {
    pub fn new(credential: Mt5Credential, url: Mt5Url) -> Self {
        Self { credential, url }
    }

    pub fn credential(&self) -> &Mt5Credential {
        &self.credential
    }

    pub fn url(&self) -> &Mt5Url {
        &self.url
    }

    pub async fn get_account_info(&self) -> Result<String, HttpClientError> {
        Ok("account_info".to_string())
    }

    pub async fn get_symbols(&self) -> Result<String, HttpClientError> {
        Ok("symbols".to_string())
    }

    pub async fn get_rates(&self) -> Result<String, HttpClientError> {
        Ok("rates".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_client_creation() {
        let cred = Mt5Credential::builder()
            .login("user")
            .password("pass")
            .server("server")
            .build()
            .unwrap();
        let url = Mt5Url::new("http://localhost");
        let client = HttpClient::new(cred, url);

        assert_eq!(client.credential().login, "user");
    }

    #[tokio::test]
    async fn test_get_account_info() {
        let cred = Mt5Credential::builder()
            .login("user")
            .password("pass")
            .server("server")
            .build()
            .unwrap();
        let url = Mt5Url::new("http://localhost");
        let client = HttpClient::new(cred, url);

        let result = client.get_account_info().await;
        assert!(result.is_ok());
    }
}
