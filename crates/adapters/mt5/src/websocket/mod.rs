//! WebSocket client implementation for MetaTrader 5 adapter.
//!
//! This module provides a complete WebSocket client for real-time communication
//! with the MT5 proxy, including message handling, parsing, and error management.

pub mod client;
pub mod enums;
pub mod error;
pub mod handler;
pub mod messages;
pub mod parse;

pub use client::Mt5WebSocketClient;
pub use enums::*;
pub use error::*;
pub use handler::*;
pub use messages::*;
pub use parse::*;