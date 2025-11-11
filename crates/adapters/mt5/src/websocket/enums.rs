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

//! WebSocket enums for the MT5 adapter.

use serde::{Deserialize, Serialize};

/// Represents the type of WebSocket message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessageType {
    /// Authentication message
    #[serde(rename = "auth")]
    Auth,
    /// Ping message
    #[serde(rename = "ping")]
    Ping,
    /// Pong message
    #[serde(rename = "pong")]
    Pong,
    /// Subscribe message
    #[serde(rename = "subscribe")]
    Subscribe,
    /// Unsubscribe message
    #[serde(rename = "unsubscribe")]
    Unsubscribe,
    /// Market data update
    #[serde(rename = "market_data")]
    MarketData,
    /// Order update
    #[serde(rename = "order_update")]
    OrderUpdate,
    /// Position update
    #[serde(rename = "position_update")]
    PositionUpdate,
    /// Error message
    #[serde(rename = "error")]
    Error,
    /// Heartbeat message
    #[serde(rename = "heartbeat")]
    Heartbeat,
}

/// Represents the WebSocket connection state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WebSocketConnectionState {
    /// Connection is initializing
    Initializing,
    /// Connection is connecting
    Connecting,
    /// Connection is connected
    Connected,
    /// Connection is reconnecting
    Reconnecting,
    /// Connection is disconnected
    Disconnected,
    /// Connection has an error
    Error,
}