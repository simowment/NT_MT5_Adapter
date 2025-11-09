//! HTTP client for communicating with the MetaTrader 5 Quant Server REST API.
//! 
//! This client interfaces with the REST API provided by the metatrader5-quant-server-python
//! which acts as a bridge to the MT5 platform.

use std::sync::Arc;
use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::common::credential::Mt5Credential;
use crate::config::Mt5Config;
use crate::http::models::{Mt5AccountInfo, Mt5Symbol, Mt5Rate};
use crate::http::query::{AccountInfoParams, SymbolsInfoParams, RatesInfoParams};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Mt5RestClientError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),
    #[error("JSON parsing failed: {0}")]
    JsonParseFailed(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

/// HTTP client for MetaTrader 5 Quant Server REST API
pub struct Mt5RestClient {
    client: Client,
    config: Mt5Config,
    credential: Mt5Credential,
    base_url: String,
}

impl Mt5RestClient {
    /// Creates a new MT5 REST client
    pub fn new(config: Mt5Config, credential: Mt5Credential) -> Result<Self, Mt5RestClientError> {
        let client = Client::builder()
            .build()
            .map_err(|e| Mt5RestClientError::ConnectionError(e.to_string()))?;

        Ok(Self {
            client,
            config,
            credential,
            base_url: config.base_url.clone(),
        })
    }

    /// Gets account information from the MT5 server
    pub async fn get_account_info(&self) -> Result<Mt5AccountInfo, Mt5RestClientError> {
        let url = format!("{}/account/info", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let account_info: Mt5AccountInfo = serde_json::from_str(&response_text)
            .map_err(|e| Mt5RestClientError::JsonParseFailed(e.to_string()))?;

        Ok(account_info)
    }

    /// Gets symbols information from the MT5 server
    pub async fn get_symbols(&self, params: &SymbolsInfoParams) -> Result<Vec<Mt5Symbol>, Mt5RestClientError> {
        let mut url = format!("{}/symbols", self.base_url);
        
        // Add query parameters if provided
        let mut query_params = Vec::new();
        if let Some(symbol) = &params.symbol {
            query_params.push(("symbol", symbol));
        }
        if let Some(group) = &params.group {
            query_params.push(("group", group));
        }
        
        if !query_params.is_empty() {
            url.push('?');
            for (i, (key, value)) in query_params.iter().enumerate() {
                if i > 0 {
                    url.push('&');
                }
                url.push_str(key);
                url.push('=');
                url.push_str(value);
            }
        }

        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let symbols: Vec<Mt5Symbol> = serde_json::from_str(&response_text)
            .map_err(|e| Mt5RestClientError::JsonParseFailed(e.to_string()))?;

        Ok(symbols)
    }

    /// Gets rates (candlestick data) from the MT5 server
    pub async fn get_rates(&self, params: &RatesInfoParams) -> Result<Vec<Mt5Rate>, Mt5RestClientError> {
        let mut url = format!("{}/rates", self.base_url);
        
        // Add query parameters
        let mut query_params = Vec::new();
        query_params.push(("symbol", &params.symbol));
        query_params.push(("timeframe", &params.timeframe));
        if let Some(from) = params.from {
            query_params.push(("from", &from.to_string()));
        }
        if let Some(to) = params.to {
            query_params.push(("to", &to.to_string()));
        }
        if let Some(count) = params.count {
            query_params.push(("count", &count.to_string()));
        }
        
        if !query_params.is_empty() {
            url.push('?');
            for (i, (key, value)) in query_params.iter().enumerate() {
                if i > 0 {
                    url.push('&');
                }
                url.push_str(key);
                url.push('=');
                url.push_str(value);
            }
        }

        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let rates: Vec<Mt5Rate> = serde_json::from_str(&response_text)
            .map_err(|e| Mt5RestClientError::JsonParseFailed(e.to_string()))?;

        Ok(rates)
    }

    /// Places an order on the MT5 server
    pub async fn place_order(&self, order_data: &Value) -> Result<Value, Mt5RestClientError> {
        let url = format!("{}/orders/place", self.base_url);
        
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(order_data)
            .send()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let result: Value = serde_json::from_str(&response_text)
            .map_err(|e| Mt5RestClientError::JsonParseFailed(e.to_string()))?;

        Ok(result)
    }

    /// Modifies an existing order on the MT5 server
    pub async fn modify_order(&self, order_id: &str, order_data: &Value) -> Result<Value, Mt5RestClientError> {
        let url = format!("{}/orders/{}/modify", self.base_url, order_id);
        
        let response = self
            .client
            .put(&url)
            .header("Content-Type", "application/json")
            .json(order_data)
            .send()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let result: Value = serde_json::from_str(&response_text)
            .map_err(|e| Mt5RestClientError::JsonParseFailed(e.to_string()))?;

        Ok(result)
    }

    /// Cancels an order on the MT5 server
    pub async fn cancel_order(&self, order_id: &str) -> Result<Value, Mt5RestClientError> {
        let url = format!("{}/orders/{}/cancel", self.base_url, order_id);
        
        let response = self
            .client
            .delete(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let result: Value = serde_json::from_str(&response_text)
            .map_err(|e| Mt5RestClientError::JsonParseFailed(e.to_string()))?;

        Ok(result)
    }

    /// Gets order information from the MT5 server
    pub async fn get_order(&self, order_id: &str) -> Result<Value, Mt5RestClientError> {
        let url = format!("{}/orders/{}", self.base_url, order_id);
        
        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let result: Value = serde_json::from_str(&response_text)
            .map_err(|e| Mt5RestClientError::JsonParseFailed(e.to_string()))?;

        Ok(result)
    }

    /// Gets all orders from the MT5 server
    pub async fn get_orders(&self) -> Result<Vec<Value>, Mt5RestClientError> {
        let url = format!("{}/orders", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let result: Vec<Value> = serde_json::from_str(&response_text)
            .map_err(|e| Mt5RestClientError::JsonParseFailed(e.to_string()))?;

        Ok(result)
    }

    /// Gets position information from the MT5 server
    pub async fn get_position(&self, position_id: &str) -> Result<Value, Mt5RestClientError> {
        let url = format!("{}/positions/{}", self.base_url, position_id);
        
        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let result: Value = serde_json::from_str(&response_text)
            .map_err(|e| Mt5RestClientError::JsonParseFailed(e.to_string()))?;

        Ok(result)
    }

    /// Gets all positions from the MT5 server
    pub async fn get_positions(&self) -> Result<Vec<Value>, Mt5RestClientError> {
        let url = format!("{}/positions", self.base_url);
        
        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| Mt5RestClientError::RequestFailed(e.to_string()))?;

        let result: Vec<Value> = serde_json::from_str(&response_text)
            .map_err(|e| Mt5RestClientError::JsonParseFailed(e.to_string()))?;

        Ok(result)
    }
}