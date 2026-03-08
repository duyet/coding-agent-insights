# @cai/cli

Query and analyze AI coding history from Claude, Codex, Git, and more.

## Installation

```bash
npm install -g @cai/cli
# or
bunx @cai/cli
```

## Usage

```bash
# Query your coding history
cai query "SELECT * FROM entries LIMIT 10"

# Import data
cai ingest claude --path ~/.claude/conversations

# View statistics
cai stats

# Interactive TUI
cai tui

# Web dashboard
cai web
```

## Features

- **SQL-like queries** - Powerful query language for coding history
- **Multi-format output** - JSON, JSONL, CSV, tables, AI-optimized
- **Multiple sources** - Claude Code, Codex CLI, Git repositories
- **Interactive UI** - Terminal UI and web dashboard
- **Fast** - Built in Rust for speed and efficiency

## Commands

| Command | Description |
|---------|-------------|
| `query` | Execute SQL query |
| `ingest` | Import coding history |
| `stats` | Display statistics |
| `tui` | Interactive terminal UI |
| `web` | Web dashboard |

## Documentation

Full documentation at [https://github.com/duyet/coding-agent-insights](https://github.com/duyet/coding-agent-insights)

## License

MIT
