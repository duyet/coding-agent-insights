//! CAI Storage - Pluggable storage backends

#![warn(missing_docs, unused_crate_dependencies)]

pub use cai_core::Result;

use async_trait::async_trait;
use cai_core::Entry;

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
