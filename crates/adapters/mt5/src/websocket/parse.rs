//! WebSocket message parsing utilities for MetaTrader 5 adapter.

use crate::websocket::messages::{Mt5WsMessage, Mt5Subscription};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WsParseError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Unknown message type: {0}")]
    UnknownType(String),
}

/// Parse a WebSocket message from JSON
pub fn parse_ws_message(data: &str) -> Result<Mt5WsMessage, WsParseError> {
    let value: Value = serde_json::from_str(data)
        .map_err(|e| WsParseError::InvalidFormat(e.to_string()))?;
    
    let msg_type = value.get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WsParseError::MissingField("type".to_string()))?;
    
    match msg_type {
        "trade" => parse_trade_message(&value),
        "quote" => parse_quote_message(&value),
        "orderbook" => parse_orderbook_message(&value),
        "connection" => parse_connection_message(&value),
        _ => Err(WsParseError::UnknownType(msg_type.to_string())),
    }
}

fn parse_trade_message(value: &Value) -> Result<Mt5WsMessage, WsParseError> {
    Ok(Mt5WsMessage::Trade {
        symbol: extract_string_field(value, "symbol")?,
        price: extract_number_field(value, "price")?,
        volume: extract_number_field(value, "volume")?,
        timestamp: extract_number_field(value, "timestamp")? as u64,
    })
}

fn parse_quote_message(value: &Value) -> Result<Mt5WsMessage, WsParseError> {
    Ok(Mt5WsMessage::Quote {
        symbol: extract_string_field(value, "symbol")?,
        bid: extract_number_field(value, "bid")?,
        ask: extract_number_field(value, "ask")?,
        timestamp: extract_number_field(value, "timestamp")? as u64,
    })
}

fn parse_orderbook_message(value: &Value) -> Result<Mt5WsMessage, WsParseError> {
    Ok(Mt5WsMessage::OrderBook {
        symbol: extract_string_field(value, "symbol")?,
        bids: parse_price_levels(value.get("bids"))?,
        asks: parse_price_levels(value.get("asks"))?,
        timestamp: extract_number_field(value, "timestamp")? as u64,
    })
}

fn parse_connection_message(value: &Value) -> Result<Mt5WsMessage, WsParseError> {
    Ok(Mt5WsMessage::Connection {
        status: extract_string_field(value, "status")?,
        message: value.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()),
    })
}

fn parse_price_levels(value: Option<&Value>) -> Result<Vec<(f64, f64)>, WsParseError> {
    match value {
        Some(arr) => {
            let levels = arr.as_array()
                .ok_or_else(|| WsParseError::InvalidFormat("Expected array".to_string()))?;
            
            let mut result = Vec::new();
            for level in levels {
                if let Some(arr) = level.as_array() {
                    if arr.len() >= 2 {
                        let price = arr[0].as_f64()
                            .ok_or_else(|| WsParseError::InvalidFormat("Invalid price".to_string()))?;
                        let size = arr[1].as_f64()
                            .ok_or_else(|| WsParseError::InvalidFormat("Invalid size".to_string()))?;
                        result.push((price, size));
                    }
                }
            }
            Ok(result)
        }
        None => Ok(Vec::new()),
    }
}

fn extract_string_field(value: &Value, field: &str) -> Result<String, WsParseError> {
    value.get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| WsParseError::MissingField(field.to_string()))
}

fn extract_number_field(value: &Value, field: &str) -> Result<f64, WsParseError> {
    value.get(field)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| WsParseError::MissingField(field.to_string()))
}