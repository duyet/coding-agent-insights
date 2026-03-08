# Contributing to CAI

Thank you for your interest in contributing to Coding Agent Insights (CAI)! This document provides guidelines for contributing to the project.

## Getting Started

### Prerequisites

- Rust 1.80 or later
- Git
- A GitHub account

### Setting Up

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/coding-agent-insights.git
   cd coding-agent-insights
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/cai-dev/coding-agent-insights.git
   ```
4. Install development dependencies:
   ```bash
   cargo install cargo-watch
   cargo install cargo-llvm-cov
   ```

## Development Workflow

### Branch Strategy

- `main`: Stable release branch
- `develop`: Development branch for next release
- `feature/*`: Feature branches
- `bugfix/*`: Bug fix branches
- `docs/*`: Documentation updates

### Creating a Feature Branch

```bash
git checkout develop
git pull upstream develop
git checkout -b feature/your-feature-name
```

### Making Changes

1. Write code following the [Style Guide](#style-guide)
2. Add tests for new functionality
3. Update documentation as needed
4. Run tests locally:
   ```bash
   cargo test --workspace
   cargo clippy --workspace
   cargo fmt --all -- --check
   ```

### Committing Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add SQLite storage backend
fix: resolve query parsing edge case
docs: update installation instructions
test: add integration tests for query engine
refactor: simplify storage interface
```

### Submitting a Pull Request

1. Push your branch:
   ```bash
   git push origin feature/your-feature-name
   ```
2. Create a pull request on GitHub
3. Fill out the PR template
4. Wait for code review
5. Address review feedback
6. Get approval and merge

## Style Guide

### Rust Code Style

- Use `rustfmt` with default settings
- Follow Rust naming conventions:
  - Types: `PascalCase`
  - Functions: `snake_case`
  - Constants: `SCREAMING_SNAKE_CASE`
- Use `#[warn(missing_docs)]` on public items
- Prefer `thiserror` for error types
- Use `async/await` for async operations

### Documentation

- Document all public items with `///` or `//!`
- Include examples in doc comments
- Use `#[doc = include_str!("...")]` for long examples
- Keep documentation up-to-date with code changes

Example:
```rust
/// Creates a new in-memory storage instance.
///
/// # Examples
///
/// ```rust
/// use cai_storage::MemoryStorage;
///
/// let storage = MemoryStorage::new();
/// ```
pub fn new() -> Self {
    // ...
}
```

### Testing

- Write unit tests for each module
- Write integration tests for cross-crate functionality
- Use `rstest` for parameterized tests
- Use `proptest` for property-based testing
- Aim for 80%+ code coverage

Example:
```rust
#[rstest::rstest]
#[case(0)]
#[case(10)]
#[case(100)]
async fn test_store_multiple(#[case] count: usize) {
    let storage = MemoryStorage::new();
    // ...
}

#[proptest::proptest]
fn test_id_roundtrip(id in "[a-zA-Z0-9_-]{1,50}") {
    // ...
}
```

## Project Structure

```
coding-agent-insights/
├── crates/              # Workspace crates
│   ├── cai-core/       # Shared types
│   ├── cai-storage/    # Storage layer
│   ├── cai-query/      # Query engine
│   └── ...
├── tests/              # Integration tests
├── docs/               # Additional documentation
├── Cargo.toml          # Workspace configuration
├── README.md           # Project overview
├── ARCHITECTURE.md     # Architecture documentation
├── CONTRIBUTING.md     # This file
└── Makefile           # Common commands
```

### Adding a New Crate

1. Create directory in `crates/`
2. Add to `Cargo.toml` workspace members
3. Create `Cargo.toml` with workspace dependencies
4. Implement crate functionality
5. Add tests
6. Update documentation

## Code Review Guidelines

### For Contributors

- Make PRs small and focused
- Include tests for new features
- Update documentation
- Respond to review feedback promptly
- Be open to suggestions

### For Reviewers

- Review within 48 hours
- Provide constructive feedback
- Approve when criteria are met
- Test changes locally if needed

## Testing Guidelines

### Unit Tests

- Test public APIs
- Test error conditions
- Use fixtures for test data
- Mock external dependencies

### Integration Tests

- Test cross-crate workflows
- Test with real storage backends
- Test error propagation
- Use realistic data volumes

### E2E Tests

- Test complete CLI workflows
- Test all output formats
- Test installation process
- Performance benchmarks

## Issue Reporting

### Bug Reports

Include:
- Rust version (`rustc --version`)
- OS and version
- Steps to reproduce
- Expected vs actual behavior
- Backtrace if applicable

### Feature Requests

Include:
- Use case description
- Proposed API/UX
- Alternatives considered
- Impact assessment

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag
4. Build release binaries
5. Publish to crates.io
6. Create GitHub release

## Community Guidelines

- Be respectful and inclusive
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Help

- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: General questions and ideas
- Discord/Slack: Real-time chat (if available)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

## Recognition

Contributors will be recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- Project documentation

Thank you for contributing to CAI!
