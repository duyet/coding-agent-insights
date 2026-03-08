# CAI Testing Infrastructure

## Overview

Comprehensive testing infrastructure for the Coding Agent Insights (CAI) project with 80%+ code coverage target.

## Test Structure

```
cai-project/
├── crates/
│   ├── cai-core/
│   │   └── tests/
│   │       └── lib_tests.rs        # Unit tests for core types
│   ├── cai-storage/
│   │   ├── tests/
│   │   │   └── memory_storage_tests.rs  # Unit tests for storage
│   │   └── benches/
│   │       └── storage_bench.rs     # Performance benchmarks
│   └── cai-output/
│       └── src/formats.rs           # Inline formatter tests
├── tests/                           # Integration tests crate
│   ├── src/
│   │   ├── integration.rs           # Integration tests
│   │   ├── fixtures.rs              # Test fixtures
│   │   └── helpers.rs               # Test utilities
├── .github/workflows/
│   └── ci.yml                       # CI/CD pipeline
└── Makefile                         # Testing commands
```

## Running Tests

### All Tests
```bash
make test
# or
cargo test --workspace
```

### Unit Tests (Per Crate)
```bash
cargo test -p cai-core
cargo test -p cai-storage
cargo test -p cai-output
```

### Integration Tests
```bash
cargo test -p cai-tests
```

### Coverage Report
```bash
make coverage
# or
cargo llvm-cov --all-features --workspace
```

### HTML Coverage Report
```bash
make coverage-html
# or
cargo llvm-cov --all-features --workspace --html
```

### Benchmarks
```bash
make benchmark
# or
cargo divan --workspace --bench
```

## Test Coverage

### cai-core (14 tests)
- Entry creation and serialization
- Source variants and equality
- Metadata with HashMap extras
- Error handling
- Property-based tests with proptest

### cai-storage (16 tests)
- MemoryStorage CRUD operations
- Filter-based queries (source, date range)
- Concurrent operations
- Count operations
- Parameterized tests for different sizes

### cai-output (7 tests)
- JSON formatter
- JSONL formatter  
- CSV formatter with escaping
- Table formatter
- AI formatter
- Stats formatter
- Text truncation

### Integration Tests (7 tests)
- Storage + query integration
- Storage + output integration
- Filtered queries
- Date range queries
- Get by ID workflow
- Concurrent operations

## CI/CD Pipeline

The `.github/workflows/ci.yml` includes:

1. **Test Job**: Runs on Ubuntu, Windows, macOS with stable and beta Rust
2. **Coverage Job**: Generates and uploads coverage to Codecov
3. **Security Job**: Runs cargo-audit for vulnerability checks
4. **Benchmark Job**: Runs benchmarks to detect regressions
5. **E2E Job**: Runs end-to-end tests (currently stubbed)
6. **Docs Job**: Validates documentation builds
7. **MSRV Job**: Validates minimum supported Rust version

## Cargo Aliases

Available in `.cargo/config.toml`:

- `cargo t` - Run all tests
- `cargo ti` - Run ignored tests
- `cargo tw` - Run tests with output
- `cargo cov` - Generate coverage
- `cargo cov-html` - Generate HTML coverage
- `cargo bench` - Run benchmarks
- `cargo lint` - Run clippy
- `cargo fmt` - Format code

## Makefile Commands

- `make test` - Run all tests
- `make unit` - Run unit tests only
- `make integration` - Run integration tests
- `make coverage` - Generate coverage report
- `make coverage-html` - Generate HTML coverage
- `make lint` - Run clippy
- `make fmt` - Format code
- `make fmt-check` - Check formatting
- `make benchmark` - Run benchmarks
- `make clean` - Clean build artifacts
- `make check` - Run test + lint
- `make all` - Build + test + lint

## Writing Tests

### Unit Tests
Place in `crates/<crate>/tests/<name>_tests.rs`:

```rust
use cai_core::{Entry, Source, Metadata};

#[test]
fn test_something() {
    let entry = Entry {
        id: "test".to_string(),
        source: Source::Claude,
        // ... rest of fields
    };
    assert_eq!(entry.id, "test");
}
```

### Property-Based Tests
Use proptest for random testing:

```rust
proptest::proptest! {
    #[test]
    fn test_property(id in "[a-zA-Z0-9_-]{1,50}") {
        // Test with generated input
    }
}
```

### Async Tests
Use tokio for async tests:

```rust
#[tokio::test]
async fn test_async_operation() {
    let storage = MemoryStorage::new();
    storage.store(&entry).await.unwrap();
}
```

### Integration Tests
Add to `tests/src/integration.rs`:

```rust
use cai_core::Entry;
use cai_storage::MemoryStorage;

#[tokio::test]
async fn test_integration() {
    // Cross-crate testing
}
```

## Fixtures

Available in `tests/src/fixtures.rs`:

- `test_entry()` - Single test entry
- `test_entries()` - Multiple test entries
- `benchmark_entries(count)` - Entry generator for benchmarks
- `sample_claude_json()` - Sample Claude conversation
- `sample_git_log()` - Sample git log output

## Test Goals

- **Coverage**: 80%+ code coverage
- **Speed**: All tests complete in < 30 seconds
- **Reliability**: No flaky tests
- **CI**: All tests pass before merge
- **Benchmarks**: Detect performance regressions
