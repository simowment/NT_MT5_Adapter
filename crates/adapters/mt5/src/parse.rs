//! Parsing utilities for MetaTrader 5 data.

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

#[cfg(test)]
mod tests {
    use super::*;
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
}
