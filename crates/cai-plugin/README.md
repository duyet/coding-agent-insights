# cai-plugin

Claude Code plugin for CAI.

## Overview

`cai-plugin` provides a WASM-based plugin for Claude Code, enabling CAI functionality directly within the Claude Code environment.

## Features

- **Query Execution**: Run CAI queries from Claude Code
- **Skill Integration**: CAI skills for common tasks
- **Data Ingestion**: Ingest conversations into CAI

## Installation

### Via Claude Code Plugin Marketplace

```bash
/plugin marketplace add cai-dev/coding-agent-insights
/plugin install cai
```

### Via NPM

```bash
npm install -g @cai/plugin
```

## Usage

### In Claude Code

```
/cai query "SELECT * FROM entries LIMIT 10"
/cai ingest
/cai stats
```

## Development

### Building WASM

```bash
cargo build --release -p cai-plugin
wasm-bindgen --out-dir ./pkg --target web
```

### Testing

```bash
cargo test -p cai-plugin
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cai-plugin = { path = "../cai-plugin" }
```

## Design Decisions

- **WASM**: Portable, sandboxed execution
- **wit-bindgen**: WebAssembly interface
- **NPM**: Easy distribution and installation

## License

MIT OR Apache-2.0
