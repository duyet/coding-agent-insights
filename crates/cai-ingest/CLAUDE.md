# cai-ingest - Claude Code Instructions

## Crate Purpose

`cai-ingest` extracts AI coding interactions from external sources:

- Parse Claude Code conversation JSON files
- Parse Codex CLI history
- Scan Git repositories for commits
- Unified ingestion API via `Ingestor`

## Architecture

### Module Structure

```
src/
├── lib.rs        - Public exports
├── ingest.rs     - Ingestor, IngestConfig (orchestrator)
├── claude.rs     - ClaudeParser (conversation JSON)
├── codex.rs      - CodexParser (CLI history)
├── git.rs        - GitScanner (commit extraction)
└── error.rs      - IngestError type
```

### Ingestion Flow

```
Ingestor::ingest_all()
    ├─> parse_claude() -> ClaudeParser::parse_all()
    ├─> parse_codex() -> CodexParser::parse_all()
    └─> scan_git() -> GitScanner::scan()
        Each returns Vec<Entry>
        Entries stored via Storage::store()
```

## Common Tasks

### Adding a New Parser

1. Create module file `src/myparser.rs`:
```rust
use crate::error::IngestError;
use cai_core::{Entry, Source};
use std::path::{Path, PathBuf};

pub struct MyParser {
    source_path: PathBuf,
}

impl MyParser {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self { source_path: path.as_ref().to_path_buf() }
    }

    pub fn parse_all(&self) -> Result<Vec<Entry>, IngestError> {
        // 1. Read files from source_path
        // 2. Parse each file
        // 3. Convert to Entry
        // 4. Return Vec<Entry>
    }
}
```

2. Add to `src/lib.rs`:
```rust
mod myparser;
pub use myparser::MyParser;
```

3. Add to `IngestConfig`:
```rust
pub struct IngestConfig {
    // ...
    pub parse_mysource: bool,
    pub mysource_path: Option<PathBuf>,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            // ...
            parse_mysource: false,
            mysource_path: None,
        }
    }
}
```

4. Integrate in `Ingestor::ingest_all()`:
```rust
if self.config.parse_mysource {
    info!("Parsing MySource");
    let parser = MyParser::new(self.config.mysource_path.unwrap());
    let entries = parser.parse_all()
        .map_err(|e| cai_core::Error::Message(e.to_string()))?;
    for entry in entries {
        storage.store(&entry).await?;
        total_count += 1;
    }
}
```

### Error Handling in Parsers

Use `IngestError` for parser-specific errors:

```rust
pub enum IngestError {
    PathNotFound(String),
    PermissionDenied(String),
    InvalidFormat(String),
    GitError(git2::Error),
    IoError(std::io::Error),
    NoFilesFound(String),
}
```

Log errors but continue processing:

```rust
match self.parse_file(&path) {
    Ok(entries) => results.extend(entries),
    Err(e) => {
        tracing::warn!("Failed to parse {}: {}", path.display(), e);
        // Continue with next file
    }
}
```

## Testing Patterns

### Parser Unit Tests

Test parsing with temporary files:

```rust
#[test]
fn test_parse_my_format() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.json");

    // Write test data
    fs::write(&test_file, test_json).unwrap();

    // Parse
    let parser = MyParser::new(temp_dir.path());
    let entries = parser.parse_all().unwrap();

    // Assert
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].source, Source::MySource);
}
```

### Ingestor Integration Tests

Test the full ingestion flow:

```rust
#[tokio::test]
async fn test_ingestor_with_mysource() {
    let config = IngestConfig {
        parse_mysource: true,
        mysource_path: Some(test_data_path()),
        ..Default::default()
    };

    let ingestor = Ingestor::new(config);
    let storage = MemoryStorage::new();

    let count = ingestor.ingest_all(&storage).await.unwrap();
    assert!(count > 0);
}
```

## Parser-Specific Notes

### ClaudeParser

- Handles two JSON formats:
  - Object format: `{ "messages": [...], "metadata": {...} }`
  - Array format: `[{ "role": "user", "content": "..." }]`
- Pairs user messages with assistant responses
- Extracts conversation metadata

### CodexParser

- Parses Codex CLI history file
- Format: timestamp + prompt + response per line
- Handles escaped characters

### GitScanner

- Uses `git2` crate for repository access
- Walks commit graph from HEAD
- Extracts author/committer info
- Returns error for empty repositories

## Dependencies

- `async-trait` - Async support
- `tokio` - Async runtime
- `git2` - Git repository access
- `serde_json` - JSON parsing
- `tracing` - Structured logging
- `dirs` - Platform-specific paths

## Getting Help

- See `README.md` for usage examples
- Check individual parser files for format details
- Review tests for parsing patterns
