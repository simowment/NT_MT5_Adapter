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
    
    // Handle login field that might be a number or string
    let login = if let Some(login_str) = value.get("login").and_then(Value::as_str) {
        login_str.to_string()
    } else if let Some(login_num) = value.get("login").and_then(Value::as_number) {
        login_num.to_string()
    } else {
        return Err(HttpParseError::MissingField("login".to_string()));
    };
    
    Ok(Mt5AccountInfo {
        login,
        balance: extract_number_field(&value, "balance").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        equity: extract_number_field(&value, "equity").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        margin: extract_number_field(&value, "margin")
            .or_else(|_| extract_number_field(&value, "margin_used"))
            .unwrap_or(0.0),
        margin_free: extract_number_field(&value, "marginFree")
            .or_else(|_| extract_number_field(&value, "margin_free"))
            .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        margin_level: extract_number_field(&value, "marginLevel")
            .or_else(|_| extract_number_field(&value, "margin_level"))
            .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
    })
}

/// Parse symbols from JSON response
pub fn parse_symbols(data: &str) -> Result<Vec<Mt5Symbol>, HttpParseError> {
    let value: Value = parse_json_response(data).map_err(|e| HttpParseError::Parse(e.to_string()))?;
    
    // Handle both direct array and wrapped object formats
    let symbols_array = if let Some(symbols) = value.get("symbols").and_then(Value::as_array) {
        // Format: {"symbols": [...]}
        symbols
    } else if let Some(array) = value.as_array() {
        // Format: [...]
        array
    } else {
        return Err(HttpParseError::Parse("Response is not a valid symbols format".to_string()));
    };
    
    let mut symbols = Vec::new();
    for item in symbols_array {
        let symbol = Mt5Symbol {
            symbol: extract_string_field(item, "symbol")
                .or_else(|_| extract_string_field(item, "name"))
                .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            digits: extract_number_field(item, "digits").map_err(|e| HttpParseError::MissingField(e.to_string()))? as u32,
            point_size: extract_number_field(item, "pointSize")
                .or_else(|_| extract_number_field(item, "point"))
                .or_else(|_| extract_number_field(item, "trade_tick_size"))
                .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            volume_min: extract_number_field(item, "volumeMin")
                .or_else(|_| Ok(0.01)) // Default value if not present
                .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            volume_max: extract_number_field(item, "volumeMax")
                .or_else(|_| Ok(100.0)) // Default value if not present
                .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            volume_step: extract_number_field(item, "volumeStep")
                .or_else(|_| extract_number_field(item, "trade_tick_size"))
                .or_else(|_| Ok(0.01)) // Default value if not present
                .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            contract_size: extract_number_field(item, "contractSize")
                .or_else(|_| extract_number_field(item, "trade_contract_size"))
                .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            margin_initial: extract_number_field(item, "marginInitial")
                .or_else(|_| Ok(0.0)) // Default value if not present
                .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            margin_maintenance: extract_number_field(item, "marginMaintenance")
                .or_else(|_| Ok(0.0)) // Default value if not present
                .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        };
        symbols.push(symbol);
    }
    
    Ok(symbols)
}

/// Parse a single symbol from JSON response
pub fn parse_single_symbol(data: &Value) -> Result<Mt5Symbol, HttpParseError> {
    Ok(Mt5Symbol {
        symbol: extract_string_field(data, "symbol")
            .or_else(|_| extract_string_field(data, "name"))
            .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        digits: extract_number_field(data, "digits").map_err(|e| HttpParseError::MissingField(e.to_string()))? as u32,
        point_size: extract_number_field(data, "pointSize")
            .or_else(|_| extract_number_field(data, "point"))
            .or_else(|_| extract_number_field(data, "trade_tick_size"))
            .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        volume_min: extract_number_field(data, "volumeMin")
            .or_else(|_| Ok(0.01)) // Default value if not present
            .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        volume_max: extract_number_field(data, "volumeMax")
            .or_else(|_| Ok(100.0)) // Default value if not present
            .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        volume_step: extract_number_field(data, "volumeStep")
            .or_else(|_| extract_number_field(data, "trade_tick_size"))
            .or_else(|_| Ok(0.01)) // Default value if not present
            .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        contract_size: extract_number_field(data, "contractSize")
            .or_else(|_| extract_number_field(data, "trade_contract_size"))
            .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        margin_initial: extract_number_field(data, "marginInitial")
            .or_else(|_| Ok(0.0)) // Default value if not present
            .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
        margin_maintenance: extract_number_field(data, "marginMaintenance")
            .or_else(|_| Ok(0.0)) // Default value if not present
            .map_err(|e| HttpParseError::MissingField(e.to_string()))?,
    })
}

/// Parse rates from JSON response
pub fn parse_rates(data: &str) -> Result<Vec<Mt5Rate>, HttpParseError> {
    let value: Value = parse_json_response(data).map_err(|e| HttpParseError::Parse(e.to_string()))?;
    
    // Handle both direct array and wrapped object formats
    let rates_array = if let Some(rates) = value.get("rates").and_then(Value::as_array) {
        // Format: {"rates": [...]}
        rates
    } else if let Some(array) = value.as_array() {
        // Format: [...]
        array
    } else {
        return Err(HttpParseError::Parse("Response is not a valid rates format".to_string()));
    };
    
    let mut rates = Vec::new();
    for item in rates_array {
        let time = extract_number_field(item, "time")
            .or_else(|_| extract_number_field(item, "timestamp"))
            .map_err(|e| HttpParseError::MissingField(e.to_string()))? as u64;
        
        // Handle different rate data formats - some have OHLC, others have bid/ask/last
        let (open, high, low, close) = if let Ok(last_price) = extract_number_field(item, "last") {
            // Format with bid/ask/last - use last as both open and close
            (last_price, last_price, last_price, last_price)
        } else {
            // Format with OHLC
            (
                extract_number_field(item, "open").unwrap_or(0.0),
                extract_number_field(item, "high").unwrap_or(0.0),
                extract_number_field(item, "low").unwrap_or(0.0),
                extract_number_field(item, "close").unwrap_or(0.0),
            )
        };
        
        let rate = Mt5Rate {
            symbol: extract_string_field(item, "symbol").map_err(|e| HttpParseError::MissingField(e.to_string()))?,
            time,
            open,
            high,
            low,
            close,
            tick_volume: extract_number_field(item, "tickVolume")
                .or_else(|_| extract_number_field(item, "volume"))
                .unwrap_or(0.0) as u64,
        };
        rates.push(rate);
    }
    
    Ok(rates)
}