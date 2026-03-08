//! Codex CLI history parser

use crate::error::IngestError;
use cai_core::{Entry, Metadata, Source};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::debug;

/// Codex CLI history entry format (JSONL)
#[derive(Debug, Serialize, Deserialize)]
struct CodexEntry {
    /// Unique ID
    id: Option<String>,
    /// Timestamp (ISO 8601 or Unix timestamp)
    timestamp: Option<String>,
    /// User prompt
    prompt: String,
    /// Assistant response
    response: Option<String>,
    /// Optional file context
    file: Option<String>,
    /// Optional language
    language: Option<String>,
    /// Session ID
    session_id: Option<String>,
}

/// Parser for Codex CLI history
pub struct CodexParser {
    /// Path to history file
    history_path: PathBuf,
}

impl CodexParser {
    /// Create a new Codex parser
    ///
    /// # Arguments
    /// * `history_path` - Path to ~/.codex/history.jsonl file
    pub fn new<P: AsRef<Path>>(history_path: P) -> Self {
        Self {
            history_path: history_path.as_ref().to_path_buf(),
        }
    }

    /// Create parser with default path
    pub fn with_default_path() -> Result<Self, IngestError> {
        let home = dirs::home_dir()
            .ok_or_else(|| IngestError::PathNotFound("Home directory not found".to_string()))?;
        Ok(Self::new(home.join(".codex/history.jsonl")))
    }

    /// Parse all entries from the history file
    pub fn parse_all(&self) -> Result<Vec<Entry>, IngestError> {
        let file = File::open(&self.history_path)
            .map_err(|e| IngestError::PathNotFound(format!("{}: {}", self.history_path.display(), e)))?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut line_num = 0;

        for line in reader.lines() {
            line_num += 1;
            let line = line.map_err(|e| IngestError::InvalidFormat(format!("read line {}: {}", line_num, e)))?;

            if line.trim().is_empty() {
                continue;
            }

            debug!("Parsing Codex entry line {}", line_num);
            match self.parse_line(&line, line_num) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    tracing::warn!("Failed to parse line {}: {}", line_num, e);
                }
            }
        }

        if entries.is_empty() {
            return Err(IngestError::NoFilesFound(self.history_path.display().to_string()));
        }

        Ok(entries)
    }

    /// Parse a single JSONL line
    fn parse_line(&self, line: &str, line_num: usize) -> Result<Entry, IngestError> {
        let codex_entry: CodexEntry = serde_json::from_str(line)
            .map_err(|e| IngestError::InvalidFormat(format!("line {}: {}", line_num, e)))?;

        let id = codex_entry.id.unwrap_or_else(|| {
            format!("codex-line-{}", line_num)
        });

        let timestamp = parse_codex_timestamp(&codex_entry.timestamp);
        let response = codex_entry.response.unwrap_or_default();

        let mut extra = HashMap::new();
        if let Some(sid) = &codex_entry.session_id {
            extra.insert("session_id".to_string(), sid.clone());
        }

        Ok(Entry {
            id,
            source: Source::Codex,
            timestamp,
            prompt: codex_entry.prompt,
            response,
            metadata: Metadata {
                file_path: codex_entry.file,
                repo_url: None,
                commit_hash: None,
                language: codex_entry.language,
                extra,
            },
        })
    }
}

fn parse_codex_timestamp(ts: &Option<String>) -> DateTime<Utc> {
    ts.as_ref()
        .and_then(|s| {
            // Try ISO 8601 first
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt: DateTime<chrono::FixedOffset>| dt.with_timezone(&Utc))
                .or_else(|| {
                    // Try Unix timestamp (seconds)
                    s.parse::<i64>()
                        .ok()
                        .map(|secs| DateTime::from_timestamp(secs, 0).unwrap_or_else(Utc::now))
                })
        })
        .unwrap_or_else(Utc::now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_codex_history() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.jsonl");

        let jsonl = r#"{"id":"entry1","timestamp":"2024-01-15T10:30:00Z","prompt":"write a function","response":"def foo(): pass","file":"main.py","language":"python"}
{"prompt":"another question","response":"answer"}
"#;

        fs::write(&history_path, jsonl).unwrap();

        let parser = CodexParser::new(&history_path);
        let entries = parser.parse_all().unwrap();

        assert_eq!(entries.len(), 2);

        let entry1 = &entries[0];
        assert_eq!(entry1.id, "entry1");
        assert_eq!(entry1.source, Source::Codex);
        assert_eq!(entry1.prompt, "write a function");
        assert_eq!(entry1.response, "def foo(): pass");
        assert_eq!(entry1.metadata.file_path, Some("main.py".to_string()));
        assert_eq!(entry1.metadata.language, Some("python".to_string()));

        let entry2 = &entries[1];
        assert!(entry2.id.starts_with("codex-line-"));
        assert_eq!(entry2.prompt, "another question");
    }

    #[test]
    fn test_parse_unix_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let history_path = temp_dir.path().join("history.jsonl");

        // Unix timestamp for 2024-01-15 10:30:00 UTC
        let jsonl = r#"{"prompt":"test","timestamp":"1705319400"}"#;

        fs::write(&history_path, jsonl).unwrap();

        let parser = CodexParser::new(&history_path);
        let entries = parser.parse_all().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].prompt, "test");
    }
}
