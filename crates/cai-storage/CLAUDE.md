# cai-storage - Claude Code Instructions

## Crate Purpose

`cai-storage` provides a flexible storage abstraction layer for CAI entries with multiple backend implementations. It's responsible for:

- Defining the `Storage` trait interface
- Providing in-memory storage implementation
- Supporting query filtering by source, date range
- Enabling pluggable storage backends (SQLite, PostgreSQL, etc.)

## Architecture

### Core Trait

The `Storage` trait is the foundation:

```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn store(&self, entry: &Entry) -> Result<()>;
    async fn get(&self, id: &str) -> Result<Option<Entry>>;
    async fn query(&self, filter: Option<&Filter>) -> Result<Vec<Entry>>;
    async fn count(&self) -> Result<usize>;
}
```

### MemoryStorage

The current implementation uses `Arc<RwLock<Vec<Entry>>>` for thread-safe in-memory storage.

### Filter Structure

Type-safe filtering:

```rust
pub struct Filter {
    pub source: Option<String>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
}
```

## Common Tasks

### Adding a New Storage Backend

1. Implement the `Storage` trait:
```rust
use cai_storage::Storage;
use cai_core::Entry;

pub struct SqliteStorage {
    // your fields
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn store(&self, entry: &Entry) -> Result<()> {
        // implementation
    }
    // ... other methods
}
```

2. Add error handling for backend-specific failures

3. Write comprehensive tests

### Testing Storage Implementations

Use the test patterns in `lib.rs`:

```rust
#[tokio::test]
async fn test_storage_store_and_retrieve() {
    let storage = MemoryStorage::new();
    let entry = Entry { /* ... */ };

    storage.store(&entry).await.unwrap();
    let retrieved = storage.get(&entry.id).await.unwrap();

    assert_eq!(Some(entry), retrieved);
}
```

### Benchmarking Storage

The crate includes benchmarks using `divan`:

```bash
cargo bench -p cai-storage
```

## Testing Patterns

### Unit Tests

Test each storage method:

```rust
#[tokio::test]
async fn test_query_with_filter() {
    let storage = MemoryStorage::new();
    // Add test entries
    // Query with filter
    // Assert results
}
```

### Integration Tests

Test with real storage backends in `tests/` directory.

### Error Cases

Test error handling:

```rust
#[tokio::test]
async fn test_get_nonexistent() {
    let storage = MemoryStorage::new();
    let result = storage.get("nonexistent").await.unwrap();
    assert!(result.is_none());
}
```

## Future Storage Backends

When adding new backends:

1. **SQLite**: Use `rusqlite` (already in dependencies)
   - Add `SqliteStorage` struct
   - Create table schema in constructor
   - Implement CRUD operations

2. **PostgreSQL**: Use `tokio-postgres`
   - Connection pooling
   - Prepared statements
   - Transaction support

3. **File-based**: JSON/JSONL files
   - Streaming reads for large datasets
   - Atomic writes

4. **Remote**: HTTP API client
   - REST/GraphQL endpoints
   - Authentication
   - Retry logic

## Dependencies

- `async-trait` - Async trait support
- `tokio` - Async runtime
- `chrono` - DateTime handling
- `cai-core` - Core types

## Performance Considerations

- MemoryStorage clones entries on query (use SQLite for large datasets)
- Consider using `Arc<Entry>` for shared references
- Filter operations are O(n) - use indexed storage for production
- Benchmarks are in `benches/storage_bench.rs`

## Getting Help

- See `README.md` for usage examples
- Check `lib.rs` for trait definitions
- Review tests for implementation patterns
