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

//! Common data models for the MT5 adapter.

use serde::{Deserialize, Serialize};

/// Represents account information from MT5.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5AccountInfo {
    /// Account number
    pub login: u64,
    /// Account name
    pub name: String,
    /// Account server
    pub server: String,
    /// Account currency
    pub currency: String,
    /// Account leverage
    pub leverage: u32,
    /// Account balance
    pub balance: f64,
    /// Account equity
    pub equity: f64,
    /// Account profit/loss
    pub profit: f64,
    /// Account margin used
    pub margin: f64,
    /// Account free margin
    pub margin_free: f64,
    /// Account margin level
    pub margin_level: f64,
}

impl Mt5AccountInfo {
    /// Creates a new MT5 account info instance.
    pub fn new(
        login: u64,
        name: String,
        server: String,
        currency: String,
        leverage: u32,
        balance: f64,
        equity: f64,
        profit: f64,
        margin: f64,
        margin_free: f64,
        margin_level: f64,
    ) -> Self {
        Self {
            login,
            name,
            server,
            currency,
            leverage,
            balance,
            equity,
            profit,
            margin,
            margin_free,
            margin_level,
        }
    }
}

/// Represents a trading position in MT5.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5Position {
    /// Position ticket number
    pub ticket: u64,
    /// Symbol of the position
    pub symbol: String,
    /// Position type (buy/sell)
    pub pos_type: Mt5PositionType,
    /// Position volume
    pub volume: f64,
    /// Position price
    pub price: f64,
    /// Position profit
    pub profit: f64,
    /// Position swap
    pub swap: f64,
    /// Position commission
    pub commission: f64,
    /// Position comment
    pub comment: String,
    /// Position magic number
    pub magic: u64,
    /// Position time
    pub time: u64,
}

impl Mt5Position {
    /// Creates a new MT5 position instance.
    pub fn new(
        ticket: u64,
        symbol: String,
        pos_type: Mt5PositionType,
        volume: f64,
        price: f64,
        profit: f64,
        swap: f64,
        commission: f64,
        comment: String,
        magic: u64,
        time: u64,
    ) -> Self {
        Self {
            ticket,
            symbol,
            pos_type,
            volume,
            price,
            profit,
            swap,
            commission,
            comment,
            magic,
            time,
        }
    }
}

/// Represents the type of position in MT5.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mt5PositionType {
    /// Buy position
    #[serde(rename = "buy")]
    Buy,
    /// Sell position
    #[serde(rename = "sell")]
    Sell,
}

/// Represents a trade in MT5.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5Trade {
    /// Trade ticket number
    pub ticket: u64,
    /// Symbol of the trade
    pub symbol: String,
    /// Trade type (buy/sell)
    pub trade_type: Mt5TradeType,
    /// Trade volume
    pub volume: f64,
    /// Trade price
    pub price: f64,
    /// Trade profit
    pub profit: f64,
    /// Trade commission
    pub commission: f64,
    /// Trade swap
    pub swap: f64,
    /// Trade comment
    pub comment: String,
    /// Trade magic number
    pub magic: u64,
    /// Trade time
    pub time: u64,
}

impl Mt5Trade {
    /// Creates a new MT5 trade instance.
    pub fn new(
        ticket: u64,
        symbol: String,
        trade_type: Mt5TradeType,
        volume: f64,
        price: f64,
        profit: f64,
        commission: f64,
        swap: f64,
        comment: String,
        magic: u64,
        time: u64,
    ) -> Self {
        Self {
            ticket,
            symbol,
            trade_type,
            volume,
            price,
            profit,
            commission,
            swap,
            comment,
            magic,
            time,
        }
    }
}

/// Represents the type of trade in MT5.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mt5TradeType {
    /// Buy trade
    #[serde(rename = "buy")]
    Buy,
    /// Sell trade
    #[serde(rename = "sell")]
    Sell,
    /// Buy limit order
    #[serde(rename = "buy_limit")]
    BuyLimit,
    /// Sell limit order
    #[serde(rename = "sell_limit")]
    SellLimit,
    /// Buy stop order
    #[serde(rename = "buy_stop")]
    BuyStop,
    /// Sell stop order
    #[serde(rename = "sell_stop")]
    SellStop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_info_creation() {
        let account = Mt5AccountInfo::new(
            12345678,
            "Test Account".to_string(),
            "TestServer".to_string(),
            "USD".to_string(),
            100,
            10000.0,
            10000.0,
            0.0,
            0.0,
            1000.0,
            0.0,
        );

        assert_eq!(account.login, 12345678);
        assert_eq!(account.name, "Test Account");
        assert_eq!(account.currency, "USD");
    }

    #[test]
    fn test_position_creation() {
        let position = Mt5Position::new(
            12345,
            "EURUSD".to_string(),
            Mt5PositionType::Buy,
            1.0,
            1.1,
            10.0,
            0.0,
            0.0,
            "Test position".to_string(),
            0,
            1234567890,
        );

        assert_eq!(position.ticket, 12345);
        assert_eq!(position.symbol, "EURUSD");
        assert_eq!(position.pos_type, Mt5PositionType::Buy);
    }

    #[test]
    fn test_trade_creation() {
        let trade = Mt5Trade::new(
            12345,
            "EURUSD".to_string(),
            Mt5TradeType::Buy,
            1.0,
            1.1,
            10.0,
            0.0,
            0.0,
            "Test trade".to_string(),
            0,
            1234567890,
        );

        assert_eq!(trade.ticket, 12345);
        assert_eq!(trade.symbol, "EURUSD");
        assert_eq!(trade.trade_type, Mt5TradeType::Buy);
    }
}