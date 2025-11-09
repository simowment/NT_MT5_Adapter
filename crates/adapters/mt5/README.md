# Nautilus Trader MetaTrader 5 Adapter

A Rust adapter for integrating MetaTrader 5 (MT5) with Nautilus Trader, providing both HTTP REST and WebSocket APIs for real-time trading data and order management.

## Features

- **HTTP Client**: REST API client for account information, symbols, and rates
- **WebSocket Client**: Real-time data streaming capabilities
- **Credential Management**: Secure handling of MT5 login credentials
- **Parsing Utilities**: JSON response parsing and field extraction
- **Python Bindings**: Optional PyO3 bindings for Python integration

## Architecture

The adapter follows a layered architecture:

```
┌─ src/
│  ├─ lib.rs              # Module orchestration
│  ├─ consts.rs           # Constants
│  ├─ enums.rs            # Enumerations
│  ├─ urls.rs             # URL management
│  ├─ credential.rs       # Credential handling
│  ├─ parse.rs            # Parsing utilities
│  ├─ bindings.rs         # Python bindings
│  └─ client/
│     ├─ mod.rs           # Client module
│     ├─ http.rs          # HTTP REST client
│     └─ ws.rs            # WebSocket client
├─ tests/                 # Integration tests
└─ README.md
```

## Building

Build the adapter crate:

```bash
cargo build -p nautilus-adapters-mt5
```

Build with Python bindings:

```bash
cargo build -p nautilus-adapters-mt5 --features python-bindings
```

## Testing

Run all tests:

```bash
cargo test -p nautilus-adapters-mt5
```

Run specific test suite:

```bash
cargo test -p nautilus-adapters-mt5 --lib
```

Run integration tests:

```bash
cargo test -p nautilus-adapters-mt5 --test '*'
```

## Dependencies

- **Core**: `nautilus-network`, `nautilus-core`
- **Async**: `tokio`
- **Serialization**: `serde`, `serde_json`, `ustr`
- **Utilities**: `derive_builder`, `thiserror`
- **Python**: `pyo3` (with `extension-module` feature for bindings)

## Dev Dependencies

- `axum`: Web framework for testing
- `tokio-test`: Testing utilities for async code
- `wiremock`: HTTP mocking for tests

## License

LGPL-3.0-or-later
