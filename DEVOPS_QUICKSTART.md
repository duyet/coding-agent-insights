# CAI DevOps Quick Reference

Quick guide for CAI developers and DevOps engineers.

## Daily Commands

### Development

```bash
# Format code
cargo fmt
# or
make fmt

# Run linter
cargo clippy --workspace --all-targets -- -D warnings
# or
make lint

# Run tests
cargo test --workspace
# or
make test

# Run tests with coverage
./scripts/test.sh -c
# or
make coverage

# Build
cargo build --workspace
# or
make build

# Build release
cargo build --workspace --release
# or
make release
```

### Git Workflow

```bash
# Start feature
git checkout develop
git pull
git checkout -b feature/my-feature

# Make changes
git add .
git commit -m "feat(scope): description"

# Push and create PR
git push origin feature/my-feature
# Create PR on GitHub

# After merge, delete branch
git checkout develop
git pull
git branch -d feature/my-feature
```

### Release

```bash
# Bump version and prepare release
./scripts/release.sh -b minor  # or -b major, -b patch

# Review changes
git diff

# Push to trigger CI/CD
git push
git push origin v1.2.3
```

## Commit Message Format

```
<type>[(scope)]: <subject>
```

**Types**: feat, fix, docs, style, refactor, perf, test, chore, ci, build
**Scopes**: core, ingest, query, storage, output, tui, web, cli, plugin, tests, ci, docs

**Examples**:
```bash
git commit -m "feat(query): add GROUP BY support"
git commit -m "fix(storage): prevent race condition"
git commit -m "docs: update installation guide"
```

## CI/CD Status

### Required Checks for Merge

- ✅ CI (all platforms)
- ✅ Commit Message Lint
- ✅ Code Review (1 approval)
- ✅ Coverage ≥80%

### Check Status

On GitHub PR page, ensure all checks pass before requesting review.

## Troubleshooting

### CI Failures

```bash
# Replicate locally
rustc --version  # Check version matches CI
cargo clean
cargo build --workspace --all-features
cargo test --workspace --all-features

# Fix issues and push
git commit -m "ci: fix CI failures"
git push
```

### Merge Conflicts

```bash
git fetch origin
git rebase origin/develop
# Resolve conflicts
git add <files>
git rebase --continue
git push --force-with-lease
```

### Release Issues

```bash
# Check tag exists
git tag -l "v1.2.3"

# Delete bad release
git tag -d v1.2.3
git push origin :refs/tags/v1.2.3

# Yank crates.io version
cargo yank --vers 1.2.3 --package cai-cli
```

## Scripts Reference

### `scripts/build.sh`

```bash
./scripts/build.sh              # Build all (release)
./scripts/build.sh -p dev       # Build all (dev)
./scripts/build.sh binary       # Build binary only
./scripts/build.sh cai-core     # Build specific crate
```

### `scripts/test.sh`

```bash
./scripts/test.sh               # Run all tests
./scripts/test.sh -c            # With coverage
./scripts/test.sh -c --html     # HTML coverage
./scripts/test.sh unit          # Unit tests only
./scripts/test.sh -b            # With benchmarks
```

### `scripts/release.sh`

```bash
./scripts/release.sh -b patch   # Bump patch
./scripts/release.sh -b minor   # Bump minor
./scripts/release.sh -b major   # Bump major
./scripts/release.sh -v 1.2.3   # Set version
./scripts/release.sh -b minor -d # Dry run
./scripts/release.sh -b minor -p # Prepare and push
```

## Makefile Targets

```bash
make build          # Build all
make release        # Build release
make test           # Run tests
make unit           # Unit tests
make integration    # Integration tests
make lint           # Run clippy
make fmt            # Format code
make fmt-check      # Check format
make coverage       # Coverage report
make coverage-html  # HTML coverage
make benchmark      # Run benchmarks
make clean          # Clean artifacts
make check          # Test + lint
make all            # Build + test + lint

# Release
make release-patch
make release-minor
make release-major
make release-dry-run
```

## GitHub Workflow

### Pull Request Process

1. Create feature branch from `develop`
2. Make changes with conventional commits
3. Push to GitHub
4. Create PR: `feature/xxx` → `develop`
5. Address CI failures
6. Address review feedback
7. Wait for approval
8. Squash merge

### Review Guidelines

- Check CI passes
- Review code changes
- Verify tests added
- Check documentation
- Approve or request changes

## Quality Checklist

Before pushing:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test --workspace

# Check commit message
git log -1 --pretty=%B  # Should follow conventional commits
```

Before requesting review:

- Update CHANGELOG.md (if user-facing)
- Add/update tests
- Update documentation
- Check all CI passes
- Add appropriate labels

## Documentation

- [GIT_WORKFLOW.md](GIT_WORKFLOW.md) - Detailed Git workflow
- [RELEASE.md](RELEASE.md) - Release process
- [DEVOPS.md](DEVOPS.md) - DevOps infrastructure
- [TESTING.md](TESTING.md) - Testing practices
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guide

## Support

- **Issues**: https://github.com/cai-dev/coding-agent-insights/issues
- **Discussions**: https://github.com/cai-dev/coding-agent-insights/discussions
- **DevOps Team**: @cai-dev/devops-team

## Quick Links

- [CI Dashboard](https://github.com/cai-dev/coding-agent-insights/actions)
- [Releases](https://github.com/cai-dev/coding-agent-insights/releases)
- [Coverage](https://codecov.io/gh/cai-dev/coding-agent-insights)
- [Crates.io](https://crates.io/crates/cai)
