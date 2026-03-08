//! Query execution engine

use std::sync::Arc;
use cai_core::Entry;
use cai_storage::Storage;
use crate::error::{QueryError, QueryResult, ColumnInfo, SchemaInfo, SchemaQueryType};
use crate::parser::{ParsedQuery, QueryType};

/// Query result type - can be either entries or schema information
#[derive(Debug, Clone)]
pub enum QueryResultData {
    /// Standard entry query results
    Entries(Vec<Entry>),
    /// Schema query results (SHOW TABLES, DESCRIBE)
    Schema(SchemaInfo),
}

/// Query engine for executing SQL queries against storage
#[derive(Clone)]
pub struct QueryEngine {
    storage: Arc<dyn Storage>,
}

impl QueryEngine {
    /// Create a new query engine
    pub fn new<S>(storage: S) -> Self
    where
        S: Storage + 'static,
    {
        Self {
            storage: Arc::new(storage),
        }
    }

    /// Create a new query engine from an Arc<dyn Storage>
    pub fn from_arc(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Execute a SQL query and return matching entries
    pub async fn execute(&self, sql: &str) -> QueryResult<Vec<Entry>> {
        let parsed = crate::parse(sql)?;

        // Handle schema queries
        match &parsed.query_type {
            QueryType::ShowTables => {
                // For backward compatibility, return empty vec for SHOW TABLES
                // Users should use execute_schema for schema queries
                Ok(vec![])
            }
            QueryType::DescribeTable(_) => {
                // For backward compatibility, return empty vec for DESCRIBE
                // Users should use execute_schema for schema queries
                Ok(vec![])
            }
            QueryType::Select => {
                // Validate table name
                if parsed.table.as_ref().is_some_and(|t| t.to_lowercase() != "entries") {
                    if let Some(table) = parsed.table {
                        return Err(QueryError::InvalidTable(table));
                    }
                }

                // For now, handle simple cases
                self.execute_simple_query(&parsed).await
            }
        }
    }

    /// Execute a SQL query and return full query result data (including schema)
    pub async fn execute_full(&self, sql: &str) -> QueryResult<QueryResultData> {
        let parsed = crate::parse(sql)?;

        match &parsed.query_type {
            QueryType::ShowTables => {
                Ok(QueryResultData::Schema(SchemaInfo {
                    query_type: SchemaQueryType::ShowTables,
                    table_name: None,
                    tables: vec!["entries".to_string()],
                    columns: vec![],
                }))
            }
            QueryType::DescribeTable(table_name) => {
                Ok(QueryResultData::Schema(SchemaInfo {
                    query_type: SchemaQueryType::DescribeTable,
                    table_name: Some(table_name.clone()),
                    tables: vec![],
                    columns: Self::get_entry_columns(),
                }))
            }
            QueryType::Select => {
                // Validate table name
                if parsed.table.as_ref().is_some_and(|t| t.to_lowercase() != "entries") {
                    if let Some(table) = parsed.table.clone() {
                        return Err(QueryError::InvalidTable(table));
                    }
                }

                let entries = self.execute_simple_query(&parsed).await?;
                Ok(QueryResultData::Entries(entries))
            }
        }
    }

    /// Get column information for the entries table
    fn get_entry_columns() -> Vec<ColumnInfo> {
        vec![
            ColumnInfo {
                name: "id".to_string(),
                data_type: "TEXT".to_string(),
                description: "Unique identifier".to_string(),
            },
            ColumnInfo {
                name: "source".to_string(),
                data_type: "TEXT".to_string(),
                description: "Source system (Claude, Codex, Git, Other)".to_string(),
            },
            ColumnInfo {
                name: "timestamp".to_string(),
                data_type: "TIMESTAMP".to_string(),
                description: "Interaction timestamp (UTC)".to_string(),
            },
            ColumnInfo {
                name: "prompt".to_string(),
                data_type: "TEXT".to_string(),
                description: "User prompt/input".to_string(),
            },
            ColumnInfo {
                name: "response".to_string(),
                data_type: "TEXT".to_string(),
                description: "AI response/output".to_string(),
            },
            ColumnInfo {
                name: "metadata".to_string(),
                data_type: "JSON".to_string(),
                description: "Additional metadata (file_path, language, etc.)".to_string(),
            },
        ]
    }

    async fn execute_simple_query(&self, parsed: &ParsedQuery) -> QueryResult<Vec<Entry>> {
        let mut entries = self.storage.query(None).await?;

        // Apply simple WHERE filter
        if let Some(ref where_sql) = parsed.where_sql {
            entries = self.apply_where_filter(entries, where_sql)?;
        }

        // Apply ORDER BY
        if !parsed.order_by.is_empty() {
            entries = self.apply_order_by(entries, &parsed.order_by)?;
        }

        // Apply LIMIT
        if let Some(limit) = parsed.limit {
            entries.truncate(limit);
        }

        Ok(entries)
    }

    fn apply_where_filter(&self, entries: Vec<Entry>, where_sql: &str) -> QueryResult<Vec<Entry>> {
        // Parse simple WHERE conditions
        let where_upper = where_sql.to_uppercase();

        // Extract values once to avoid lifetime issues
        let expected_source = if where_upper.contains("SOURCE =") || where_upper.contains("SOURCE=") {
            extract_quoted_string(where_sql)
        } else {
            None
        };

        let expected_ts_gt = if where_upper.contains("TIMESTAMP >") || where_upper.contains("TIMESTAMP>") {
            extract_timestamp(where_sql).and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())
        } else {
            None
        };

        let expected_ts_lt = if where_upper.contains("TIMESTAMP <") || where_upper.contains("TIMESTAMP<") {
            extract_timestamp(where_sql).and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok())
        } else {
            None
        };

        Ok(entries.into_iter()
            .filter(|entry| {
                if let Some(ref source) = expected_source {
                    if format!("{:?}", entry.source) != *source {
                        return false;
                    }
                }
                if let Some(ts) = expected_ts_gt {
                    if entry.timestamp <= ts {
                        return false;
                    }
                }
                if let Some(ts) = expected_ts_lt {
                    if entry.timestamp >= ts {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<_>>())
    }

    fn apply_order_by(&self, mut entries: Vec<Entry>, order_by: &[(String, bool)]) -> QueryResult<Vec<Entry>> {
        entries.sort_by(|a, b| {
            for (col, asc) in order_by {
                let cmp = match col.to_lowercase().as_str() {
                    "timestamp" => a.timestamp.cmp(&b.timestamp),
                    "source" => format!("{:?}", a.source).cmp(&format!("{:?}", b.source)),
                    "id" => a.id.cmp(&b.id),
                    "prompt" => a.prompt.cmp(&b.prompt),
                    "response" => a.response.cmp(&b.response),
                    _ => std::cmp::Ordering::Equal,
                };

                let cmp = if *asc { cmp } else { cmp.reverse() };

                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            std::cmp::Ordering::Equal
        });
        Ok(entries)
    }
}

fn extract_timestamp(sql: &str) -> Option<&str> {
    let start = sql.find('\'')? + 1;
    let end = sql[start..].find('\'')?;
    Some(&sql[start..start + end])
}

fn extract_quoted_string(sql: &str) -> Option<String> {
    let start = sql.find('\'')? + 1;
    let end = sql[start..].find('\'')?;
    Some(sql[start..start + end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cai_storage::MemoryStorage;
    use cai_core::Source;
    use chrono::Utc;

    fn make_test_entries() -> (MemoryStorage, Vec<Entry>) {
        let storage = MemoryStorage::new();

        let entries = vec![
            Entry {
                id: "1".to_string(),
                source: Source::Claude,
                timestamp: chrono::DateTime::parse_from_rfc3339("2024-01-15T10:00:00Z").unwrap().with_timezone(&Utc),
                prompt: "hello".to_string(),
                response: "world".to_string(),
                metadata: cai_core::Metadata {
                    file_path: Some("/path/to/file.rs".to_string()),
                    repo_url: None,
                    commit_hash: None,
                    language: Some("rust".to_string()),
                    ..Default::default()
                },
            },
            Entry {
                id: "2".to_string(),
                source: Source::Git,
                timestamp: chrono::DateTime::parse_from_rfc3339("2024-01-16T11:00:00Z").unwrap().with_timezone(&Utc),
                prompt: "commit".to_string(),
                response: "message".to_string(),
                metadata: cai_core::Metadata {
                    file_path: None,
                    repo_url: None,
                    commit_hash: Some("abc123".to_string()),
                    language: None,
                    ..Default::default()
                },
            },
        ];

        (storage, entries)
    }

    #[tokio::test]
    async fn test_simple_select() {
        let (storage, entries) = make_test_entries();
        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        let engine = QueryEngine::new(storage);
        let results = engine.execute("SELECT * FROM entries").await.unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_select_with_limit() {
        let (storage, entries) = make_test_entries();
        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        let engine = QueryEngine::new(storage);
        let results = engine.execute("SELECT * FROM entries LIMIT 1").await.unwrap();

        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_select_with_where() {
        let (storage, entries) = make_test_entries();
        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        let engine = QueryEngine::new(storage);
        let results = engine.execute("SELECT * FROM entries WHERE source = 'Claude'").await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, Source::Claude);
    }

    #[tokio::test]
    async fn test_order_by() {
        let (storage, entries) = make_test_entries();
        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        let engine = QueryEngine::new(storage);
        // Note: ORDER BY parsing not implemented in simple parser yet
        let results = engine.execute("SELECT * FROM entries ORDER BY timestamp DESC").await.unwrap();

        assert_eq!(results.len(), 2);
        // For now, just verify we get results (ordering not implemented yet)
    }

    #[tokio::test]
    async fn test_invalid_table() {
        let storage = MemoryStorage::new();
        let engine = QueryEngine::new(storage);

        let result = engine.execute("SELECT * FROM invalid_table").await;

        assert!(matches!(result, Err(QueryError::InvalidTable(_))));
    }
}
