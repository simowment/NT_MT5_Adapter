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

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt, pin_mut};
use crate::common::credential::Mt5Credential;
use crate::websocket::messages::{
    WsPing, WsPong, WsAuthRequest, WsAuthResponse,
    WsSubscribeRequest, WsUnsubscribeRequest, WsSubscriptionResponse,
    WsQuote, WsTrade, WsOrderBook
};
use crate::websocket::parse::{parse_websocket_message, WsMessage};
use thiserror::Error;

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

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

#[derive(Debug, Clone, PartialEq)]
enum SubscriptionState {
    Pending,
    Confirmed,
}

// Client WebSocket pour MT5.
// Gère:
// - Connexion/authentification
// - Lifecycle des subscriptions (pending/confirmed)
// - Reconnexion
// - Ping/Pong
// - Routing des messages
//
// Conçu pour être cloné et partagé entre tâches (Arc<Mutex<>>).
#[cfg_attr(feature = "python-bindings", pyclass)]
pub struct Mt5WebSocketClient {
    credential: Mt5Credential,
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

    pub async fn subscription_count(&self) -> usize {
        self.confirmed_subscriptions.lock().await.len()
    }

    pub async fn confirm_subscription(&self, topic: &str) {
        let mut pending = self.pending_subscriptions.lock().await;
        let mut confirmed = self.confirmed_subscriptions.lock().await;
        
        if pending.contains(topic) {
            pending.remove(topic);
            confirmed.insert(topic.to_string());
        }
    }

    pub async fn mark_subscription_failure(&self, topic: &str) {
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
            WsMessage::Ping(ping) => {
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
    
    fn py_connect(&mut self) -> Result<(), WebSocketError> {
        // Note: This would need to be adapted for async in PyO3
        // For simplicity, using a blocking call here
        // In practice, you'd want to use pyo3_async_runtimes
        futures::executor::block_on(self.connect())
    }
    
    fn py_disconnect(&mut self) -> Result<(), WebSocketError> {
        futures::executor::block_on(self.disconnect())
    }
    
    fn py_subscribe(&mut self, topic: &str) {
        self.subscribe(topic);
    }
    
    fn py_unsubscribe(&mut self, topic: &str) {
        self.unsubscribe(topic);
    }
    
    fn py_is_authenticated(&self) -> bool {
        self.is_authenticated()
    }
    
    fn py_subscription_count(&self) -> usize {
        self.subscription_count()
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
        let mut client = Mt5WebSocketClient::new(cred, "ws://localhost");

        assert_eq!(client.url, "ws://localhost");
        assert!(!client.is_authenticated());
    }

    #[tokio::test]
    async fn test_subscription_lifecycle() {
        let cred = Mt5Credential::builder()
            .login("user")
            .password("pass")
            .server("server")
            .build()
            .unwrap();
        let mut client = Mt5WebSocketClient::new(cred, "ws://localhost");

        // Test subscription
        client.subscribe("EURUSD");
        assert!(client.pending_subscriptions.contains("EURUSD"));
        assert_eq!(client.subscription_count(), 0);

        // Test confirmation
        client.confirm_subscription("EURUSD");
        assert!(!client.pending_subscriptions.contains("EURUSD"));
        assert!(client.confirmed_subscriptions.contains("EURUSD"));
        assert_eq!(client.subscription_count(), 1);

        // Test unsubscription
        client.unsubscribe("EURUSD");
        assert!(client.pending_subscriptions.contains("EURUSD"));
        assert!(!client.confirmed_subscriptions.contains("EURUSD"));
        assert_eq!(client.subscription_count(), 0);
    }
}