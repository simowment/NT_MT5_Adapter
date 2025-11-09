# NT_MT5_Adapter

Nautilus Trader MetaTrader 5 Adapter - A Rust adapter for integrating MetaTrader 5 (MT5) with Nautilus Trader.

## Architecture Overview

This repository is organized as a Cargo workspace with the following structure:

```
.
├── crates/
│   └── adapters/
│       └── mt5/                    # MT5 adapter crate
│           ├── src/
│           │   ├── lib.rs          # Module orchestration
│           │   ├── client/         # HTTP and WebSocket clients
│           │   ├── consts.rs       # Constants
│           │   ├── enums.rs        # Enumerations
│           │   ├── urls.rs         # URL management
│           │   ├── credential.rs   # Credential handling
│           │   ├── parse.rs        # Parsing utilities
│           │   └── bindings.rs     # Python bindings
│           ├── tests/              # Integration tests
│           ├── Cargo.toml
│           └── README.md
├── test_data/                      # Shared test fixtures and MT5 payloads
├── Cargo.toml                      # Workspace configuration
├── rust-toolchain.toml             # Rust version specification
└── README.md                       # This file
```

## Key Features

- **Modular Architecture**: Clean separation of concerns with dedicated modules for different functionality
- **Async Runtime**: Built on Tokio for high-performance async operations
- **HTTP and WebSocket Support**: Dual connectivity options for REST API and real-time streaming
- **Python Integration**: Optional PyO3 bindings for Python applications
- **Type-Safe**: Comprehensive use of Rust's type system with serde for serialization
- **Error Handling**: Structured error types using thiserror for reliable error management
- **Testing**: Built-in test infrastructure with mocking and async test support

## Building

### Prerequisites

- Rust 1.70 or later (specified in `rust-toolchain.toml`)
- Cargo

### Build Commands

Build the entire workspace:

```bash
cargo build
```

Build the MT5 adapter crate specifically:

```bash
cargo build -p nautilus-adapters-mt5
```

Build with Python bindings enabled:

```bash
cargo build -p nautilus-adapters-mt5 --features python-bindings
```

Build in release mode:

```bash
cargo build --release
```

## Testing

Run all workspace tests:

```bash
cargo test
```

Run tests for the MT5 adapter:

```bash
cargo test -p nautilus-adapters-mt5
```

Run tests with output:

```bash
cargo test -p nautilus-adapters-mt5 -- --nocapture
```

Run integration tests:

```bash
cargo test -p nautilus-adapters-mt5 --test '*'
```

## Workspace Dependencies

### Core Dependencies

- **nautilus-network**: Networking utilities
- **nautilus-core**: Core Nautilus Trader functionality

### Async & Runtime

- **tokio**: Async runtime and utilities

### Serialization

- **serde**: Serialization framework
- **serde_json**: JSON serialization

### Utilities

- **ustr**: Interned strings for efficient memory usage
- **derive_builder**: Builder pattern derivation
- **thiserror**: Error type derivation

### Python Integration

- **pyo3**: Python bindings with `extension-module` feature

### Dev Dependencies

- **axum**: Web framework for test utilities
- **tokio-test**: Async testing utilities
- **wiremock**: HTTP mocking for tests

## Development

### Adding New Dependencies

Edit the workspace `Cargo.toml` to add new dependencies that should be shared across crates, or edit the crate-specific `Cargo.toml` for crate-local dependencies.

### Module Organization

Each module in the adapter serves a specific purpose:

- `consts.rs`: Configuration constants
- `enums.rs`: Enumerated types for MT5 operations
- `urls.rs`: URL building and management
- `credential.rs`: MT5 credential storage and validation
- `parse.rs`: Response parsing and data extraction
- `client/http.rs`: HTTP REST client implementation
- `client/ws.rs`: WebSocket streaming client
- `bindings.rs`: Python-Rust interoperability

### Testing Patterns

Tests are included alongside their implementations using Rust's module system. For integration tests, add test files to the `crates/adapters/mt5/tests/` directory.

### Test Data

MT5 payload fixtures and canonical test data should be stored in `test_data/` directory for easy access and version control.

## Features and Configuration

The adapter supports feature flags for conditional compilation:

- `python-bindings`: Enable PyO3 Python bindings (default: disabled)

Enable features when building:

```bash
cargo build --features python-bindings
```

## License

LGPL-3.0-or-later

## Contributing

Please follow the existing code style and patterns. Ensure all tests pass before submitting changes.
