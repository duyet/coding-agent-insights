#!/usr/bin/env bash
# Generate SHA256 checksum for Homebrew formula

set -e

VERSION=${1:-$(grep -m 1 'version =' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')}
DOWNLOAD_URL="https://github.com/duyet/coding-agent-insights/archive/refs/tags/v${VERSION}.tar.gz"
TEMP_FILE="/tmp/cai-${VERSION}.tar.gz"

echo "Fetching CAI v${VERSION} from ${DOWNLOAD_URL}..."
curl -L -o "${TEMP_FILE}" "${DOWNLOAD_URL}"

echo "Calculating SHA256..."
SHA256=$(shasum -a 256 "${TEMP_FILE}" | awk '{print $1}')

echo "SHA256: ${SHA256}"

# Update the formula
sed -i.bak "s/sha256 \".*\"/sha256 \"${SHA256}\"/" homebrew/cai.rb
rm -f homebrew/cai.rb.bak

echo "Updated homebrew/cai.rb with SHA256: ${SHA256}"
rm -f "${TEMP_FILE}"
