# /cai stats

Display summary statistics about your coding history.

## Usage

```
/cai stats [options]
```

## Examples

```bash
# Overall statistics
/cai stats

# By agent
/cai stats --by agent

# Time series
/cai stats --timeline daily

# Top repositories
/cai stats --top repos
```

## Options

| Option | Description |
|--------|-------------|
| `--by` | Group by: agent, repo, language, date |
| `--timeline` | Time granularity: hourly, daily, weekly, monthly |
| `--top` | Show top N items |
| `--output` | Format: table, json |

## Output

```
Total Entries: 1,234
Date Range: 2025-01-01 to 2025-03-08
Unique Agents: 5
Top Repositories:
  - project-a: 456 entries
  - project-b: 234 entries
```

## See Also

- `/cai query` - Custom SQL queries
- `/cai ingest` - Import more data
