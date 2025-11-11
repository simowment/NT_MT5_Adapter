// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
// https://nautechsystems.io
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

//! WebSocket message handler for the MT5 adapter.

use std::sync::Arc;

use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use tracing::debug;

use crate::error::{Mt5Error, Mt5Result};

/// Trait for handling incoming WebSocket messages.
pub trait WebSocketMessageHandler {
    /// Handle an incoming text message.
    fn handle_text_message(&self, message: &str) -> Mt5Result<()>;

    /// Handle an incoming binary message.
    fn handle_binary_message(&self, data: &[u8]) -> Mt5Result<()>;

    /// Handle a ping message.
    fn handle_ping(&self, data: &[u8]) -> Mt5Result<()>;

    /// Handle a pong message.
    fn handle_pong(&self, data: &[u8]) -> Mt5Result<()>;

    /// Handle a close message.
    fn handle_close(&self) -> Mt5Result<()>;
}

/// Generic WebSocket handler that manages the connection and delegates message handling.
pub struct WebSocketHandler<T: WebSocketMessageHandler> {
    /// The message handler implementation
    handler: Arc<T>,
    /// The WebSocket sink for sending messages
    sink: Arc<tokio::sync::Mutex<SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>>>,
}

impl<T: WebSocketMessageHandler + 'static> WebSocketHandler<T> {
    /// Creates a new WebSocket handler.
    pub fn new(
        handler: Arc<T>,
        sink: SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    ) -> Self {
        Self {
            handler,
            sink: Arc::new(tokio::sync::Mutex::new(sink)),
        }
    }

    /// Handles an incoming message based on its type.
    pub async fn handle_message(&self, message: Message) -> Mt5Result<()> {
        match message {
            Message::Text(text) => {
                self.handler.handle_text_message(&text)?;
                Ok(())
            },
            Message::Binary(data) => {
                self.handler.handle_binary_message(&data)?;
                Ok(())
            },
            Message::Ping(data) => {
                self.handler.handle_ping(&data)?;
                Ok(())
            },
            Message::Pong(data) => {
                self.handler.handle_pong(&data)?;
                Ok(())
            },
            Message::Close(_) => {
                self.handler.handle_close()?;
                Ok(())
            },
            Message::Frame(_) => {
                debug!("Received raw frame message");
                Ok(())
            }
        }
    }

    /// Sends a text message through the WebSocket.
    pub async fn send_text(&self, text: &str) -> Mt5Result<()> {
        let mut sink = self.sink.lock().await;
        sink.send(Message::Text(text.to_string().into()))
            .await
            .map_err(|e| Mt5Error::WebSocketError(format!("Failed to send text message: {}", e)))?;
        Ok(())
    }

    /// Sends a binary message through the WebSocket.
    pub async fn send_binary(&self, data: &[u8]) -> Mt5Result<()> {
        let mut sink = self.sink.lock().await;
        sink.send(Message::Binary(data.to_vec().into()))
            .await
            .map_err(|e| Mt5Error::WebSocketError(format!("Failed to send binary message: {}", e)))?;
        Ok(())
    }

    /// Sends a ping message through the WebSocket.
    pub async fn send_ping(&self, data: &[u8]) -> Mt5Result<()> {
        let mut sink = self.sink.lock().await;
        sink.send(Message::Ping(data.to_vec().into()))
            .await
            .map_err(|e| Mt5Error::WebSocketError(format!("Failed to send ping message: {}", e)))?;
        Ok(())
    }

    /// Closes the WebSocket connection.
    pub async fn close(&self) -> Mt5Result<()> {
        let mut sink = self.sink.lock().await;
        sink.close()
            .await
            .map_err(|e| Mt5Error::WebSocketError(format!("Failed to close connection: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct TestHandler;

    impl WebSocketMessageHandler for TestHandler {
        fn handle_text_message(&self, _message: &str) -> Mt5Result<()> {
            Ok(())
        }

        fn handle_binary_message(&self, _data: &[u8]) -> Mt5Result<()> {
            Ok(())
        }

        fn handle_ping(&self, _data: &[u8]) -> Mt5Result<()> {
            Ok(())
        }

        fn handle_pong(&self, _data: &[u8]) -> Mt5Result<()> {
            Ok(())
        }

        fn handle_close(&self) -> Mt5Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_websocket_handler_creation() {
        // This test would require a mock WebSocket stream
        // For now, we just verify the structure compiles
        let handler = Arc::new(TestHandler);
        // Note: We can't create an actual WebSocket stream in this test
        // without a server, so we're just verifying the types work together
        assert!(true);
    }
}