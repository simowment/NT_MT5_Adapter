# NautilusTrader MT5 Adapter

A high-performance **MetaTrader 5 (MT5)** adapter for **NautilusTrader**.

This adapter connects NautilusTrader to MT5 via a lightweight **Open Source REST Middleware** (Python-based), enabling both **Live Trading** and **Data Acquisition**. It leverages **Rust** for performance-critical data handling and JSON parsing.

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
*   **Rust Core Acceleration**: Historical bar fetching is implemented in Rust for high-performance pagination and parsing.
*   **Automatic Pagination**: Request unlimited historical data. The adapter automatically chunks requests (e.g., 30 days for M1 bars, 6 hours for Ticks).
*   **Gap Handling**: Gracefully handles market closures and missing data.
*   **Correct Instrument Mapping**: Automatically detects **Forex** vs **CFDs** (Indices, Crypto, Stocks).

### 2. Live Order Execution
*   **Netting Mode**: Optimized for Netting accounts (standard for algorithmic trading).
*   **Real-time Lifecycle**: Full support for order lifecycle events (Submitted, Accepted, Rejected, Filled).
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

This adapter uses a mixed **Python/Rust** architecture.

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

### 1. Backtest Data Test (Pagination Check)
Fetches historical data and runs a sample EMACross strategy to verify the full pipeline.
```bash
python tests/test_backtest.py
```

### 2. Data Streaming Test
Verifies that quote and trade ticks are correctly received via long-polling.
```bash
python tests/test_data.py
```

### 3. Live Order Test
Tests order placement, modification, and cancellation (Demo account recommended!).
```bash
python tests/test_live_orders.py
```

### 4. Live Strategy Test
Runs a simple periodic market order strategy.
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

1.  **Account Type**: Currently optimized for **Netting** accounts. Hedging accounts are functional but may cause position tracking intricacies in complex scenarios.
2.  **HTTP Overhead**: Fetching millions of M1 bars is subject to HTTP latency. Use local caching (Nautilus standard `Cache`) for repeated backtests.
3.  **Order Matching**: Order modification and cancellation are supported but depend on the broker's MT5 execution rules.

---

## 📜 License
Licensed under the GNU Lesser General Public License Version 3.0 (LGPL-3.0).
