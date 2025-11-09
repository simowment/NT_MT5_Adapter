//! Response parsing functions for HTTP client in the MetaTrader 5 adapter.

use crate::common::parse::{parse_json_response, extract_string_field, extract_number_field};
use crate::http::models::{Mt5AccountInfo, Mt5Symbol, Mt5Rate};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpParseError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Field missing: {0}")]
    MissingField(String),
}

/// Parse account info from JSON response
pub fn parse_account_info(data: &str) -> Result<Mt5AccountInfo, HttpParseError> {
    let value: Value = parse_json_response(data).map_err(|e| HttpParseError::Parse(e.to_string()))?;
    
    Ok(Mt5AccountInfo {
        login: extract_string_field(&value, "login").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        balance: extract_number_field(&value, "balance").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        equity: extract_number_field(&value, "equity").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        margin: extract_number_field(&value, "margin").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        margin_free: extract_number_field(&value, "marginFree").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        margin_level: extract_number_field(&value, "marginLevel").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
    })
}

/// Parse symbols from JSON response
pub fn parse_symbols(data: &str) -> Result<Vec<Mt5Symbol>, HttpParseError> {
    let value: Value = parse_json_response(data).map_err(|e| HttpParseError::Parse(e.to_string()))?;
    
    let symbols_array = value.as_array().ok_or_else(|| HttpParseError::Parse("Response is not an array".to_string()))?;
    
    let mut symbols = Vec::new();
    for item in symbols_array {
        let symbol = Mt5Symbol {
            symbol: extract_string_field(item, "symbol").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            digits: extract_number_field(item, "digits").map_err(|e| HttpParseError::MissingField(e.to_string()))? as u32,
            point_size: extract_number_field(item, "pointSize").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            volume_min: extract_number_field(item, "volumeMin").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            volume_max: extract_number_field(item, "volumeMax").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            volume_step: extract_number_field(item, "volumeStep").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            contract_size: extract_number_field(item, "contractSize").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            margin_initial: extract_number_field(item, "marginInitial").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            margin_maintenance: extract_number_field(item, "marginMaintenance").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        };
        symbols.push(symbol);
    }
    
    Ok(symbols)
}

/// Parse rates from JSON response
pub fn parse_rates(data: &str) -> Result<Vec<Mt5Rate>, HttpParseError> {
    let value: Value = parse_json_response(data).map_err(|e| HttpParseError::Parse(e.to_string()))?;
    
    let rates_array = value.as_array().ok_or_else(|| HttpParseError::Parse("Response is not an array".to_string()))?;
    
    let mut rates = Vec::new();
    for item in rates_array {
        let rate = Mt5Rate {
            symbol: extract_string_field(item, "symbol").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            time: extract_number_field(item, "time").map_err(|e| HttpParseError::MissingField(e.to_string()))? as u64,
            open: extract_number_field(item, "open").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            high: extract_number_field(item, "high").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            low: extract_number_field(item, "low").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            close: extract_number_field(item, "close").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            tick_volume: extract_number_field(item, "tickVolume").map_err(|e| HttpParseError::MissingField(e.to_string()))? as u64,
        };
        rates.push(rate);
    }
    
    Ok(rates)
}