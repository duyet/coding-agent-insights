//! E2E tests for query execution workflows
//!
//! These tests verify complete query flows from various sources.

#[cfg(test)]
use cai_core::{Entry, Metadata, Source};
#[cfg(test)]
use chrono::{DateTime, Utc};

#[cfg(test)]
fn setup_test_entries() -> Vec<Entry> {
    vec![
        Entry {
            id: "query-test-1".to_string(),
            source: Source::Claude,
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            prompt: "Write a function".to_string(),
            response: "fn test() {}".to_string(),
            metadata: Metadata {
                file_path: Some("src/test.rs".to_string()),
                repo_url: Some("https://github.com/test/repo".to_string()),
                commit_hash: Some("abc123".to_string()),
                language: Some("Rust".to_string()),
                extra: std::collections::HashMap::new(),
            },
        },
        Entry {
            id: "query-test-2".to_string(),
            source: Source::Codex,
            timestamp: DateTime::parse_from_rfc3339("2024-01-02T14:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            prompt: "Add error handling".to_string(),
            response: "Result<T, E>".to_string(),
            metadata: Metadata {
                file_path: Some("src/error.rs".to_string()),
                repo_url: Some("https://github.com/test/repo".to_string()),
                commit_hash: Some("def456".to_string()),
                language: Some("Rust".to_string()),
                extra: std::collections::HashMap::new(),
            },
        },
        Entry {
            id: "query-test-3".to_string(),
            source: Source::Git,
            timestamp: DateTime::parse_from_rfc3339("2024-01-03T09:15:00Z")
                .unwrap()
                .with_timezone(&Utc),
            prompt: "Commit changes".to_string(),
            response: "Changes committed".to_string(),
            metadata: Metadata {
                file_path: None,
                repo_url: Some("https://github.com/test/repo".to_string()),
                commit_hash: Some("ghi789".to_string()),
                language: None,
                extra: std::collections::HashMap::new(),
            },
        },
        Entry {
            id: "query-test-4".to_string(),
            source: Source::Claude,
            timestamp: DateTime::parse_from_rfc3339("2024-01-04T16:45:00Z")
                .unwrap()
                .with_timezone(&Utc),
            prompt: "Implement trait".to_string(),
            response: "trait Trait {}".to_string(),
            metadata: Metadata {
                file_path: Some("src/trait.rs".to_string()),
                repo_url: Some("https://github.com/test/repo".to_string()),
                commit_hash: Some("jkl012".to_string()),
                language: Some("Rust".to_string()),
                extra: std::collections::HashMap::from([(
                    "complexity".to_string(),
                    "high".to_string(),
                )]),
            },
        },
    ]
}

#[cfg(test)]
mod basic_query_tests {
    use cai_core::Source;
    use cai_storage::{Filter, MemoryStorage, Storage};
    use chrono::{DateTime, Utc};

    /// Test simple SELECT all query
    #[tokio::test]
    async fn test_query_all_entries() {
        let storage = MemoryStorage::new();
        let entries = super::setup_test_entries();

        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        let results = storage.query(None).await.unwrap();
        assert_eq!(results.len(), 4, "Should retrieve all 4 entries");
    }

    /// Test WHERE clause by source
    #[tokio::test]
    async fn test_query_by_source() {
        let storage = MemoryStorage::new();
        let entries = super::setup_test_entries();

        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        // Query only Claude entries
        let filter = Filter {
            source: Some("Claude".to_string()),
            after: None,
            before: None,
        };
        let results = storage.query(Some(&filter)).await.unwrap();
        assert_eq!(results.len(), 2, "Should find 2 Claude entries");
        assert!(results.iter().all(|e| matches!(e.source, Source::Claude)));
    }

    /// Test WHERE clause with date range (after)
    #[tokio::test]
    async fn test_query_after_date() {
        let storage = MemoryStorage::new();
        let entries = super::setup_test_entries();

        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        // Query entries after Jan 2, 2024
        let filter = Filter {
            source: None,
            after: Some(
                DateTime::parse_from_rfc3339("2024-01-02T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            before: None,
        };
        let results = storage.query(Some(&filter)).await.unwrap();
        assert_eq!(results.len(), 3, "Should find 3 entries after Jan 2");
    }

    /// Test WHERE clause with date range (before)
    #[tokio::test]
    async fn test_query_before_date() {
        let storage = MemoryStorage::new();
        let entries = super::setup_test_entries();

        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        // Query entries before Jan 3, 2024
        let filter = Filter {
            source: None,
            after: None,
            before: Some(
                DateTime::parse_from_rfc3339("2024-01-03T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };
        let results = storage.query(Some(&filter)).await.unwrap();
        assert_eq!(results.len(), 2, "Should find 2 entries before Jan 3");
    }

    /// Test WHERE clause with both source and date range
    #[tokio::test]
    async fn test_query_combined_filter() {
        let storage = MemoryStorage::new();
        let entries = super::setup_test_entries();

        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        // Query Claude entries after Jan 1
        let filter = Filter {
            source: Some("Claude".to_string()),
            after: Some(
                DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            before: None,
        };
        let results = storage.query(Some(&filter)).await.unwrap();
        assert_eq!(
            results.len(),
            1,
            "Should find 1 Claude entry after midday Jan 1"
        );
        assert_eq!(results[0].id, "query-test-4");
    }
}

#[cfg(test)]
mod output_format_tests {
    use cai_output::{Formatter, JsonFormatter, JsonlFormatter};
    use cai_storage::{MemoryStorage, Storage};
    use std::io::Cursor;

    /// Test JSON output format
    #[tokio::test]
    async fn test_json_output() {
        let storage = MemoryStorage::new();
        let entries = super::setup_test_entries();

        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        let results = storage.query(None).await.unwrap();

        let mut buffer = Cursor::new(Vec::new());
        let formatter = JsonFormatter::default();
        formatter.format(&results, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();

        // Verify JSON structure
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.as_array().is_some(), "Output should be a JSON array");
        assert_eq!(parsed.as_array().unwrap().len(), 4);
    }

    /// Test JSONL output format
    #[tokio::test]
    async fn test_jsonl_output() {
        let storage = MemoryStorage::new();
        let entries = super::setup_test_entries();

        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        let results = storage.query(None).await.unwrap();

        let mut buffer = Cursor::new(Vec::new());
        let formatter = JsonlFormatter::default();
        formatter.format(&results, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();

        // Verify JSONL structure (one JSON object per line)
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4, "Should have 4 lines for 4 entries");

        for line in lines {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(parsed.is_object(), "Each line should be a JSON object");
        }
    }

    /// Test empty result set
    #[tokio::test]
    async fn test_empty_result_output() {
        let storage = MemoryStorage::new();

        let results = storage.query(None).await.unwrap();
        assert_eq!(results.len(), 0);

        let mut buffer = Cursor::new(Vec::new());
        let formatter = JsonFormatter::default();
        formatter.format(&results, &mut buffer).unwrap();

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 0);
    }
}

#[cfg(test)]
mod edge_case_tests {
    use cai_storage::{Filter, MemoryStorage, Storage};
    use chrono::{DateTime, Utc};

    /// Test query with no matches
    #[tokio::test]
    async fn test_query_no_matches() {
        let storage = MemoryStorage::new();
        let entries = super::setup_test_entries();

        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        // Query for non-existent source
        let filter = Filter {
            source: Some("NonExistent".to_string()),
            after: None,
            before: None,
        };
        let results = storage.query(Some(&filter)).await.unwrap();
        assert_eq!(results.len(), 0, "Should find no entries");
    }

    /// Test query with date range outside data range
    #[tokio::test]
    async fn test_query_outside_date_range() {
        let storage = MemoryStorage::new();
        let entries = super::setup_test_entries();

        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        // Query for dates far in the future
        let filter = Filter {
            source: None,
            after: Some(
                DateTime::parse_from_rfc3339("2099-01-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            before: None,
        };
        let results = storage.query(Some(&filter)).await.unwrap();
        assert_eq!(results.len(), 0, "Should find no future entries");
    }

    /// Test get by ID
    #[tokio::test]
    async fn test_get_by_id() {
        let storage = MemoryStorage::new();
        let entries = super::setup_test_entries();

        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        let entry = storage.get("query-test-2").await.unwrap();
        assert!(entry.is_some(), "Should find entry by ID");
        assert_eq!(entry.unwrap().source, cai_core::Source::Codex);

        let missing = storage.get("non-existent").await.unwrap();
        assert!(missing.is_none(), "Should not find non-existent entry");
    }

    /// Test count operation
    #[tokio::test]
    async fn test_count() {
        let storage = MemoryStorage::new();

        assert_eq!(
            storage.count().await.unwrap(),
            0,
            "Initial count should be 0"
        );

        let entries = super::setup_test_entries();
        for entry in &entries {
            storage.store(entry).await.unwrap();
        }

        assert_eq!(storage.count().await.unwrap(), 4, "Count should be 4");
    }
}
