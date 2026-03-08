# cai-ingest

Data ingestion from various AI coding sources.

## Overview

`cai-ingest` provides parsers and scanners for extracting AI coding interactions from different platforms and formats.

## Supported Sources

- **Claude Code**: Parse conversation JSON files
- **Codex CLI**: Parse history JSONL files
- **Git**: Scan repository commits

## Usage

### Claude Code Parser

```rust
use cai_ingest::claude::ClaudeParser;

let parser = ClaudeParser::new();
let entries = parser.parse_file("conversation.json")?;
```

### Codex CLI Parser

```rust
use cai_ingest::codex::CodexParser;

let parser = CodexParser::new();
let entries = parser.parse_file("history.jsonl")?;
```

### Git Scanner

```rust
use cai_ingest::git::GitScanner;

let scanner = GitScanner::new();
let entries = scanner.scan("/path/to/repo")?;
```

## Data Format

Each parser produces `cai_core::Entry` instances:

```rust
use cai_core::{Entry, Source, Metadata};
use chrono::Utc;

Entry {
    id: "unique-id".to_string(),
    source: Source::Claude,
    timestamp: Utc::now(),
    prompt: "Original prompt".to_string(),
    response: "AI response".to_string(),
    metadata: Metadata {
        file_path: Some("src/main.rs".to_string()),
        repo_url: Some("https://github.com/user/repo".to_string()),
        commit_hash: Some("abc123".to_string()),
        language: Some("Rust".to_string()),
        extra: std::collections::HashMap::new(),
    },
}
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cai-ingest = { path = "../cai-ingest" }
```

## Design Decisions

- **Streaming**: Support large files without loading entirely into memory
- **Error recovery**: Continue parsing on malformed entries
- **Extensible**: Easy to add new source parsers

## Testing

```bash
cargo test -p cai-ingest
```

## License

MIT OR Apache-2.0
