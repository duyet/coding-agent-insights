//! Unit tests for cai-storage MemoryStorage

use cai_storage::{MemoryStorage, Storage, Filter};
use cai_core::{Entry, Source, Metadata};
use chrono::{DateTime, Utc};
use std::str::FromStr;

fn create_test_entry(id: &str, timestamp: &str) -> Entry {
    Entry {
        id: id.to_string(),
        source: Source::Claude,
        timestamp: DateTime::parse_from_rfc3339(timestamp)
            .unwrap()
            .with_timezone(&Utc),
        prompt: "Test prompt".to_string(),
        response: "Test response".to_string(),
        metadata: Metadata::default(),
    }
}

#[tokio::test]
async fn test_memory_storage_new() {
    let storage = MemoryStorage::new();
    assert_eq!(storage.count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_memory_storage_default() {
    let storage = MemoryStorage::default();
    assert_eq!(storage.count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_memory_storage_store() {
    let storage = MemoryStorage::new();
    let entry = create_test_entry("test-1", "2024-01-01T12:00:00Z");

    storage.store(&entry).await.unwrap();
    assert_eq!(storage.count().await.unwrap(), 1);
}

#[tokio::test]
async fn test_memory_storage_store_multiple() {
    let storage = MemoryStorage::new();

    for i in 0..5 {
        let entry = create_test_entry(&format!("entry-{}", i), "2024-01-01T12:00:00Z");
        storage.store(&entry).await.unwrap();
    }

    assert_eq!(storage.count().await.unwrap(), 5);
}

#[tokio::test]
async fn test_memory_storage_get_existing() {
    let storage = MemoryStorage::new();
    let entry = create_test_entry("test-1", "2024-01-01T12:00:00Z");

    storage.store(&entry).await.unwrap();
    let retrieved = storage.get("test-1").await.unwrap();

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "test-1");
}

#[tokio::test]
async fn test_memory_storage_get_missing() {
    let storage = MemoryStorage::new();
    let entry = create_test_entry("test-1", "2024-01-01T12:00:00Z");

    storage.store(&entry).await.unwrap();
    let retrieved = storage.get("non-existent").await.unwrap();

    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_memory_storage_query_no_filter() {
    let storage = MemoryStorage::new();

    for i in 0..3 {
        let entry = create_test_entry(&format!("entry-{}", i), "2024-01-01T12:00:00Z");
        storage.store(&entry).await.unwrap();
    }

    let results = storage.query(None).await.unwrap();
    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn test_memory_storage_query_with_source_filter() {
    let storage = MemoryStorage::new();

    let mut entry1 = create_test_entry("entry-1", "2024-01-01T12:00:00Z");
    entry1.source = Source::Claude;

    let mut entry2 = create_test_entry("entry-2", "2024-01-02T12:00:00Z");
    entry2.source = Source::Git;

    storage.store(&entry1).await.unwrap();
    storage.store(&entry2).await.unwrap();

    let filter = Filter {
        source: Some("Claude".to_string()),
        after: None,
        before: None,
    };

    let results = storage.query(Some(&filter)).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "entry-1");
}

#[tokio::test]
async fn test_memory_storage_query_with_after_filter() {
    let storage = MemoryStorage::new();

    storage.store(&create_test_entry("entry-1", "2024-01-01T12:00:00Z")).await.unwrap();
    storage.store(&create_test_entry("entry-2", "2024-01-02T12:00:00Z")).await.unwrap();
    storage.store(&create_test_entry("entry-3", "2024-01-03T12:00:00Z")).await.unwrap();

    let filter = Filter {
        source: None,
        after: Some(DateTime::from_str("2024-01-02T00:00:00Z").unwrap()),
        before: None,
    };

    let results = storage.query(Some(&filter)).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.id != "entry-1"));
}

#[tokio::test]
async fn test_memory_storage_query_with_before_filter() {
    let storage = MemoryStorage::new();

    storage.store(&create_test_entry("entry-1", "2024-01-01T12:00:00Z")).await.unwrap();
    storage.store(&create_test_entry("entry-2", "2024-01-02T12:00:00Z")).await.unwrap();
    storage.store(&create_test_entry("entry-3", "2024-01-03T12:00:00Z")).await.unwrap();

    let filter = Filter {
        source: None,
        after: None,
        before: Some(DateTime::from_str("2024-01-02T23:59:59Z").unwrap()),
    };

    let results = storage.query(Some(&filter)).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.id != "entry-3"));
}

#[tokio::test]
async fn test_memory_storage_query_with_range_filter() {
    let storage = MemoryStorage::new();

    storage.store(&create_test_entry("entry-1", "2024-01-01T12:00:00Z")).await.unwrap();
    storage.store(&create_test_entry("entry-2", "2024-01-02T12:00:00Z")).await.unwrap();
    storage.store(&create_test_entry("entry-3", "2024-01-03T12:00:00Z")).await.unwrap();
    storage.store(&create_test_entry("entry-4", "2024-01-04T12:00:00Z")).await.unwrap();

    let filter = Filter {
        source: None,
        after: Some(DateTime::from_str("2024-01-02T00:00:00Z").unwrap()),
        before: Some(DateTime::from_str("2024-01-03T23:59:59Z").unwrap()),
    };

    let results = storage.query(Some(&filter)).await.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.id == "entry-2" || e.id == "entry-3"));
}

#[tokio::test]
async fn test_memory_storage_concurrent_stores() {
    let storage = std::sync::Arc::new(MemoryStorage::new());
    let mut handles = Vec::new();

    for i in 0..10 {
        let storage_clone = storage.clone();
        let handle = tokio::spawn(async move {
            let entry = create_test_entry(&format!("entry-{}", i), "2024-01-01T12:00:00Z");
            storage_clone.store(&entry).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    assert_eq!(storage.count().await.unwrap(), 10);
}

#[tokio::test]
async fn test_memory_storage_count_zero() {
    test_memory_storage_count_helper(0).await
}

#[tokio::test]
async fn test_memory_storage_count_one() {
    test_memory_storage_count_helper(1).await
}

#[tokio::test]
async fn test_memory_storage_count_ten() {
    test_memory_storage_count_helper(10).await
}

#[tokio::test]
async fn test_memory_storage_count_hundred() {
    test_memory_storage_count_helper(100).await
}

async fn test_memory_storage_count_helper(count: usize) {
    let storage = MemoryStorage::new();

    for i in 0..count {
        let entry = create_test_entry(&format!("entry-{}", i), "2024-01-01T12:00:00Z");
        storage.store(&entry).await.unwrap();
    }

    assert_eq!(storage.count().await.unwrap(), count);
}
