//! Integration tests for CAI

use cai_core::{Entry, Source, Metadata};
use cai_storage::{MemoryStorage, Storage, Filter};
use cai_output::{JsonFormatter, JsonlFormatter, Formatter};
use chrono::{DateTime, Utc};
use std::io::Cursor;

fn setup_test_data() -> Vec<Entry> {
    vec![
        Entry {
            id: "int-1".to_string(),
            source: Source::Claude,
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            prompt: "Create a user model".to_string(),
            response: "struct User { id: Uuid, name: String }".to_string(),
            metadata: Metadata {
                file_path: Some("src/models/user.rs".to_string()),
                repo_url: Some("https://github.com/example/app".to_string()),
                commit_hash: Some("a1b2c3".to_string()),
                language: Some("Rust".to_string()),
                extra: std::collections::HashMap::new(),
            },
        },
        Entry {
            id: "int-2".to_string(),
            source: Source::Git,
            timestamp: DateTime::parse_from_rfc3339("2024-01-02T14:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            prompt: "Commit user model".to_string(),
            response: "Add user model implementation".to_string(),
            metadata: Metadata {
                file_path: None,
                repo_url: Some("https://github.com/example/app".to_string()),
                commit_hash: Some("d4e5f6".to_string()),
                language: None,
                extra: std::collections::HashMap::new(),
            },
        },
        Entry {
            id: "int-3".to_string(),
            source: Source::Claude,
            timestamp: DateTime::parse_from_rfc3339("2024-01-03T09:15:00Z")
                .unwrap()
                .with_timezone(&Utc),
            prompt: "Add authentication".to_string(),
            response: "impl Auth { fn login() {} }".to_string(),
            metadata: Metadata {
                file_path: Some("src/auth.rs".to_string()),
                repo_url: Some("https://github.com/example/app".to_string()),
                commit_hash: Some("g7h8i9".to_string()),
                language: Some("Rust".to_string()),
                extra: std::collections::HashMap::from([("complexity".to_string(), "high".to_string())]),
            },
        },
    ]
}

#[tokio::test]
async fn test_storage_and_query_integration() {
    let storage = MemoryStorage::new();
    let entries = setup_test_data();

    // Store all entries
    for entry in &entries {
        storage.store(entry).await.unwrap();
    }

    // Verify count
    assert_eq!(storage.count().await.unwrap(), 3);

    // Query all
    let all = storage.query(None).await.unwrap();
    assert_eq!(all.len(), 3);

    // Query by source
    let filter = Filter {
        source: Some("Claude".to_string()),
        after: None,
        before: None,
    };
    let claude_entries = storage.query(Some(&filter)).await.unwrap();
    assert_eq!(claude_entries.len(), 2);
}

#[tokio::test]
async fn test_storage_and_output_integration() {
    let storage = MemoryStorage::new();
    let entries = setup_test_data();

    for entry in &entries {
        storage.store(entry).await.unwrap();
    }

    let retrieved = storage.query(None).await.unwrap();

    // Format as JSON
    let mut buffer = Cursor::new(Vec::new());
    let formatter = JsonFormatter::default();
    formatter.format(&retrieved, &mut buffer).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    assert!(output.contains("\"int-1\""));
    assert!(output.contains("\"int-2\""));
    assert!(output.contains("\"int-3\""));
}

#[tokio::test]
async fn test_filtered_query_and_output() {
    let storage = MemoryStorage::new();
    let entries = setup_test_data();

    for entry in &entries {
        storage.store(entry).await.unwrap();
    }

    // Query only Claude entries
    let filter = Filter {
        source: Some("Claude".to_string()),
        after: None,
        before: None,
    };
    let claude_entries = storage.query(Some(&filter)).await.unwrap();

    assert_eq!(claude_entries.len(), 2);
    assert!(claude_entries.iter().all(|e| matches!(e.source, Source::Claude)));

    // Format filtered results
    let mut buffer = Cursor::new(Vec::new());
    let formatter = JsonFormatter::default();
    formatter.format(&claude_entries, &mut buffer).unwrap();

    let output = String::from_utf8(buffer.into_inner()).unwrap();
    assert!(output.contains("\"int-1\""));
    assert!(output.contains("\"int-3\""));
    assert!(!output.contains("\"int-2\""));
}

#[tokio::test]
async fn test_date_range_query() {
    let storage = MemoryStorage::new();
    let entries = setup_test_data();

    for entry in &entries {
        storage.store(entry).await.unwrap();
    }

    // Query entries after a specific date
    let filter = Filter {
        source: None,
        after: Some(DateTime::parse_from_rfc3339("2024-01-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)),
        before: None,
    };
    let recent = storage.query(Some(&filter)).await.unwrap();

    assert_eq!(recent.len(), 2);
    assert!(recent.iter().all(|e| e.id == "int-2" || e.id == "int-3"));
}

#[tokio::test]
async fn test_complex_filter_and_output() {
    let storage = MemoryStorage::new();
    let entries = setup_test_data();

    for entry in &entries {
        storage.store(entry).await.unwrap();
    }

    // Complex filter: Claude source + date range
    let filter = Filter {
        source: Some("Claude".to_string()),
        after: Some(DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)),
        before: Some(DateTime::parse_from_rfc3339("2024-01-02T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc)),
    };
    let results = storage.query(Some(&filter)).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "int-1");

    // Output in different formats
    {
        let mut buffer = Cursor::new(Vec::new());
        let formatter = JsonFormatter::default();
        formatter.format(&results, &mut buffer).unwrap();
        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(!output.is_empty());
    }
    {
        let mut buffer = Cursor::new(Vec::new());
        let formatter = JsonlFormatter::default();
        formatter.format(&results, &mut buffer).unwrap();
        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(!output.is_empty());
    }
}

#[tokio::test]
async fn test_get_by_id_workflow() {
    let storage = MemoryStorage::new();
    let entries = setup_test_data();

    for entry in &entries {
        storage.store(entry).await.unwrap();
    }

    // Get specific entry
    let entry = storage.get("int-2").await.unwrap();
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().source, Source::Git);

    // Get non-existent entry
    let missing = storage.get("non-existent").await.unwrap();
    assert!(missing.is_none());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let storage = std::sync::Arc::new(MemoryStorage::new());
    let entries = setup_test_data();

    // Concurrent stores
    let mut handles = Vec::new();
    for entry in &entries {
        let storage_clone = storage.clone();
        let entry_clone = entry.clone();
        let handle = tokio::spawn(async move {
            storage_clone.store(&entry_clone).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Verify all stored
    assert_eq!(storage.count().await.unwrap(), 3);

    // Concurrent queries
    let mut query_handles = Vec::new();
    for _ in 0..5 {
        let storage_clone = storage.clone();
        let handle = tokio::spawn(async move {
            storage_clone.query(None).await
        });
        query_handles.push(handle);
    }

    for handle in query_handles {
        let results = handle.await.unwrap().unwrap();
        assert_eq!(results.len(), 3);
    }
}
