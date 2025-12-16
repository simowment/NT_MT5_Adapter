# NautilusTrader MT5 Adapter

A high-performance **MetaTrader 5 (MT5)** adapter for **NautilusTrader**.

This adapter connects NautilusTrader to MT5 via a lightweight **Open Source REST Middleware** (Python-based), enabling both **Live Trading** and **Data Acquisition**.

---

## ⚠️ Prerequisites & Middleware

**This adapter requires the MT5 REST Server middleware.**
MetaTrader 5 does not support REST/WebSockets natively. You must run the middleware server alongside your MT5 terminal (on the same machine).

**Middleware Repository:**  
👉 [MT5_REST_Server (GitHub)](https://github.com/simowment/MT5_REST_Server)

### Setup Steps
1.  **Install MT5** and log in to your broker account.
2.  Enable **"Allow automated trading"** in MT5 (Tools -> Options -> Expert Advisors).
3.  **Clone & Run the Middleware**: `python server.py` (Default: `http://localhost:5000`).
4.  **Important**: In MT5 -> Tools -> Options -> Charts, set **"Max bars in chart"** to **Unlimited** to allow downloading large history.

---

## ⚡ Features

### 1. Robust Data Fetching
*   **Automatic Pagination**: Request unlimited historical data. The adapter automatically chunks requests (e.g., 30 days for M1 bars, 6 hours for Ticks) to bypass HTTP and API limits.
*   **Gap Handling**: Gracefully handles market closures and missing data.
*   **Correct Instrument Mapping**: Automatically detects **Forex** vs **CFDs** (Indices, Crypto, Stocks) based on instrument path.

### 2. Live Order Execution
*   **Netting Mode**: Designed for Netting accounts (standard for algo trading).
*   **Advanced Order Tags**: Support for Stop Loss and Take Profit via Nautilus tags.
    ```python
    # Example: Send Market Buy with SL/TP
    order = self.order_factory.market(
        instrument_id,
        quantity,
        tags={"sl": 1.0500, "tp": 1.0800} 
    )
    self.submit_order(order)
    ```

---

## 🛠️ Installation & Building

This adapter uses **Rust** for performance handling of JSON and HTTP.

1.  **Install dependencies**:
    ```bash
    pip install maturin nautilus_trader
    ```

2.  **Build and Install**:
    ```bash
    maturin develop --features python-bindings
    ```

---

## 🧪 Testing

We provide a comprehensive test suite to verify connectivity and logic.

### 1. Basic Connection Test
Verifies that the adapter can talk to the MT5 Middleware.
```bash
python tests/test_bindings.py
```

### 2. Backtest Data Test (Pagination Check)
Fetches a large range of historical data (e.g., 30 days of M1 bars) to verify pagination logic.
```bash
python tests/test_backtest.py
```

### 3. Live Trading Test
Places a real order (Demo recommended!) to verify execution logic.
```bash
python tests/test_live_strategy.py
```

---

## 📝 Configuration

Configure the adapter in your NautilusTrader script:

```python
from nautilus_mt5 import Mt5DataClientConfig, Mt5ExecClientConfig, Mt5InstrumentProviderConfig

# Adapter Config
provider_config = Mt5InstrumentProviderConfig(
    base_url="http://localhost:5000",
    filter_cfds=True,                # Include Indices/Crypto
    filter_currencies=("EUR", "USD") # Optional filter
)

data_config = Mt5DataClientConfig(
    base_url="http://localhost:5000",
    instrument_provider=provider_config
)

exec_config = Mt5ExecClientConfig(
    base_url="http://localhost:5000",
    instrument_provider=provider_config,
    max_concurrent_orders=50
)
```

## 🔍 Known Limitations

1.  **Speed**: Fetching millions of M1 bars takes time (HTTP overhead). Use local caching for backtests after the first download.
2.  **Ticks**: Tick data is heavy. Requesting large tick ranges will be slow; use M1 bars where possible.
3.  **Account Type**: Currently optimized for **Netting** accounts. Hedging accounts may cause position tracking desyncs.
