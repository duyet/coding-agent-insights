//! Test helpers and utilities for CAI testing

use cai_core::{Entry, Source, Metadata};
use chrono::{DateTime, Utc};

/// Create a test entry with default values
pub fn make_entry(id: &str) -> Entry {
    Entry {
        id: id.to_string(),
        source: Source::Claude,
        timestamp: Utc::now(),
        prompt: format!("Test prompt {}", id),
        response: format!("Test response {}", id),
        metadata: Metadata {
            file_path: Some(format!("src/{}.rs", id)),
            repo_url: Some("https://github.com/test/repo".to_string()),
            commit_hash: Some(format!("commit_{}", id)),
            language: Some("Rust".to_string()),
            extra: std::collections::HashMap::new(),
        },
    }
}

/// Create a test entry with custom prompt
pub fn make_entry_with_prompt(id: &str, prompt: &str) -> Entry {
    let mut entry = make_entry(id);
    entry.prompt = prompt.to_string();
    entry
}

/// Create a test entry with custom source
pub fn make_entry_with_source(id: &str, source: Source) -> Entry {
    let mut entry = make_entry(id);
    entry.source = source;
    entry
}

/// Create a test entry with timestamp
pub fn make_entry_with_timestamp(id: &str, timestamp: DateTime<Utc>) -> Entry {
    let mut entry = make_entry(id);
    entry.timestamp = timestamp;
    entry
}

/// Create multiple test entries
pub fn make_entries(count: usize) -> Vec<Entry> {
    (0..count).map(|i| make_entry(&format!("entry-{}", i))).collect()
}

/// Assert that two entries are equal
pub fn assert_entries_equal(a: &Entry, b: &Entry) {
    assert_eq!(a.id, b.id);
    assert_eq!(a.source, b.source);
    assert_eq!(a.prompt, b.prompt);
    assert_eq!(a.response, b.response);
}
