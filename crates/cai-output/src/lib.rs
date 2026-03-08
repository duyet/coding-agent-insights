//! CAI Output - Output formatters
//!
//! Trait-based output formatting for AI coding interaction data.

#![warn(missing_docs, unused_crate_dependencies)]

pub mod formats;
pub mod formatter;

pub use formatter::{Formatter, FormatterConfig};

/// Re-export formatters
pub use formats::{JsonFormatter, JsonlFormatter, CsvFormatter, TableFormatter, AiFormatter, StatsFormatter};
