#!/usr/bin/env bash
# Quick version bump script for CAI
# Usage: bump-version.sh [major|minor|patch|VERSION]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

CARGO_TOML="$PROJECT_ROOT/Cargo.toml"
NPM_PACKAGE="$PROJECT_ROOT/npm/cli/package.json"

get_current_version() {
    grep -m 1 '^version =' "$CARGO_TOML" | sed 's/version = "\(.*\)"/\1/'
}

bump_version() {
    local current=$1
    local type=$2

    local major=$(echo "$current" | cut -d. -f1)
    local minor=$(echo "$current" | cut -d. -f2)
    local patch=$(echo "$current" | cut -d. -f3 | cut -d- -f1)

    case $type in
        major) major=$((major + 1)); minor=0; patch=0 ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        patch) patch=$((patch + 1)) ;;
        *) echo "$current" ;;
    esac

    echo "${major}.${minor}.${patch}"
}

update_cargo_version() {
    local version=$1
    sed -i.bak "s/^version = \".*\"/version = \"$version\"/" "$CARGO_TOML"
    rm -f "${CARGO_TOML}.bak"
}

update_npm_version() {
    local version=$1
    if [ -f "$NPM_PACKAGE" ]; then
        sed -i.bak "s/\"version\": \".*\"/\"version\": \"$version\"/" "$NPM_PACKAGE"
        rm -f "${NPM_PACKAGE}.bak"
    fi
}

# Main
CURRENT=$(get_current_version)
NEW_VERSION="${1:-}"

if [[ -z "$NEW_VERSION" ]]; then
    echo "Current version: $CURRENT"
    echo "Usage: $0 [major|minor|patch|VERSION]"
    exit 1
fi

# Check if it's a bump type or specific version
if [[ "$NEW_VERSION" =~ ^(major|minor|patch)$ ]]; then
    NEW_VERSION=$(bump_version "$CURRENT" "$NEW_VERSION")
else
    # Validate semver
    if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        echo "Invalid semantic version: $NEW_VERSION"
        exit 1
    fi
fi

echo "Bumping version: $CURRENT -> $NEW_VERSION"
update_cargo_version "$NEW_VERSION"
update_npm_version "$NEW_VERSION"

echo "Version updated to $NEW_VERSION"
echo "Run 'scripts/release.sh' to continue with release preparation"
