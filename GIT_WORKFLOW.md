# CAI Git Workflow

This document describes the Git workflow and branch protection rules for the CAI project.

## Branch Strategy

CAI uses a simplified Git workflow optimized for parallel development by multiple agents.

### Main Branches

```
main           → Production-ready code
develop        → Integration branch for features
feature/*      → Feature branches
fix/*          → Bug fix branches
docs/*         → Documentation updates
release/*      → Release preparation
```

### Branch Rules

- **`main`**: Protected, requires PR + CI checks
- **`develop`**: Integration branch, requires PR + CI checks
- **`feature/*`**: Short-lived branches from `develop`
- **`fix/*`**: Bug fixes from `develop` (or `main` for hotfixes)
- **`docs/*`**: Documentation updates
- **`release/*`**: Release preparation (created by release script)

## Workflow Process

### Feature Development

```bash
# 1. Create feature branch from develop
git checkout develop
git pull origin develop
git checkout -b feature/add-group-by-support

# 2. Make changes with conventional commits
git add .
git commit -m "feat(query): add GROUP BY clause support"

# 3. Push and create PR
git push origin feature/add-group-by-support
# Create PR on GitHub: feature/add-group-by-support → develop

# 4. Address review feedback
git add .
git commit -m "refactor(query): simplify GROUP BY logic"
git push

# 5. After approval and CI passes, squash merge
# (Merge via GitHub UI)
```

### Bug Fixes

```bash
# For bugs in current development:
git checkout develop
git checkout -b fix/fix-null-pointer-crash

# For production hotfixes:
git checkout main
git checkout -b fix/critical-data-corruption

# Follow same process as features
```

### Release Process

See [RELEASE.md](./RELEASE.md) for detailed release instructions.

```bash
# 1. Prepare release (from develop or main)
./scripts/release.sh -b minor

# 2. Push changes and tag
git push
git push origin v1.2.3

# 3. Monitor GitHub Actions
# 4. Verify release on GitHub
```

## Commit Message Format

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[(scope)]: <subject>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test additions/changes
- `chore`: Maintenance tasks
- `ci`: CI/CD changes
- `build`: Build system changes

### Scopes

- `core`: Core functionality
- `ingest`: Data ingestion
- `query`: Query engine
- `storage`: Storage layer
- `output`: Output formatting
- `tui`: Terminal UI
- `web`: Web interface
- `cli`: CLI interface
- `plugin`: Plugin system
- `tests`: Test infrastructure
- `ci`: CI/CD
- `docs`: Documentation
- `release`: Release management

### Examples

```
feat(query): add GROUP BY clause support
fix(storage): prevent data race in SQLite backend
docs: update README with installation instructions
test(cli): add integration tests for ingest command
perf(query): optimize query execution with caching
refactor(core): simplify entry type hierarchy
chore: update dependencies to latest versions
```

## Pull Request Guidelines

### PR Title

PR titles must follow conventional commits format:

```
feat(query): add GROUP BY clause support
fix(storage): prevent data race in SQLite backend
docs: update installation guide
```

### PR Description

Use the provided PR template. Include:

- Summary of changes
- Type of change (feature, fix, etc.)
- Testing performed
- Checklist items
- Related issues/PRs

### PR Labels

PRs are auto-labeled based on changed files:

- `cai-core`, `cai-query`, etc.: Affected crates
- `bug`, `enhancement`: Issue type
- `documentation`: Docs changes
- `ci-cd`: CI/CD updates
- `dependencies`: Dependency updates

### Review Process

1. **Automated checks**
   - Commitlint validates commit messages
   - CI runs tests on all platforms
   - Code coverage requirements
   - Clippy and formatting checks

2. **Code review**
   - At least one approval required
   - Code owner review for core changes
   - All feedback addressed

3. **Merge**
   - Squash merge preferred
   - PR title becomes commit message
   - Branch deleted automatically

## Branch Protection Rules

### `main` Branch

- **Required status checks**: All CI checks must pass
- **Required pull request reviews**: At least 1 approval
- **Require code owner reviews**: Enabled
- **Dismiss stale reviews**: Enabled
- **Restrict who can push**: Only maintainers
- **Do not allow bypassing settings**: Enabled

### `develop` Branch

- **Required status checks**: All CI checks must pass
- **Required pull request reviews**: At least 1 approval
- **Require code owner reviews**: Enabled
- **Dismiss stale reviews**: Enabled

## CI/CD Integration

### On Push

- **To `main` or `develop`**: Full CI pipeline runs
  - Tests (Ubuntu, macOS, Windows)
  - Coverage report
  - Security audit
  - Benchmarks
  - Documentation checks

### On Pull Request

- Additional PR checks run:
  - Commit message linting
  - PR size warning
  - Label validation
  - DCO check
  - Quick pre-merge checks

### On Tag Push

- Release workflow triggers:
  - Build release binaries
  - Create GitHub release
  - Publish to crates.io
  - Update Homebrew formula

## Parallel Development

CAI supports 7 agents working in parallel:

### Agent Coordination

1. **Work Assignment**
   - Leader agent assigns tasks to specific agents
   - Each agent works on isolated feature branches
   - Branch naming: `feature/<agent>/<feature-name>`

2. **Avoiding Conflicts**
   - Work in separate crates when possible
   - Use feature flags for incomplete features
   - Coordinate through issue assignments

3. **Integration**
   - All PRs target `develop` branch
   - Continuous integration prevents breakage
   - Regular syncs to `develop` reduce conflicts

### Example Parallel Workflow

```
Agent 1: feature/agent1/core-types
Agent 2: feature/agent2/sql-parser
Agent 3: feature/agent3/sql-executor
Agent 4: feature/agent4/json-output
Agent 5: feature/agent5/cli-commands
Agent 6: feature/agent6/tui-scaffold
Agent 7: feature/agent7/test-infrastructure
```

All work independently, PR to `develop`, CI validates each.

## Best Practices

### DO ✅

- Keep branches focused and short-lived
- Write descriptive commit messages
- Update documentation with code changes
- Add tests for new features
- Run tests locally before pushing
- Review PRs promptly
- Delete merged branches

### DON'T ❌

- Commit directly to `main` or `develop`
- Mix unrelated changes in one branch
- Ignore CI failures
- Bypass code review
- Leave PRs open for weeks
- Push broken code

## Troubleshooting

### Merge Conflicts

```bash
# 1. Update your branch
git checkout feature/your-feature
git fetch origin
git rebase origin/develop

# 2. Resolve conflicts
# Edit conflicting files
git add <resolved-files>
git rebase --continue

# 3. Force push (carefully!)
git push origin feature/your-feature --force-with-lease
```

### Failed CI Checks

1. Check CI logs for specific failure
2. Replicate locally with same Rust version
3. Fix the issue
4. Commit and push fix
5. CI will re-run automatically

### Stale PRs

PRs inactive for 60 days are auto-marked as stale:
- Comment to unmark
- Will close after 30 more days if no activity
- Use `keep` label to prevent auto-close

## Related Documentation

- [RELEASE.md](./RELEASE.md) - Release process
- [CONTRIBUTING.md](./CONTRIBUTING.md) - Contribution guide
- [TESTING.md](./TESTING.md) - Testing practices
- [.commitlintrc.yml](../.commitlintrc.yml) - Commit linting rules
