//! Message parsing functions for WebSocket client in the MetaTrader 5 adapter.

use crate::websocket::messages::{
    WsPing, WsPong, WsAuthRequest, WsAuthResponse, 
    WsSubscribeRequest, WsUnsubscribeRequest, WsSubscriptionResponse,
    WsQuote, WsTrade, WsOrderBook
};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WsParseError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Message type not recognized: {0}")]
    UnknownMessageType(String),
}

/// Parse WebSocket message based on its type
pub fn parse_websocket_message(data: &str) -> Result<WsMessage, WsParseError> {
    let value: Value = serde_json::from_str(data)
        .map_err(|e| WsParseError::Parse(e.to_string()))?;
    
    // Extract the operation type from the message
    if let Some(op_value) = value.get("op") {
        let op = op_value.as_str().ok_or_else(|| 
            WsParseError::Parse("Operation field is not a string".to_string()))?;
        
        match op {
            "ping" => {
                let ping: WsPing = serde_json::from_value(value)
                    .map_err(|e| WsParseError::Parse(e.to_string()))?;
                Ok(WsMessage::Ping(ping))
            },
            "pong" => {
                let pong: WsPong = serde_json::from_value(value)
                    .map_err(|e| WsParseError::Parse(e.to_string()))?;
                Ok(WsMessage::Pong(pong))
            },
            "auth" => {
                // Determine if it's a request or response based on other fields
                if value.get("login").is_some() {
                    let auth_req: WsAuthRequest = serde_json::from_value(value)
                        .map_err(|e| WsParseError::Parse(e.to_string()))?;
                    Ok(WsMessage::AuthRequest(auth_req))
                } else {
                    let auth_resp: WsAuthResponse = serde_json::from_value(value)
                        .map_err(|e| WsParseError::Parse(e.to_string()))?;
                    Ok(WsMessage::AuthResponse(auth_resp))
                }
            },
            "subscribe" => {
                let sub_req: WsSubscribeRequest = serde_json::from_value(value)
                    .map_err(|e| WsParseError::Parse(e.to_string()))?;
                Ok(WsMessage::SubscribeRequest(sub_req))
            },
            "unsubscribe" => {
                let unsub_req: WsUnsubscribeRequest = serde_json::from_value(value)
                    .map_err(|e| WsParseError::Parse(e.to_string()))?;
                Ok(WsMessage::UnsubscribeRequest(unsub_req))
            },
            "subscription" => {
                let sub_resp: WsSubscriptionResponse = serde_json::from_value(value)
                    .map_err(|e| WsParseError::Parse(e.to_string()))?;
                Ok(WsMessage::SubscriptionResponse(sub_resp))
            },
            "quote" => {
                let quote: WsQuote = serde_json::from_value(value)
                    .map_err(|e| WsParseError::Parse(e.to_string()))?;
                Ok(WsMessage::Quote(quote))
            },
            "trade" => {
                let trade: WsTrade = serde_json::from_value(value)
                    .map_err(|e| WsParseError::Parse(e.to_string()))?;
                Ok(WsMessage::Trade(trade))
            },
            "orderbook" => {
                let orderbook: WsOrderBook = serde_json::from_value(value)
                    .map_err(|e| WsParseError::Parse(e.to_string()))?;
                Ok(WsMessage::OrderBook(orderbook))
            },
            _ => Err(WsParseError::UnknownMessageType(op.to_string())),
        }
    } else {
        // If no 'op' field, try to determine message type by other fields
        if value.get("bid").is_some() && value.get("ask").is_some() {
            let quote: WsQuote = serde_json::from_value(value)
                .map_err(|e| WsParseError::Parse(e.to_string()))?;
            Ok(WsMessage::Quote(quote))
        } else if value.get("price").is_some() && value.get("volume").is_some() {
            let trade: WsTrade = serde_json::from_value(value)
                .map_err(|e| WsParseError::Parse(e.to_string()))?;
            Ok(WsMessage::Trade(trade))
        } else if value.get("bids").is_some() && value.get("asks").is_some() {
            let orderbook: WsOrderBook = serde_json::from_value(value)
                .map_err(|e| WsParseError::Parse(e.to_string()))?;
            Ok(WsMessage::OrderBook(orderbook))
        } else {
            Err(WsParseError::UnknownMessageType("unknown".to_string()))
        }
    }
}

/// Enum representing different types of WebSocket messages
#[derive(Debug, Clone)]
pub enum WsMessage {
    Ping(WsPing),
    Pong(WsPong),
    AuthRequest(WsAuthRequest),
    AuthResponse(WsAuthResponse),
    SubscribeRequest(WsSubscribeRequest),
    UnsubscribeRequest(WsUnsubscribeRequest),
    SubscriptionResponse(WsSubscriptionResponse),
    Quote(WsQuote),
    Trade(WsTrade),
    OrderBook(WsOrderBook),
}