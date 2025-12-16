# MT5 REST API - Complete Endpoints Documentation

## Overview
This document provides complete documentation for all MT5 REST API endpoints based on actual test results from `MT5_Endpoints_Test.py`.

## Base URL
```
http://localhost:5000/api
```

## Important Note
**All endpoints use POST method only** - This API does not support GET requests. All requests must be sent as POST with JSON parameters in the request body.

## Response Format
All endpoints return JSON in the following format:

**Success Response:**
```json
{
  "result": <data>
}
```

**Error Response:**
```json
{
  "error": "error message"
}
```

## Endpoints Documentation

### 1. BASIC INFORMATION

#### version
**Method:** POST `/api/version`
**Description:** Returns MT5 terminal version information
**Response:**
```json
{
  "result": [500, 5370, "17 Oct 2025"]
}
```
- **500:** Terminal type (500 = MetaTrader 5)
- **5370:** Build number
- **Date:** Last update date

#### terminal_info
**Method:** POST `/api/terminal_info`
**Description:** Returns detailed terminal information
**Response:**
```json
{
  "result": {
    "build": 5370,
    "codepage": 1252,
    "commondata_path": "C:\\Users\\simow\\AppData\\Roaming\\MetaQuotes\\Terminal\\Common",
    "community_account": false,
    "community_balance": 0.0,
    "company": "FTMO Global Markets Ltd",
    "connected": true,
    "currency": "USD",
    "demo_account": false,
    "expert_enabled": true,
    "trade_allowed": true,
    "tradeapi_disabled": false
  }
}
```

#### account_info
**Method:** POST `/api/account_info`
**Description:** Returns current trading account information
**Response:**
```json
{
  "result": {
    "assets": 0.0,
    "balance": 9999.31,
    "commission_blocked": 0.0,
    "company": "FTMO Global Markets Ltd",
    "credit": 0.0,
    "currency": "USD",
    "currency_digits": 2,
    "equity": 9999.31,
    "fifo_close": false,
    "free_margin": 9999.31,
    "free_margin_check": true,
    "free_margin_mode": 0,
    "leverage": 100,
    "liquidation_mode": false,
    "margin": 0.0,
    "margin_free": 9999.31,
    "margin_level": 0.0,
    "margin_so_call": 50.0,
    "margin_so_so": 30.0,
    "name": "Test Account",
    "number": 123456789,
    "profit": 0.0,
    "server": "FTMO-Server",
    "stopout_level": 50.0,
    "stopout_mode": 0,
    "trade_allowed": true,
    "trade_expert": true,
    "trade_mode": 0,
    "trade_server_time": 1762984717,
    "waf": false
  }
}
```

### 2. SYMBOLS MANAGEMENT

#### symbols_total
**Method:** POST `/api/symbols_total`
**Description:** Returns total number of available symbols
**Response:**
```json
{
  "result": 131
}
```

#### symbols_get
**Method:** POST `/api/symbols_get`
**Parameters:** `{"start": 0, "count": 10}`
**Description:** Returns array of symbol information
**Response:**
```json
{
  "result": [
    {
      "ask": 1.15862,
      "askhigh": 1.15978,
      "asklow": 1.15631,
      "bank": "",
      "basis": "",
      "bid": 1.15862,
      "bidhigh": 1.15978,
      "bidlow": 1.15628,
      "category": "",
      "chart_mode": 0,
      "currency_base": "EUR",
      "currency_profit": "USD",
      "currency_margin": "EUR",
      "currency_exchange": "USD",
      "description": "Euro vs US Dollar",
      "exchange": "",
      "formula": "",
      "page": "https://www.investing.com/currencies/eur-usd",
      "path": "Forex\\Major\\EURUSD",
      "point": 0.00001,
      "session_deals": 0,
      "session_buy_orders": 0,
      "session_sell_orders": 0,
      "session_turnover": 0,
      "session_interest": 0,
      "session_awp": 0,
      "session_price_settlement": 0,
      "session_trade_orders": 0,
      "session_trade_deals": 0,
      "session_trade_volume": 0,
      "session_trade_profit": 0,
      "session_open_interest": 0,
      "session_sso_buy_orders": 0,
      "session_sso_sell_orders": 0,
      "session_margin": 0.0,
      "spread": 15,
      "stops_level": 10,
      "swap_long": -3.37,
      "swap_short": 0.1,
      "swap_rollover3days": 3,
      "swap_type": 0,
      "sync_delay": 0,
      "time": 1762984717,
      "time_digits": 5,
      "volume_high": 100,
      "volume_low": 0.01,
      "volume_limit": 0.0,
      "volume_step": 0.01,
      "volume_max": 100,
      "visible": true,
      "description": "Euro vs US Dollar",
      "custom": false,
      "background_color": 16777215
    }
  ]
}
```

#### symbol_info
**Method:** POST `/api/symbol_info`
**Parameters:** `["EURUSD"]`
**Description:** Returns detailed information about specific symbol
**Response:** Same format as `symbols_get` entry

#### symbol_info_tick
**Method:** POST `/api/symbol_info_tick`
**Parameters:** `["EURUSD"]`
**Description:** Returns last tick information for symbol
**Response:**
```json
{
  "result": {
    "ask": 1.15862,
    "bid": 1.15862,
    "flags": 1030,
    "last": 0.0,
    "time": 1762984717,
    "time_msc": 1762984717507,
    "volume": 0,
    "volume_real": 0.0
  }
}
```

#### symbol_select
**Method:** POST `/api/symbol_select`
**Parameters:** `["EURUSD", true]`
**Description:** Selects/deselects symbol in Market Watch
**Parameters:**
- `symbol`: Symbol name
- `select`: true to select, false to deselect
**Response:**
```json
{
  "result": true
}
```

### 3. MARKET DATA

#### copy_ticks_from
**Method:** POST `/api/copy_ticks_from`
**Parameters:** `["EURUSD", <timestamp>, 100, 1]`
**Description:** Copies tick data starting from specified time
**Response:**
```json
{
  "result": [
    [
      1762977463,    // time
      1.1592,        // bid
      1.1592,        // ask
      0.0,           // last
      0,             // volume
      1762977463507, // time_msc
      1030,          // flags
      0.0            // volume_real
    ]
  ]
}
```

#### copy_ticks_range
**Method:** POST `/api/copy_ticks_range`
**Parameters:** `["EURUSD", <start_time>, <end_time>, 1]`
**Description:** Copies tick data for specified time range
**Response:** Same format as `copy_ticks_from`

#### copy_rates_from
**Method:** POST `/api/copy_rates_from`
**Parameters:** `["EURUSD", 1, <timestamp>, 10]`
**Description:** Copies bar data from specified time
**Response:**
```json
{
  "result": [
    [
      1762973340,    // time
      1.15913,       // open
      1.15915,       // high
      1.15907,       // low
      1.15913,       // close
      53,            // tick_volume
      0,             // spread
      0              // real_volume
    ]
  ]
}
```

#### copy_rates_range
**Method:** POST `/api/copy_rates_range`
**Parameters:** `["EURUSD", 1, <start_time>, <end_time>]`
**Description:** Copies bar data for specified time range
**Response:** Same format as `copy_rates_from`

### 4. ORDERS AND POSITIONS

#### orders_total
**Method:** POST `/api/orders_total`
**Description:** Returns number of active pending orders
**Response:**
```json
{
  "result": 0
}
```

#### orders_get
**Method:** POST `/api/orders_get`
**Description:** Returns array of pending orders
**Response:**
```json
{
  "result": []
}
```
*(Empty when no pending orders)*

#### positions_total
**Method:** POST `/api/positions_total`
**Description:** Returns number of open positions
**Response:**
```json
{
  "result": 0
}
```

#### positions_get
**Method:** POST `/api/positions_get`
**Description:** Returns array of open positions
**Response:**
```json
{
  "result": []
}
```
*(Empty when no open positions)*

### 5. HISTORY DATA

#### history_orders_total
**Method:** POST `/api/history_orders_total`
**Parameters:** `[<start_time>, <end_time>]`
**Description:** Returns number of historical orders in specified period
**Response:**
```json
{
  "result": 20
}
```

#### history_orders_get
**Method:** POST `/api/history_orders_get`
**Parameters:** `[<start_time>, <end_time>]`
**Description:** Returns array of historical orders
**Response:**
```json
{
  "result": [
    {
      "comment": "",
      "external_id": "345869922-O",
      "magic": 0,
      "position_by_id": 0,
      "position_id": 345869922,
      "price_current": 105225.09,
      "price_open": 0.0,
      "price_stoplimit": 0.0,
      "reason": 2,
      "request_id": 0,
      "retcode": 10004,
      "sl": 0.0,
      "status": 1,
      "symbol": "XAUUSD",
      "ticket": 345869922,
      "time": 1762977403,
      "time_done": 1762977406,
      "time_setup": 1762977403,
      "tp": 0.0,
      "type": 2,
      "type_filling": 2,
      "type_time": 0,
      "volume": 0.01,
      "volume_done": 0.01
    }
  ]
}
```

#### history_deals_total
**Method:** POST `/api/history_deals_total`
**Parameters:** `[<start_time>, <end_time>]`
**Description:** Returns number of historical deals in specified period
**Response:**
```json
{
  "result": 20
}
```

#### history_deals_get
**Method:** POST `/api/history_deals_get`
**Parameters:** `[<start_time>, <end_time>]`
**Description:** Returns array of historical deals
**Response:**
```json
{
  "result": [
    {
      "comment": "",
      "commission": -0.34,
      "entry": 0,
      "external_id": "319025734",
      "fee": 0.0,
      "magic": 0,
      "order": 345869922,
      "position_id": 345869922,
      "price": 105225.43,
      "profit": 0.0,
      "reason": 2,
      "retcode": 10004,
      "sl": 0.0,
      "swap": 0.0,
      "symbol": "XAUUSD",
      "ticket": 319025734,
      "time": 1762977406,
      "tp": 0.0,
      "type": 2,
      "volume": 0.01
    }
  ]
}
```

### 6. CALCULATIONS

#### order_calc_margin
**Method:** POST `/api/order_calc_margin`
**Parameters:** `[0, "EURUSD", 0.1, 1.0]`
**Description:** Calculates margin requirement for order
**Parameters:**
- `type`: Order type (0 = buy)
- `symbol`: Symbol name
- `volume`: Order volume
- `price`: Order price
**Response:**
```json
{
  "result": 333.33
}
```
*(Margin amount in account currency)*

#### order_calc_profit
**Method:** POST `/api/order_calc_profit`
**Parameters:** `[0, "EURUSD", 0.1, 1.0, 1.001]`
**Description:** Calculates profit for order
**Parameters:**
- `type`: Order type (0 = buy)
- `symbol`: Symbol name  
- `volume`: Order volume
- `price_open`: Open price
- `price_close`: Close price
**Response:**
```json
{
  "result": 10.0
}
```
*(Profit amount in account currency)*

#### order_check
**Method:** POST `/api/order_check`
**Parameters:** Complex order object
**Description:** Checks if order can be placed
**Response:**
```json
{
  "result": {
    "balance": 0.0,
    "comment": "Invalid request",
    "equity": 0.0,
    "margin": 0.0,
    "margin_free": 0.0,
    "margin_level": 0.0,
    "profit": 0.0,
    "request": {...},
    "retcode": 10016
  }
}
```

### 7. ORDER PLACEMENT

#### order_send
**Method:** POST `/api/order_send`
**Description:** Places market/pending order
**Parameters:** Complex order object
**Response:**
```json
{
  "result": {
    "ask": 0.0,
    "bid": 0.0,
    "comment": "Only position closing is allowed",
    "deal": 0,
    "order": 0,
    "price": 0.0,
    "request": {...},
    "retcode": 10016,
    "retcode_external": 0,
    "volume": 0.0,
    "volume_ext": 0.0
  }
}
```

### 8. MARKET BOOK (MICROSTRUCTURE)

#### market_book_add
**Method:** POST `/api/market_book_add`
**Parameters:** `["EURUSD"]`
**Description:** Adds symbol to market book for depth data
**Response:**
```json
{
  "result": true
}
```

#### market_book_get
**Method:** POST `/api/market_book_get`
**Parameters:** `["EURUSD"]`
**Description:** Returns market depth data for symbol
**Response:**
```json
{
  "result": [
    {
      "price": 1.1587,
      "type": 1,
      "volume": 5000000,
      "volume_dbl": 5000000.0
    },
    {
      "price": 1.15868,
      "type": 1,
      "volume": 3000000,
      "volume_dbl": 3000000.0
    }
  ]
}
```
**Types:**
- `type: 1` = Ask (sell orders)
- `type: 2` = Bid (buy orders)

#### market_book_release
**Method:** POST `/api/market_book_release`
**Parameters:** `["EURUSD"]`
**Description:** Removes symbol from market book
**Response:**
```json
{
  "result": true
}
```

### 9. SYSTEM

#### last_error
**Method:** POST `/api/last_error`
**Description:** Returns last MT5 error information
**Response:**
```json
{
  "result": [1, "Success"]
}
```

### 10. SESSION MANAGEMENT

#### initialize
**Method:** POST `/api/initialize`
**Description:** Initializes MT5 terminal connection
**Response:**
```json
{
  "result": true
}
```

#### login
**Method:** POST `/api/login`
**Description:** Login to trading account
**Response:**
```json
{
  "result": false
}
```
*(False for demo account or already logged in)*

#### shutdown
**Method:** POST `/api/shutdown`
**Description:** Shuts down MT5 terminal connection
**Response:**
```json
{
  "result": true
}
```

## Error Codes Reference

- **1:** Success
- **10004:** TRADE_RETCODE_DONE
- **10016:** TRADE_RETCODE_INVALID_VOLUME
- **10017:** TRADE_RETCODE_INVALID_PRICE

## Rate Limiting
- No explicit rate limiting observed
- API calls are synchronous and may block on heavy operations
- Recommended polling intervals: 500ms for live data

## Authentication
- No authentication required for local MT5 API
- Uses MT5 terminal's existing login session

## Common Usage Patterns

### Get Live Market Data
```bash
# 1. Add to market book
curl -X POST http://localhost:5000/api/market_book_add -H "Content-Type: application/json" -d '["EURUSD"]'

# 2. Get depth data (poll every 500ms)
curl -X POST http://localhost:5000/api/market_book_get -H "Content-Type: application/json" -d '["EURUSD"]'

# 3. Release when done
curl -X POST http://localhost:5000/api/market_book_release -H "Content-Type: application/json" -d '["EURUSD"]'
```

### Get Historical Data
```bash
# Get recent ticks
curl -X POST http://localhost:5000/api/copy_ticks_range -H "Content-Type: application/json" -d '["EURUSD", <start_time>, <end_time>, 1]'

# Get candles
curl -X POST http://localhost:5000/api/copy_rates_range -H "Content-Type: application/json" -d '["EURUSD", 1, <start_time>, <end_time>]'
```

### Check Account Status
```bash
curl -X POST http://localhost:5000/api/account_info
curl -X POST http://localhost:5000/api/positions_get
curl -X POST http://localhost:5000/api/orders_get
```

## Notes

- All timestamps are Unix epoch (seconds since 1970)
- Volumes are returned as integers and decimal values
- Prices are returned as float with appropriate precision
- Order types: 0=Buy, 1=Sell, 2=Buy Limit, 3=Sell Limit, 4=Buy Stop, 5=Sell Stop
- Market book types: 1=Ask, 2=Bid
- All amounts are in account base currency unless specified otherwise