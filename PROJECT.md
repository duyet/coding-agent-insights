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

### Phase 1: Foundation (PR #1) ✅
- [x] Project setup (workspace, crates)
- [x] Core data models
- [x] Basic CLI scaffolding
- [x] Test framework setup

### Phase 2: Data Ingestion (PR #2) ✅
- [x] Claude Code parser
- [x] Codex CLI parser
- [x] Git repository scanner
- [x] Storage layer

### Phase 3: Query Engine (PR #3) ✅
- [x] SQL-like language parser
- [x] Query execution engine
- [x] Built-in functions
- [x] Optimization

### Phase 4: Output Formatters (PR #4) ✅
- [x] JSON/JSONL output
- [x] CSV output
- [x] Table output
- [x] AI-optimized output

### Phase 5: Interactive UI (PR #5) ✅
- [x] TUI with ratatui
- [x] Web UI with Axum
- [x] Real-time updates

### Phase 6: Plugin Integration (PR #6) ✅
- [x] Claude Code plugin
- [x] NPM package for npx/bunx
- [x] Skill definitions

### Phase 7: Testing & Polish (PR #7) ✅
- [x] Unit tests (71 tests, 100% pass rate)
- [x] Integration tests
- [x] E2E tests
- [x] Performance benchmarks (divan)

### Phase 8: Documentation & Release (PR #8) ✅
- [x] README and docs
- [x] Release binaries (v0.1.0)
- [x] Homebrew formula
- [x] NPM package (ready for publish)

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

- **Language**: Rust 2021 edition
- **SQL**: sqlparser-rust + custom engine with FunctionRegistry
- **CLI**: clap 4.x + ratatui for TUI
- **Web**: Axum + WebSocket support + vanilla HTML/JS dashboard
- **Storage**: MemoryStorage (pluggable backends)
- **Testing**: rstest + tempfile + E2E framework
- **Output**: JSON, JSONL, CSV, Table, AI, Stats formatters
- **Plugin**: C FFI exports for WASM compatibility
