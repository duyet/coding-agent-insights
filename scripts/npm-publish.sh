#!/usr/bin/env bash
# Publish NPM package for CAI CLI

set -e

VERSION=${1:-$(grep '"version"' npm/cli/package.json | head -1 | sed 's/.*": "\(.*\)".*/\1/')}

echo "Publishing @cai/cli v${VERSION}..."

# Navigate to NPM package directory
cd npm/cli

# Run dry-run first
echo "Running dry-run..."
npm publish --dry-run

# Ask for confirmation
read -p "Publish to npm? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Publish to npm
    npm publish

    echo "Published @cai/cli@${VERSION} successfully!"
    echo "Install with: npm install -g @cai/cli"
else
    echo "Publishing cancelled."
fi
