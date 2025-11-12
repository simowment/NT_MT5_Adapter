//! HTTP client implementation for MetaTrader 5 adapter.
//!
//! This module provides a complete HTTP client for interacting with the MT5 proxy,
//! including authentication, error handling, and all supported API endpoints.

pub mod client;
pub mod error;
pub mod models;
pub mod parse;
pub mod query;

pub use client::Mt5HttpClient;
pub use error::*;
pub use models::*;
pub use parse::*;
pub use query::*;