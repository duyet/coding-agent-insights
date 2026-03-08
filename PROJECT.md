# Coding Agent Insights - Better DevSQL

## Project Overview
Build a superior Rust-based tool for analyzing AI coding history with:
- Multi-format outputs (json, jsonl, csv, TUI, Web, AI-optimized stdout)
- Claude Code Plugin + Skill installation via npx/bunx
- SQL-like query language with enhanced features
- Comprehensive testing (unit, integration, e2e)
- Lightweight, fast, agent-optimized

## Architecture

### Core Crates
1. **cai-core** - Shared types, traits, utilities
2. **cai-ingest** - Data ingestion from Claude, Codex, Git
3. **cai-query** - SQL parser and query engine
4. **cai-storage** - Pluggable storage backends (SQLite, memory, JSON)
5. **cai-output** - Output formatters for all formats
6. **cai-tui** - Terminal UI
7. **cai-web** - Local web interface
8. **cai-cli** - Main CLI tool
9. **cai-plugin** - Claude Code plugin

### Data Sources
- Claude Code: ~/.claude/conversations/
- Codex CLI: ~/.codex/history.jsonl
- Git: Local repositories
- Future: Cursor, Windsurf, Copilot

### Output Formats
- json - Structured data
- jsonl - Streaming JSON Lines
- csv - Spreadsheet compatible
- table - Pretty terminal tables
- tui - Interactive terminal UI
- web - Local web dashboard (localhost)
- ai - Compact AI-optimized format
- stats - Summary statistics

## Implementation Phases

### Phase 1: Foundation (PR #1)
- [ ] Project setup (workspace, crates)
- [ ] Core data models
- [ ] Basic CLI scaffolding
- [ ] Test framework setup

### Phase 2: Data Ingestion (PR #2)
- [ ] Claude Code parser
- [ ] Codex CLI parser
- [ ] Git repository scanner
- [ ] Storage layer

### Phase 3: Query Engine (PR #3)
- [ ] SQL-like language parser
- [ ] Query execution engine
- [ ] Built-in functions
- [ ] Optimization

### Phase 4: Output Formatters (PR #4)
- [ ] JSON/JSONL output
- [ ] CSV output
- [ ] Table output
- [ ] AI-optimized output

### Phase 5: Interactive UI (PR #5)
- [ ] TUI with ratatui
- [ ] Web UI with Axum
- [ ] Real-time updates

### Phase 6: Plugin Integration (PR #6)
- [ ] Claude Code plugin
- [ ] NPM package for npx/bunx
- [ ] Skill definitions

### Phase 7: Testing & Polish (PR #7)
- [ ] Unit tests (80%+ coverage)
- [ ] Integration tests
- [ ] E2E tests
- [ ] Performance benchmarks

### Phase 8: Documentation & Release (PR #8)
- [ ] README and docs
- [ ] Release binaries
- [ ] Homebrew formula
- [ ] NPM publish

## Team Structure

### Development Team (parallel workstreams)
1. **rust-core-team** - Core crates (storage, query engine)
2. **ingest-team** - Data ingestion parsers
3. **output-team** - Output formatters
4. **ui-team** - TUI and Web UI
5. **plugin-team** - Claude Code plugin + NPM
6. **test-team** - Testing infrastructure

### Review Team
7. **code-review-team** - Continuous code review

## Commands

### Installation
```bash
# Via NPM (recommended)
npm install -g @cai/cli

# Via Bun
bunx @cai/cli

# Via Cargo
cargo install cai-cli

# Claude Code Plugin
/plugin marketplace add user/repo
/plugin install cai
```

### Usage
```bash
# Basic query
cai query "SELECT * FROM history LIMIT 10"

# Output formats
cai query --output json "..."
cai query --output csv "..."
cai query --output table "..."

# Interactive
cai tui

# Web UI
cai web

# AI agent mode
cai ai "most productive prompts last week"
```

## Tech Stack

- **Language**: Rust 2024 edition
- **SQL**: sqlparser-rust + custom engine
- **CLI**: clap 4.x + ratatui
- **Web**: Axum + HTMX + Tailwind
- **Storage**: SQLite (rusqlite) + in-memory
- **Testing**: rstest + proptest
- **Plugin**: WASM for portability
