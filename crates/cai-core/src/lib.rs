//! # CAI Core - Shared types, traits, and utilities
//!
//! This crate provides the foundational data structures and shared utilities
//! used throughout the CAI (Coding Agent Insights) workspace.
//!
//! ## Overview
//!
//! `cai-core` defines the core domain model for representing AI coding interactions.
//! All other crates in the workspace depend on these types to ensure consistency
//! across the system.
//!
//! ## Core Types
//!
//! - [`Entry`] - Represents a single AI coding interaction with all associated data
//! - [`Source`] - Identifies the origin system (Claude, Codex, Git, etc.)
//! - [`Metadata`] - Extensible metadata for additional context
//! - [`Error`] - Unified error type for CAI operations
//! - [`Result<T>`] - Type alias for `Result<T, Error>`
//!
//! ## Usage
//!
//! Creating a basic entry:
//!
//! ```rust
//! use cai_core::{Entry, Source, Metadata};
//! use chrono::Utc;
//!
//! let entry = Entry {
//!     id: "test-entry-123".to_string(),
//!     source: Source::Claude,
//!     timestamp: Utc::now(),
//!     prompt: "Help me refactor this function".to_string(),
//!     response: "Here's the improved version...".to_string(),
//!     metadata: Metadata::default(),
//! };
//! ```
//!
//! Working with metadata:
//!
//! ```rust
//! use cai_core::Metadata;
//! use std::collections::HashMap;
//!
//! let mut metadata = Metadata {
//!     file_path: Some("src/main.rs".to_string()),
//!     repo_url: Some("https://github.com/user/repo".to_string()),
//!     commit_hash: Some("abc123def".to_string()),
//!     language: Some("Rust".to_string()),
//!     extra: HashMap::new(),
//! };
//!
//! // Add custom fields
//! metadata.extra.insert("complexity".to_string(), "high".to_string());
//! metadata.extra.insert("reviewed".to_string(), "true".to_string());
//! ```
//!
//! Error handling:
//!
//! ```rust
//! use cai_core::{Error, Result, Entry};
//!
//! fn process_entry(entry: &Entry) -> Result<()> {
//!     if entry.prompt.is_empty() {
//!         return Err(Error::Message("Prompt cannot be empty".to_string()));
//!     }
//!     // Process the entry...
//!     Ok(())
//! }
//! ```
//!
//! ## Design Principles
//!
//! - **Minimal dependencies**: Only essential dependencies (serde, chrono)
//! - **Serialization**: All public types implement `Serialize` and `Deserialize`
//! - **Extensibility**: `Metadata.extra` allows custom fields without breaking changes
//! - **Type safety**: Enums use `#[non_exhaustive]` for future-proofing
//!
//! ## Testing
//!
//! ```bash
//! # Run all tests
//! cargo test -p cai-core
//!
//! # Run with coverage
//! cargo llvm-cov -p cai-core
//! ```

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Metadata {
    /// Optional file path if applicable
    pub file_path: Option<String>,
    /// Optional repository URL
    pub repo_url: Option<String>,
    /// Optional commit hash
    pub commit_hash: Option<String>,
    /// Optional language/technology
    pub language: Option<String>,
    /// Additional custom fields (as key-value pairs)
    #[serde(default)]
    pub extra: std::collections::HashMap<String, String>,
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
