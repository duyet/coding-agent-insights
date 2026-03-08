# cai-ingest

Data ingestion from AI coding platforms.

## Overview

`cai-ingest` provides parsers and scanners for extracting AI coding interactions from various sources including Claude Code conversations, Codex CLI history, and Git commit history.

## Key Features

- **Claude Code parser** - Parse conversation JSON files
- **Codex CLI parser** - Parse Codex command history
- **Git scanner** - Extract commits as entries
- **Unified API** - Single `Ingestor` for all sources
- **Extensible** - Easy to add new parsers

## Usage

### Using Ingestor

```rust
use cai_ingest::{Ingestor, IngestConfig};
use cai_storage::MemoryStorage;

#[tokio::main]
async fn main() -> cai_core::Result<()> {
    let storage = MemoryStorage::new();

    // Default configuration (Claude + Codex)
    let ingestor = Ingestor::with_defaults();
    let count = ingestor.ingest_all(&storage).await?;

    println!("Ingested {} entries", count);
    Ok(())
}
```

### Individual Parsers

```rust
use cai_ingest::{ClaudeParser, CodexParser, GitScanner};

// Parse Claude conversations
let claude_parser = ClaudeParser::with_default_path()?;
let entries = claude_parser.parse_all()?;

// Parse Codex history
let codex_parser = CodexParser::with_default_path()?;
let entries = codex_parser.parse_all()?;

// Scan Git repository
let git_scanner = GitScanner::new("~/my-project");
let entries = git_scanner.scan()?;
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
