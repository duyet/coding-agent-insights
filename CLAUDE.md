# CAI - Claude Code Project Instructions

## Project Context

CAI (Coding Agent Insights) is a Rust-based tool for analyzing AI coding history. Think "supercharged devsql" - designed specifically for AI-agent workflows.

## Quick Reference

### Project Structure
- `crates/` - Workspace crates (modular architecture)
- `tests/` - Integration tests
- `.github/workflows/` - CI/CD pipelines

### Key Commands
```bash
# Build
cargo build --workspace

# Test
make test                    # All tests
make unit                    # Unit tests only
make integration             # Integration tests
make coverage                # Coverage report

# Lint
make lint                    # Run clippy
make fmt                     # Format code
make fmt-check               # Check formatting

# Development
cargo watch -x 'test --workspace'    # Continuous testing
```

### Crate Responsibilities
- **cai-core**: Shared types (Entry, Source, Metadata)
- **cai-storage**: Storage backends (MemoryStorage, SQLiteStorage)
- **cai-query**: SQL-like query engine
- **cai-ingest**: Data ingestion parsers (Claude, Codex, Git)
- **cai-output**: Output formatters (JSON, CSV, Table, AI)
- **cai-tui**: Terminal UI with ratatui
- **cai-web**: Web dashboard with Axum
- **cai-cli**: Command-line interface
- **cai-plugin**: Claude Code WASM plugin

### Code Style
- Use `rustfmt` with default settings
- Follow Rust naming conventions
- Document all public items with `///`
- Use `thiserror` for error types
- Use `async/await` for async operations

### Testing Requirements
- Aim for 80%+ code coverage
- Write unit tests for each module
- Use `rstest` for parameterized tests
- Use `proptest` for property-based tests
- All tests must pass before PR merge

### Commit Convention
Follow Conventional Commits:
- `feat: ` - New feature
- `fix: ` - Bug fix
- `docs: ` - Documentation
- `test: ` - Tests
- `refactor: ` - Refactoring

### Common Workflows

#### Add New Data Source
1. Create parser in `cai-ingest`
2. Add `Source` variant to `cai-core`
3. Implement ingestion logic
4. Add tests
5. Update documentation

#### Add New Output Format
1. Implement `Formatter` trait in `cai-output`
2. Add tests
3. Register with CLI
4. Update documentation

#### Add Query Function
1. Define function in `cai-query`
2. Implement executor logic
3. Add to function registry
4. Document and test

### CI/CD
- Tests run on push/PR to main/develop
- Coverage uploaded to Codecov
- Benchmarks detect regressions
- All checks must pass before merge

### Dependencies
- Rust 1.80+
- Workspace dependencies defined in root `Cargo.toml`
- Use workspace versions for consistency

### Get Help
- `ARCHITECTURE.md` - System design
- `CONTRIBUTING.md` - Development guidelines
- `TESTING.md` - Testing guide
- Crate READMEs - Specific crate docs
