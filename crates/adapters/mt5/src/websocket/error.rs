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

//! WebSocket error types for the MT5 adapter.

use std::fmt::Debug;

use thiserror::Error;

use crate::error::Mt5Error;

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Send error: {0}")]
    SendError(String),
    
    #[error("Receive error: {0}")]
    ReceiveError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Timeout error")]
    TimeoutError,
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

pub type WebSocketResult<T> = Result<T, WebSocketError>;

impl From<WebSocketError> for Mt5Error {
    fn from(err: WebSocketError) -> Self {
        match err {
            WebSocketError::AuthenticationError(msg) => Mt5Error::AuthenticationError(msg),
            WebSocketError::TimeoutError => Mt5Error::TimeoutError,
            WebSocketError::SerializationError(_) => Mt5Error::SerializationError(err.to_string()),
            _ => Mt5Error::WebSocketError(err.to_string()),
        }
    }
}