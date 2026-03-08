# /cai ingest

Import AI coding history from various sources.

## Usage

```
/cai ingest <source> [options]
```

## Sources

```bash
# Claude Code conversations
/cai ingest claude --path ~/.claude/conversations

# Codex CLI history
/cai ingest codex --path ~/.codex/history.jsonl

# Git repository
/cai ingest git --path /path/to/repo --since "2025-01-01"

# All sources (auto-discover)
/cai ingest all
```

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `--path` | Path to data source | auto |
| `--since` | Start date for import | all |
| `--until` | End date for import | now |
| `--storage` | Storage backend | sqlite |
| `--merge` | Merge with existing data | replace |

## Examples

```bash
# Import recent Claude conversations
/cai ingest claude --since "2025-03-01"

# Import specific Git repo
/cai ingest git --path ./my-project --since "2025-01-01"

# Import everything and merge
/cai ingest all --merge
```

## See Also

- `/cai query` - Query imported data
- `/cai stats` - View statistics
