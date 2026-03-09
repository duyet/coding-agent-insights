//! Output format implementations

use crate::formatter::Truncate;
use crate::{Formatter, FormatterConfig};
use cai_core::Entry;
use cai_core::Result;
use std::io::Write;

/// JSON array formatter
#[derive(Debug, Clone, Default)]
pub struct JsonFormatter {
    config: FormatterConfig,
}

impl JsonFormatter {
    /// Create a new formatter instance
    pub fn new() -> Self {
        Self::default()
    }
}

impl Formatter for JsonFormatter {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()> {
        serde_json::to_writer(writer, entries)?;
        Ok(())
    }

    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()> {
        serde_json::to_writer(&mut *writer, entry)?;
        writeln!(writer)?;
        Ok(())
    }

    fn config(&self) -> &FormatterConfig {
        &self.config
    }

    fn set_config(&mut self, config: FormatterConfig) {
        self.config = config;
    }
}

/// JSON Lines (newline-delimited JSON) formatter
#[derive(Debug, Clone, Default)]
pub struct JsonlFormatter {
    config: FormatterConfig,
}

impl JsonlFormatter {
    /// Create a new formatter instance
    pub fn new() -> Self {
        Self::default()
    }
}

impl Formatter for JsonlFormatter {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()> {
        for entry in entries {
            self.format_one(entry, writer)?;
        }
        Ok(())
    }

    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()> {
        serde_json::to_writer(&mut *writer, entry)?;
        writeln!(writer)?;
        Ok(())
    }

    fn config(&self) -> &FormatterConfig {
        &self.config
    }

    fn set_config(&mut self, config: FormatterConfig) {
        self.config = config;
    }
}

/// CSV formatter
#[derive(Debug, Clone, Default)]
pub struct CsvFormatter {
    config: FormatterConfig,
}

impl CsvFormatter {
    /// Create a new formatter instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Escape CSV fields containing quotes or commas
    fn escape_field(value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }
}

impl Formatter for CsvFormatter {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()> {
        // Write header
        writeln!(writer, "id,source,timestamp,prompt,response")?;

        for entry in entries {
            self.format_one(entry, writer)?;
        }
        Ok(())
    }

    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()> {
        writeln!(
            writer,
            "{},{},{},{},{}",
            Self::escape_field(&entry.id),
            Self::escape_field(&format!("{:?}", entry.source)),
            Self::escape_field(&entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()),
            Self::escape_field(&entry.prompt),
            Self::escape_field(&entry.response)
        )?;
        Ok(())
    }

    fn config(&self) -> &FormatterConfig {
        &self.config
    }

    fn set_config(&mut self, config: FormatterConfig) {
        self.config = config;
    }
}

/// Table formatter for terminal output
#[derive(Debug, Clone, Default)]
pub struct TableFormatter {
    config: FormatterConfig,
}

impl TableFormatter {
    /// Create a new formatter instance
    pub fn new() -> Self {
        Self::default()
    }
}

impl Formatter for TableFormatter {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()> {
        // Simple table format for now
        for entry in entries {
            writeln!(
                writer,
                "[{}] {:?}",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                entry.source
            )?;
            writeln!(
                writer,
                "  Prompt: {}",
                self.config.truncate_text(&entry.prompt, 80)
            )?;
            writeln!(writer)?;
        }
        Ok(())
    }

    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()> {
        writeln!(
            writer,
            "[{}] {:?}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.source
        )?;
        writeln!(
            writer,
            "  Prompt: {}",
            self.config.truncate_text(&entry.prompt, 80)
        )?;
        writeln!(writer)?;
        Ok(())
    }

    fn config(&self) -> &FormatterConfig {
        &self.config
    }

    fn set_config(&mut self, config: FormatterConfig) {
        self.config = config;
    }
}

/// AI-optimized compact formatter
#[derive(Debug, Clone, Default)]
pub struct AiFormatter {
    config: FormatterConfig,
}

impl AiFormatter {
    /// Create a new formatter instance
    pub fn new() -> Self {
        Self::default()
    }
}

impl Formatter for AiFormatter {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()> {
        for entry in entries {
            writeln!(
                writer,
                "[{}] {:?}: {}",
                entry.timestamp.format("%Y-%m-%d %H:%M"),
                entry.source,
                self.config.truncate_text(&entry.prompt, 60)
            )?;
            writeln!(
                writer,
                "  -> {}",
                self.config.truncate_text(&entry.response, 100)
            )?;
            writeln!(writer)?;
        }
        Ok(())
    }

    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()> {
        writeln!(
            writer,
            "[{}] {:?}: {}",
            entry.timestamp.format("%Y-%m-%d %H:%M"),
            entry.source,
            self.config.truncate_text(&entry.prompt, 60)
        )?;
        writeln!(
            writer,
            "  -> {}",
            self.config.truncate_text(&entry.response, 100)
        )?;
        writeln!(writer)?;
        Ok(())
    }

    fn config(&self) -> &FormatterConfig {
        &self.config
    }

    fn set_config(&mut self, config: FormatterConfig) {
        self.config = config;
    }
}

/// Statistics summary formatter
#[derive(Debug, Clone, Default)]
pub struct StatsFormatter {
    config: FormatterConfig,
}

impl StatsFormatter {
    /// Create a new formatter instance
    pub fn new() -> Self {
        Self::default()
    }
}

impl Formatter for StatsFormatter {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()> {
        writeln!(writer, "=== Summary Statistics ===")?;
        writeln!(writer, "Total entries: {}", entries.len())?;

        let mut by_source = std::collections::HashMap::new();
        for entry in entries {
            *by_source.entry(format!("{:?}", entry.source)).or_insert(0) += 1;
        }

        writeln!(writer, "\nBy source:")?;
        for (source, count) in by_source {
            writeln!(writer, "  {}: {}", source, count)?;
        }

        Ok(())
    }

    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()> {
        writeln!(
            writer,
            "[{}] {:?}",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.source
        )?;
        Ok(())
    }

    fn config(&self) -> &FormatterConfig {
        &self.config
    }

    fn set_config(&mut self, config: FormatterConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cai_core::{Entry, Metadata, Source};
    use chrono::Utc;

    fn mock_entry() -> Entry {
        Entry {
            id: "test-1".to_string(),
            source: Source::Claude,
            timestamp: Utc::now(),
            prompt: "Write a function".to_string(),
            response: "Here is the function".to_string(),
            metadata: Metadata {
                file_path: Some("src/main.rs".to_string()),
                repo_url: None,
                commit_hash: None,
                language: Some("Rust".to_string()),
                extra: std::collections::HashMap::new(),
            },
        }
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::default();
        let entries = vec![mock_entry()];
        let mut buf = Vec::new();
        formatter.format(&entries, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Verify valid JSON output
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_jsonl_formatter() {
        let formatter = JsonlFormatter::default();
        let entries = vec![mock_entry()];
        let mut buf = Vec::new();
        formatter.format(&entries, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Verify valid JSONL output (one JSON per line)
        for line in output.lines() {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(parsed.is_object());
        }
        assert_eq!(output.lines().count(), 1);
    }

    #[test]
    fn test_csv_formatter() {
        let formatter = CsvFormatter::default();
        let entries = vec![mock_entry()];
        let mut buf = Vec::new();
        formatter.format(&entries, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with("id,source,timestamp"));
        assert!(output.contains("test-1"));
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(CsvFormatter::escape_field("simple"), "simple");
        assert_eq!(CsvFormatter::escape_field("with, comma"), "\"with, comma\"");
        assert_eq!(
            CsvFormatter::escape_field("with\"quote"),
            "\"with\"\"quote\""
        );
    }

    #[test]
    fn test_ai_formatter() {
        let formatter = AiFormatter::default();
        let entry = mock_entry();
        let mut buf = Vec::new();
        formatter.format_one(&entry, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Write a function"));
        assert!(output.contains("->"));
    }

    #[test]
    fn test_stats_formatter() {
        let formatter = StatsFormatter::default();
        let entries = vec![mock_entry()];
        let mut buf = Vec::new();
        formatter.format(&entries, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Summary Statistics"));
        assert!(output.contains("By source"));
        assert!(output.contains("Claude"));
    }

    #[test]
    fn test_truncate() {
        let config = FormatterConfig::default();
        assert_eq!(config.truncate_text("short", 100), "short");
        assert_eq!(config.truncate_text("hello world", 8), "hello...");
        assert_eq!(config.truncate_text("test", 0), "test");
    }
}
