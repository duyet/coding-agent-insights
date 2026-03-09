# CAI - Coding Agent Insights

> Query and analyze your AI coding history with SQL-like queries

[![CI](https://github.com/duyet/coding-agent-insights/workflows/CI/badge.svg)](https://github.com/duyet/coding-agent-insights/actions)
[![codecov](https://codecov.io/gh/duyet/coding-agent-insights/branch/main/graph/badge.svg)](https://codecov.io/gh/duyet/coding-agent-insights)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/cai-cli.svg)](https://crates.io/cai-cli)
[![npm](https://img.shields.io/npm/v/@cai/cli.svg)](https://npmjs.com/package/@cai/cli)

## Overview

CAI (Coding Agent Insights) is a fast, lightweight tool for querying and analyzing your AI coding history across multiple platforms. Think of it as "supercharged git history" - designed specifically for AI-agent workflows.

### What Problems Does CAI Solve?

- **Discover Patterns**: Understand how you use AI coding assistants over time
- **Search Context**: Find similar problems you've solved with AI help
- **Measure Impact**: Track which AI-assisted sessions produced the most code
- **Optimize Workflows**: Identify your most productive AI pairing patterns

### Key Features

- **Multi-Source Support**: Claude Code, Codex CLI, Git repositories
- **SQL-like Queries**: Powerful query language with aggregates, filtering, and date functions
- **Multiple Output Formats**: JSON, JSONL, CSV, tables, TUI, web dashboard, AI-optimized
- **Blazing Fast**: Written in Rust with efficient storage backends
- **Agent-Optimized**: Compact outputs designed for LLM consumption
- **Extensible**: Plugin system for adding custom data sources

## Installation

### NPM / Bun (Recommended)

```bash
# Using npm
npm install -g @cai/cli

# Using bun (faster)
bun install -g @cai/cli

# Run directly without installing
npx @cai/cli --help
bunx @cai/cli --help
```

### Homebrew

```bash
brew install duyet/tap/cai
```

### Cargo

```bash
cargo install cai-cli
```

### Binaries

Download pre-built binaries from the [releases page](https://github.com/duyet/coding-agent-insights/releases).

## Quick Start

### 1. Create Sample Data

First, let's create some sample data to explore:

```bash
# Create a sample directory
mkdir -p ~/.cai-demo

# Use included sample data
cai ingest --source claude --path /path/to/coding-agent-insights/tests/fixtures
```

### 2. Query Your Data

```bash
# View recent entries
cai query "SELECT * FROM entries LIMIT 10"

# Count entries by source
cai query "SELECT source, COUNT(*) as count FROM entries GROUP BY source"

# Find Claude conversations from a specific date
cai query "SELECT * FROM entries WHERE source = 'Claude' AND timestamp > '2024-01-01'"

# Daily activity patterns
cai query "SELECT time_bucket(timestamp, 'day') as day, COUNT(*) FROM entries GROUP BY day ORDER BY day DESC"
```

### 3. View Statistics

```bash
cai stats
```

**Output:**
```
CAI Statistics
================================================================================
Total Entries:        1,247
Sources:              3 (Claude, Codex, Git)
Date Range:           2024-01-01 to 2024-03-09
Storage Backend:      Memory

Top Sources:
  Claude:             892 entries (71.5%)
  Git:                301 entries (24.1%)
  Codex:              54 entries (4.3%)

Activity (Last 7 Days):
  2024-03-09:         42 entries
  2024-03-08:         38 entries
  2024-03-07:         15 entries
  ...
```

### 4. Interactive TUI

```bash
cai tui
```

Launch an interactive terminal UI for browsing and filtering your coding history.

### 5. Web Dashboard

```bash
cai web --port 3000
```

Open http://localhost:3000 for a visual dashboard.

## Query Examples

### Basic Queries

```sql
-- Show all entries
SELECT * FROM entries LIMIT 10

-- Count total entries
SELECT COUNT(*) FROM entries

-- Get entries from a specific source
SELECT * FROM entries WHERE source = 'Claude'

-- Filter by date range
SELECT * FROM entries WHERE timestamp > '2024-01-01'
```

### Advanced Queries

```sql
-- Daily activity over time
SELECT
  time_bucket(timestamp, 'day') as day,
  COUNT(*) as entries
FROM entries
GROUP BY day
ORDER BY day DESC

-- Filter by date range
SELECT * FROM entries WHERE date_range(timestamp, '2024-01-01', '2024-01-31')

-- Find longest prompts
SELECT prompt FROM entries ORDER BY LENGTH(prompt) DESC LIMIT 10
```

### Output Formats

```bash
# JSON output
cai query --output json "SELECT * FROM entries LIMIT 5"

# CSV for spreadsheets
cai query --output csv "SELECT source, COUNT(*) FROM entries GROUP BY source"

# AI-optimized format
cai query --output ai "SELECT * FROM entries WHERE prompt LIKE '%rust%'"

# Pretty table (default)
cai query "SELECT * FROM entries LIMIT 10"
```

## Features

### Query Engine

CAI supports SQL-like queries optimized for coding history:

| Feature | Example |
|---------|---------|
| Wildcard selection | `SELECT * FROM entries` |
| Filtering | `WHERE source = 'Claude'` |
| Comparison | `WHERE timestamp > '2024-01-01'` |
| Aggregation | `COUNT(*)`, `SUM(field)` |
| Grouping | `GROUP BY source` |
| Time buckets | `time_bucket(timestamp, 'day')` |
| Date ranges | `date_range(timestamp, '2024-01-01', '2024-12-31')` |
| Limiting | `LIMIT 100` |
| Ordering | `ORDER BY timestamp DESC` |

### Data Sources

#### Claude Code
```bash
cai ingest --source claude --path ~/.claude/conversations
```

#### Codex CLI
```bash
cai ingest --source codex --path ~/.codex/history.jsonl
```

#### Git Repository
```bash
cai ingest --source git --path /path/to/repo
```

### Schema Command

View the data schema:

```bash
cai schema
```

**Output:**
```
CAI Schema
================================================================================
Table: entries

Columns:
  id         TEXT      Unique identifier
  prompt     TEXT      User prompt/request
  response   TEXT      AI response
  timestamp  DATETIME  Entry timestamp
  source     TEXT      Data source (Claude, Codex, Git)
  metadata   JSON      Additional metadata

Indexes:
  - source
  - timestamp

Custom Functions:
  - time_bucket(timestamp, unit)  - Group by time period (hour, day, week, month)
  - date_range(timestamp, start, end)  - Filter by date range
```

## Architecture

CAI is organized as a Cargo workspace with modular crates:

```
cai-workspace/
├── cai-core       # Shared types and utilities
├── cai-ingest     # Data ingestion from sources
├── cai-storage    # Pluggable storage backends (Memory, SQLite)
├── cai-query      # SQL parser and engine
├── cai-output     # Output formatters (JSON, CSV, Table, AI)
├── cai-tui        # Terminal UI with ratatui
├── cai-web        # Web dashboard with Axum
├── cai-cli        # Command-line interface
└── cai-plugin     # Claude Code WASM plugin
```

### Storage Backends

- **Memory**: Fast, in-memory storage (default)
- **SQLite**: Persistent, queryable storage (coming soon)

### CLI Commands

| Command | Description |
|---------|-------------|
| `query` | Execute SQL query against entries |
| `ingest` | Import data from sources |
| `stats` | Display statistics and insights |
| `schema` | Show database schema |
| `tui` | Interactive terminal UI |
| `web` | Web dashboard |

## Development

### Prerequisites

- Rust 1.80 or later
- Git

### Building

```bash
# Clone the repository
git clone https://github.com/duyet/coding-agent-insights.git
cd coding-agent-insights

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run with coverage
cargo llvm-cov --workspace
```

### Using Make

```bash
make test          # Run all tests
make unit          # Unit tests only
make integration   # Integration tests
make coverage      # Coverage report
make lint          # Run clippy
make fmt           # Format code
make fmt-check     # Check formatting
```

### Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Roadmap

### v0.1.0 (Current)
- ✅ Core data models
- ✅ Storage layer (memory, SQLite)
- ✅ Basic CLI
- ✅ Query engine
- ✅ Output formatters
- ✅ Testing infrastructure

### v0.2.0 (In Progress)
- [ ] Chart visualization in CLI
- [ ] Additional data sources (Cursor, Windsurf, Copilot)
- [ ] Performance benchmarks
- [ ] Query optimization

### v0.3.0 (Future)
- [ ] Additional data sources (Cursor, Windsurf, Copilot)
- [ ] Query optimization
- [ ] Performance benchmarks
- [ ] Visual insights and charts

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Inspired by [devsql](https://github.com/anthropics/devsql) but built from scratch to be:
- Faster and more memory-efficient
- More extensible with pluggable backends
- Optimized for AI-agent workflows
- Written in idiomatic Rust

## Contact

- GitHub: [duyet/coding-agent-insights](https://github.com/duyet/coding-agent-insights)
- Issues: [GitHub Issues](https://github.com/duyet/coding-agent-insights/issues)
- Discussions: [GitHub Discussions](https://github.com/duyet/coding-agent-insights/discussions)
