#!/usr/bin/env bash
# CAI Release Script
# Automates version bumping, changelog generation, and release preparation

set -eEuo pipefail
trap 'echo "::error::Release preparation failed at line $LINENO"' ERR

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
CHANGELOG_FILE="$PROJECT_ROOT/CHANGELOG.md"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
COMMITLINT_CONFIG="$PROJECT_ROOT/.commitlintrc.yml"

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

check_git_status() {
    log_step "Checking git status..."

    if [ -n "$(git status --porcelain)" ]; then
        log_error "Working directory is not clean. Please commit or stash changes."
        git status --short
        exit 1
    fi

    local BRANCH=$(git rev-parse --abbrev-ref HEAD)
    if [ "$BRANCH" != "main" ] && [ "$BRANCH" != "develop" ]; then
        log_warn "Not on main or develop branch (current: $BRANCH)"
    fi
}

get_current_version() {
    grep -m 1 '^version =' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/'
}

validate_version() {
    local version=$1

    # Validate semantic version format
    if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        log_error "Invalid semantic version: $version"
        log_error "Expected format: X.Y.Z or X.Y.Z-<pre-release>"
        exit 1
    fi
}

bump_version() {
    local current_version=$1
    local bump_type=$2

    local MAJOR=$(echo "$current_version" | cut -d. -f1)
    local MINOR=$(echo "$current_version" | cut -d. -f2)
    local PATCH=$(echo "$current_version" | cut -d. -f3 | cut -d- -f1)

    case $bump_type in
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        patch)
            PATCH=$((PATCH + 1))
            ;;
        *)
            log_error "Invalid bump type: $bump_type"
            log_error "Must be: major, minor, or patch"
            exit 1
            ;;
    esac

    echo "${MAJOR}.${MINOR}.${PATCH}"
}

update_version() {
    local new_version=$1

    log_step "Updating version to $new_version..."

    # Update workspace version in Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    rm -f "${CARGO_TOML}.bak"

    log_info "Updated version in $CARGO_TOML"
}

generate_changelog() {
    local version=$1
    local previous_version=$2

    log_step "Generating changelog for v$version..."

    # Get commits since previous version
    local COMMITS=""
    if [ -n "$previous_version" ]; then
        COMMITS=$(git log "v${previous_version}..HEAD" --pretty=format:"- %s (%h)" --reverse)
    else
        COMMITS=$(git log --pretty=format:"- %s (%h)" --reverse | head -20)
    fi

    # Categorize commits
    local FEATURES=$(echo "$COMMITS" | grep -i "^- feat" || echo "")
    local FIXES=$(echo "$COMMITS" | grep -i "^- fix" || echo "")
    local CHANGES=$(echo "$COMMITS" | grep -i "^- refactor\|^- perf" || echo "")
    local DOCS=$(echo "$COMMITS" | grep -i "^- docs" || echo "")
    local OTHER=$(echo "$COMMITS" | grep -iv "^- feat\|^- fix\|^- refactor\|^- perf\|^- docs" || echo "")

    # Build changelog entry
    local ENTRY="## [$version] - $(date +%Y-%m-%d)\n\n"

    if [ -n "$FEATURES" ]; then
        ENTRY="${ENTRY}### Added\n\n${FEATURES}\n\n"
    fi

    if [ -n "$CHANGES" ]; then
        ENTRY="${ENTRY}### Changed\n\n${CHANGES}\n\n"
    fi

    if [ -n "$FIXES" ]; then
        ENTRY="${ENTRY}### Fixed\n\n${FIXES}\n\n"
    fi

    if [ -n "$DOCS" ]; then
        ENTRY="${ENTRY}### Documentation\n\n${DOCS}\n\n"
    fi

    if [ -n "$OTHER" ]; then
        ENTRY="${ENTRY}### Other\n\n${OTHER}\n\n"
    fi

    # Insert into CHANGELOG.md
    if [ -f "$CHANGELOG_FILE" ]; then
        # Insert after [Unreleased] section
        awk -v entry="$ENTRY" '
            /^## \[Unreleased\]/ {
                print
                print ""
                print entry
                getline
                while (/^$/) { getline }  # Skip empty lines
            }
            { print }
        ' "$CHANGELOG_FILE" > "${CHANGELOG_FILE}.tmp"
        mv "${CHANGELOG_FILE}.tmp" "$CHANGELOG_FILE"
    else
        echo "# Changelog\n\n$ENTRY" > "$CHANGELOG_FILE"
    fi

    log_info "Generated changelog entry"
    log_info "Please review and update $CHANGELOG_FILE before committing"
}

commit_release() {
    local version=$1

    log_step "Committing release changes..."

    git add "$CARGO_TOML" "$CHANGELOG_FILE"
    git commit -m "chore(release): prepare v$version"

    log_info "Committed release changes"
}

create_tag() {
    local version=$1

    log_step "Creating tag v$version..."

    git tag -a "v$version" -m "Release v$version"

    log_info "Created tag v$version"
}

push_release() {
    local version=$1

    log_step "Pushing release..."

    git push
    git push origin "v$version"

    log_info "Pushed release to remote"
}

print_usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Prepare and create a release for CAI.

OPTIONS:
    -h, --help              Show this help message
    -v, --version VERSION   Specific version to set (e.g., 1.2.3)
    -b, --bump TYPE         Version bump type: major, minor, or patch
    -d, --dry-run           Prepare changes without committing or tagging
    -s, --skip-changelog    Skip changelog generation
    -c, --commit-only       Only commit changes, don't create tag
    -p, --push              Push to remote after creating tag

EXAMPLES:
    $(basename "$0") -b patch              # Bump patch version
    $(basename "$0") -v 1.2.3              # Set specific version
    $(basename "$0") -b minor -p            # Bump minor and push
    $(basename "$0") -b major -d            # Dry run major bump

NOTES:
    - Version must follow semantic versioning (X.Y.Z)
    - Updates version in Cargo.toml workspace
    - Generates changelog from git commits
    - Creates git tag for the release
    - Use --dry-run to preview changes

EOF
}

# Parse arguments
DRY_RUN=false
SKIP_CHANGELOG=false
COMMIT_ONLY=false
PUSH=false
BUMP_TYPE=""
VERSION=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            print_usage
            exit 0
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -b|--bump)
            BUMP_TYPE="$2"
            shift 2
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -s|--skip-changelog)
            SKIP_CHANGELOG=true
            shift
            ;;
        -c|--commit-only)
            COMMIT_ONLY=true
            shift
            ;;
        -p|--push)
            PUSH=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Validate arguments
if [ -z "$VERSION" ] && [ -z "$BUMP_TYPE" ]; then
    log_error "Must specify either --version or --bump"
    print_usage
    exit 1
fi

if [ -n "$VERSION" ] && [ -n "$BUMP_TYPE" ]; then
    log_error "Cannot specify both --version and --bump"
    exit 1
fi

# Main execution
log_info "Starting release preparation..."

check_git_status

CURRENT_VERSION=$(get_current_version)
log_info "Current version: $CURRENT_VERSION"

# Determine new version
if [ -n "$VERSION" ]; then
    validate_version "$VERSION"
    NEW_VERSION="$VERSION"
else
    NEW_VERSION=$(bump_version "$CURRENT_VERSION" "$BUMP_TYPE")
fi

log_info "New version: $NEW_VERSION"

if [ "$DRY_RUN" = true ]; then
    log_warn "DRY RUN MODE - No changes will be made"
    log_info "Would update: $CURRENT_VERSION -> $NEW_VERSION"
    exit 0
fi

# Update version
update_version "$NEW_VERSION"

# Generate changelog
if [ "$SKIP_CHANGELOG" = false ]; then
    # Get previous version from tags
    PREVIOUS_VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    generate_changelog "$NEW_VERSION" "$PREVIOUS_VERSION"
fi

# Commit changes
commit_release "$NEW_VERSION"

# Create tag (unless commit-only)
if [ "$COMMIT_ONLY" = false ]; then
    create_tag "$NEW_VERSION"

    # Push if requested
    if [ "$PUSH" = true ]; then
        push_release "$NEW_VERSION"
    fi
fi

log_info "Release preparation complete!"
log_info "Version: $NEW_VERSION"
log_info "Next steps:"
if [ "$COMMIT_ONLY" = false ] && [ "$PUSH" = false ]; then
    log_info "  1. Review changes: git diff"
    log_info "  2. Push to remote: git push"
    log_info "  3. Push tag: git push origin v$NEW_VERSION"
elif [ "$COMMIT_ONLY" = false ] && [ "$PUSH" = true ]; then
    log_info "  1. Monitor CI/CD pipeline"
    log_info "  2. Check GitHub release"
else
    log_info "  1. Review commit"
    log_info "  2. Create tag: git tag v$NEW_VERSION"
    log_info "  3. Push to remote: git push && git push origin v$NEW_VERSION"
fi
