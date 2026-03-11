//! File loader - load and parse conversation files

use crate::Result;
use cai_core::{Entry, Metadata, Source};
use serde_json::Value;
use std::io::Read;
use std::path::Path;
use tracing::debug;

/// Load and parse conversation files
pub struct FileLoader;

impl FileLoader {
    /// Load single file, auto-detect format
    pub fn load(&self, path: &Path) -> Result<Vec<Entry>> {
        let format = self.detect_format(path)?;

        match format {
            FormatVersion::V1 => self.load_v1(path),
            FormatVersion::V2 => self.load_v2(path),
            FormatVersion::Unknown => Err(crate::Error::Message(format!(
                "Unknown format for file: {:?}",
                path
            ))),
        }
    }

    /// Load multiple files in parallel
    pub async fn load_many(&self, paths: &[std::path::PathBuf]) -> Result<Vec<Entry>> {
        use futures::stream::{self, StreamExt};

        let loader = self;
        let entries = stream::iter(paths)
            .map(|path| async move { loader.load(path) })
            .buffer_unordered(10) // Process up to 10 files concurrently
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flat_map(|result| result.unwrap_or_else(|e| {
                debug!("Failed to load file: {}", e);
                Vec::new()
            }))
            .collect();

        Ok(entries)
    }

    /// Detect format version from file content
    pub fn detect_format(&self, path: &Path) -> Result<FormatVersion> {
        // Read first 1KB to detect format
        let file = std::fs::File::open(path)
            .map_err(|e| crate::Error::Message(format!("Failed to open file: {}", e)))?;
        let mut reader = std::io::BufReader::new(file);
        let mut buffer = [0u8; 1024];
        let n = reader
            .read(&mut buffer)
            .map_err(|e| crate::Error::Message(format!("Failed to read file: {}", e)))?;

        let content = std::str::from_utf8(&buffer[..n])
            .map_err(|e| crate::Error::Message(format!("Invalid UTF-8: {}", e)))?;

        Ok(detect_format(content))
    }

    /// Load V1 format (initial Claude Code format)
    fn load_v1(&self, path: &Path) -> Result<Vec<Entry>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::Error::Message(format!("Failed to read file: {}", e)))?;

        let json: Value = serde_json::from_str(&content)
            .map_err(|e| crate::Error::Message(format!("Failed to parse JSON: {}", e)))?;

        // V1 format: { "messages": [...], ... }
        let messages = json["messages"]
            .as_array()
            .ok_or_else(|| crate::Error::Message("Missing messages array".to_string()))?;

        let mut entries = Vec::new();
        for (i, msg) in messages.iter().enumerate() {
            if let Some(role) = msg["role"].as_str() {
                if role == "user" {
                    entries.push(Entry {
                        id: format!("{}-{}", path.display(), i),
                        source: Source::Claude,
                        timestamp: chrono::Utc::now(), // V1 doesn't have timestamps
                        prompt: msg["content"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        response: String::new(), // Would need to pair with assistant message
                        metadata: Metadata::default(),
                    });
                }
            }
        }

        Ok(entries)
    }

    /// Load V2 format (with tool_use)
    fn load_v2(&self, path: &Path) -> Result<Vec<Entry>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::Error::Message(format!("Failed to read file: {}", e)))?;

        let json: Value = serde_json::from_str(&content)
            .map_err(|e| crate::Error::Message(format!("Failed to parse JSON: {}", e)))?;

        // V2 format has more structure
        let messages = json["messages"]
            .as_array()
            .ok_or_else(|| crate::Error::Message("Missing messages array".to_string()))?;

        let mut entries = Vec::new();
        let mut user_prompt = String::new();

        for msg in messages {
            let role = msg["role"].as_str().unwrap_or("unknown");

            match role {
                "user" => {
                    user_prompt = msg["content"]
                        .as_str()
                        .or_else(|| msg["text"].as_str())
                        .unwrap_or("")
                        .to_string();
                }
                "assistant" => {
                    let response = msg["content"]
                        .as_str()
                        .or_else(|| msg["text"].as_str())
                        .unwrap_or("")
                        .to_string();

                    if !user_prompt.is_empty() {
                        entries.push(Entry {
                            id: format!("{}-{}", path.display(), entries.len()),
                            source: Source::Claude,
                            timestamp: chrono::Utc::now(),
                            prompt: user_prompt.clone(),
                            response,
                            metadata: Metadata::default(),
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(entries)
    }
}

/// Format version for conversation files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatVersion {
    /// V1: Initial Claude Code format
    V1,
    /// V2: Current format with tool_use support
    V2,
    /// Unknown format
    Unknown,
}

/// Detect format from JSON content
pub fn detect_format(content: &str) -> FormatVersion {
    // Check for V2 markers
    if content.contains("\"tool_use\"") || content.contains("\"tool_calls\"") {
        return FormatVersion::V2;
    }

    // Check for V1 markers
    if content.contains("\"messages\"") {
        return FormatVersion::V1;
    }

    FormatVersion::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_v2() {
        let content = r#"{"messages": [{"role": "user", "tool_use": true}]}"#;
        assert_eq!(detect_format(content), FormatVersion::V2);
    }

    #[test]
    fn test_detect_format_v1() {
        let content = r#"{"messages": [{"role": "user", "content": "hello"}]}"#;
        assert_eq!(detect_format(content), FormatVersion::V1);
    }

    #[test]
    fn test_detect_format_unknown() {
        let content = r#"{"foo": "bar"}"#;
        assert_eq!(detect_format(content), FormatVersion::Unknown);
    }

    #[test]
    fn test_detect_format_empty() {
        let content = "";
        assert_eq!(detect_format(content), FormatVersion::Unknown);
    }
}
