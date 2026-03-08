# cai-query

SQL-like query engine for CAI entries.

## Overview

`cai-query` provides a SQL parser and execution engine for querying AI coding interactions stored via `cai-storage`. Supports SELECT statements with WHERE, ORDER BY, LIMIT, and custom aggregate functions.

## Usage

### Basic Query

```rust
use cai_query::{QueryEngine, sql};
use cai_storage::MemoryStorage;

#[tokio::main]
async fn main() -> cai_core::Result<()> {
    let storage = MemoryStorage::new();
    let engine = QueryEngine::new(storage);

    // Execute a query
    let results = engine.execute(
        "SELECT * FROM entries LIMIT 10"
    ).await?;

    Ok(())
}
```

### Convenience Function

```rust
use cai_query::sql;

let results = sql(
    "SELECT * FROM entries WHERE source = 'Claude'",
    &storage
).await?;
```

## Query Language

### SELECT

```sql
SELECT * FROM entries LIMIT 10
SELECT source, COUNT(*) FROM entries GROUP BY source
```

### WHERE

```sql
SELECT * FROM entries WHERE source = 'Claude'
SELECT * FROM entries WHERE timestamp > '2024-01-01'
```

### Aggregate Functions

```sql
SELECT COUNT(*) FROM entries
SELECT source, COUNT(*), AVG(length(prompt)) FROM entries GROUP BY source
```

### Date Functions

```sql
-- Date range filter
SELECT * FROM entries
WHERE date_range(timestamp, '2024-01-01', '2024-01-31')

-- Time bucket grouping
SELECT time_bucket(timestamp, 'day') as day, COUNT(*)
FROM entries
GROUP BY day
```

## Built-in Functions

- **COUNT**: Count entries
- **SUM**: Sum numeric values
- **AVG**: Average of values
- **MIN/MAX**: Minimum/Maximum values
- **date_range**: Filter by date range
- **time_bucket**: Group by time periods

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cai-query = { path = "../cai-query" }
```

## Design Decisions

- **sqlparser-rust**: Compatible SQL syntax
- **Custom executor**: Optimized for CAI's data model
- **Extensible functions**: Easy to add custom functions

## Testing

```bash
cargo test -p cai-query
```

## License

MIT OR Apache-2.0
