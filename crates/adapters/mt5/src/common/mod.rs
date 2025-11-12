//! Common types and utilities for the MetaTrader 5 adapter.
//!
//! This module contains shared functionality including:
//! - Constants and configuration values
//! - Authentication credentials
//! - Common enums and data structures
//! - Parsing utilities
//! - URL management
//! - Testing utilities

pub mod consts;
pub mod credential;
pub mod enums;
pub mod models;
pub mod parse;
pub mod symbol;
pub mod urls;
pub mod testing;

pub use consts::*;
pub use credential::*;
pub use enums::*;
pub use models::*;
pub use parse::*;
pub use symbol::*;
pub use urls::*;
