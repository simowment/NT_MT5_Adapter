// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2025 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Parsing utilities for MetaTrader 5 data.

use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Invalid symbol format: {0}")]
    InvalidSymbol(String),
    #[error("Invalid price: {0}")]
    InvalidPrice(String),
    #[error("Invalid volume: {0}")]
    InvalidVolume(String),
}

pub fn parse_json_response(data: &str) -> Result<Value, ParseError> {
    serde_json::from_str(data).map_err(|e| ParseError::InvalidJson(e.to_string()))
}

pub fn extract_string_field(obj: &Value, field: &str) -> Result<String, ParseError> {
    obj.get(field)
        .and_then(Value::as_str)
        .map(|s| s.to_string())
        .ok_or_else(|| ParseError::MissingField(field.to_string()))
}

pub fn extract_number_field(obj: &Value, field: &str) -> Result<f64, ParseError> {
    obj.get(field)
        .and_then(Value::as_f64)
        .ok_or_else(|| ParseError::MissingField(field.to_string()))
}

pub fn extract_u64_field(obj: &Value, field: &str) -> Result<u64, ParseError> {
    obj.get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| ParseError::MissingField(field.to_string()))
}

pub fn extract_i64_field(obj: &Value, field: &str) -> Result<i64, ParseError> {
    obj.get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| ParseError::MissingField(field.to_string()))
}

/// Parse MT5 timestamp (seconds since epoch) to DateTime<Utc>
pub fn parse_mt5_timestamp(timestamp: i64) -> Result<DateTime<Utc>, ParseError> {
    DateTime::from_timestamp(timestamp, 0)
        .ok_or_else(|| ParseError::InvalidTimestamp(timestamp.to_string()))
}

/// Parse symbol to determine instrument type
pub fn parse_instrument_symbol(symbol: &str) -> Result<InstrumentType, ParseError> {
    // FX pairs (6 characters: AAABBB where AAA and BBB are currency codes)
    if symbol.len() == 6 {
        let base = &symbol[..3];
        let quote = &symbol[3..];
        
        // Common currency codes
        let currencies = ["USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD", "NZD", "NOK", "SEK", "DKK"];
        
        if currencies.contains(&base) && currencies.contains(&quote) {
            return Ok(InstrumentType::CurrencyPair {
                base_currency: base.to_string(),
                quote_currency: quote.to_string(),
            });
        }
    }
    
    // CFD indices (like US30, SPX500, etc.)
    if symbol.len() <= 8 && symbol.chars().all(|c| c.is_alphanumeric()) {
        return Ok(InstrumentType::Cfd { symbol: symbol.to_string() });
    }
    
    // Futures contracts (like GC2024, ES2024, etc.)
    if symbol.len() >= 5 && symbol.chars().take(symbol.len() - 4).all(|c| c.is_alphabetic())
        && symbol.chars().skip(symbol.len() - 4).all(|c| c.is_numeric()) {
        return Ok(InstrumentType::FuturesContract {
            symbol: symbol.to_string(),
            expiry_year: symbol.chars().skip(symbol.len() - 4).collect::<String>().parse()
                .map_err(|_| ParseError::InvalidSymbol(symbol.to_string()))?
        });
    }
    
    Err(ParseError::InvalidSymbol(symbol.to_string()))
}

/// Parse price with proper precision
pub fn parse_price(price: f64, digits: u8) -> Result<f64, ParseError> {
    if price.is_finite() && price >= 0.0 {
        let factor = 10f64.powi(digits as i32);
        Ok((price * factor).round() / factor)
    } else {
        Err(ParseError::InvalidPrice(price.to_string()))
    }
}

/// Parse volume/lot size
pub fn parse_volume(volume: f64, min_lot: f64, max_lot: f64, lot_step: f64) -> Result<f64, ParseError> {
    if volume.is_finite() && volume >= min_lot && volume <= max_lot {
        // Round to nearest lot step
        let normalized = (volume / lot_step).round() * lot_step;
        Ok((normalized * 10000.0).round() / 10000.0) // Round to 4 decimal places
    } else {
        Err(ParseError::InvalidVolume(volume.to_string()))
    }
}

/// Extract and parse instrument metadata
pub fn parse_instrument_metadata(obj: &Value) -> Result<InstrumentMetadata, ParseError> {
    let symbol = extract_string_field(obj, "symbol")?;
    let digits = extract_u64_field(obj, "digits")? as u8;
    let point_size = extract_number_field(obj, "point_size")?;
    let volume_min = extract_number_field(obj, "volume_min")?;
    let volume_max = extract_number_field(obj, "volume_max")?;
    let volume_step = extract_number_field(obj, "volume_step")?;
    let contract_size = extract_number_field(obj, "contract_size")?;
    
    let instrument_type = parse_instrument_symbol(&symbol)?;
    
    Ok(InstrumentMetadata {
        symbol,
        digits,
        point_size,
        volume_min,
        volume_max,
        volume_step,
        contract_size,
        instrument_type,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstrumentType {
    CurrencyPair {
        base_currency: String,
        quote_currency: String,
    },
    Cfd {
        symbol: String,
    },
    FuturesContract {
        symbol: String,
        expiry_year: i32,
    },
}

#[derive(Debug, Clone)]
pub struct InstrumentMetadata {
    pub symbol: String,
    pub digits: u8,
    pub point_size: f64,
    pub volume_min: f64,
    pub volume_max: f64,
    pub volume_step: f64,
    pub contract_size: f64,
    pub instrument_type: InstrumentType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use serde_json::json;

    #[test]
    fn test_parse_json_response() {
        let data = r#"{"status": "ok"}"#;
        let result = parse_json_response(data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_json_invalid() {
        let data = "invalid json";
        let result = parse_json_response(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_string_field() {
        let obj = json!({"name": "test"});
        let result = extract_string_field(&obj, "name");
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_extract_string_field_missing() {
        let obj = json!({"name": "test"});
        let result = extract_string_field(&obj, "missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_mt5_timestamp() {
        let timestamp = 1640995200; // 2022-01-01 00:00:00 UTC
        let result = parse_mt5_timestamp(timestamp);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2022);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn test_parse_instrument_symbol_eurusd() {
        let result = parse_instrument_symbol("EURUSD");
        assert!(result.is_ok());
        if let Ok(InstrumentType::CurrencyPair { base_currency, quote_currency }) = result {
            assert_eq!(base_currency, "EUR");
            assert_eq!(quote_currency, "USD");
        }
    }

    #[test]
    fn test_parse_instrument_symbol_cfd() {
        let result = parse_instrument_symbol("US30");
        assert!(result.is_ok());
        if let Ok(InstrumentType::Cfd { symbol }) = result {
            assert_eq!(symbol, "US30");
        }
    }

    #[test]
    fn test_parse_price_with_precision() {
        let result = parse_price(1.23456789, 5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1.23457);
    }

    #[test]
    fn test_parse_volume() {
        let result = parse_volume(0.1, 0.01, 100.0, 0.01);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.1);
    }

    #[test]
    fn test_parse_instrument_metadata() {
        let obj = json!({
            "symbol": "EURUSD",
            "digits": 5,
            "point_size": 0.00001,
            "volume_min": 0.01,
            "volume_max": 100.0,
            "volume_step": 0.01,
            "contract_size": 100000.0
        });
        let result = parse_instrument_metadata(&obj);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.symbol, "EURUSD");
        assert_eq!(metadata.digits, 5);
    }
}
