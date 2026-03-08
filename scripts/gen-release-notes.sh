#!/usr/bin/env bash
# Generate release notes from CHANGELOG for GitHub releases
# Usage: gen-release-notes.sh [VERSION]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

CHANGELOG="$PROJECT_ROOT/CHANGELOG.md"
VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
    # Try to get version from git tag
    VERSION=$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//')
    if [[ -z "$VERSION" ]]; then
        echo "Usage: $0 [VERSION]"
        exit 1
    fi
fi

# Remove 'v' prefix if present
VERSION="${VERSION#v}"

echo "Generating release notes for v${VERSION}..."
echo ""

# Extract the version section from CHANGELOG
awk -v version="$VERSION" '
    BEGIN { found = 0; printing = 0; skipped = 0 }

    # Find the version header (with optional date)
    /^## \[/ {
        if (index($0, "[" version "]") > 0) {
            found = 1
            printing = 1
            next
        }
    }

    # Stop at next version header or development phases
    (/^## \[/ && !index($0, "[" version "]")) && found && printing {
        exit
    }
    /^## Development Phases/ && found && printing {
        exit
    }

    # Print content while in the section (skip empty lines at start)
    printing {
        if (NF > 0 || skipped > 0) {
            print
            skipped++
        }
    }
' "$CHANGELOG"

echo ""
echo "---"
echo ""
echo "## Installation"
echo ""
echo "\`\`\`bash"
echo "# Via cargo"
echo "cargo install cai-cli --version ${VERSION}"
echo ""
echo "# Via NPM"
echo "npm install -g @cai/cli"
echo ""
echo "# Via Homebrew (coming soon)"
echo "brew install cai"
echo "\`\`\`"
