pub mod bindings;

/// Python bindings for the MetaTrader 5 adapter using PyO3.
///
/// This module re-exports Rust functionality to Python through PyO3,
/// making it available to the Python layer of the adapter.

// This module intentionally left minimal - all PyO3 bindings are in bindings.rs