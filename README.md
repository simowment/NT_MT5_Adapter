# NT_MT5_Adapter

MetaTrader 5 Adapter for NautilusTrader

## ⚠️ MT5 REST Server Required

**This adapter requires a running MT5 REST API server** to communicate with MetaTrader 5. Since MT5 doesn't natively support REST APIs, you must use a middleware server that exposes MT5 functionality via HTTP.

This one is made to work with this adapter: https://github.com/simowment/MT5_REST_Server.git

The MT5 REST server should:
- Run on `http://localhost:5000` (default, configurable)
- Expose endpoints like `/api/account_info`, `/api/symbol_info`, `/api/order_send`, etc.
- Use POST requests with JSON payloads
- Return JSON responses in the format `{"result": ...}` or `{"error": "..."}`

See `MT5_REST_API_Documentation.md` for the complete API specification.

---

## Adapter Structure

This adapter is divided into two main parts:

1. **Rust Part**: `crates/adapters/mt5/`
   - HTTP client for communicating with MT5 REST server
   - Python bindings via PyO3
   - Low-level data models and parsing

2. **Python Part**: `nautilus_mt5_adapter`
   - NautilusTrader integration (Data/Execution clients)
   - Factory classes for TradingNode
   - Instrument providers and configuration

---

## Quick Start


### 1. Compile the Rust Bindings

```bash
# Development mode
maturin develop --features python-bindings

# Or release mode
maturin build --release --features python-bindings
```

### 2. Start the MT5 REST Server

Ensure your MT5 REST middleware server is running (`http://localhost:5000` by default).

### 3. Verify Installation

```bash
python -c "import nautilus_mt5; print(nautilus_mt5.__version__)"
```

### 3. Run Tests

```bash
# Basic connectivity test
python testbindings.py

# Order placement test (requires AutoTrading enabled in MT5)
python test_live_strategy.py

# Historical data fetch for backtesting
python Adapter_Backtest_Test.py
```

---

## Usage with NautilusTrader

```python
from nautilus_trader.adapters.metatrader5 import (
    MT5_VENUE,
    Mt5DataClientConfig,
    Mt5ExecClientConfig,
    Mt5LiveDataClientFactory,
    Mt5LiveExecClientFactory,
)

# Add factories to the trading node
node.add_data_client_factory("MT5", Mt5LiveDataClientFactory)
node.add_exec_client_factory("MT5", Mt5LiveExecClientFactory)

# Configure clients
config = TradingNodeConfig(
    data_clients={
        "MT5": Mt5DataClientConfig(
            mt5_base_url="http://localhost:5000",
        ),
    },
    exec_clients={
        "MT5": Mt5ExecClientConfig(
            mt5_base_url="http://localhost:5000",
        ),
    },
)
```

---

## Requirements

- **MT5 REST Server** - A middleware server exposing MT5 via HTTP (required)
- **Rust** - Version specified in `rust-toolchain.toml`
- **Python** - 3.8+
- **maturin** - For building Python bindings

---
