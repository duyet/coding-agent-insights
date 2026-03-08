# cai-storage

Pluggable storage backends for CAI entries.

## Overview

`cai-storage` provides a flexible storage abstraction with multiple backend implementations. Use it to store and retrieve AI coding interactions.

## Storage Trait

The core `Storage` trait defines the interface:

```rust
use cai_storage::{Storage, MemoryStorage};
use cai_core::Entry;

#[tokio::main]
async fn main() -> cai_core::Result<()> {
    let storage = MemoryStorage::new();

    // Store an entry
    storage.store(&entry).await?;

    // Retrieve by ID
    let entry = storage.get("entry-id").await?;

    // Query with filters
    let results = storage.query(None).await?;

    // Count entries
    let count = storage.count().await?;

    Ok(())
}
```

## Implementations

### MemoryStorage

In-memory storage for testing and temporary data:

```rust
use cai_storage::MemoryStorage;

let storage = MemoryStorage::new();
```

### Filter

Query entries with filters:

```rust
use cai_storage::Filter;
use chrono::{Utc, DateTime};

let filter = Filter {
    source: Some("Claude".to_string()),
    after: Some(DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")?.into()),
    before: None,
};

let results = storage.query(Some(&filter)).await?;
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cai-storage = { path = "../cai-storage" }
```

## Design Decisions

- **Async trait**: Non-blocking operations for better performance
- **Generic filters**: Type-safe query filtering
- **Arc<RwLock>**: Thread-safe interior mutability

## Testing

```bash
cargo test -p cai-storage
```

## License

MIT OR Apache-2.0
