//! CAI Storage - Pluggable storage backends

#![warn(missing_docs, unused_crate_dependencies)]

pub use cai_core::{Error, Result};

use async_trait::async_trait;
use cai_core::Entry;

#[cfg(feature = "sqlite")]
pub mod sqlite;

#[cfg(feature = "sqlite")]
pub use sqlite::SqliteStorage;

/// Storage backend trait
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store an entry
    async fn store(&self, entry: &Entry) -> Result<()>;

    /// Retrieve an entry by ID
    async fn get(&self, id: &str) -> Result<Option<Entry>>;

    /// Query entries with optional filter
    async fn query(&self, filter: Option<&Filter>) -> Result<Vec<Entry>>;

    /// Count entries
    async fn count(&self) -> Result<usize>;
}

#[cfg(feature = "duckdb")]
pub mod duckdb;

#[cfg(feature = "duckdb")]
pub use duckdb::DuckDBStorage;

/// Query filter
#[derive(Debug, Clone, Default)]
pub struct Filter {
    /// Source to filter by
    pub source: Option<String>,
    /// Minimum timestamp
    pub after: Option<chrono::DateTime<chrono::Utc>>,
    /// Maximum timestamp
    pub before: Option<chrono::DateTime<chrono::Utc>>,
}

/// In-memory storage implementation
pub struct MemoryStorage {
    entries: std::sync::Arc<tokio::sync::RwLock<Vec<Entry>>>,
}

impl MemoryStorage {
    /// Create new in-memory storage
    pub fn new() -> Self {
        Self {
            entries: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage {
    /// Create new in-memory storage with mock data for testing
    pub fn with_mock_data() -> Self {
        use cai_core::{Entry, Metadata, Source};
        use chrono::Utc;

        let storage = Self::new();

        let mock_entries = vec![
            Entry {
                id: "1".to_string(),
                source: Source::Claude,
                timestamp: Utc::now() - chrono::Duration::hours(2),
                prompt: "Help me refactor this Rust function to be more idiomatic".to_string(),
                response: "Here's a more idiomatic version using iterators and pattern matching...".to_string(),
                metadata: Metadata {
                    file_path: Some("src/main.rs".to_string()),
                    language: Some("Rust".to_string()),
                    ..Default::default()
                },
            },
            Entry {
                id: "2".to_string(),
                source: Source::Claude,
                timestamp: Utc::now() - chrono::Duration::hours(4),
                prompt: "Write a unit test for this module".to_string(),
                response: "Here are comprehensive unit tests using rstest...".to_string(),
                metadata: Metadata {
                    file_path: Some("src/storage.rs".to_string()),
                    language: Some("Rust".to_string()),
                    ..Default::default()
                },
            },
            Entry {
                id: "3".to_string(),
                source: Source::Claude,
                timestamp: Utc::now() - chrono::Duration::hours(6),
                prompt: "Explain how async/await works in Rust".to_string(),
                response: "Async/await in Rust is built on futures...".to_string(),
                metadata: Metadata {
                    language: Some("Rust".to_string()),
                    ..Default::default()
                },
            },
            Entry {
                id: "4".to_string(),
                source: Source::Codex,
                timestamp: Utc::now() - chrono::Duration::hours(8),
                prompt: "Implement a binary search function".to_string(),
                response: "fn binary_search(arr: &[i32], target: i32) -> Option<usize> { ... }".to_string(),
                metadata: Metadata {
                    language: Some("Rust".to_string()),
                    ..Default::default()
                },
            },
            Entry {
                id: "5".to_string(),
                source: Source::Git,
                timestamp: Utc::now() - chrono::Duration::hours(10),
                prompt: "feat: Add TUI implementation".to_string(),
                response: "Implemented terminal UI with ratatui...".to_string(),
                metadata: Metadata {
                    commit_hash: Some("abc123def".to_string()),
                    repo_url: Some("https://github.com/cai-dev/coding-agent-insights".to_string()),
                    ..Default::default()
                },
            },
            Entry {
                id: "6".to_string(),
                source: Source::Claude,
                timestamp: Utc::now() - chrono::Duration::hours(12),
                prompt: "What's the difference between Arc and Rc in Rust?".to_string(),
                response: "Arc (Atomic Reference Counting) is thread-safe...".to_string(),
                metadata: Metadata {
                    language: Some("Rust".to_string()),
                    ..Default::default()
                },
            },
            Entry {
                id: "7".to_string(),
                source: Source::Claude,
                timestamp: Utc::now() - chrono::Duration::days(1),
                prompt: "Design a REST API for user management".to_string(),
                response: "Here's a RESTful API design using axum...".to_string(),
                metadata: Metadata {
                    language: Some("Rust".to_string()),
                    ..Default::default()
                },
            },
            Entry {
                id: "8".to_string(),
                source: Source::Claude,
                timestamp: Utc::now() - chrono::Duration::days(2),
                prompt: "Debug this segmentation fault".to_string(),
                response: "The segfault is caused by a dangling reference...".to_string(),
                metadata: Metadata {
                    file_path: Some("src/parser.rs".to_string()),
                    language: Some("Rust".to_string()),
                    ..Default::default()
                },
            },
        ];

        // Use tokio runtime to store entries
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            for entry in mock_entries {
                let _ = storage.store(&entry).await;
            }
        });

        storage
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn store(&self, entry: &Entry) -> Result<()> {
        self.entries.write().await.push(entry.clone());
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Entry>> {
        Ok(self
            .entries
            .read()
            .await
            .iter()
            .find(|e| e.id == id)
            .cloned())
    }

    async fn query(&self, filter: Option<&Filter>) -> Result<Vec<Entry>> {
        let entries = self.entries.read().await;
        Ok(if let Some(f) = filter {
            entries
                .iter()
                .filter(|e| {
                    if let Some(ref src) = f.source {
                        if format!("{:?}", e.source) != *src {
                            return false;
                        }
                    }
                    if let Some(after) = f.after {
                        if e.timestamp < after {
                            return false;
                        }
                    }
                    if let Some(before) = f.before {
                        if e.timestamp > before {
                            return false;
                        }
                    }
                    true
                })
                .cloned()
                .collect()
        } else {
            entries.clone()
        })
    }

    async fn count(&self) -> Result<usize> {
        Ok(self.entries.read().await.len())
    }
}
