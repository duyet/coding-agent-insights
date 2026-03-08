//! CAI Core - Shared types, traits, and utilities

#![warn(missing_docs, unused_crate_dependencies)]

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Core entry representing a single AI coding interaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    /// Unique identifier
    pub id: String,
    /// Source system (claude, codex, git, etc.)
    pub source: Source,
    /// Timestamp of the interaction
    pub timestamp: DateTime<Utc>,
    /// The prompt/input provided
    pub prompt: String,
    /// The response/output received
    pub response: String,
    /// Associated metadata
    pub metadata: Metadata,
}

/// Source system identifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Source {
    /// Claude Code conversations
    Claude,
    /// Codex CLI history
    Codex,
    /// Git repository activity
    Git,
    /// Other source
    Other(String),
}

/// Metadata associated with an entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    /// Optional file path if applicable
    pub file_path: Option<String>,
    /// Optional repository URL
    pub repo_url: Option<String>,
    /// Optional commit hash
    pub commit_hash: Option<String>,
    /// Optional language/technology
    pub language: Option<String>,
    /// Additional custom fields
    #[serde(flatten)]
    pub extra: Vec<(String, String)>,
}

/// Result type for CAI operations
pub type Result<T> = std::result::Result<T, Error>;

/// Core error types
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic error message
    #[error("{0}")]
    Message(String),
}
