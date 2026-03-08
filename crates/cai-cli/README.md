# cai-cli

Command-line interface for CAI.

## Overview

`cai-cli` provides the main `cai` command-line tool for interacting with CAI functionality.

## Installation

```bash
# From crates.io
cargo install cai-cli

# From source
cargo build --release --bin cai
```

## Usage

### Query Command

```bash
cai query "SELECT * FROM entries LIMIT 10"
cai query --output json "SELECT * FROM entries"
cai query --output csv "SELECT * FROM entries"
```

### Ingest Command

```bash
cai ingest --source claude --path ~/.claude/conversations
cai ingest --source codex --path ~/.codex/history.jsonl
cai ingest --source git --path /path/to/repo
```

### Interactive TUI

```bash
cai tui
```

### Web Dashboard

```bash
cai web --port 3000
```

## Output Formats

- `table` - Pretty terminal table (default)
- `json` - JSON array
- `jsonl` - JSON Lines (streaming)
- `csv` - CSV with headers
- `ai` - AI-optimized compact format
- `stats` - Summary statistics

## Usage in Code

```rust
use cai_cli::Cli;
use clap::Parser;

let cli = Cli::parse();
// Handle commands...
```

## Design Decisions

- **clap**: Type-safe argument parsing
- **colored**: User-friendly colored output
- **async**: Non-blocking command execution

## License

MIT OR Apache-2.0
