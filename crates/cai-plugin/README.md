# cai-plugin

Claude Code plugin interface for CAI.

## Overview

`cai-plugin` provides the plugin interface for integrating CAI with Claude Code. It defines the Plugin trait, manages skill registration, and handles session lifecycle hooks.

## Key Features

- **Plugin trait** - Interface for CAI integration
- **Skill registration** - Define available skills
- **Command handling** - Process CLI commands
- **Session hooks** - Initialize/cleanup on session start/end
- **FFI exports** - WASM-compatible C interface

## Usage

```rust
use cai_plugin::{Plugin, CaiPlugin, PluginConfig};

// Create plugin instance
let plugin = CaiPlugin::new();

// Get plugin configuration
let config = plugin.config();

// Handle skill invocation
let params = serde_json::json!({"sql": "SELECT * FROM entries"});
let result = plugin.handle_skill("cai.query", &params)?;

// Handle command
let result = plugin.handle_command("cai.query", &["--limit", "10"])?;
```

## Available Skills

- **cai.query** - Execute SQL queries
- **cai.ingest** - Ingest data from sources
- **cai.stats** - Get statistics
- **cai.tui** - Launch terminal UI
- **cai.web** - Launch web dashboard

## Plugin Configuration

```rust
let config = PluginConfig {
    version: "0.1.0".to_string(),
    skills: vec!["cai.query".into(), "cai.ingest".into()],
    commands: vec!["cai.query".into(), "cai.ingest".into()],
};
```

## Design Decisions

- **Trait-based** - Easy to implement custom plugins
- **Skill params** - JSON parameters for flexibility
- **Session context** - Access to working directory and environment
- **FFI exports** - Compatible with WASM runtimes

## API Documentation

Full API docs at [docs.rs](https://docs.rs/cai-plugin)

## License

MIT OR Apache-2.0
