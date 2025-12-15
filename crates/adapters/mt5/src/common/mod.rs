//! Common types and utilities for the MetaTrader 5 adapter.
//!
//! This module contains shared functionality including:
//! - Authentication credentials
//! - Common enums and data structures
//! - Parsing utilities
//! - Symbol handling
//! - Testing utilities

pub mod credential;
pub mod enums;
pub mod models;
pub mod parse;
pub mod symbol;
pub mod testing;

pub use credential::*;
pub use enums::*;
pub use models::*;
pub use parse::*;
pub use symbol::*;
