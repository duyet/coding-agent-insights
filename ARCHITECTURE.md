# CAI Architecture

This document describes the architecture and design decisions of the Coding Agent Insights (CAI) project.

## Overview

CAI is designed as a modular, extensible system for analyzing AI coding history. The architecture follows these principles:

1. **Separation of Concerns**: Each crate has a single, well-defined responsibility
2. **Extensibility**: Pluggable storage, ingestion, and output systems
3. **Performance**: Efficient data structures and query execution
4. **Type Safety**: Leverage Rust's type system for correctness

## Crate Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        cai-cli                              │
│                   (Command Interface)                       │
└───────────────────────────┬─────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│  cai-ingest  │   │  cai-query   │   │   cai-tui    │
│              │   │              │   │              │
│ - Claude     │   │ - Parser     │   │ - Dashboard  │
│ - Codex      │   │ - Executor   │   │ - Interactive│
│ - Git        │   │ - Functions  │   │              │
└──────┬───────┘   └──────┬───────┘   └──────────────┘
       │                   │
       │                   │
       ▼                   ▼
┌──────────────┐   ┌──────────────┐
│  cai-storage │   │  cai-output  │
│              │   │              │
│ - Memory     │◄──┤ - JSON       │
│ - SQLite     │   │ - JSONL      │
│ - JSON       │   │ - CSV        │
└──────┬───────┘   │ - Table      │
       │           │ - AI         │
       │           └──────────────┘
       │
       ▼
┌──────────────┐
│   cai-core   │
│              │
│ - Entry      │
│ - Source     │
│ - Metadata   │
│ - Error      │
└──────────────┘
```

## Core Crates

### cai-core

**Purpose**: Shared types, traits, and utilities

**Responsibilities**:
- Define core data structures (`Entry`, `Source`, `Metadata`)
- Provide error types and result aliases
- Shared utilities used across crates

**Key Types**:
```rust
pub struct Entry {
    pub id: String,
    pub source: Source,
    pub timestamp: DateTime<Utc>,
    pub prompt: String,
    pub response: String,
    pub metadata: Metadata,
}

pub enum Source {
    Claude,
    Codex,
    Git,
    Other(String),
}
```

**Design Decisions**:
- Uses `HashMap` for metadata extensibility
- `Source` is `#[non_exhaustive]` for future expansion
- `DateTime<Utc>` for consistent timezone handling

### cai-storage

**Purpose**: Pluggable storage backends

**Responsibilities**:
- Define storage interface (`Storage` trait)
- Provide implementations (MemoryStorage, SQLiteStorage)
- Handle query filtering

**Key Trait**:
```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn store(&self, entry: &Entry) -> Result<()>;
    async fn get(&self, id: &str) -> Result<Option<Entry>>;
    async fn query(&self, filter: Option<&Filter>) -> Result<Vec<Entry>>;
    async fn count(&self) -> Result<usize>;
}
```

**Design Decisions**:
- Async trait for non-blocking operations
- Filter struct for type-safe queries
- Generic over storage backend

### cai-ingest

**Purpose**: Data ingestion from various sources

**Responsibilities**:
- Parse Claude Code conversations
- Parse Codex CLI history
- Scan Git repositories
- Convert to `Entry` format

**Design Decisions**:
- Separate parsers for each source
- Streaming ingestion for large datasets
- Error recovery for malformed data

### cai-query

**Purpose**: SQL-like query language and execution

**Responsibilities**:
- Parse SQL queries
- Execute queries against storage
- Provide built-in functions (aggregates, date functions)

**Query Language**:
```sql
SELECT source, COUNT(*) as count
FROM entries
WHERE timestamp > '2024-01-01'
GROUP BY source
ORDER BY count DESC
LIMIT 10
```

**Design Decisions**:
- Uses `sqlparser-rust` for compatibility
- Custom executor for CAI-specific features
- Extensible function system

### cai-output

**Purpose**: Output formatting for multiple formats

**Responsibilities**:
- Define formatter interface
- Implement formatters (JSON, JSONL, CSV, Table, AI)
- Handle output configuration

**Key Trait**:
```rust
pub trait Formatter: Send + Sync {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()>;
    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()>;
}
```

**Design Decisions**:
- Generic over `Write` for flexibility
- Streaming support via `format_one`
- Configurable truncation and limits

### cai-tui

**Purpose**: Interactive terminal UI

**Responsibilities**:
- Display entries in table format
- Handle keyboard input
- Support filtering and search

**Design Decisions**:
- Uses `ratatui` for cross-platform TUI
- Event-driven architecture
- Async data updates

### cai-web

**Purpose**: Web dashboard interface

**Responsibilities**:
- HTTP server with WebSocket support
- REST API for queries
- Real-time updates

**Design Decisions**:
- Uses `Axum` for async HTTP
- WebSocket for live updates
- Stateless design for horizontal scaling

### cai-cli

**Purpose**: Command-line interface

**Responsibilities**:
- Parse command-line arguments
- Route to appropriate subsystem
- Handle user errors gracefully

**Design Decisions**:
- Uses `clap` for argument parsing
- Subcommands for each feature
- Colored output for better UX

### cai-plugin

**Purpose**: Claude Code plugin integration

**Responsibilities**:
- WASM plugin for Claude Code
- NPM package for distribution
- Skill definitions

**Design Decisions**:
- WASM for portability
- Component-based design
- Minimal dependencies

## Data Flow

### Ingestion Flow

```
Source File → Parser → Entry → Storage
                        ↓
                     Validation
                        ↓
                     Indexing
```

### Query Flow

```
SQL Query → Parser → AST → Executor → Storage → Results → Formatter → Output
```

### TUI Flow

```
User Input → Event Handler → State Update → Render → Display
     ↑                                                    ↓
     └──────────────── Key Processing ←──────────────────┘
```

## Error Handling

CAI uses a layered error handling strategy:

1. **cai-core**: Base `Error` enum for fundamental errors
2. **Each crate**: Crate-specific error types
3. **Propagation**: Use `?` operator for clean error propagation
4. **User-facing**: Clear, actionable error messages

Example:
```rust
// cai-storage
pub enum StorageError {
    NotFound(String),
    Io(std::io::Error),
    Database(String),
}

// cai-query
pub enum QueryError {
    InvalidSql(String),
    StorageError(StorageError),
    ExecutionError(String),
}
```

## Testing Strategy

### Unit Tests
- Test each crate in isolation
- Use fixtures for test data
- Property-based testing with proptest

### Integration Tests
- Test cross-crate workflows
- Mock external dependencies
- Test error conditions

### E2E Tests
- Test complete CLI workflows
- Validate output formats
- Performance benchmarks

Target: 80%+ code coverage

## Performance Considerations

1. **Storage**: Use indexes for common queries
2. **Query**: Lazy evaluation where possible
3. **Output**: Stream large datasets
4. **Memory**: Reuse allocations, avoid cloning

## Future Extensibility

### Adding New Data Sources

1. Create parser in `cai-ingest`
2. Add `Source` variant to `cai-core`
3. Implement ingestion logic
4. Add tests

### Adding New Output Formats

1. Implement `Formatter` trait
2. Add to `cai-output`
3. Register with CLI
4. Add tests

### Adding New Query Functions

1. Define function in `cai-query`
2. Implement executor logic
3. Add to function registry
4. Document and test

## Dependencies

### External Dependencies

- **tokio**: Async runtime
- **serde/serde_json**: Serialization
- **rusqlite**: SQLite bindings
- **sqlparser**: SQL parsing
- **clap**: CLI argument parsing
- **ratatui**: Terminal UI
- **axum**: Web framework

### Internal Dependencies

```
cai-core ← All crates depend on this
cai-storage ← cai-query, cai-ingest depend on this
cai-output ← cai-cli, cai-tui depend on this
```

## Security Considerations

1. **Input Validation**: Validate all user inputs
2. **SQL Injection**: Use parameterized queries
3. **Path Traversal**: Validate file paths
4. **Secrets**: Never log sensitive data

## Release Strategy

1. **Semantic Versioning**: Follow SemVer for public API
2. **MSRV**: Pin minimum supported Rust version
3. **Backward Compatibility**: Don't break public APIs
4. **Deprecation**: Mark deprecated features clearly
