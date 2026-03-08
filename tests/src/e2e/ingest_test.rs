//! E2E tests for data ingestion workflows
//!
//! These tests verify complete ingestion flows from various sources.

use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    // CARGO_MANIFEST_DIR for cai-tests is the tests/ directory
    // Fixtures are at tests/fixtures/
    PathBuf::from(format!("{}/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name))
}

#[cfg(test)]
mod ingest_tests {
    use super::*;
    use std::fs;

    /// Test ingesting Claude conversation from fixture file
    #[tokio::test]
    async fn test_ingest_claude_conversation() {
        let fixture_path = fixture_path("claude_conversation.json");
        assert!(fixture_path.exists(), "Fixture file should exist");

        let content = fs::read_to_string(&fixture_path)
            .expect("Should read fixture file");

        // Verify JSON structure
        let json: serde_json::Value = serde_json::from_str(&content)
            .expect("Should parse valid JSON");

        assert_eq!(json["version"], "1.0");
        assert_eq!(json["api"], "anthropic");
        assert!(json["messages"].as_array().unwrap().len() >= 2);

        // Verify we have user-assistant pairs
        let messages = json["messages"].as_array().unwrap();
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["role"], "assistant");
    }

    /// Test ingesting Codex history from fixture file
    #[tokio::test]
    async fn test_ingest_codex_history() {
        let fixture_path = fixture_path("codex_history.jsonl");
        assert!(fixture_path.exists(), "Fixture file should exist");

        let content = fs::read_to_string(&fixture_path)
            .expect("Should read fixture file");

        // Verify JSONL structure - each line is valid JSON
        let lines: Vec<&str> = content.lines().collect();
        assert!(lines.len() >= 3, "Should have at least 3 entries");

        for line in lines {
            let json: serde_json::Value = serde_json::from_str(line)
                .expect("Each line should be valid JSON");

            assert!(json.get("timestamp").is_some());
            assert!(json.get("prompt").is_some());
            assert!(json.get("response").is_some());
        }
    }

    /// Test ingesting Git log from fixture file
    #[tokio::test]
    async fn test_ingest_git_log() {
        let fixture_path = fixture_path("git_log.txt");
        assert!(fixture_path.exists(), "Fixture file should exist");

        let content = fs::read_to_string(&fixture_path)
            .expect("Should read fixture file");

        // Verify git log format: hash|timestamp|name|email|message
        let lines: Vec<&str> = content.lines().collect();
        assert!(lines.len() >= 3, "Should have at least 3 commits");

        for line in lines {
            let parts: Vec<&str> = line.split('|').collect();
            assert_eq!(parts.len(), 5, "Each line should have 5 pipe-delimited fields");

            // Verify hash format (short git hash is 7-15 chars)
            assert!(parts[0].len() >= 7 && parts[0].len() <= 40, "Commit hash should be 7-40 chars, got {}", parts[0].len());

            // Verify timestamp is ISO 8601
            parts[1].parse::<chrono::DateTime<chrono::Utc>>()
                .expect("Timestamp should be valid ISO 8601");
        }
    }

    /// Test error handling for non-existent file
    #[tokio::test]
    async fn test_ingest_nonexistent_file() {
        let fixture_path = fixture_path("nonexistent.json");
        assert!(!fixture_path.exists());

        let result = fs::read_to_string(&fixture_path);
        assert!(result.is_err(), "Should fail to read non-existent file");
    }

    /// Test error handling for invalid JSON
    #[tokio::test]
    async fn test_ingest_invalid_json() {
        let invalid_json = "{ this is not valid json }";
        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err(), "Should fail to parse invalid JSON");
    }

    /// Test error handling for malformed JSONL
    #[tokio::test]
    async fn test_ingest_malformed_jsonl() {
        let malformed_jsonl = r#"{"valid": "json"}
{not valid json}
{"also": "valid"}"#;

        let mut valid_count = 0;
        let mut invalid_count = 0;

        for line in malformed_jsonl.lines() {
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(_) => valid_count += 1,
                Err(_) => invalid_count += 1,
            }
        }

        assert_eq!(valid_count, 2, "Should have 2 valid JSON lines");
        assert_eq!(invalid_count, 1, "Should have 1 invalid JSON line");
    }

    /// Test error handling for malformed git log
    #[tokio::test]
    async fn test_ingest_malformed_git_log() {
        let malformed_log = r#"abc123|2024-01-15T10:00:00Z|Name|email@example.com|Valid commit
invalid-format-commit
def456|invalid-timestamp|Name|email@example.com|Invalid timestamp"#;

        let valid_count = malformed_log.lines()
            .filter(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() != 5 {
                    return false;
                }
                parts[1].parse::<chrono::DateTime<chrono::Utc>>().is_ok()
            })
            .count();

        assert_eq!(valid_count, 1, "Only first line should be valid");
    }
}

#[cfg(test)]
mod integration {
    use cai_core::{Entry, Source, Metadata};
    use cai_storage::{MemoryStorage, Storage};

    /// Test full ingestion workflow with storage
    #[tokio::test]
    async fn test_ingest_and_store_workflow() {
        let storage = MemoryStorage::new();

        // Create test entry
        let entry = Entry {
            id: "e2e-test-1".to_string(),
            source: Source::Claude,
            timestamp: chrono::Utc::now(),
            prompt: "E2E test prompt".to_string(),
            response: "E2E test response".to_string(),
            metadata: Metadata::default(),
        };

        // Store entry
        storage.store(&entry).await
            .expect("Should store entry successfully");

        // Verify storage count
        let count = storage.count().await
            .expect("Should get count successfully");
        assert_eq!(count, 1, "Should have 1 entry");

        // Retrieve entry
        let retrieved = storage.get("e2e-test-1").await
            .expect("Should get entry successfully");
        assert!(retrieved.is_some(), "Should find stored entry");
        assert_eq!(retrieved.unwrap().id, "e2e-test-1");
    }

    /// Test batch ingestion workflow
    #[tokio::test]
    async fn test_batch_ingestion_workflow() {
        let storage = MemoryStorage::new();

        let entries: Vec<Entry> = (0..10).map(|i| Entry {
            id: format!("batch-entry-{}", i),
            source: Source::Codex,
            timestamp: chrono::Utc::now(),
            prompt: format!("Prompt {}", i),
            response: format!("Response {}", i),
            metadata: Metadata::default(),
        }).collect();

        // Store all entries
        for entry in &entries {
            storage.store(entry).await
                .expect("Should store each entry");
        }

        // Verify all stored
        let count = storage.count().await
            .expect("Should get count");
        assert_eq!(count, 10, "Should have 10 entries");

        // Query all
        let all = storage.query(None).await
            .expect("Should query all entries");
        assert_eq!(all.len(), 10);
    }
}
