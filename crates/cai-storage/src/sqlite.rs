//! SQLite storage backend
//!
//! Provides persistent storage using SQLite database.

use crate::{Filter, Result, Storage};
use cai_core::Entry;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// SQLite-based storage for CAI entries
pub struct SqliteStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStorage {
    /// Create a new SQLite storage with the given database path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SQLite database file (will be created if it doesn't exist)
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| crate::Error::Message(format!("Failed to open database: {}", e)))?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema()?;
        Ok(storage)
    }

    /// Create in-memory SQLite storage (useful for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| {
            crate::Error::Message(format!("Failed to open in-memory database: {}", e))
        })?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema()?;
        Ok(storage)
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| crate::Error::Message(format!("Failed to lock connection: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS entries (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                prompt TEXT NOT NULL,
                response TEXT NOT NULL,
                file_path TEXT,
                repo_url TEXT,
                commit_hash TEXT,
                language TEXT,
                metadata_json TEXT
            )",
            [],
        )
        .map_err(|e| crate::Error::Message(format!("Failed to create table: {}", e)))?;

        // Create indexes for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_source ON entries(source)",
            [],
        )
        .map_err(|e| crate::Error::Message(format!("Failed to create index: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON entries(timestamp)",
            [],
        )
        .map_err(|e| crate::Error::Message(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    /// Convert database row to Entry
    fn row_to_entry(row: &rusqlite::Row) -> Result<Entry> {
        use cai_core::Source;

        let source_str: String = row.get("source")?;
        let source = match source_str.as_str() {
            "Claude" => Source::Claude,
            "Codex" => Source::Codex,
            "Git" => Source::Git,
            _ => Source::Other(source_str),
        };

        let timestamp_str: String = row.get("timestamp")?;
        let timestamp = timestamp_str
            .parse::<DateTime<Utc>>()
            .map_err(|e| crate::Error::Message(format!("Invalid timestamp: {}", e)))?;

        let mut metadata = cai_core::Metadata::default();
        if let Ok(Some(fp)) = row.get::<_, Option<String>>("file_path") {
            metadata.file_path = fp;
        }
        if let Ok(Some(url)) = row.get::<_, Option<String>>("repo_url") {
            metadata.repo_url = url;
        }
        if let Ok(Some(hash)) = row.get::<_, Option<String>>("commit_hash") {
            metadata.commit_hash = hash;
        }
        if let Ok(Some(lang)) = row.get::<_, Option<String>>("language") {
            metadata.language = lang;
        }

        // TODO: Parse metadata_json for extra fields

        Ok(Entry {
            id: row.get("id")?,
            source,
            timestamp,
            prompt: row.get("prompt")?,
            response: row.get("response")?,
            metadata,
        })
    }
}

#[async_trait::async_trait]
impl Storage for SqliteStorage {
    async fn store(&self, entry: &Entry) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| crate::Error::Message(format!("Failed to lock connection: {}", e)))?;

        let source_str = format!("{:?}", entry.source);
        let timestamp_str = entry
            .timestamp
            .format("%Y-%m-%dT%H:%M:%S%.6f%:z")
            .to_string();

        conn.execute(
            "INSERT OR REPLACE INTO entries (
                id, source, timestamp, prompt, response,
                file_path, repo_url, commit_hash, language
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entry.id,
                source_str,
                timestamp_str,
                entry.prompt,
                entry.response,
                entry.metadata.file_path,
                entry.metadata.repo_url,
                entry.metadata.commit_hash,
                entry.metadata.language,
            ],
        )
        .map_err(|e| crate::Error::Message(format!("Failed to insert entry: {}", e)))?;

        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Entry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| crate::Error::Message(format!("Failed to lock connection: {}", e)))?;

        let mut stmt = conn
            .prepare("SELECT * FROM entries WHERE id = ?1")
            .map_err(|e| crate::Error::Message(format!("Failed to prepare query: {}", e)))?;

        let entry = stmt
            .query_row(params![id], |row| Self::row_to_entry(row))
            .optional()
            .map_err(|e| crate::Error::Message(format!("Failed to query entry: {}", e)))?;

        Ok(entry)
    }

    async fn query(&self, filter: Option<&Filter>) -> Result<Vec<Entry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| crate::Error::Message(format!("Failed to lock connection: {}", e)))?;

        let (sql, params) = if let Some(f) = filter {
            let mut conditions = Vec::new();
            let mut params = Vec::new();

            if let Some(ref src) = f.source {
                conditions.push("source = ?");
                params.push(src.clone());
            }

            if let Some(after) = f.after {
                conditions.push("timestamp > ?");
                params.push(after.format("%Y-%m-%dT%H:%M:%S%.6f%:z").to_string());
            }

            if let Some(before) = f.before {
                conditions.push("timestamp < ?");
                params.push(before.format("%Y-%m-%dT%H:%M:%S%.6f%:z").to_string());
            }

            let sql = if conditions.is_empty() {
                "SELECT * FROM entries".to_string()
            } else {
                format!("SELECT * FROM entries WHERE {}", conditions.join(" AND "))
            };

            (sql, params)
        } else {
            ("SELECT * FROM entries".to_string(), Vec::new())
        };

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| crate::Error::Message(format!("Failed to prepare query: {}", e)))?;

        let params_refs: Vec<_> = params.iter().map(|s| s.as_str()).collect();
        let entries = stmt
            .query_map(rusqlite::params_from_iter(params_refs), |row| {
                Self::row_to_entry(row)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| crate::Error::Message(format!("Failed to execute query: {}", e)))??;

        Ok(entries)
    }

    async fn count(&self) -> Result<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| crate::Error::Message(format!("Failed to lock connection: {}", e)))?;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
            .map_err(|e| crate::Error::Message(format!("Failed to count entries: {}", e)))?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cai_core::Source;

    #[tokio::test]
    async fn test_sqlite_store_and_get() {
        let storage = SqliteStorage::in_memory().unwrap();

        let entry = Entry {
            id: "test-1".to_string(),
            source: Source::Claude,
            timestamp: Utc::now(),
            prompt: "Test prompt".to_string(),
            response: "Test response".to_string(),
            metadata: cai_core::Metadata::default(),
        };

        storage.store(&entry).await.unwrap();

        let retrieved = storage.get("test-1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-1");
    }

    #[tokio::test]
    async fn test_sqlite_query_with_filter() {
        let storage = SqliteStorage::in_memory().unwrap();

        let entry1 = Entry {
            id: "test-1".to_string(),
            source: Source::Claude,
            timestamp: Utc::now(),
            prompt: "Test".to_string(),
            response: "Response".to_string(),
            metadata: cai_core::Metadata::default(),
        };

        let entry2 = Entry {
            id: "test-2".to_string(),
            source: Source::Git,
            timestamp: Utc::now(),
            prompt: "Test".to_string(),
            response: "Response".to_string(),
            metadata: cai_core::Metadata::default(),
        };

        storage.store(&entry1).await.unwrap();
        storage.store(&entry2).await.unwrap();

        let filter = Filter {
            source: Some("Claude".to_string()),
            after: None,
            before: None,
        };

        let results = storage.query(Some(&filter)).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test-1");
    }

    #[tokio::test]
    async fn test_sqlite_count() {
        let storage = SqliteStorage::in_memory().unwrap();
        assert_eq!(storage.count().await.unwrap(), 0);

        let entry = Entry {
            id: "test-1".to_string(),
            source: Source::Claude,
            timestamp: Utc::now(),
            prompt: "Test".to_string(),
            response: "Response".to_string(),
            metadata: cai_core::Metadata::default(),
        };

        storage.store(&entry).await.unwrap();
        assert_eq!(storage.count().await.unwrap(), 1);
    }
}
