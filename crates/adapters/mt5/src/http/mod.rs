//! HTTP client implementation for MetaTrader 5 REST API.
//!
//! This module provides a simple HTTP client for interacting with the MT5 REST server,
//! which exposes all MT5 Python API functions via HTTP endpoints.

pub mod client;
pub mod error;
pub mod models;

pub use client::Mt5HttpClient;
pub use error::*;
pub use models::*;