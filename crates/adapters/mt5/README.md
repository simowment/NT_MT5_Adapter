# Nautilus Trader MetaTrader 5 Adapter

A Rust adapter for integrating MetaTrader 5 (MT5) with Nautilus Trader, providing both HTTP REST and WebSocket APIs for real-time trading data and order management.

## Features

- **HTTP Client**: REST API client for account information, symbols, and rates
- **WebSocket Client**: Real-time data streaming capabilities
- **Credential Management**: Secure handling of MT5 login credentials
- **Parsing Utilities**: JSON response parsing and field extraction
- **Python Bindings**: Optional PyO3 bindings for Python integration

## Architecture

The adapter follows a layered architecture pattern with:

- **Rust core** for networking clients and performance-critical operations
- **Python layer** (optional) for integrating into the legacy system

The Rust layer handles:

- HTTP client: Raw API communication, request signing, rate limiting
- WebSocket client: Low-latency streaming connections, message parsing
- Parsing: Fast conversion of venue data to Nautilus domain models
- Python bindings: PyO3 exports to make Rust functionality available to Python

Typical Rust structure:

```
crates/adapters/your_adapter/
├── src/
│   ├── common/           # Shared types and utilities
│   │   ├── consts.rs     # Venue constants / broker IDs
│   │   ├── credential.rs # API key storage and signing helpers
│   │   ├── enums.rs      # Venue enums mirrored in REST/WS payloads
│   │   ├── urls.rs       # Environment & product aware base-url resolvers
│   │   ├── parse.rs      # Shared parsing helpers
│   │   └── testing.rs    # Fixtures reused across unit tests
│   ├── http/             # HTTP client implementation
│   │   ├── client.rs     # HTTP client with authentication
│   │   ├── models.rs     # Structs for REST payloads
│   │   ├── query.rs      # Request and query builders
│   │   └── parse.rs      # Response parsing functions
│   ├── websocket/        # WebSocket implementation
│   │   ├── client.rs     # WebSocket client
│   │   ├── messages.rs   # Structs for stream payloads
│   │   └── parse.rs      # Message parsing functions
│   ├── python/           # PyO3 Python bindings
│   ├── config.rs         # Configuration structures
│   └── lib.rs            # Library entry point
└── tests/                # Integration tests with mock servers
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
