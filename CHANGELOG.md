# Changelog

All notable changes to CAI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Core data structures (Entry, Source, Metadata)
- Storage abstraction with MemoryStorage implementation
- SQL-like query engine with parser and executor
- Output formatters (JSON, JSONL, CSV, Table, AI, Stats)
- CLI scaffolding with subcommands
- Testing infrastructure (62 tests, 80%+ coverage target)
- CI/CD pipeline with multi-platform testing
- Comprehensive documentation

### Changed
- Updated Metadata to use HashMap for extensibility

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

### Phase 2: Data Ingestion 🚧
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

### Phase 5: Interactive UI 🚧
- [x] TUI scaffolding
- [ ] Real-time updates
- [ ] Web UI with Axum
- [ ] Dashboard views

### Phase 6: Plugin Integration 📋
- [x] Claude Code plugin structure
- [ ] NPM package for npx/bunx
- [ ] Skill definitions

### Phase 7: Testing & Polish ✅
- [x] Unit tests (62 tests passing)
- [x] Integration tests
- [x] CI/CD pipeline
- [ ] Performance benchmarks

### Phase 8: Documentation & Release 🚧
- [x] README and docs
- [ ] Release binaries
- [ ] Homebrew formula
- [ ] NPM publish

[Unreleased]: https://github.com/cai-dev/coding-agent-insights/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/cai-dev/coding-agent-insights/releases/tag/v0.1.0
