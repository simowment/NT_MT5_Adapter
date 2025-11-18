# NT_MT5_Adapter (WIP)

MetaTrader 5 Adapter for NautilusTrader

## Adapter Structure

This adapter is divided into two main parts:

1. **Rust Part**: in `crates/adapters/mt5/`
   - Handles HTTP communications with the MT5 bridge
   - Provides Python bindings via PyO3
   - Implements low-level functionality

2. **Python Part**: in `nautilus_trader/adapters/metatrader5/`
   - Integrates the adapter into the NautilusTrader system
   - Provides data and execution clients
   - Manages configuration and instrument providers

## Compilation

To compile the adapter with Python bindings:

```bash
# Compile in development mode
maturin develop --features python-bindings

# Or compile in release mode
maturin build --release --features python-bindings
```

Or with Cargo directly:

```bash
cargo build --features python-bindings
```

## Requirements

- Rust (version specified in `rust-toolchain.toml`)
- Python 3.8+
- maturin or pyo3 for Python bindings
- A running MT5 bridge service (e.g., an HTTP server that communicates with MT5)

## Testing

To test the adapter:

```bash
python Adapter_Backtest_Test.py
```

