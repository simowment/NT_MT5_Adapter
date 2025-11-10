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

//! MT5 Data Client implementation.
//!
//! This module implements the data client for the MetaTrader 5 adapter,
//! providing market data functionality including subscriptions and requests.

use crate::config::Mt5DataClientConfig;
use crate::http::client::Mt5HttpClient;
use crate::websocket::client::Mt5WebSocketClient;
use nautilus_core::clock::Clock;
use nautilus_core::message_bus::MessageBus;
use nautilus_model::data::Data;
use nautilus_model::data::{Bar, QuoteTick, TradeTick};
use nautilus_model::identifiers::InstrumentId;
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

#[derive(Debug, Error)]
pub enum DataClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    #[error("Subscription error: {0}")]
    SubscriptionError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

pub struct Mt5DataClient {
    config: Mt5DataClientConfig,
    http_client: Arc<Mt5HttpClient>,
    ws_client: Option<Arc<Mt5WebSocketClient>>,
    message_bus: Arc<MessageBus>,
    clock: Clock,
    connected: Arc<RwLock<bool>>,
    subscriptions: Arc<RwLock<HashSet<String>>>,
    data_sender: mpsc::UnboundedSender<Data>,
}

impl Mt5DataClient {
    /// Creates a new instance of the MT5 data client.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the data client.
    /// * `message_bus` - The message bus for publishing data.
    ///
    /// # Returns
    ///
    /// A new instance of the data client.
    pub fn new(
        config: Mt5DataClientConfig,
        message_bus: Arc<MessageBus>,
    ) -> Result<Self, DataClientError> {
        let http_client = Arc::new(Mt5HttpClient::new(
            config.base_url.clone(),
            config.credentials.clone(),
        )?);

        let (data_sender, _) = mpsc::unbounded_channel::<Data>();

        Ok(Self {
            config,
            http_client,
            ws_client: None,
            message_bus,
            clock: Clock::new(),
            connected: Arc::new(RwLock::new(false)),
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
            data_sender,
        })
    }

    /// Establishes a connection to the MT5 server.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn connect(&mut self) -> Result<(), DataClientError> {
        // Connect HTTP
        self.http_client.login().await
            .map_err(|e| DataClientError::ConnectionError(e.to_string()))?;

        // Connect WebSocket if enabled
        if self.config.subscribe_quotes || self.config.subscribe_trades {
            self.ws_client = Some(Arc::new(Mt5WebSocketClient::new(
                self.config.credentials.clone(),
                self.config.ws_url.clone(),
            )?));
            
            let ws_client = self.ws_client.as_ref().unwrap();
            ws_client.connect().await
                .map_err(|e| DataClientError::WebSocketError(e.to_string()))?;
                
            // Authenticate
            ws_client.authenticate().await
                .map_err(|e| DataClientError::WebSocketError(e.to_string()))?;
        }

        // Update connection status
        {
            let mut connected = self.connected.write().await;
            *connected = true;
        }

        Ok(())
    }

    /// Disconnects from the MT5 server.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn disconnect(&mut self) -> Result<(), DataClientError> {
        // Disconnect WebSocket
        if let Some(ws_client) = &self.ws_client {
            ws_client.disconnect().await
                .map_err(|e| DataClientError::WebSocketError(e.to_string()))?;
        }

        // Update connection status
        {
            let mut connected = self.connected.write().await;
            *connected = false;
        }

        Ok(())
    }

    /// Subscribes to quote ticks for the specified instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument ID to subscribe to.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn subscribe_quotes(&self, instrument_id: &InstrumentId) -> Result<(), DataClientError> {
        self.check_connection().await?;
        
        let symbol = instrument_id.to_string();
        
        // Subscribe via WebSocket
        if let Some(ws_client) = &self.ws_client {
            ws_client.subscribe_quotes(&symbol).await
                .map_err(|e| DataClientError::SubscriptionError(e.to_string()))?;
        }

        // Track subscription
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(format!("quote:{}", symbol));
        }

        Ok(())
    }

    /// Subscribes to trade ticks for the specified instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument ID to subscribe to.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn subscribe_trades(&self, instrument_id: &InstrumentId) -> Result<(), DataClientError> {
        self.check_connection().await?;
        
        let symbol = instrument_id.to_string();
        
        // Subscribe via WebSocket
        if let Some(ws_client) = &self.ws_client {
            ws_client.subscribe_trades(&symbol).await
                .map_err(|e| DataClientError::SubscriptionError(e.to_string()))?;
        }

        // Track subscription
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(format!("trade:{}", symbol));
        }

        Ok(())
    }

    /// Subscribes to order book data for the specified instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument ID to subscribe to.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn subscribe_order_book(&self, instrument_id: &InstrumentId) -> Result<(), DataClientError> {
        self.check_connection().await?;
        
        let symbol = instrument_id.to_string();
        
        // Subscribe via WebSocket
        if let Some(ws_client) = &self.ws_client {
            ws_client.subscribe_order_book(&symbol).await
                .map_err(|e| DataClientError::SubscriptionError(e.to_string()))?;
        }

        // Track subscription
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(format!("orderbook:{}", symbol));
        }

        Ok(())
    }

    /// Unsubscribes from data for the specified instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument ID to unsubscribe from.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn unsubscribe(&self, instrument_id: &InstrumentId) -> Result<(), DataClientError> {
        let symbol = instrument_id.to_string();
        
        // Unsubscribe via WebSocket
        if let Some(ws_client) = &self.ws_client {
            ws_client.unsubscribe(&symbol).await
                .map_err(|e| DataClientError::SubscriptionError(e.to_string()))?;
        }

        // Remove from tracking
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(&format!("quote:{}", symbol));
            subscriptions.remove(&format!("trade:{}", symbol));
            subscriptions.remove(&format!("orderbook:{}", symbol));
        }

        Ok(())
    }

    /// Requests instruments from the MT5 server.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn request_instruments(&self) -> Result<(), DataClientError> {
        self.check_connection().await?;
        
        let instruments = self.http_client.request_symbols().await
            .map_err(|e| DataClientError::ConnectionError(e.to_string()))?;

        // Process instruments (this would publish to message bus)
        for instrument in instruments {
            tracing::debug!("Received instrument: {}", instrument.symbol);
        }

        Ok(())
    }

    /// Requests quote tick data for the specified instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument ID to request data for.
    ///
    /// # Returns
    ///
    /// A vector of quote ticks.
    pub async fn request_quote_ticks(&self, instrument_id: &InstrumentId) -> Result<Vec<QuoteTick>, DataClientError> {
        self.check_connection().await?;
        
        // Request quote tick data
        let symbol = instrument_id.to_string();
        
        // This would fetch actual quote tick data
        let ticks = Vec::new(); // Placeholder - would fetch from MT5
        
        Ok(ticks)
    }

    /// Requests trade tick data for the specified instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument ID to request data for.
    ///
    /// # Returns
    ///
    /// A vector of trade ticks.
    pub async fn request_trade_ticks(&self, instrument_id: &InstrumentId) -> Result<Vec<TradeTick>, DataClientError> {
        self.check_connection().await?;
        
        // Request trade tick data
        let symbol = instrument_id.to_string();
        
        // This would fetch actual trade tick data
        let ticks = Vec::new(); // Placeholder
        
        Ok(ticks)
    }

    /// Requests bar data for the specified instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument ID to request data for.
    ///
    /// # Returns
    ///
    /// A vector of bars.
    pub async fn request_bars(&self, instrument_id: &InstrumentId) -> Result<Vec<Bar>, DataClientError> {
        self.check_connection().await?;
        
        // Request bar data
        let symbol = instrument_id.to_string();
        let rates = self.http_client.request_rates(&symbol, "M1", Some(100)).await
            .map_err(|e| DataClientError::ConnectionError(e.to_string()))?;

        // Convert rates to bars
        let mut bars = Vec::new();
        for rate in rates {
            // Convert rate to bar (simplified)
            let bar = Bar::new(
                "MT5".to_string(),
                rate.symbol,
                rate.open,
                rate.high,
                rate.low,
                rate.close,
                rate.tick_volume as f64,
                /* timestamp */ std::time::SystemTime::now(),
            );
            bars.push(bar);
        }

        Ok(bars)
    }

    /// Requests an order book snapshot for the specified instrument.
    ///
    /// # Arguments
    ///
    /// * `instrument_id` - The instrument ID to request data for.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub async fn request_order_book_snapshot(&self, instrument_id: &InstrumentId) -> Result<(), DataClientError> {
        self.check_connection().await?;
        
        let symbol = instrument_id.to_string();
        
        // Request order book snapshot
        // This would be implemented based on MT5 bridge capabilities
        
        Ok(())
    }

    async fn check_connection(&self) -> Result<(), DataClientError> {
        let connected = *self.connected.read().await;
        if !connected {
            return Err(DataClientError::ConnectionError("Not connected".to_string()));
        }
        Ok(())
    }

    /// Checks if the client is connected.
    ///
    /// # Returns
    ///
    /// True if connected, false otherwise.
    pub fn is_connected(&self) -> bool {
        // This is a simplified check
        self.connected.blocking_read().clone()
    }
}