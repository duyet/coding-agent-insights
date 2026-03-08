# CAI Release Process

This document describes the automated release process for CAI.

## Prerequisites

- Repository admin access
- `CRATES_IO_TOKEN` configured in GitHub secrets
- Clean working directory (no uncommitted changes)

## Automated Release Process

The release process is fully automated via GitHub Actions when you push a version tag:

```bash
# 1. Use the release script to prepare the release
./scripts/release.sh -b minor  # or -b major, -b patch

# 2. The script will:
#    - Bump version in Cargo.toml
#    - Update CHANGELOG.md from commits
#    - Commit changes with conventional commit format
#    - Create git tag

# 3. Push to trigger CI/CD
git push
git push origin v1.2.3

# 4. GitHub Actions will automatically:
#    - Run all tests
#    - Build binaries for multiple platforms
#    - Create GitHub release with CHANGELOG notes
#    - Upload release binaries
#    - Publish crates to crates.io (if not pre-release)
#    - Generate Homebrew formula
```

## Manual Release Steps

If you need to perform a manual release:

```bash
# 1. Update version
# Edit Cargo.toml workspace.version
vim Cargo.toml

# 2. Update CHANGELOG
# Add release section under [Unreleased]
vim CHANGELOG.md

# 3. Commit changes
git add Cargo.toml CHANGELOG.md
git commit -m "chore(release): prepare v1.2.3"

# 4. Create tag
git tag -a v1.2.3 -m "Release v1.2.3"

# 5. Push
git push
git push origin v1.2.3
```

## Release Script Usage

The `scripts/release.sh` script provides several options:

```bash
# Bump version automatically
./scripts/release.sh -b patch   # 1.2.3 -> 1.2.4
./scripts/release.sh -b minor   # 1.2.3 -> 1.3.0
./scripts/release.sh -b major   # 1.2.3 -> 2.0.0

# Set specific version
./scripts/release.sh -v 2.0.0

# Dry run (preview changes)
./scripts/release.sh -b minor -d

# Prepare changes only (no tag)
./scripts/release.sh -b minor -c

# Prepare and push
./scripts/release.sh -b minor -p

# Skip changelog generation
./scripts/release.sh -b minor -s
```

## Version Bumping Rules

Follow semantic versioning:

- **MAJOR** (X.0.0): Incompatible API changes
- **MINOR** (0.X.0): Backwards-compatible functionality
- **PATCH** (0.0.X): Backwards-compatible bug fixes

Examples:
- Breaking query syntax change: `1.2.3 -> 2.0.0`
- New output format: `1.2.3 -> 1.3.0`
- Bug fix in parsing: `1.2.3 -> 1.2.4`

## Pre-release Versions

For alpha, beta, or RC releases:

```bash
# Set version with pre-release identifier
./scripts/release.sh -v 1.3.0-alpha.1
./scripts/release.sh -v 1.3.0-beta.1
./scripts/release.sh -v 1.3.0-rc.1
```

Pre-release versions:
- Create GitHub release as "Pre-release"
- Skip crates.io publishing
- Don't update Homebrew formula

## Changelog Format

The changelog follows [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [1.2.3] - 2024-03-08

### Added
- New feature description
- Another new feature

### Changed
- Modified existing feature

### Fixed
- Bug fix description
- Another bug fix

### Security
- Security fix description
```

## Release Artifacts

Each release produces:

### Binaries
- `cai-linux-x86_64.tar.gz`
- `cai-linux-aarch64.tar.gz`
- `cai-macos-x86_64.tar.gz`
- `cai-macos-aarch64.tar.gz`

### Checksums
- `cai-*.tar.gz.sha256` for each binary

### Documentation
- Release notes from CHANGELOG
- GitHub release page

### Package Publishing
- Crates published to crates.io (stable releases only)
- Homebrew formula generated (stable releases only)

## Post-Release Tasks

After a successful release:

1. **Verify GitHub release**
   - Check all binaries uploaded
   - Verify release notes

2. **Verify crates.io**
   - Check all crates published
   - Verify version numbers

3. **Update documentation**
   - Update installation instructions
   - Update version badges

4. **Announce release**
   - Update README badges
   - Create release blog post
   - Announce on social media

## Rollback Procedure

If a release needs to be rolled back:

```bash
# 1. Delete GitHub release (via web UI)
# 2. Delete tag
git tag -d v1.2.3
git push origin :refs/tags/v1.2.3

# 3. Yank crates.io version (if published)
cargo yank --vers 1.2.3 --package cai-cli

# 4. Create patch release
./scripts/release.sh -b patch
```

## Troubleshooting

### Release workflow fails

Check the Actions tab for specific failure:
- Tests failing: Fix tests and retry
- Build failing: Check CI logs, fix build
- Publish failing: Check CRATES_IO_TOKEN secret

### Changelog not generated

Ensure commits follow conventional commits format:
- `feat:` for new features
- `fix:` for bug fixes
- etc.

### Binaries not uploaded

Check release.yml workflow logs:
- Verify build matrix completed
- Check upload steps for errors

## CI/CD Architecture

```
Push Tag (v1.2.3)
        ↓
GitHub Actions Triggered
        ↓
┌──────────────────────────┐
│  Create GitHub Release   │
│  - Extract CHANGELOG     │
│  - Create release        │
└──────────────────────────┘
        ↓
┌──────────────────────────┐
│  Build Binaries          │
│  - Linux (x86_64, aarch64)│
│  - macOS (x86_64, aarch64)│
└──────────────────────────┘
        ↓
┌──────────────────────────┐
│  Upload Artifacts        │
│  - Binaries to release   │
│  - Checksums             │
└──────────────────────────┘
        ↓
┌──────────────────────────┐
│  Publish Packages        │
│  - crates.io (stable)    │
│  - Homebrew formula      │
└──────────────────────────┘
```

## Related Files

- `.github/workflows/release.yml` - Release workflow definition
- `.github/workflows/ci.yml` - CI workflow (runs on release)
- `.commitlintrc.yml` - Commit message validation
- `scripts/release.sh` - Release automation script
- `CHANGELOG.md` - Changelog entries
