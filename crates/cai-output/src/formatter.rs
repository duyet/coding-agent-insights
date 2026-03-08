//! Core formatter trait and configuration

use cai_core::{Entry, Result};
use std::io::Write;

/// Configuration for formatters
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct FormatterConfig {
    /// Maximum width for table output (0 = no limit)
    pub max_width: usize,
    /// Enable colored output where supported
    pub colorize: bool,
    /// Truncate long fields to this length (0 = no truncation)
    pub truncate: usize,
    /// Number of entries to include (0 = all)
    pub limit: usize,
}

/// Core formatter trait for output formats
pub trait Formatter: Send + Sync {
    /// Format entries and write to output
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()>;

    /// Format a single entry for streaming output
    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()>;

    /// Get the configuration for this formatter
    fn config(&self) -> &FormatterConfig;

    /// Set configuration for this formatter
    fn set_config(&mut self, config: FormatterConfig);
}

/// Helper trait for formatters that need to truncate text
pub(crate) trait Truncate {
    fn truncate_text(&self, text: &str, limit: usize) -> String;
}

impl Truncate for FormatterConfig {
    fn truncate_text(&self, text: &str, limit: usize) -> String {
        if limit == 0 || text.len() <= limit {
            text.to_string()
        } else {
            format!("{}...", text.chars().take(limit.saturating_sub(3)).collect::<String>())
        }
    }
}
