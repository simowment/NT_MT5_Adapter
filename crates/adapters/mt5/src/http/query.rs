//! Structs for HTTP query parameters in the MetaTrader 5 adapter.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(setter(into, strip_option), default)]
pub struct AccountInfoParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
}

impl Default for AccountInfoParams {
    fn default() -> Self {
        Self {
            login: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(setter(into, strip_option), default)]
pub struct SymbolsInfoParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl Default for SymbolsInfoParams {
    fn default() -> Self {
        Self {
            symbol: None,
            group: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(setter(into, strip_option), default)]
pub struct RatesInfoParams {
    pub symbol: String,
    pub timeframe: String,
    pub from: Option<u64>,
    pub to: Option<u64>,
    pub count: Option<u32>,
}

impl Default for RatesInfoParams {
    fn default() -> Self {
        Self {
            symbol: "EURUSD".to_string(),
            timeframe: "M1".to_string(),
            from: None,
            to: None,
            count: Some(100),
        }
    }
}