# cai-files - Claude Code Instructions

## Crate Purpose

`cai-files` provides direct file operations for CAI. No storage layer, no database - just read files and query them.

## Architecture

```
FileScanner → finds conversation files on disk
    ↓
FileLoader → loads and parses files
    ↓
FileFilterOps → filters results in-memory
```

## Core Types

### FileScanner

Find conversation files:

```rust
let scanner = FileScanner::new();
let all_files = scanner.scan()?;

// Filter by source
let filter = ScanFilter {
    source: Some(Source::Claude),
    ..Default::default()
};
let claude_files = scanner.find(&filter)?;
```

### FileLoader

Load and parse files:

```rust
let loader = FileLoader;

// Single file
let entries = loader.load(path)?;

// Multiple files in parallel
let entries = loader.load_many(&paths).await?;
```

### FormatVersion

Auto-detect conversation format:

- `V1` - Initial Claude Code format
- `V2` - Current format with tool_use
- `Unknown` - Unrecognized format

### FileFilterOps

Filter loaded entries:

```rust
let filtered = FileFilterOps::by_source(entries, &["Claude"]);
let filtered = FileFilterOps::by_date_range(entries, after, before);
let filtered = FileFilterOps::by_text(entries, "rust");
```

## File Paths

Default paths (configurable):

- **Claude**: `~/.claude/conversations/*.json`
- **Codex**: `~/.codex/history.jsonl`

## Testing

```bash
# Run tests
cargo test -p cai-files

# Run specific test
cargo test -p cai-files test_detect_format_v2
```

## Adding New Formats

To support a new conversation format:

1. Add variant to `FormatVersion` enum
2. Update `detect_format()` to recognize the format
3. Add `load_v3()` (or similar) method in `FileLoader`
4. Update `load()` to call the new loader

## Performance Notes

- Scanning is fast (just file glob)
- Parsing is synchronous (JSON parsing)
- `load_many()` processes files in parallel (up to 10 concurrent)
- No caching - every call loads fresh data
