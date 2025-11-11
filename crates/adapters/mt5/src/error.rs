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

//! Error types for the MT5 adapter.

use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Mt5Error {
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error("Connection timeout")]
    TimeoutError,
    
    #[error("Rate limit exceeded")]
    RateLimitError,
    
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    
    #[error("Order error: {0}")]
    OrderError(String),
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

pub type Mt5Result<T> = Result<T, Mt5Error>;