//! Unit tests for cai-core

use cai_core::{Entry, Error, Metadata, Source};
use chrono::{DateTime, Utc};
use serde_json::{from_value, json, to_value};

#[test]
fn test_entry_creation() {
    let timestamp = DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let entry = Entry {
        id: "test-id".to_string(),
        source: Source::Claude,
        timestamp,
        prompt: "Test prompt".to_string(),
        response: "Test response".to_string(),
        metadata: Metadata::default(),
    };

    assert_eq!(entry.id, "test-id");
    assert_eq!(entry.source, Source::Claude);
    assert_eq!(entry.prompt, "Test prompt");
    assert_eq!(entry.response, "Test response");
}

#[test]
fn test_source_variants() {
    assert!(matches!(Source::Claude, Source::Claude));
    assert!(matches!(Source::Codex, Source::Codex));
    assert!(matches!(Source::Git, Source::Git));
    assert!(matches!(
        Source::Other("custom".to_string()),
        Source::Other(_)
    ));
}

#[test]
fn test_source_equality() {
    assert_eq!(Source::Claude, Source::Claude);
    assert_eq!(
        Source::Other("test".to_string()),
        Source::Other("test".to_string())
    );
    assert_ne!(Source::Claude, Source::Codex);
}

#[test]
fn test_metadata_default() {
    let metadata = Metadata::default();
    assert!(metadata.file_path.is_none());
    assert!(metadata.repo_url.is_none());
    assert!(metadata.commit_hash.is_none());
    assert!(metadata.language.is_none());
    assert!(metadata.extra.is_empty());
}

#[test]
fn test_metadata_with_fields() {
    let metadata = Metadata {
        file_path: Some("src/main.rs".to_string()),
        repo_url: Some("https://github.com/test/repo".to_string()),
        commit_hash: Some("abc123".to_string()),
        language: Some("Rust".to_string()),
        extra: std::collections::HashMap::from([
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]),
    };

    assert_eq!(metadata.file_path, Some("src/main.rs".to_string()));
    assert_eq!(
        metadata.repo_url,
        Some("https://github.com/test/repo".to_string())
    );
    assert_eq!(metadata.commit_hash, Some("abc123".to_string()));
    assert_eq!(metadata.language, Some("Rust".to_string()));
    assert_eq!(metadata.extra.len(), 2);
}

#[rstest::rstest]
#[case("simple")]
#[case("with spaces")]
#[case("with special chars: !@#$%")]
fn test_prompt_variations(#[case] prompt: &str) {
    let entry = Entry {
        id: "test".to_string(),
        source: Source::Claude,
        timestamp: Utc::now(),
        prompt: prompt.to_string(),
        response: "response".to_string(),
        metadata: Metadata::default(),
    };

    assert_eq!(entry.prompt, prompt);
}

#[test]
fn test_entry_serialization() {
    let entry = Entry {
        id: "test-id".to_string(),
        source: Source::Claude,
        timestamp: DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
        prompt: "Test prompt".to_string(),
        response: "Test response".to_string(),
        metadata: Metadata {
            file_path: Some("src/main.rs".to_string()),
            repo_url: None,
            commit_hash: None,
            language: Some("Rust".to_string()),
            extra: std::collections::HashMap::new(),
        },
    };

    let json = to_value(&entry).unwrap();
    assert_eq!(json["id"], "test-id");
    assert_eq!(json["source"], "Claude");
    assert_eq!(json["prompt"], "Test prompt");
}

#[test]
fn test_entry_deserialization() {
    let json = json!({
        "id": "test-id",
        "source": "Claude",
        "timestamp": "2024-01-01T12:00:00Z",
        "prompt": "Test prompt",
        "response": "Test response",
        "metadata": {
            "file_path": "src/main.rs",
            "repo_url": null,
            "commit_hash": null,
            "language": "Rust",
            "extra": {}
        }
    });

    let entry: Entry = from_value(json).unwrap();
    assert_eq!(entry.id, "test-id");
    assert_eq!(entry.source, Source::Claude);
    assert_eq!(entry.prompt, "Test prompt");
    assert_eq!(entry.metadata.file_path, Some("src/main.rs".to_string()));
}

#[test]
fn test_error_display() {
    let io_error = Error::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "file not found",
    ));
    assert!(io_error.to_string().contains("I/O error"));

    let msg_error = Error::Message("custom error".to_string());
    assert_eq!(msg_error.to_string(), "custom error");
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let error: Error = io_err.into();
    assert!(matches!(error, Error::Io(_)));
}

proptest::proptest! {
    #[test]
    fn test_entry_id_roundtrip(id in "[a-zA-Z0-9_-]{1,50}") {
        let entry = Entry {
            id: id.clone(),
            source: Source::Claude,
            timestamp: Utc::now(),
            prompt: "test".to_string(),
            response: "test".to_string(),
            metadata: Metadata::default(),
        };

        let serialized = to_value(&entry).unwrap();
        let deserialized: Entry = from_value(serialized).unwrap();
        assert_eq!(deserialized.id, id);
    }

    #[test]
    fn test_prompt_length_property(prompt in "\\PC{0,1000}") {
        let entry = Entry {
            id: "test".to_string(),
            source: Source::Claude,
            timestamp: Utc::now(),
            prompt: prompt.clone(),
            response: "test".to_string(),
            metadata: Metadata::default(),
        };

        // After serialization roundtrip, prompt should be preserved
        let serialized = to_value(&entry).unwrap();
        let deserialized: Entry = from_value(serialized).unwrap();
        assert_eq!(deserialized.prompt, prompt);
    }
}
