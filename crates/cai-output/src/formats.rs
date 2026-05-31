//! Output format implementations
//!
//! Provides formatters for JSON, JSONL, CSV, terminal tables,
//! AI-optimized, and statistics output formats.

use crate::formatter::Truncate;
use crate::{Formatter, FormatterConfig};
use cai_core::Entry;
use cai_core::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::io::Write;
use tabled::builder::Builder;
use tabled::settings::style::Style;
use tabled::settings::{Alignment, Modify};
use tabled::Tabled;

/// A display-friendly row for table output.
#[derive(Tabled)]
struct Row {
    timestamp: String,
    source: String,
    prompt: String,
}

impl From<&cai_core::Entry> for Row {
    fn from(e: &cai_core::Entry) -> Self {
        Self {
            timestamp: e.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            source: format!("{:?}", e.source),
            prompt: e.prompt.chars().take(60).collect(),
        }
    }
}

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
        serde_json::to_writer_pretty(&mut *writer, entries)?;
        writeln!(writer)?;
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

/// Source color helper - returns a colored string for each source type
fn colorize_source(source: &str, colorize: bool) -> String {
    if !colorize {
        return source.to_string();
    }
    match source {
        "Claude" => source.blue().to_string(),
        "Codex" => source.yellow().to_string(),
        "Git" => source.green().to_string(),
        _ => source.to_string(),
    }
}

/// Table formatter for terminal output using tabled
#[derive(Debug, Clone, Default)]
pub struct TableFormatter {
    config: FormatterConfig,
}

impl TableFormatter {
    /// Create a new formatter instance
    pub fn new() -> Self {
        Self::default()
    }

    fn write_table<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()> {
        let rows: Vec<Row> = entries.iter().map(Row::from).collect();
        let mut builder = Builder::new();

        if self.config.show_header {
            builder.push_record(["Timestamp", "Source", "Prompt"]);
        }

        for row in &rows {
            builder.push_record([&row.timestamp, &row.source, &row.prompt]);
        }

        let mut table = builder.build();
        if !self.config.compact {
            table.with(Style::rounded());
        } else {
            table.with(Style::empty());
        }

        write!(writer, "{table}")?;
        writeln!(writer)?;
        Ok(())
    }
}

impl Formatter for TableFormatter {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()> {
        self.write_table(entries, writer)
    }

    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()> {
        self.write_table(std::slice::from_ref(entry), writer)
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
        let colorize = self.config.colorize;

        for (i, entry) in entries.iter().enumerate() {
            let marker = if colorize {
                format!("{}", format!("─▸ {}", i + 1).cyan())
            } else {
                format!("─▸ {}", i + 1)
            };

            writeln!(writer, "{}", marker)?;
            let source_label = if colorize { "source:".dimmed().to_string() } else { "source:".to_string() };
            let time_label = if colorize { "time:".dimmed().to_string() } else { "time:".to_string() };
            let ask_label = if colorize { "ask:".dimmed().to_string() } else { "ask:".to_string() };
            let ans_label = if colorize { "ans:".dimmed().to_string() } else { "ans:".to_string() };

            writeln!(
                writer,
                "  {} {}",
                source_label,
                format!("{:?}", entry.source)
            )?;
            writeln!(
                writer,
                "  {} {}",
                time_label,
                entry.timestamp.format("%Y-%m-%d %H:%M")
            )?;
            writeln!(
                writer,
                "  {} {}",
                ask_label,
                self.config.truncate_text(&entry.prompt, 80)
            )?;
            writeln!(
                writer,
                "  {} {}",
                ans_label,
                self.config.truncate_text(&entry.response, 120)
            )?;

            if i < entries.len() - 1 {
                writeln!(writer)?;
            }
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

/// Statistics summary formatter with tabled layout
#[derive(Debug, Clone, Default)]
pub struct StatsFormatter {
    config: FormatterConfig,
}

impl StatsFormatter {
    /// Create a new formatter instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Count entries by source
    fn count_by_source(&self, entries: &[Entry]) -> HashMap<String, usize> {
        let mut by_source = HashMap::new();
        for entry in entries {
            *by_source.entry(format!("{:?}", entry.source)).or_insert(0) += 1;
        }
        by_source
    }

    /// Find the earliest and latest timestamps
    fn date_range(&self, entries: &[Entry]) -> (String, String) {
        let mut min_ts = &entries[0].timestamp;
        let mut max_ts = &entries[0].timestamp;
        for entry in entries {
            if entry.timestamp < *min_ts {
                min_ts = &entry.timestamp;
            }
            if entry.timestamp > *max_ts {
                max_ts = &entry.timestamp;
            }
        }
        (
            min_ts.format("%Y-%m-%d").to_string(),
            max_ts.format("%Y-%m-%d").to_string(),
        )
    }

    /// Build the overview table section
    fn build_overview_table(&self, entries: &[Entry]) -> String {
        let colorize = self.config.colorize;
        let total = entries.len();
        let (min_date, max_date) = self.date_range(entries);
        let by_source = self.count_by_source(entries);
        let source_count = by_source.len();

        let overview_label = if colorize {
            "CAI Statistics".cyan().bold().to_string()
        } else {
            "CAI Statistics".to_string()
        };

        let mut builder = Builder::new();
        builder.push_record(["Metric", "Value"]);

        let total_str = if colorize {
            format!("{}", total.to_string().bold())
        } else {
            total.to_string()
        };
        builder.push_record(["Total Entries", &total_str]);

        let sources_str = format!(
            "{} ({})",
            source_count,
            by_source
                .keys()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        builder.push_record(["Sources", &sources_str]);
        builder.push_record(["Date Range", &format!("{} to {}", min_date, max_date)]);

        let mut table = builder.build();
        table.with(Style::rounded());
        table.with(Modify::new(tabled::settings::object::Columns::first()).with(Alignment::left()));

        format!("{}\n{}", overview_label, table)
    }

    /// Build the source breakdown table section
    fn build_sources_table(&self, entries: &[Entry]) -> String {
        let colorize = self.config.colorize;
        let total = entries.len() as f64;
        let by_source = self.count_by_source(entries);

        let header = if colorize {
            "Top Sources".cyan().bold().to_string()
        } else {
            "Top Sources".to_string()
        };

        let mut builder = Builder::new();
        builder.push_record(["Source", "Entries", "%"]);

        let mut sorted: Vec<_> = by_source.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        for (source, count) in &sorted {
            let pct = *count as f64 / total * 100.0;
            let pct_str = format!("{:.1}%", pct);

            let source_display = colorize_source(source, colorize);

            let count_str = if colorize {
                format!("{}", count.to_string().bold())
            } else {
                count.to_string()
            };

            builder.push_record([&source_display, &count_str, &pct_str]);
        }

        let mut table = builder.build();
        table.with(Style::rounded());
        table.with(
            Modify::new(tabled::settings::object::Columns::last())
                .with(Alignment::right()),
        );

        format!("\n{}\n{}", header, table)
    }
}

impl Formatter for StatsFormatter {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()> {
        if entries.is_empty() {
            writeln!(
                writer,
                "{}",
                if self.config.colorize {
                    "No entries found.".dimmed().to_string()
                } else {
                    "No entries found.".to_string()
                }
            )?;
            return Ok(());
        }

        writeln!(writer, "{}", self.build_overview_table(entries))?;
        writeln!(writer, "{}", self.build_sources_table(entries))?;
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
    use chrono::{TimeZone, Utc};

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

    fn multiple_entries() -> Vec<Entry> {
        vec![
            Entry {
                id: "1".to_string(),
                source: Source::Claude,
                timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
                prompt: "Refactor this code".to_string(),
                response: "Here's the refactored version".to_string(),
                metadata: Metadata::default(),
            },
            Entry {
                id: "2".to_string(),
                source: Source::Git,
                timestamp: Utc.with_ymd_and_hms(2024, 1, 2, 14, 30, 0).unwrap(),
                prompt: "feat: add login".to_string(),
                response: "Added OAuth2 login flow".to_string(),
                metadata: Metadata::default(),
            },
            Entry {
                id: "3".to_string(),
                source: Source::Codex,
                timestamp: Utc.with_ymd_and_hms(2024, 1, 3, 9, 15, 0).unwrap(),
                prompt: "Parse JSON data".to_string(),
                response: "Here's a JSON parser using serde".to_string(),
                metadata: Metadata::default(),
            },
        ]
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
    fn test_table_formatter() {
        let formatter = TableFormatter::default();
        let entries = multiple_entries();
        let mut buf = Vec::new();
        formatter.format(&entries, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should contain table borders (rounded style uses unicode)
        assert!(output.contains("╭"));
        assert!(output.contains("╮"));
        assert!(output.contains("╰"));
        assert!(output.contains("╯"));
        // Should contain column headers
        assert!(output.contains("Source"));
        assert!(output.contains("Timestamp"));
        assert!(output.contains("Prompt"));
        // Should contain entry data
        assert!(output.contains("Refactor"));
        assert!(output.contains("Claude"));
    }

    #[test]
    fn test_table_formatter_empty() {
        let formatter = TableFormatter::default();
        let entries: Vec<Entry> = vec![];
        let mut buf = Vec::new();
        formatter.format(&entries, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should render a header-only table (show_header defaults to true)
        assert!(output.contains("Timestamp"));
        assert!(output.contains("Source"));
        assert!(output.contains("Prompt"));
    }

    #[test]
    fn test_table_formatter_no_header() {
        let mut config = FormatterConfig::default();
        config.show_header = false;
        config.compact = true;
        let mut formatter = TableFormatter::default();
        formatter.set_config(config);

        let entries = multiple_entries();
        let mut buf = Vec::new();
        formatter.format(&entries, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // No header, compact mode
        assert!(!output.contains("Timestamp"));
        assert!(output.contains("Refactor"));
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
        let entries = multiple_entries();
        let mut buf = Vec::new();
        formatter.format(&entries, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Should contain the overview section
        assert!(output.contains("CAI Statistics"));
        assert!(output.contains("Total Entries"));
        assert!(output.contains("3"));
        // Should contain source breakdown
        assert!(output.contains("Claude"));
        assert!(output.contains("Git"));
        assert!(output.contains("Codex"));
        // Should have date range
        assert!(output.contains("2024-01-01"));
    }

    #[test]
    fn test_stats_formatter_empty() {
        let formatter = StatsFormatter::default();
        let entries: Vec<Entry> = vec![];
        let mut buf = Vec::new();
        formatter.format(&entries, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No entries found"));
    }

    #[test]
    fn test_truncate() {
        let config = FormatterConfig::default();
        assert_eq!(config.truncate_text("short", 100), "short");
        assert_eq!(config.truncate_text("hello world", 8), "hello...");
        assert_eq!(config.truncate_text("test", 0), "test");
    }
}
