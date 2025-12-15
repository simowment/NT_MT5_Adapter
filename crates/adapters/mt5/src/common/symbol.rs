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

//! Symbol handling for the MT5 adapter.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents an MT5 symbol with associated metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5Symbol {
    /// The raw symbol string as used by MT5
    pub symbol: String,
    /// The symbol type (e.g., forex, cfd, futures)
    pub symbol_type: Mt5SymbolType,
    /// The base currency
    pub base_currency: String,
    /// The quote currency
    pub quote_currency: String,
    /// The settlement currency
    pub settlement_currency: String,
    /// The price precision (number of decimal places)
    pub price_precision: u8,
    /// The volume precision
    pub volume_precision: u8,
    /// The minimum lot size
    pub min_lot_size: f64,
    /// The maximum lot size
    pub max_lot_size: f64,
    /// The lot step
    pub lot_step: f64,
    /// The pip size
    pub pip_size: f64,
    /// Whether the symbol is enabled for trading
    pub enabled: bool,
}

impl Mt5Symbol {
    /// Creates a new MT5 symbol.
    pub fn new(
        symbol: String,
        symbol_type: Mt5SymbolType,
        base_currency: String,
        quote_currency: String,
        settlement_currency: String,
        price_precision: u8,
        volume_precision: u8,
        min_lot_size: f64,
        max_lot_size: f64,
        lot_step: f64,
        pip_size: f64,
        enabled: bool,
    ) -> Self {
        Self {
            symbol,
            symbol_type,
            base_currency,
            quote_currency,
            settlement_currency,
            price_precision,
            volume_precision,
            min_lot_size,
            max_lot_size,
            lot_step,
            pip_size,
            enabled,
        }
    }

    /// Returns the symbol as a string reference.
    pub fn as_str(&self) -> &str {
        &self.symbol
    }

    /// Returns the trading pair as "BASE_QUOTE" format.
    pub fn trading_pair(&self) -> String {
        format!("{}_{}", self.base_currency, self.quote_currency)
    }
}

impl fmt::Display for Mt5Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

/// Represents the type of MT5 symbol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mt5SymbolType {
    /// Forex currency pair
    #[serde(rename = "forex")]
    Forex,
    /// Stock CFD
    #[serde(rename = "stock")]
    Stock,
    /// Index CFD
    #[serde(rename = "index")]
    Index,
    /// Commodity CFD
    #[serde(rename = "commodity")]
    Commodity,
    /// Energy CFD
    #[serde(rename = "energy")]
    Energy,
    /// Futures
    #[serde(rename = "futures")]
    Futures,
    /// Cryptocurrency
    #[serde(rename = "crypto")]
    Crypto,
    /// Other instrument type
    #[serde(rename = "other")]
    Other,
}

impl fmt::Display for Mt5SymbolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mt5SymbolType::Forex => write!(f, "Forex"),
            Mt5SymbolType::Stock => write!(f, "Stock"),
            Mt5SymbolType::Index => write!(f, "Index"),
            Mt5SymbolType::Commodity => write!(f, "Commodity"),
            Mt5SymbolType::Energy => write!(f, "Energy"),
            Mt5SymbolType::Futures => write!(f, "Futures"),
            Mt5SymbolType::Crypto => write!(f, "Crypto"),
            Mt5SymbolType::Other => write!(f, "Other"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mt5_symbol_creation() {
        let symbol = Mt5Symbol::new(
            "EURUSD".to_string(),
            Mt5SymbolType::Forex,
            "EUR".to_string(),
            "USD".to_string(),
            "USD".to_string(),
            5,
            2,
            0.01,
            100000.0,
            0.01,
            0.0001,
            true,
        );

        assert_eq!(symbol.symbol, "EURUSD");
        assert_eq!(symbol.symbol_type, Mt5SymbolType::Forex);
        assert_eq!(symbol.base_currency, "EUR");
        assert_eq!(symbol.quote_currency, "USD");
        assert_eq!(symbol.trading_pair(), "EUR_USDX");
    }

    #[test]
    fn test_symbol_display() {
        let symbol = Mt5Symbol::new(
            "EURUSD".to_string(),
            Mt5SymbolType::Forex,
            "EUR".to_string(),
            "USD".to_string(),
            "USD".to_string(),
            5,
            2,
            0.01,
            100000.0,
            0.01,
            0.0001,
            true,
        );

        assert_eq!(format!("{}", symbol), "EURUSD");
    }
}