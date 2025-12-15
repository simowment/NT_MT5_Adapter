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
// See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! HTTP error types for the MT5 adapter.

pub use nautilus_network::http::HttpClientError;
use thiserror::Error;

use crate::error::Mt5Error;

#[derive(Error, Debug)]
pub enum Mt5HttpError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Request error: {0}")]
    RequestError(String),

    #[error("HTTP error: {0} - {1}")]
    HttpError(u16, String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Invalid request: {0}")]
    InvalidRequestError(String),

    #[error("Resource not found: {0}")]
    NotFoundError(String),

    #[error("JSON decode error: {0}")]
    JsonDecodeError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

impl Mt5HttpError {
    /// Determines if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Mt5HttpError::ConnectionError(_)
                | Mt5HttpError::RequestError(_)
                | Mt5HttpError::ServerError(_)
                | Mt5HttpError::TimeoutError(_)
                | Mt5HttpError::RateLimitError(_)
                | Mt5HttpError::NetworkError(_)
        )
    }

    /// Determines if the error is non-retryable (should fail immediately)
    pub fn is_non_retryable(&self) -> bool {
        matches!(
            self,
            Mt5HttpError::AuthenticationError(_)
                | Mt5HttpError::AuthorizationError(_)
                | Mt5HttpError::InvalidRequestError(_)
                | Mt5HttpError::NotFoundError(_)
                | Mt5HttpError::JsonDecodeError(_)
                | Mt5HttpError::ParseError(_)
        )
    }

    /// Determines if the error is fatal (should terminate the connection)
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Mt5HttpError::AuthenticationError(_) | Mt5HttpError::AuthorizationError(_)
        )
    }

    /// Maps HTTP status codes to appropriate error variants
    pub fn from_http_status(status: u16, message: String) -> Self {
        match status {
            400 => Mt5HttpError::InvalidRequestError(message),
            401 => Mt5HttpError::AuthenticationError(message),
            403 => Mt5HttpError::AuthorizationError(message),
            404 => Mt5HttpError::NotFoundError(message),
            429 => Mt5HttpError::RateLimitError(message),
            500..=599 => Mt5HttpError::ServerError(message),
            _ => Mt5HttpError::HttpError(status, message),
        }
    }
}

impl From<Mt5HttpError> for Mt5Error {
    fn from(err: Mt5HttpError) -> Self {
        match err {
            Mt5HttpError::AuthenticationError(msg) => Mt5Error::AuthenticationError(msg),
            Mt5HttpError::AuthorizationError(msg) => Mt5Error::AuthenticationError(msg),
            Mt5HttpError::RateLimitError(_) => Mt5Error::RateLimitError,
            Mt5HttpError::NotFoundError(msg) => Mt5Error::SymbolNotFound(msg),
            Mt5HttpError::JsonDecodeError(_) | Mt5HttpError::ParseError(_) => {
                Mt5Error::SerializationError(err.to_string())
            }
            _ => Mt5Error::HttpError(err.to_string()),
        }
    }
}

// We don't need direct conversion between Mt5HttpError and HttpClientError
// since HttpClientError is from nautilus_network and has different variants
// Instead, we'll use the error handling through the ? operator where appropriate