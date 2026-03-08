# CAI Homebrew Tap

This tap contains the formula for installing [CAI (Coding Agent Insights)](https://github.com/duyet/coding-agent-insights).

## Installation

```bash
# Install CAI using this tap
brew install duyet/tap/cai

# OR tap and install
brew tap duyet/tap
brew install cai
```

## Usage

```bash
# Show help
cai help

# Query your coding history
cai query "SELECT * FROM entries LIMIT 10"

# Start TUI
cai tui

# Start web dashboard
cai web
```

## Documentation

See [CAI README](https://github.com/duyet/coding-agent-insights#readme) for full documentation.

## Formula

The `cai.rb` formula is generated from the CAI repository. It:
- Downloads the release tarball from GitHub
- Builds the CLI using Cargo
- Installs the `cai` binary to your Homebrew prefix

## Updating

```bash
brew upgrade cai
```

## Development

To update the formula for a new version:

1. Update the `url` and `sha256` in `cai.rb`
2. Test locally: `brew install --build-from-source ./cai.rb`
3. Commit and push to this tap
