//! CAI Output - Output formatters
//!
//! Trait-based output formatting for AI coding interaction data.

#![warn(missing_docs)]

pub mod formats;
pub mod formatter;

pub use formatter::{Formatter, FormatterConfig};

/// Re-export formatters
pub use formats::{
    AiFormatter, CsvFormatter, JsonFormatter, JsonlFormatter, StatsFormatter, TableFormatter,
};

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OutputFormat {
    /// Standard JSON array
    Json,
    /// JSON Lines (newline-delimited JSON)
    Jsonl,
    /// CSV with headers
    Csv,
    /// Pretty terminal table
    Table,
    /// AI-optimized compact format
    Ai,
    /// Summary statistics
    Stats,
}

/// Dynamic formatter that can hold any formatter type
#[non_exhaustive]
pub enum DynFormatter {
    /// JSON array formatter
    Json(JsonFormatter),
    /// JSON Lines formatter
    Jsonl(JsonlFormatter),
    /// CSV formatter
    Csv(CsvFormatter),
    /// Table formatter
    Table(TableFormatter),
    /// AI-optimized formatter
    Ai(AiFormatter),
    /// Statistics formatter
    Stats(StatsFormatter),
}

impl DynFormatter {
    /// Create formatter from output format
    pub fn from_format(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Json => Self::Json(JsonFormatter::default()),
            OutputFormat::Jsonl => Self::Jsonl(JsonlFormatter::default()),
            OutputFormat::Csv => Self::Csv(CsvFormatter::default()),
            OutputFormat::Table => Self::Table(TableFormatter::default()),
            OutputFormat::Ai => Self::Ai(AiFormatter::default()),
            OutputFormat::Stats => Self::Stats(StatsFormatter::default()),
        }
    }

    /// Get the config for this formatter
    pub fn config(&self) -> &FormatterConfig {
        match self {
            Self::Json(f) => f.config(),
            Self::Jsonl(f) => f.config(),
            Self::Csv(f) => f.config(),
            Self::Table(f) => f.config(),
            Self::Ai(f) => f.config(),
            Self::Stats(f) => f.config(),
        }
    }

    /// Set config for this formatter
    pub fn set_config(&mut self, config: FormatterConfig) {
        match self {
            Self::Json(f) => f.set_config(config),
            Self::Jsonl(f) => f.set_config(config),
            Self::Csv(f) => f.set_config(config),
            Self::Table(f) => f.set_config(config),
            Self::Ai(f) => f.set_config(config),
            Self::Stats(f) => f.set_config(config),
        }
    }
}

// Delegate Formatter trait methods
macro_rules! delegate_formatter {
    ($($format:ident),*) => {
        impl Formatter for DynFormatter {
            fn format<W: std::io::Write>(&self, entries: &[cai_core::Entry], writer: &mut W) -> cai_core::Result<()> {
                match self {
                    $(Self::$format(f) => f.format(entries, writer),)*
                }
            }

            fn format_one<W: std::io::Write>(&self, entry: &cai_core::Entry, writer: &mut W) -> cai_core::Result<()> {
                match self {
                    $(Self::$format(f) => f.format_one(entry, writer),)*
                }
            }

            fn config(&self) -> &FormatterConfig {
                match self {
                    $(Self::$format(f) => f.config(),)*
                }
            }

            fn set_config(&mut self, config: FormatterConfig) {
                match self {
                    $(Self::$format(f) => f.set_config(config),)*
                }
            }
        }
    }
}

delegate_formatter!(Json, Jsonl, Csv, Table, Ai, Stats);
