# CAI - Coding Agent Insights

> A superior Rust-based tool for analyzing AI coding history with SQL-like queries, multiple output formats, and agent-optimized workflows.

[![CI](https://github.com/cai-dev/coding-agent-insights/workflows/CI/badge.svg)](https://github.com/cai-dev/coding-agent-insights/actions)
[![codecov](https://codecov.io/gh/cai-dev/coding-agent-insights/branch/main/graph/badge.svg)](https://codecov.io/gh/cai-dev/coding-agent-insights)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Overview

CAI (Coding Agent Insights) is a fast, lightweight tool for querying and analyzing your AI coding history across multiple platforms. Think of it as "supercharged devsql" - designed specifically for AI-agent workflows.

### Key Features

- **Multi-Source Support**: Claude Code, Codex CLI, Git repositories, and more
- **SQL-like Queries**: Powerful query language with aggregates, filtering, and date functions
- **Multiple Output Formats**: JSON, JSONL, CSV, tables, TUI, web dashboard, AI-optimized
- **Blazing Fast**: Written in Rust with efficient storage backends
- **Agent-Optimized**: Compact outputs designed for LLM consumption
- **Extensible**: Plugin system for adding custom data sources

## Quick Start

### Installation

```bash
# Via Cargo
cargo install cai-cli

# Via Homebrew (coming soon)
brew install cai-cli

# Build from source
git clone https://github.com/cai-dev/coding-agent-insights.git
cd coding-agent-insights
cargo build --release
```

### Basic Usage

```bash
# Ingest data from Claude Code
cai ingest --source claude --path ~/.claude/conversations

# Query your coding history
cai query "SELECT * FROM entries LIMIT 10"

# Filter by date and source
cai query "SELECT * FROM entries WHERE source = 'Claude' AND timestamp > '2024-01-01'"

# Output formats
cai query --output json "SELECT COUNT(*) FROM entries GROUP BY source"
cai query --output csv "SELECT * FROM entries LIMIT 100"

# Interactive terminal UI
cai tui

# Web dashboard
cai web --port 3000
```

## Query Language

CAI supports SQL-like queries optimized for coding history:

```sql
-- Basic queries
SELECT * FROM entries LIMIT 10

-- Filtering
SELECT * FROM entries WHERE source = 'Claude' AND timestamp > '2024-01-01'

-- Aggregation
SELECT source, COUNT(*) as count FROM entries GROUP BY source

-- Date ranges
SELECT * FROM entries WHERE date_range(timestamp, '2024-01-01', '2024-01-31')

-- Time buckets
SELECT time_bucket(timestamp, 'day') as day, COUNT(*)
FROM entries
GROUP BY day
ORDER BY day DESC
```

## Architecture

CAI is organized as a Cargo workspace with modular crates:

```
cai-workspace/
├── cai-core       # Shared types and utilities
├── cai-ingest     # Data ingestion from sources
├── cai-storage    # Pluggable storage backends
├── cai-query      # SQL parser and engine
├── cai-output     # Output formatters
├── cai-tui        # Terminal UI
├── cai-web        # Web dashboard
├── cai-cli        # Command-line interface
└── cai-plugin     # Claude Code plugin
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed design documentation.

## Development

### Prerequisites

- Rust 1.80 or later
- Git

### Building

```bash
# Build all crates
cargo build --workspace

# Build release version
cargo build --workspace --release

# Run tests
cargo test --workspace

# Run with coverage
cargo llvm-cov --workspace
```

### Testing

```bash
# All tests
make test

# Unit tests only
make unit

# Integration tests
make integration

# Coverage report
make coverage
```

See [TESTING.md](TESTING.md) for detailed testing information.

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

### v0.2.0 (Next)
- [ ] Complete parser implementations
- [ ] TUI with real-time updates
- [ ] Web dashboard
- [ ] Claude Code plugin
- [ ] NPM package

### v0.3.0 (Future)
- [ ] Additional data sources (Cursor, Windsurf, Copilot)
- [ ] Query optimization
- [ ] Performance benchmarks
- [ ] Homebrew formula

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

- GitHub: [cai-dev/coding-agent-insights](https://github.com/cai-dev/coding-agent-insights)
- Issues: [GitHub Issues](https://github.com/cai-dev/coding-agent-insights/issues)
- Discussions: [GitHub Discussions](https://github.com/cai-dev/coding-agent-insights/discussions)
