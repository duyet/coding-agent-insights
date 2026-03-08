# /cai query

Query your AI coding history using SQL-like syntax.

## Usage

```
/cai query "<sql>"
```

## Examples

```sql
-- Recent activity
/cai query "SELECT * FROM entries ORDER BY timestamp DESC LIMIT 20"

-- Filter by agent
/cai query "SELECT agent, COUNT(*) FROM entries GROUP BY agent"

-- Time range
/cai query "SELECT * FROM entries WHERE timestamp > '2025-01-01'"

-- Productivity analysis
/cai query "SELECT DATE(timestamp) as day, COUNT(*) as commits FROM entries GROUP BY day"
```

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `--output, -o` | Format: json, jsonl, csv, table, ai | table |
| `--save` | Save results to file | - |
| `--storage` | Storage backend: memory, sqlite, json | sqlite |

## Output Formats

- `table` - Pretty terminal table (default)
- `json` - Structured JSON array
- `jsonl` - Streaming JSON Lines
- `csv` - Comma-separated values
- `ai` - Compact AI-optimized format

## See Also

- `/cai ingest` - Import coding history
- `/cai stats` - Summary statistics
