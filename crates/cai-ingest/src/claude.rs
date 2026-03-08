//! Claude Code conversation parser

use crate::error::IngestError;
use cai_core::{Entry, Metadata, Source};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tracing::debug;

/// Claude Code conversation file format
#[derive(Debug, Serialize, Deserialize)]
struct ClaudeConversation {
    /// Conversation ID (filename)
    #[serde(skip)]
    id: String,
    /// Messages in the conversation
    messages: Vec<ClaudeMessage>,
    /// Optional metadata
    #[serde(default)]
    metadata: ClaudeMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    /// Role (user/assistant)
    role: String,
    /// Message content
    content: String,
    /// Timestamp (ISO 8601)
    #[serde(default)]
    timestamp: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ClaudeMetadata {
    /// Project path
    #[serde(rename = "projectPath")]
    project_path: Option<String>,
    /// Repository URL
    repo_url: Option<String>,
}

/// Parser for Claude Code conversations
pub struct ClaudeParser {
    /// Path to conversations directory
    conversations_dir: PathBuf,
}

impl ClaudeParser {
    /// Create a new Claude parser
    ///
    /// # Arguments
    /// * `conversations_dir` - Path to ~/.claude/conversations directory
    pub fn new<P: AsRef<Path>>(conversations_dir: P) -> Self {
        Self {
            conversations_dir: conversations_dir.as_ref().to_path_buf(),
        }
    }

    /// Create parser with default path
    pub fn with_default_path() -> Result<Self, IngestError> {
        let home = dirs::home_dir()
            .ok_or_else(|| IngestError::PathNotFound("Home directory not found".to_string()))?;
        Ok(Self::new(home.join(".claude/conversations")))
    }

    /// Parse all conversations from the directory
    pub fn parse_all(&self) -> Result<Vec<Entry>, IngestError> {
        let entries = fs::read_dir(&self.conversations_dir)
            .map_err(|e| IngestError::PathNotFound(format!("{}: {}", self.conversations_dir.display(), e)))?;

        let mut results = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| IngestError::PermissionDenied(format!("read dir: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            debug!("Parsing Claude conversation: {}", path.display());
            match self.parse_file(&path) {
                Ok(conversation_entries) => {
                    results.extend(conversation_entries);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse {}: {}", path.display(), e);
                }
            }
        }

        if results.is_empty() {
            return Err(IngestError::NoFilesFound(self.conversations_dir.display().to_string()));
        }

        Ok(results)
    }

    /// Parse a single conversation file
    fn parse_file(&self, path: &Path) -> Result<Vec<Entry>, IngestError> {
        let content = fs::read_to_string(path)
            .map_err(|e| IngestError::InvalidFormat(format!("read failed: {}", e)))?;

        let conversation_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Try parsing as direct Entry array first (newer format)
        if let Ok(entries) = serde_json::from_str::<Vec<ClaudeMessage>>(&content) {
            return self.messages_to_entries(&conversation_id, entries, &None);
        }

        // Try parsing as Conversation object (older format)
        let mut conv: ClaudeConversation = serde_json::from_str(&content)
            .map_err(|e| IngestError::InvalidFormat(format!("JSON parse: {}", e)))?;
        conv.id = conversation_id;

        self.messages_to_entries(&conv.id, conv.messages, &Some(conv.metadata))
    }

    fn messages_to_entries(
        &self,
        conversation_id: &str,
        messages: Vec<ClaudeMessage>,
        metadata: &Option<ClaudeMetadata>,
    ) -> Result<Vec<Entry>, IngestError> {
        let mut entries = Vec::new();
        let mut i = 0;

        // Pair user messages with assistant responses
        while i < messages.len() {
            let msg = &messages[i];

            if msg.role == "user" {
                let prompt = msg.content.clone();
                let timestamp = parse_timestamp(&msg.timestamp);

                // Look for assistant response
                let response = if i + 1 < messages.len() && messages[i + 1].role == "assistant" {
                    messages[i + 1].content.clone()
                } else {
                    String::new()
                };

                let meta = metadata.as_ref().map(|m| Metadata {
                    file_path: Some(m.project_path.clone().unwrap_or_default()),
                    repo_url: m.repo_url.clone(),
                    commit_hash: None,
                    language: None,
                    extra: HashMap::from([
                        ("conversation_id".to_string(), conversation_id.to_string()),
                        ("message_index".to_string(), i.to_string()),
                    ]),
                }).unwrap_or_else(|| Metadata {
                    file_path: None,
                    repo_url: None,
                    commit_hash: None,
                    language: None,
                    extra: HashMap::from([
                        ("conversation_id".to_string(), conversation_id.to_string()),
                    ]),
                });

                entries.push(Entry {
                    id: format!("claude-{}-{}", conversation_id, i),
                    source: Source::Claude,
                    timestamp,
                    prompt,
                    response,
                    metadata: meta,
                });
            }

            i += 1;
        }

        Ok(entries)
    }
}

fn parse_timestamp(ts: &Option<String>) -> DateTime<Utc> {
    ts.as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt: DateTime<chrono::FixedOffset>| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_claude_conversation() {
        let temp_dir = TempDir::new().unwrap();
        let conv_path = temp_dir.path().join("test-conversation.json");

        let json = r#"{
            "messages": [
                {
                    "role": "user",
                    "content": "help me write a function",
                    "timestamp": "2024-01-15T10:30:00Z"
                },
                {
                    "role": "assistant",
                    "content": "Here's how to write a function...",
                    "timestamp": "2024-01-15T10:30:01Z"
                }
            ],
            "metadata": {
                "projectPath": "/Users/user/project"
            }
        }"#;

        fs::write(&conv_path, json).unwrap();

        let parser = ClaudeParser::new(temp_dir.path());
        let entries = parser.parse_all().unwrap();

        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert!(entry.id.starts_with("claude-test-conversation-"));
        assert_eq!(entry.source, Source::Claude);
        assert_eq!(entry.prompt, "help me write a function");
        assert_eq!(entry.response, "Here's how to write a function...");
        assert_eq!(entry.metadata.file_path, Some("/Users/user/project".to_string()));
    }

    #[test]
    fn test_parse_conversation_array_format() {
        let temp_dir = TempDir::new().unwrap();
        let conv_path = temp_dir.path().join("array-format.json");

        let json = r#"[
            {
                "role": "user",
                "content": "test question"
            },
            {
                "role": "assistant",
                "content": "test answer"
            }
        ]"#;

        fs::write(&conv_path, json).unwrap();

        let parser = ClaudeParser::new(temp_dir.path());
        let entries = parser.parse_all().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].prompt, "test question");
        assert_eq!(entries[0].response, "test answer");
    }
}
