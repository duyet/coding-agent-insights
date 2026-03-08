# Changelog

All notable changes to CAI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **CLI query execution** - Full SQL query support with all output formatters
  - `cai query "SELECT..." --output table|json|jsonl|csv|ai|stats`
  - Async mock data initialization to avoid nested runtime issues
  - Proper error handling with user-friendly messages
- **CLI configuration file support** (~/.cai/config.toml)
  - Storage backend configuration (memory/sqlite types)
  - Output format preferences
  - Environment variable override support (CAI_STORAGE_TYPE, etc.)
  - XDG config directory support (~/.config/cai/)
- Performance benchmark suite (divan) for output formatting
- E2E test framework with CLI testing support
- --version flag to CLI with proper version output
- Core data structures (Entry, Source, Metadata)
- Storage abstraction with MemoryStorage implementation
- Storage test helper: `with_mock_data()` for convenient test data
- SQL-like query engine with parser and executor
- SQL function system with FunctionRegistry for extensibility
- Built-in SQL functions:
  - `date_format(timestamp, format)` - Date formatting
  - `concat(...args)` - String concatenation
  - `length(str)` - String length
  - `upper(str)`/`lower(str)` - Case conversion
  - `substring(str, start, length)` - Extract substring
  - `coalesce(...args)` - First non-null value
  - `now()` - Current timestamp
- Output formatters (JSON, JSONL, CSV, Table, AI, Stats)
- CLI scaffolding with subcommands
- Testing infrastructure (71 tests, 100% pass rate)
- CI/CD pipeline with multi-platform testing
- TUI enhancements:
  - Sort columns (timestamp, source, prompt)
  - Sort order toggle (asc/desc)
  - Detail view for selected entry
  - Enhanced search with next/prev navigation
  - Improved keyboard handling
- Web UI enhancements:
  - WebSocket support for live updates
  - Enhanced dashboard with statistics cards
  - Improved responsive layout and styling
  - Real-time entry streaming and filtering
- Comprehensive documentation
- DevOps automation:
  - Release script with version bumping
  - Pull request templates and issue templates
  - Automated labeling workflow
  - Stale issue/PR management
  - Dependabot configuration
  - Codeowners for review assignment
  - Security policy and code of conduct
  - Funding configuration
  - Git workflow documentation
  - DevOps infrastructure documentation

### Fixed
- Divan benchmark compilation errors (types -> args)
- DynFormatter enum missing documentation
- Doctest failures in cai-core, cai-tui, and cai-web
- E2E test fixture path resolution
- CLI test assertions for case-insensitive command matching
- Compiler warnings from unused dependencies across all crates

### Changed
- Updated Metadata to use HashMap for extensibility
- Enhanced Makefile with release targets (patch/minor/major/dry-run/push)
- Added benches to workspace members
- Updated dependencies: axum with ws feature, tokio-tungstenite 0.28
- Improved NPM package scripts and configuration
- Enhanced crate README documentation

## [0.1.0] - TBD

### Added
- Initial project structure
- Workspace configuration
- Basic crate implementations

## Development Phases

### Phase 1: Foundation ✅
- [x] Project setup (workspace, crates)
- [x] Core data models
- [x] Basic CLI scaffolding
- [x] Test framework setup

### Phase 2: Data Ingestion ✅
- [x] Claude Code parser
- [x] Codex CLI parser
- [x] Git repository scanner
- [x] Storage layer

### Phase 3: Query Engine ✅
- [x] SQL-like language parser
- [x] Query execution engine
- [x] Built-in functions

### Phase 4: Output Formatters ✅
- [x] JSON/JSONL output
- [x] CSV output
- [x] Table output
- [x] AI-optimized output

### Phase 5: Interactive UI ✅
- [x] TUI scaffolding with ratatui
- [x] Web UI with Axum
- [x] Dashboard views with real-time updates

### Phase 6: Plugin Integration ✅
- [x] Claude Code plugin structure
- [x] NPM package for npx/bunx
- [x] Skill definitions

### Phase 7: Testing & Polish ✅
- [x] Unit tests (71 tests, 100% pass rate)
- [x] Integration tests
- [x] CI/CD pipeline
- [x] Performance benchmarks (divan)

### Phase 8: Documentation & Release ✅
- [x] README and docs
- [x] Release binaries (v0.1.0 release created)
- [x] Homebrew formula
- [x] NPM publish (script ready, manual publish required)

[Unreleased]: https://github.com/duyet/coding-agent-insights/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/duyet/coding-agent-insights/releases/tag/v0.1.0
