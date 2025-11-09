//! Client implementations for MetaTrader 5 connections.

pub mod http;
pub mod ws;

pub use http::HttpClient;
pub use ws::WebSocketClient;
