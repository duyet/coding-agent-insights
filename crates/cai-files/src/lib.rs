//! CAI Files - Direct file operations, no storage layer
//!
//! This crate provides file scanning, loading, and filtering for CAI.
//! Files are the source of truth - no database, no caching, no syncing.

#![warn(missing_docs)]

pub use cai_core::{Error, Result};

mod scanner;
mod loader;
mod filter;

pub use scanner::{FileScanner, ScanFilter};
pub use loader::{FileLoader, FormatVersion};
pub use filter::FileFilterOps;
