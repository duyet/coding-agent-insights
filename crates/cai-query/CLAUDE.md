# cai-query - Claude Code Instructions

## Crate Purpose

`cai-query` provides SQL query capabilities for CAI entries:

- Parse SQL SELECT statements
- Execute queries against Storage backends
- Support filtering, sorting, limiting
- Custom aggregate and date functions

## Architecture

### Module Structure

```
src/
├── lib.rs        - Public API, exports
├── parser.rs     - SQL parsing (ParsedQuery struct)
├── executor.rs   - QueryEngine, executes queries
├── functions.rs  - Custom functions (date_range, time_bucket)
├── eval.rs       - Expression evaluation
└── error.rs      - QueryError, QueryResult types
```

### Query Flow

1. **Parse** - SQL string → `ParsedQuery`
2. **Validate** - Table name, columns
3. **Execute** - Fetch entries, apply filters
4. **Return** - Vector of Entry objects

### ParsedQuery Structure

```rust
pub struct ParsedQuery {
    pub select_wildcard: bool,
    pub columns: Vec<String>,
    pub table: Option<String>,
    pub where_sql: Option<String>,
    pub group_by: Vec<String>,
    pub order_by: Vec<(String, bool)>,
    pub limit: Option<usize>,
    pub has_aggregates: bool,
}
```

## Common Tasks

### Adding a New SQL Function

1. Add function signature in `functions.rs`:
```rust
pub fn my_function(args: &[Expr]) -> QueryResult<Value> {
    // implementation
}
```

2. Register in executor (when calling functions)

3. Add tests in `functions.rs`

### Extending WHERE Clause Support

Edit `executor.rs::apply_where_filter()`:

```rust
fn apply_where_filter(&self, entries: Vec<Entry>, where_sql: &str) -> QueryResult<Vec<Entry>> {
    // Add your condition parsing logic
    if where_sql.contains("prompt LIKE") {
        // extract and apply LIKE filter
    }
    // ... existing conditions
}
```

### Adding ORDER BY Parsing

Update `parser.rs::parse()` to extract ORDER BY clause:

```rust
// Check for ORDER BY
let order_by = if sql_upper.contains("ORDER BY") {
    let order_idx = sql_upper.find("ORDER BY ").unwrap() + 9;
    // Parse column and direction
    vec![(column_name, is_asc)]
} else {
    vec![]
};
```

## Testing

### Parser Tests

Test SQL parsing in `parser.rs`:

```rust
#[test]
fn test_parse_with_limit() {
    let result = parse("SELECT * FROM entries LIMIT 10");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().limit, Some(10));
}
```

### Executor Tests

Test query execution in `executor.rs`:

```rust
#[tokio::test]
async fn test_select_with_where() {
    let storage = setup_test_storage();
    let engine = QueryEngine::new(storage);
    let results = engine.execute("SELECT * FROM entries WHERE source = 'Claude'").await.unwrap();
    assert_eq!(results.len(), 1);
}
```

## Current Limitations

1. **Single table** - Only `entries` table supported
2. **Basic WHERE** - Simple equality and comparison only
3. **No JOIN** - Single table queries only
4. **No GROUP BY** - Parsing present but execution incomplete
5. **No subqueries** - Flat queries only
6. **Case-sensitive** - Source matching is case-sensitive

## Future Enhancements

- **Full sqlparser-rust integration** - Replace manual parsing
- **Aggregate execution** - Implement COUNT, SUM, AVG
- **Expression evaluation** - Full arithmetic and logical expressions
- **Column selection** - Return only requested columns
- **Prepared statements** - Query planning and caching

## Dependencies

- `sqlparser` - SQL query parsing (currently manual)
- `cai-storage` - Storage trait for queries
- `cai-core` - Entry types
- `chrono` - DateTime handling
- `tracing` - Debug logging

## Performance Notes

- Current implementation loads all entries then filters
- For large datasets, use SQLite backend with native SQL
- Future: push filters down to storage layer

## Getting Help

- See `README.md` for query syntax
- Check `parser.rs` tests for parsing examples
- Review `executor.rs` for query execution patterns
