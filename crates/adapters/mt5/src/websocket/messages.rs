//! Structs for WebSocket stream payloads in the MetaTrader 5 adapter.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsPing {
    pub op: String,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsPong {
    pub op: String,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsAuthRequest {
    pub op: String,
    pub login: String,
    pub password: String,
    pub server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsAuthResponse {
    pub op: String,
    pub result: bool,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsSubscribeRequest {
    pub op: String,
    pub topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsUnsubscribeRequest {
    pub op: String,
    pub topic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsSubscriptionResponse {
    pub op: String,
    pub topic: String,
    pub result: bool,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsQuote {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub bid_volume: f64,
    pub ask_volume: f64,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsTrade {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub time: u64,
    pub direction: String, // BUY or SELL
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsOrderBook {
    pub symbol: String,
    pub bids: Vec<(f64, f64)>, // (price, volume)
    pub asks: Vec<(f64, f64)>, // (price, volume)
    pub time: u64,
}