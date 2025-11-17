// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
// Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! WebSocket client implementation for MetaTrader 5 real-time data streaming.
//!
//! This module handles connection state, authentication, subscriptions, and reconnection logic.

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error};

use crate::common::credential::Mt5Credential;
use crate::websocket::messages::{WsPong};
use crate::websocket::parse::WsMessage;
use thiserror::Error;

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;
#[cfg(feature = "python-bindings")]
use pyo3_async_runtimes::tokio::future_into_py;

#[derive(Debug, Error)]
pub enum WebSocketError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),

    #[error("Message parse error: {0}")]
    ParseError(String),

    #[error("Send error: {0}")]
    SendError(String),

    #[error("Receive error: {0}")]
    ReceiveError(String),
}

impl WebSocketError {
    pub fn is_reconnectable(&self) -> bool {
        matches!(
            self,
            WebSocketError::ConnectionFailed(_) | WebSocketError::ReceiveError(_)
        )
    }
}

// WebSocket client for MT5.
// Handles:
// - Connection/authentication
// - Subscription lifecycle (pending/confirmed)
// - Reconnection
// - Ping/Pong
// - Message routing
//
// Designed to be cloned and shared between tasks (Arc<Mutex<>>).
#[cfg_attr(feature = "python-bindings", pyclass)]
pub struct Mt5WebSocketClient {
    #[cfg_attr(feature = "python-bindings", pyo3(get))]
    credential: Mt5Credential,
    #[cfg_attr(feature = "python-bindings", pyo3(get))]
    url: String,
    authenticated: Arc<Mutex<bool>>,
    pending_subscriptions: Arc<Mutex<HashSet<String>>>,
    confirmed_subscriptions: Arc<Mutex<HashSet<String>>>,
    // Additional connection state fields would go here
}

impl Mt5WebSocketClient {
    pub fn new(credential: Mt5Credential, url: impl Into<String>) -> Self {
        Self {
            credential,
            url: url.into(),
            authenticated: Arc::new(Mutex::new(false)),
            pending_subscriptions: Arc::new(Mutex::new(HashSet::new())),
            confirmed_subscriptions: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn credential(&self) -> &Mt5Credential {
        &self.credential
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub async fn connect(&self) -> Result<(), WebSocketError> {
        // In a real implementation, this would establish the WebSocket connection
        // For now, just setting the authenticated flag to true
        *self.authenticated.lock().await = true;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), WebSocketError> {
        // In a real implementation, this would close the WebSocket connection
        *self.authenticated.lock().await = false;
        Ok(())
    }

    pub async fn is_authenticated(&self) -> bool {
        *self.authenticated.lock().await
    }

    // Subscription management
    pub async fn subscribe(&self, topic: &str) {
        self.pending_subscriptions.lock().await.insert(topic.to_string());
    }

    pub async fn unsubscribe(&self, topic: &str) {
        let mut confirmed = self.confirmed_subscriptions.lock().await;
        let mut pending = self.pending_subscriptions.lock().await;
        
        if confirmed.contains(topic) {
            // Move from confirmed to pending for unsubscribe
            confirmed.remove(topic);
            pending.insert(topic.to_string());
        } else {
            // Remove from pending if it was only pending
            pending.remove(topic);
        }
    }

    pub async fn confirm_subscription(&self, topic: &str) {
        let mut pending = self.pending_subscriptions.lock().await;
        let mut confirmed = self.confirmed_subscriptions.lock().await;
        
        if pending.contains(topic) {
            pending.remove(topic);
            confirmed.insert(topic.to_string());
        }
    }

    pub async fn subscription_count(&self) -> usize {
        self.confirmed_subscriptions.lock().await.len()
    }

    // Specific subscription methods for data client
    pub async fn subscribe_quotes(&self, symbol: &str) -> Result<(), WebSocketError> {
        let topic = format!("quote:{}", symbol);
        self.subscribe(&topic).await;
        Ok(())
    }

    pub async fn subscribe_trades(&self, symbol: &str) -> Result<(), WebSocketError> {
        let topic = format!("trade:{}", symbol);
        self.subscribe(&topic).await;
        Ok(())
    }

    pub async fn subscribe_order_book(&self, symbol: &str) -> Result<(), WebSocketError> {
        let topic = format!("orderbook:{}", symbol);
        self.subscribe(&topic).await;
        Ok(())
    }

    pub async fn mark_subscription_failure(&self, _topic: &str) {
        // Keep it in pending for retry on reconnect
        // The topic should already be in pending_subscriptions
    }

    // Authentication methods
    pub async fn authenticate(&self) -> Result<(), WebSocketError> {
        // In a real implementation, this would send authentication request
        // and wait for response
        *self.authenticated.lock().await = true;
        Ok(())
    }

    // Reconnection logic
    pub async fn reconnect(&self) -> Result<(), WebSocketError> {
        // Disconnect first
        let _ = self.disconnect().await;
        
        // Connect again
        self.connect().await?;
        
        // Re-authenticate
        self.authenticate().await?;
        
        // Restore subscriptions (only public ones in real implementation)
        let confirmed_topics: Vec<String> = {
            let confirmed = self.confirmed_subscriptions.lock().await;
            confirmed.iter().cloned().collect()
        };
        
        for topic in confirmed_topics {
            self.subscribe(&topic).await;
        }
        
        // Also restore pending subscriptions
        // In real implementation, you'd need to send subscription requests again
        Ok(())
    }

    // Ping/Pong handling
    pub fn handle_ping(&self) -> WsPong {
        WsPong {
            op: "pong".to_string(),
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    // Message routing
    pub async fn route_message(&self, message: WsMessage) -> Result<(), WebSocketError> {
        match message {
            WsMessage::Ping(_ping) => {
                // Respond to ping
                let pong = self.handle_ping();
                // In real implementation, would send pong back
                println!("Received ping, sending pong: {:?}", pong);
            },
            WsMessage::AuthResponse(auth_resp) => {
                if auth_resp.result {
                    *self.authenticated.lock().await = true;
                    println!("Authentication successful");
                } else {
                    *self.authenticated.lock().await = false;
                    return Err(WebSocketError::AuthenticationFailed(
                        auth_resp.msg.unwrap_or_else(|| "Authentication failed".to_string())
                    ));
                }
            },
            WsMessage::SubscriptionResponse(sub_resp) => {
                if sub_resp.result {
                    self.confirm_subscription(&sub_resp.topic).await;
                    println!("Subscription confirmed for topic: {}", sub_resp.topic);
                } else {
                    self.mark_subscription_failure(&sub_resp.topic).await;
                    println!("Subscription failed for topic: {}", sub_resp.topic);
                }
            },
            WsMessage::Quote(quote) => {
                // Handle quote message
                println!("Received quote: {} bid: {} ask: {}", quote.symbol, quote.bid, quote.ask);
            },
            WsMessage::Trade(trade) => {
                // Handle trade message
                println!("Received trade: {} price: {} volume: {}", trade.symbol, trade.price, trade.volume);
            },
            WsMessage::OrderBook(orderbook) => {
                // Handle order book message
                println!("Received order book for: {} bids: {} asks: {}",
                    orderbook.symbol, orderbook.bids.len(), orderbook.asks.len());
            },
            _ => {
                // Other message types handled as needed
                println!("Received other message type");
            }
        }
        Ok(())
    }

    pub fn should_reconnect(&self, error: &WebSocketError) -> bool {
        error.is_reconnectable()
    }
}

#[cfg(feature = "python-bindings")]
#[pymethods]
impl Mt5WebSocketClient {
    #[new]
    fn new_py(credential: Mt5Credential, url: &str) -> Self {
        Self::new(credential, url)
    }
    
    #[pyo3(name = "connect")]
    fn py_connect(&mut self) -> PyResult<PyObject> {
        // Use pyo3_async_runtimes to convert Rust future to Python awaitable
        future_into_py(self.py().unwrap(), async move {
            self.connect().await
        })
    }
    
    #[pyo3(name = "disconnect")]
    fn py_disconnect(&mut self) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.disconnect().await
        })
    }
    
    #[pyo3(name = "subscribe")]
    fn py_subscribe(&mut self, topic: &str) {
        Python::with_gil(|py| {
            future_into_py(py, async move {
                self.subscribe(topic).await;
                Ok(())
            })
        })
    }
    
    #[pyo3(name = "unsubscribe")]
    fn py_unsubscribe(&mut self, topic: &str) {
        Python::with_gil(|py| {
            future_into_py(py, async move {
                self.unsubscribe(topic).await;
                Ok(())
            })
        })
    }
    
    #[pyo3(name = "is_authenticated")]
    fn py_is_authenticated(&self) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.is_authenticated().await
        })
    }
    
    #[pyo3(name = "subscription_count")]
    fn py_subscription_count(&self) -> PyResult<PyObject> {
        future_into_py(self.py().unwrap(), async move {
            self.subscription_count().await
        })
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
        let client = Mt5WebSocketClient::new(cred, "ws://localhost");

        assert_eq!(client.url, "ws://localhost");
        // CORRECTION: is_authenticated() is async, requires await
        assert!(!client.is_authenticated().await);
    }

    #[tokio::test]
    async fn test_subscription_lifecycle() {
        let cred = Mt5Credential::builder()
            .login("user")
            .password("pass")
            .server("server")
            .build()
            .unwrap();
        let client = Mt5WebSocketClient::new(cred, "ws://localhost");

        // Test subscription
        client.subscribe("EURUSD").await;
        assert!(client.pending_subscriptions.lock().await.contains("EURUSD"));
        assert_eq!(client.subscription_count().await, 0);

        // Test confirmation
        client.confirm_subscription("EURUSD").await;
        assert!(!client.pending_subscriptions.lock().await.contains("EURUSD"));
        assert!(client.confirmed_subscriptions.lock().await.contains("EURUSD"));
        assert_eq!(client.subscription_count().await, 1);

        // Test unsubscription
        client.unsubscribe("EURUSD").await;
        assert!(client.pending_subscriptions.lock().await.contains("EURUSD"));
        assert!(!client.confirmed_subscriptions.lock().await.contains("EURUSD"));
        assert_eq!(client.subscription_count().await, 0);
    }
}