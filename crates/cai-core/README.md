# cai-core

Core types, traits, and utilities for the Coding Agent Insights (CAI) project.

## Overview

`cai-core` provides the foundational data structures and shared utilities used across all CAI crates. It defines the core domain model and error handling.

## Key Types

### Entry

Represents a single AI coding interaction:

```rust
use cai_core::{Entry, Source, Metadata};
use chrono::Utc;

let entry = Entry {
    id: "unique-id".to_string(),
    source: Source::Claude,
    timestamp: Utc::now(),
    prompt: "Write a function".to_string(),
    response: "fn test() {}".to_string(),
    metadata: Metadata::default(),
};
```

### Source

Represents the origin system of an entry:

```rust
use cai_core::Source;

let source = Source::Claude;
let custom = Source::Other("custom-ai".to_string());
```

### Metadata

Extensible metadata for entries:

```rust
use cai_core::Metadata;
use std::collections::HashMap;

let mut extra = HashMap::new();
extra.insert("complexity".to_string(), "high".to_string());

let metadata = Metadata {
    file_path: Some("src/main.rs".to_string()),
    repo_url: Some("https://github.com/user/repo".to_string()),
    commit_hash: Some("abc123".to_string()),
    language: Some("Rust".to_string()),
    extra,
};
```

### Error Handling

```rust
use cai_core::{Error, Result};

fn do_something() -> Result<()> {
    Err(Error::Message("Something went wrong".to_string()))
}
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cai-core = { path = "../cai-core" }
```

## Design Decisions

- **HashMap for metadata**: Allows flexible, extensible metadata without changing the core structure
- **DateTime<Utc>**: Consistent timezone handling across the system
- **Non-exhaustive enums**: Future-proofing for adding new sources

## Testing

```bash
cargo test -p cai-core
```

## License

MIT OR Apache-2.0
