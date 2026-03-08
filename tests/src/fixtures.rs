//! Test fixtures for CAI testing

use cai_core::{Entry, Source, Metadata};
use chrono::{DateTime, Utc};
use serde_json::json;

/// Create a test entry with default values
pub fn test_entry() -> Entry {
    Entry {
        id: "test-id-1".to_string(),
        source: Source::Claude,
        timestamp: DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
        prompt: "Write a function".to_string(),
        response: "Here's the function".to_string(),
        metadata: Metadata {
            file_path: Some("src/main.rs".to_string()),
            repo_url: Some("https://github.com/test/repo".to_string()),
            commit_hash: Some("abc123".to_string()),
            language: Some("Rust".to_string()),
            extra: std::collections::HashMap::new(),
        },
    }
}

/// Create multiple test entries
pub fn test_entries() -> Vec<Entry> {
    vec![
        Entry {
            id: "entry-1".to_string(),
            source: Source::Claude,
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            prompt: "Create a struct".to_string(),
            response: "struct User { name: String }".to_string(),
            metadata: Metadata {
                file_path: Some("src/models.rs".to_string()),
                repo_url: Some("https://github.com/test/repo".to_string()),
                commit_hash: Some("commit1".to_string()),
                language: Some("Rust".to_string()),
                extra: std::collections::HashMap::new(),
            },
        },
        Entry {
            id: "entry-2".to_string(),
            source: Source::Claude,
            timestamp: DateTime::parse_from_rfc3339("2024-01-02T14:30:00Z")
                .unwrap()
                .with_timezone(&Utc),
            prompt: "Add error handling".to_string(),
            response: "Result<T, Error>".to_string(),
            metadata: Metadata {
                file_path: Some("src/error.rs".to_string()),
                repo_url: Some("https://github.com/test/repo".to_string()),
                commit_hash: Some("commit2".to_string()),
                language: Some("Rust".to_string()),
                extra: std::collections::HashMap::new(),
            },
        },
        Entry {
            id: "entry-3".to_string(),
            source: Source::Git,
            timestamp: DateTime::parse_from_rfc3339("2024-01-03T09:15:00Z")
                .unwrap()
                .with_timezone(&Utc),
            prompt: "Commit changes".to_string(),
            response: "Changes committed".to_string(),
            metadata: Metadata {
                file_path: None,
                repo_url: Some("https://github.com/test/repo".to_string()),
                commit_hash: Some("commit3".to_string()),
                language: None,
                extra: std::collections::HashMap::new(),
            },
        },
    ]
}

/// Create test entries for benchmarking
pub fn benchmark_entries(count: usize) -> Vec<Entry> {
    (0..count)
        .map(|i| Entry {
            id: format!("bench-entry-{}", i),
            source: if i % 3 == 0 { Source::Claude } else if i % 3 == 1 { Source::Codex } else { Source::Git },
            timestamp: Utc::now() - chrono::Duration::seconds(i as i64),
            prompt: format!("Benchmark prompt {}", i),
            response: format!("Benchmark response {}", i),
            metadata: Metadata {
                file_path: Some(format!("src/file{}.rs", i % 10)),
                repo_url: Some("https://github.com/test/repo".to_string()),
                commit_hash: Some(format!("commit{}", i)),
                language: Some("Rust".to_string()),
                extra: std::collections::HashMap::new(),
            },
        })
        .collect()
}

/// Sample Claude conversation JSON
pub fn sample_claude_json() -> serde_json::Value {
    json!({
        "conversation": [
            {
                "role": "user",
                "content": "Write a Rust function",
                "timestamp": "2024-01-01T12:00:00Z"
            },
            {
                "role": "assistant",
                "content": "fn hello() { println!(\"Hello\"); }",
                "timestamp": "2024-01-01T12:00:01Z"
            }
        ]
    })
}

/// Sample git log output
pub fn sample_git_log() -> String {
    r#"abc123def|2024-01-01T12:00:00Z|John Doe|john@example.com|Add new feature
def456ghi|2024-01-02T14:30:00Z|Jane Smith|jane@example.com|Fix bug in parser
ghi789jkl|2024-01-03T09:15:00Z|Bob Wilson|bob@example.com|Update documentation"#.to_string()
}
