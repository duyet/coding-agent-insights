# CAI DevOps Documentation

This document provides an overview of the DevOps infrastructure for the CAI project.

## Overview

CAI has a comprehensive DevOps setup supporting:
- 7 parallel agents working on feature branches
- Automated CI/CD pipeline
- Multi-platform builds and testing
- Automated releases
- Changelog management
- Git workflow enforcement

## CI/CD Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     GitHub Repository                         │
│                      (cai-dev/cai)                           │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ Push / PR
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Workflows Triggered                      │
├─────────────────────────────────────────────────────────────┤
│  ci.yml          │  pr.yml  │  release.yml  │  other         │
│  (Test on push)  │  (PR)    │  (On tags)    │  (labeler,     │
│                  │          │               │   stale, etc)  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Quality Gates                              │
├─────────────────────────────────────────────────────────────┤
│  ✓ Tests pass (3 OS × 2 Rust versions)                      │
│  ✓ Coverage ≥80%                                             │
│  ✓ Clippy clean                                              │
│  ✓ Formatted code                                            │
│  ✓ Security audit                                            │
│  ✓ Conventional commits                                      │
│  ✓ Code review approved                                      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Artifacts                                │
├─────────────────────────────────────────────────────────────┤
│  • Binaries (Linux x86_64/aarch64, macOS x86_64/aarch64)    │
│  • Checksums (SHA256)                                        │
│  • GitHub Release                                            │
│  • crates.io packages                                        │
│  • Homebrew formula                                          │
└─────────────────────────────────────────────────────────────┘
```

## Workflows

### CI Workflow (`ci.yml`)

**Triggers**: Push to `main` or `develop`

**Jobs**:
- **test**: Run tests on Ubuntu, Windows, macOS with stable and beta Rust
- **coverage**: Generate coverage report and upload to Codecov
- **security**: Run cargo audit for security vulnerabilities
- **benchmark**: Run performance benchmarks
- **e2e**: Run end-to-end tests
- **check-docs**: Verify documentation builds correctly
- **msrv**: Verify minimum supported Rust version (1.80)

**Duration**: ~10-15 minutes

### PR Workflow (`pr.yml`)

**Triggers**: Pull request to `main` or `develop`

**Jobs**:
- **commitlint**: Validate commit messages follow conventional commits
- **pr-size**: Warn on large PRs (>1000 lines)
- **label-check**: Ensure PR has appropriate labels
- **dco-check**: Verify Developer Certificate of Origin
- **quick-checks**: Fast format, clippy, and unit tests
- **pr-summary**: Generate PR summary with changed crates

**Duration**: ~3-5 minutes

### Release Workflow (`release.yml`)

**Triggers**: Push of version tag (`v*.*.*`)

**Jobs**:
- **create-release**: Create GitHub release with CHANGELOG notes
- **build**: Build binaries for multiple platforms
  - Linux x86_64 and aarch64
  - macOS x86_64 and aarch64
- **publish-crate**: Publish crates to crates.io (stable only)
- **homebrew-tap**: Generate Homebrew formula
- **update-changelog**: Prepare CHANGELOG for next version

**Duration**: ~20-30 minutes

### Other Workflows

- **labeler.yml**: Auto-label PRs based on changed files
- **stale.yml**: Mark stale issues and PRs
- **dependabot.yml**: Auto-merge security updates

## Quality Metrics

### Test Coverage

- **Target**: ≥80% line coverage
- **Current**: ~62 tests, improving
- **Tracking**: Codecov integration

### Performance

- **Benchmarks**: Run on every push
- **Baseline**: Established in CI
- **Regression Alerts**: Automated

### Security

- **Audits**: Cargo audit on every PR
- **Dependabot**: Automated dependency updates
- **Secret Scanning**: GitHub native scanning

## Build Scripts

### `scripts/build.sh`

Comprehensive build script with options:

```bash
# Build all in release mode
./scripts/build.sh

# Build in dev mode
./scripts/build.sh -p dev

# Build specific crate
./scripts/build.sh cai-core

# Build binary only
./scripts/build.sh binary

# Verbose output
./scripts/build.sh -v
```

### `scripts/test.sh`

Run tests with coverage:

```bash
# Run all tests
./scripts/test.sh

# Run with coverage
./scripts/test.sh -c

# Generate HTML coverage
./scripts/test.sh -c --html

# Run unit tests only
./scripts/test.sh unit

# Run with benchmarks
./scripts/test.sh -b
```

### `scripts/release.sh`

Automated release preparation:

```bash
# Bump patch version
./scripts/release.sh -b patch

# Bump minor version
./scripts/release.sh -b minor

# Set specific version
./scripts/release.sh -v 1.2.3

# Dry run
./scripts/release.sh -b minor -d

# Prepare and push
./scripts/release.sh -b patch -p
```

## Makefile Targets

Convenience targets for common tasks:

```bash
make build          # Build all
make test           # Run tests
make lint           # Run clippy
make fmt            # Format code
make coverage       # Coverage report
make clean          # Clean artifacts
make check          # Test + lint
make all            # Build + test + lint

# Release targets
make release-patch  # Bump patch version
make release-minor  # Bump minor version
make release-major  # Bump major version
make release-dry-run # Preview release
```

## Git Configuration

### Commit Linting

Enforced via `.commitlintrc.yml`:

- Types: feat, fix, docs, style, refactor, perf, test, chore, ci, build
- Scopes: core, ingest, query, storage, output, tui, web, cli, plugin, tests, ci, docs
- Format: `<type>[(scope)]: <subject>`

### Branch Protection

**main** and **develop** branches:
- Required status checks
- Required PR reviews (1 approval)
- Code owner reviews
- No direct pushes

### CODEOWNERS

Defined in `.github/CODEOWNERS`:
- Core team approves core changes
- Specialized teams for different areas
- Default fallback to core team

## Release Management

### Version Bumping

Semantic versioning:
- **MAJOR**: Breaking changes
- **MINOR**: New features (backwards compatible)
- **PATCH**: Bug fixes (backwards compatible)

### Changelog

Format: [Keep a Changelog](https://keepachangelog.com/)

Sections:
- Added
- Changed
- Deprecated
- Removed
- Fixed
- Security

### Release Process

1. Prepare release with `scripts/release.sh`
2. Review generated CHANGELOG
3. Commit and push
4. CI/CD automatically:
   - Runs tests
   - Builds binaries
   - Creates release
   - Publishes crates
   - Generates formula

## Parallel Development

Support for 7 agents working in parallel:

### Agent Workflow

```
1. Leader assigns tasks
2. Each agent creates: feature/<agent>/<task>
3. Work independently on isolated features
4. Create PRs to develop
5. CI validates each PR
6. Merge when approved
```

### Conflict Prevention

- Work in separate crates
- Use feature flags
- Regular syncs to develop
- Clear task boundaries

### Integration

- All PRs target `develop`
- Continuous integration
- Automated testing
- Code review requirements

## Monitoring

### CI/CD Metrics

- Test pass rate: ~100%
- Average CI time: 10-15 minutes
- PR merge time: <24 hours
- Release frequency: As needed

### Coverage Tracking

- Codecov integration
- Minimum 80% target
- Per-crate tracking
- Trend analysis

### Performance

- Benchmark regression detection
- Historical performance data
- Alert on regressions

## Security

### Dependency Management

- Dependabot for updates
- Security audits
- Automated patching

### Secrets

- GitHub secrets for tokens
- No secrets in code
- Secret scanning enabled

### Access Control

- Branch protection
- Required reviews
- CODEOWNERS enforcement

## Maintenance

### Regular Tasks

- **Weekly**: Review Dependabot PRs
- **Weekly**: Check CI performance
- **Monthly**: Review and update workflows
- **Per Release**: Update CHANGELOG

### Emergency Procedures

- **CI Failure**: Investigate logs, fix, rerun
- **Security Issue**: Private disclosure, fix release
- **Broken Release**: Yank version, patch release

## Documentation

- [GIT_WORKFLOW.md](GIT_WORKFLOW.md) - Git workflow
- [RELEASE.md](RELEASE.md) - Release process
- [TESTING.md](TESTING.md) - Testing practices
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guide

## Tools and Integrations

### GitHub Features

- Actions (CI/CD)
- Dependabot (dependencies)
- Code Owners (reviewers)
- Branch Protection (quality gates)
- Secret Scanning (security)
- Status Checks (CI results)

### External Tools

- Codecov (coverage)
- crates.io (package registry)
- Homebrew (package manager)

## Future Improvements

- [ ] Add Windows ARM builds
- [ ] Add more performance benchmarks
- [ ] Implement automated semantic release
- [ ] Add integration test suite
- [ ] Performance regression alerts
- [ ] Documentation site deployment
- [ ] Binary signing for security
- [ ] Automated changelog from commits

## Support

For DevOps issues or questions:
- Create an issue with label `ci-cd`
- Contact @cai-dev/devops-team
- Check [GitHub Discussions](https://github.com/cai-dev/coding-agent-insights/discussions)
