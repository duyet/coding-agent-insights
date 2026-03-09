//! DuckDB storage backend
//!
//! Provides high-performance analytical storage using DuckDB.

use crate::{Filter, Result, Storage};
use cai_core::{Entry, Error, Source};
use chrono::{DateTime, Utc};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// DuckDB-based storage for CAI entries optimized for analytics queries
pub struct DuckDBStorage {
    conn: Arc<Mutex<duckdb::Connection>>,
}

impl DuckDBStorage {
    /// Create a new DuckDB storage with the given database path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the DuckDB database file (will be created if it doesn't exist)
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = duckdb::Connection::open(path)
            .map_err(|e| Error::Message(format!("Failed to open database: {}", e)))?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema()?;
        Ok(storage)
    }

    /// Create in-memory DuckDB storage (useful for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = duckdb::Connection::open_in_memory()
            .map_err(|e| Error::Message(format!("Failed to open in-memory database: {}", e)))?;

        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        storage.init_schema()?;
        Ok(storage)
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::Message(format!("Failed to create runtime: {}", e)))?;
        rt.block_on(async {
            let conn = self.conn.lock().await;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS entries (
                    id VARCHAR PRIMARY KEY,
                    source VARCHAR NOT NULL,
                    timestamp VARCHAR NOT NULL,
                    prompt TEXT NOT NULL,
                    response TEXT NOT NULL,
                    file_path TEXT,
                    repo_url TEXT,
                    commit_hash TEXT,
                    language TEXT,
                    metadata_json VARCHAR
                )",
                [],
            )
            .map_err(|e| Error::Message(format!("Failed to create table: {}", e)))?;

            Ok::<(), Error>(())
        })
    }

    /// Convert database row to Entry
    fn row_to_entry(row: &duckdb::Row) -> std::result::Result<Entry, duckdb::Error> {
        let source_str: String = row.get("source")?;
        let source = match source_str.as_str() {
            "Claude" => Source::Claude,
            "Codex" => Source::Codex,
            "Git" => Source::Git,
            _ => Source::Other(source_str),
        };

        let timestamp_str: String = row.get("timestamp")?;
        let timestamp = timestamp_str.parse::<DateTime<Utc>>().map_err(|e| {
            duckdb::Error::InvalidParameterName(format!("Invalid timestamp: {}", e))
        })?;

        let mut metadata = cai_core::Metadata::default();
        if let Ok(fp) = row.get::<_, Option<String>>("file_path") {
            metadata.file_path = fp;
        }
        if let Ok(url) = row.get::<_, Option<String>>("repo_url") {
            metadata.repo_url = url;
        }
        if let Ok(hash) = row.get::<_, Option<String>>("commit_hash") {
            metadata.commit_hash = hash;
        }
        if let Ok(lang) = row.get::<_, Option<String>>("language") {
            metadata.language = lang;
        }

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
impl Storage for DuckDBStorage {
    async fn store(&self, entry: &Entry) -> Result<()> {
        let conn = self.conn.lock().await;

        let source_str = format!("{:?}", entry.source);
        let timestamp_str = entry
            .timestamp
            .format("%Y-%m-%dT%H:%M:%S%.6f%:z")
            .to_string();

        // Build parameters manually - convert None to empty string
        let file_path = entry
            .metadata
            .file_path
            .as_deref()
            .unwrap_or("")
            .to_string();
        let repo_url = entry.metadata.repo_url.as_deref().unwrap_or("").to_string();
        let commit_hash = entry
            .metadata
            .commit_hash
            .as_deref()
            .unwrap_or("")
            .to_string();
        let language = entry.metadata.language.as_deref().unwrap_or("").to_string();

        conn.execute(
            "INSERT OR REPLACE INTO entries VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                &entry.id,
                &source_str,
                &timestamp_str,
                &entry.prompt,
                &entry.response,
                &file_path,
                &repo_url,
                &commit_hash,
                &language,
            ],
        )
        .map_err(|e| Error::Message(format!("Failed to insert entry: {}", e)))?;

        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Entry>> {
        use duckdb::OptionalExt;

        let conn = self.conn.lock().await;

        let mut stmt = conn
            .prepare("SELECT * FROM entries WHERE id = ?")
            .map_err(|e| Error::Message(format!("Failed to prepare query: {}", e)))?;

        let result = stmt
            .query_row([id], |row| Self::row_to_entry(row))
            .optional()
            .map_err(|e| Error::Message(format!("Failed to query entry: {}", e)))?;

        Ok(result)
    }

    async fn query(&self, filter: Option<&Filter>) -> Result<Vec<Entry>> {
        let conn = self.conn.lock().await;

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
            .map_err(|e| Error::Message(format!("Failed to prepare query: {}", e)))?;

        // For variable number of params, iterate over rows
        let mut entries = Vec::new();

        // Create a params slice that matches the query
        let params_refs: Vec<&str> = params.iter().map(|s| s.as_str()).collect();

        // Use query_map with the right parameters
        if params_refs.is_empty() {
            let mut rows = stmt
                .query([])
                .map_err(|e| Error::Message(format!("Failed to execute query: {}", e)))?;

            while let Some(row) = rows
                .next()
                .map_err(|e| Error::Message(format!("Failed to fetch row: {}", e)))?
            {
                let entry = Self::row_to_entry(&row)
                    .map_err(|e| Error::Message(format!("Failed to parse row: {}", e)))?;
                entries.push(entry);
            }
        } else if params_refs.len() == 1 {
            let mut rows = stmt
                .query([params_refs[0]])
                .map_err(|e| Error::Message(format!("Failed to execute query: {}", e)))?;

            while let Some(row) = rows
                .next()
                .map_err(|e| Error::Message(format!("Failed to fetch row: {}", e)))?
            {
                let entry = Self::row_to_entry(&row)
                    .map_err(|e| Error::Message(format!("Failed to parse row: {}", e)))?;
                entries.push(entry);
            }
        } else if params_refs.len() == 2 {
            let mut rows = stmt
                .query([params_refs[0], params_refs[1]])
                .map_err(|e| Error::Message(format!("Failed to execute query: {}", e)))?;

            while let Some(row) = rows
                .next()
                .map_err(|e| Error::Message(format!("Failed to fetch row: {}", e)))?
            {
                let entry = Self::row_to_entry(&row)
                    .map_err(|e| Error::Message(format!("Failed to parse row: {}", e)))?;
                entries.push(entry);
            }
        } else {
            return Err(Error::Message("Too many filter conditions".to_string()));
        }

        Ok(entries)
    }

    async fn count(&self) -> Result<usize> {
        let conn = self.conn.lock().await;

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entries", [], |row| row.get(0))
            .map_err(|e| Error::Message(format!("Failed to count entries: {}", e)))?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_duckdb_store_and_get() {
        let storage = DuckDBStorage::in_memory().unwrap();

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
    async fn test_duckdb_query_with_filter() {
        let storage = DuckDBStorage::in_memory().unwrap();

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
    async fn test_duckdb_count() {
        let storage = DuckDBStorage::in_memory().unwrap();
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
