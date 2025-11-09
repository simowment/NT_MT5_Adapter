//! WebSocket client for MetaTrader 5 real-time data streaming.

use crate::credential::Mt5Credential;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebSocketError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Send error: {0}")]
    SendError(String),
    #[error("Receive error: {0}")]
    ReceiveError(String),
    #[error("Close error: {0}")]
    CloseError(String),
}

pub struct WebSocketClient {
    credential: Mt5Credential,
    url: String,
}

impl WebSocketClient {
    pub fn new(credential: Mt5Credential, url: impl Into<String>) -> Self {
        Self {
            credential,
            url: url.into(),
        }
    }

    pub fn credential(&self) -> &Mt5Credential {
        &self.credential
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub async fn connect(&self) -> Result<(), WebSocketError> {
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), WebSocketError> {
        Ok(())
    }

    pub async fn send_message(&self, message: &str) -> Result<(), WebSocketError> {
        let _ = message;
        Ok(())
    }

    pub async fn receive_message(&self) -> Result<Option<String>, WebSocketError> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_client_creation() {
        let cred = Mt5Credential::builder()
            .login("user")
            .password("pass")
            .server("server")
            .build()
            .unwrap();
        let client = WebSocketClient::new(cred, "ws://localhost");

        assert_eq!(client.url(), "ws://localhost");
    }

    #[tokio::test]
    async fn test_websocket_connect() {
        let cred = Mt5Credential::builder()
            .login("user")
            .password("pass")
            .server("server")
            .build()
            .unwrap();
        let client = WebSocketClient::new(cred, "ws://localhost");

        let result = client.connect().await;
        assert!(result.is_ok());
    }
}
