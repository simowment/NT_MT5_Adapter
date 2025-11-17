//! WebSocket client implementation for MetaTrader 5 adapter.
//!
//! This module provides WebSocket connectivity for real-time data streaming
//! and execution updates from the MetaTrader 5 platform.

pub mod client;
pub mod messages;
pub mod parse;

pub use client::*;
pub use messages::*;
pub use parse::*;