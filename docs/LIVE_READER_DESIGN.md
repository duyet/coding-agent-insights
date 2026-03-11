# CAI Direct File Query Design

## Problem Statement

The current "ingest-to-database" approach is over-engineered:
1. **Unnecessary complexity**: Storage layer, ingest step, database sync
2. **Data sync issues**: Database vs actual files
3. **No persistence needed**: We just want to query and analyze

## Solution: Direct File Query

**No storage layer**. Files are the source of truth. SQL queries map directly to file operations.

## Architecture

```
CLI Command
    ↓
cai-query (SQL parser)
    ↓
cai-files (file operations) ← NEW: Maps SQL to file ops
    ↓
File System (Claude Code conversations, etc.)
    ↓
cai-output (format results)
```

## Simplified Design

```
cai-files (new lightweight crate)
├── scanner.rs      # Find conversation files
├── loader.rs       # Load and parse files
└── filter.rs       # In-memory filter (no DB)

cai-query (enhanced)
├── parser.rs       # Parse SQL
├── planner.rs      # Map SQL → file ops (NEW)
└── executor.rs     # Execute against files
```

## Core Components

### 1. FileScanner (cai-files)

```rust
/// Find conversation files on disk
pub struct FileScanner {
    claude_path: PathBuf,  // ~/.claude/conversations
    codex_path: PathBuf,   // ~/.codex/history
}

impl FileScanner {
    /// List all conversation files
    pub fn scan(&self) -> Result<Vec<PathBuf>>;

    /// Find files matching criteria
    pub fn find(&self, filter: &FileFilter) -> Result<Vec<PathBuf>>;
}

pub struct FileFilter {
    pub source: Option<Source>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
}
```

### 2. FileLoader (cai-files)

```rust
/// Load and parse conversation files
pub struct FileLoader;

impl FileLoader {
    /// Load single file, auto-detect format
    pub fn load(&self, path: &Path) -> Result<Conversation>;

    /// Load multiple files in parallel
    pub fn load_many(&self, paths: &[PathBuf]) -> Result<Vec<Conversation>>;

    /// Detect format version from file content
    pub fn detect_format(&self, path: &Path) -> Result<FormatVersion>;
}
```

### 3. QueryPlanner (cai-query)

```rust
/// Map SQL query to file operations
pub struct QueryPlanner;

impl QueryPlanner {
    /// Parse SQL and create file operation plan
    pub fn plan(&self, sql: &str) -> Result<FilePlan>;
}

pub struct FilePlan {
    /// Which files to scan
    pub sources: Vec<Source>,

    /// Filters to apply
    pub filters: Vec<Filter>,

    /// Fields to extract
    pub fields: Vec<Field>,
}
```

### 4. FileExecutor (cai-query)

```rust
/// Execute query against files directly
pub struct FileExecutor {
    scanner: FileScanner,
    loader: FileLoader,
}

impl FileExecutor {
    /// Execute SQL query against files
    pub async fn execute(&self, sql: &str) -> Result<Vec<Entry>> {
        // 1. Parse SQL
        // 2. Plan file operations
        // 3. Scan relevant files
        // 4. Load and filter in-memory
        // 5. Return results
    }
}
```

## Format Version Detection

### Claude Code Conversation Format

Files are stored in `~/.claude/conversations/` as JSON.

### Version Detection Strategy

1. **File header check**: Read first 1KB, check for version field
2. **Schema detection**: Look for known field patterns
3. **Fallback**: Try parsers in order

```rust
pub enum FormatVersion {
    V1,    // Initial format
    V2,    // Current format (with tool_use)
}

pub fn detect_format(content: &str) -> FormatVersion {
    if content.contains("\"tool_use\"") {
        FormatVersion::V2
    } else if content.contains("\"messages\"") {
        FormatVersion::V1
    } else {
        FormatVersion::Unknown
    }
}
```

## Query Execution Flow

```rust
// User runs: cai query "SELECT * FROM entries WHERE source = 'Claude'"

async fn execute_query(sql: &str) -> Result<Vec<Entry>> {
    // 1. Parse SQL
    let ast = parse_sql(sql)?;

    // 2. Plan file operations
    let plan = QueryPlanner::plan(&ast)?;
    // -> sources: [Claude]
    // -> filters: [source = "Claude"]
    // -> fields: [*]

    // 3. Scan files
    let scanner = FileScanner::new();
    let paths = scanner.scan()?
        .into_iter()
        .filter(|p| p.starts_with("~/.claude/conversations"))
        .collect::<Vec<_>>();

    // 4. Load files in parallel
    let loader = FileLoader;
    let conversations = loader.load_many(&paths).await?;

    // 5. Filter in-memory
    let results = conversations
        .into_iter()
        .filter(|c| c.source == Source::Claude)
        .map(|c| c.to_entry())
        .collect();

    Ok(results)
}
```

## CLI Integration

### Simple Usage

```bash
# Query directly, no setup needed
cai query "SELECT * FROM entries"

# Filter by source
cai query "SELECT * FROM entries WHERE source = 'Claude'"

# Date range
cai query "SELECT * FROM entries WHERE date > '2024-01-01'"

# Stats
cai stats
# -> Scans files and reports
```

### No Config Needed

Default paths are hardcoded:
- Claude: `~/.claude/conversations`
- Codex: `~/.codex/history.jsonl`
- Git: `./.git` (current repo)

Optional config for custom paths:
```bash
cai --claude-path /custom/path query "..."
```

## Migration: Remove Storage Layer

### Before (current architecture)
```
CLI -> Query Engine -> Storage Trait -> MemoryStorage/SqliteStorage
```

### After (simplified)
```
CLI -> Query Engine -> File Operations -> Files
```

### Changes Required

1. **Remove** `cai-storage` crate (or simplify to just types)
2. **Create** `cai-files` crate for file operations
3. **Update** `cai-query` to use files instead of storage trait
4. **Update** `cai-cli` to remove ingest command (no longer needed)
5. **Keep** `cai-ingest` only for importing external data formats

## Benefits

1. **Zero setup**: Just run queries, no ingest/config
2. **Always fresh**: Data comes directly from files
3. **Simpler**: No storage abstraction, no database
4. **Transparent**: Users know where data comes from
5. **Debuggable**: Can inspect files directly

## Trade-offs

### Pros
- **Simple**: Fewer moving parts
- **Direct**: No sync issues between storage and files
- **Fast enough**: For typical usage (<1000 files)
- **Obvious**: Data flow is clear

### Cons
- **Slower for large datasets**: Scans files every query
- **No complex queries**: No joins, aggregations across runs
- **No optimization**: Can't build indexes for repeated queries

## Performance Targets

- **Small dataset** (<100 files): < 100ms per query
- **Medium dataset** (100-1000 files): < 500ms per query
- **Large dataset** (1000+ files): < 2s per query

For large datasets, users can export to a proper database if needed.
