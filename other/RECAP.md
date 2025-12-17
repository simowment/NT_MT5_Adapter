# Project Status Recap: MT5 Adapter

## Current Status: ✅ Working

### Backtesting
- **Status**: ✅ Fully functional
- Fetches bars from MT5 REST API
- Converts to NautilusTrader `Bar` objects with correct precision
- Runs backtests with EMACross strategy

### Live Trading
- **Status**: ✅ Working
- Receives live quote ticks via long-polling
- Instruments load correctly
- Order submission working (market orders tested)
- Timer callbacks working
- Account state properly registered with cache

---

## Quick Start

### Live Trading

```python
from nautilus_mt5 import (
    MT5,
    Mt5DataClientConfig,
    Mt5ExecClientConfig,
    Mt5LiveDataClientFactory,
    Mt5LiveExecClientFactory,
)
from nautilus_trader.config import TradingNodeConfig
from nautilus_trader.live.node import TradingNode

# Configure (uses localhost:5000 by default)
config = TradingNodeConfig(
    trader_id="TRADER-001",
    data_clients={MT5: Mt5DataClientConfig()},
    exec_clients={MT5: Mt5ExecClientConfig()},
)

# Build and run
node = TradingNode(config=config)
node.add_data_client_factory(MT5, Mt5LiveDataClientFactory)
node.add_exec_client_factory(MT5, Mt5LiveExecClientFactory)
node.build()
node.run()
```

### Research / Scripting

```python
from nautilus_mt5 import Mt5Client

async def main():
    client = Mt5Client()
    await client.connect()
    bars = await client.fetch_bars("EURUSD", "H1", count=1000)
    await client.disconnect()
```

---

## Architecture

```
nautilus_mt5/
├── __init__.py          # Package exports
├── bindings.py          # PyO3 extension loader
├── client.py            # High-level Mt5Client for scripting
├── common.py            # MT5, MT5_VENUE constants
├── config.py            # Configuration classes
├── constants.py         # MT5 enums (timeframes, order types, etc.)
├── data.py              # Mt5DataClient (LiveMarketDataClient)
├── execution.py         # Mt5ExecutionClient (LiveExecutionClient)
├── factories.py         # Factory classes for TradingNode
└── providers.py         # Mt5InstrumentProvider
```

---

## Key Components

### 1. Data Client (`data.py`)
- Implements `LiveMarketDataClient`
- Long-polling mechanism with adaptive intervals
- Deduplication of ticks
- Proper conversion to `QuoteTick`, `TradeTick`, `Bar`

### 2. Execution Client (`execution.py`)
- Implements `LiveExecutionClient`
- Order submit/modify/cancel
- Order lifecycle events (Submitted, Accepted, Rejected, Filled)
- Report generation (positions, fills, orders)

### 3. Instrument Provider (`providers.py`)
- Loads instrument definitions from MT5
- Creates proper `CurrencyPair` objects with correct precision

### 4. High-Level Client (`client.py`)
- `Mt5Client` for research/scripting
- Abstracts REST API complexities
- `fetch_bars()`, `fetch_ticks()` methods

---


## Session Changes (2025-12-16)

### Fixed
1. **Order submission** - Now properly emits order lifecycle events:
   - `generate_order_submitted()` before HTTP request
   - `generate_order_accepted()` on success
   - `generate_order_rejected()` on failure
   - `generate_order_filled()` for market orders
2. **Account registration** - Uses `_set_account_id()` and `generate_account_state()` to properly register account with cache
3. **Order attribute access** - Fixed `order.side` (was incorrectly using `order.order_side`)
4. **Long-polling mechanism** - Implemented with:
   - Adaptive intervals (100ms-5s)
   - Tick deduplication via `time_msc` tracking
   - Uses `_handle_data()` for streaming individual ticks

### Added
1. **Order event generation** - Full order lifecycle event emission
2. **Account state initialization** - Generates initial `AccountState` on connect
3. **Mt5Client** (`client.py`) - High-level client for scripting
4. **test_live.py** - Live trading test with simple strategy

---

## TODOs / Improvements Needed

### Code Quality
1. **Reduce code repetition in execution.py**:
   - Multiple `_generate_order_*` helper methods have similar patterns
   - Extract common logic into reusable utilities
   - Consider creating an `OrderEventFactory` class

2. **Consolidate order type conversion**:
   - `_convert_order_type()` logic is duplicated in multiple places
   - Move to shared utility

3. **Simplify report parsing**:
   - `_parse_order_status_report()`, `_parse_fill_report()`, `_parse_position_status_report()` have overlapping logic
   - Extract common field extraction into base method

### Missing Features
1. **Trade tick polling** - `_poll_trades` is placeholder only
2. **Limit/Stop order handling** - Market orders work, but limit/stop orders need full testing
3. **Order modification events** - `_generate_order_updated()` needs cache order lookup
4. **SL/TP extraction** - Currently not implemented for bracket orders
5. **Position reconciliation** - Not fully tested

### Error Handling
1. **Better error messages** - More descriptive MT5 error code translation
2. **Retry logic** - Consider adding retry for transient network errors

### Testing
1. **Unit tests** - Add proper unit tests for execution client
2. **Integration tests** - Test all order types (limit, stop, etc.)
3. **Error case testing** - Test rejection scenarios

---

## Test Files

| File | Purpose |
|------|---------|
| `tests/test_backtest.py` | Backtest with MT5 data + EMACross strategy |
| `tests/test_live.py` | Live trading test with periodic buy orders |
| `tests/test_live_strategy.py` | Direct order placement test |

---

## Reference Commands

```bash
# Rebuild Rust extension
maturin develop --features python-bindings

# Run backtest
python tests/test_backtest.py

# Run live test (requires MT5 middleware running)
python tests/test_live.py
```

---

## Dependencies

- NautilusTrader >= 1.200
- MT5 REST API middleware running at `http://localhost:5000`
- Python 3.12+
