# cai-cli - Claude Code Instructions

## Crate Purpose

`cai-cli` is the main command-line interface for CAI. It orchestrates all other crates to provide a unified user experience.

## Architecture

The CLI is built as a thin orchestrator layer:

```
cai-cli (main.rs)
    ├── Parses commands with clap
    ├── Initializes storage
    ├── Delegates to specialized crates
    └── Formats output for user
```

## Command Structure

### Main Commands

```rust
#[derive(Subcommand, Clone)]
enum Commands {
    Query { query: String, output: String },
    Ingest { source: String, path: Option<String> },
    Tui,
    Web { port: u16 },
}
```

### Command Flow

1. **Parse** - clap parses arguments into structured types
2. **Initialize** - Set up storage and dependencies
3. **Execute** - Delegate to appropriate crate
4. **Output** - Format and display results

## Working with Commands

### Adding a New Command

```rust
#[derive(Subcommand, Clone)]
enum Commands {
    // Existing commands...
    /// My new command
    NewCommand {
        /// Description of argument
        #[arg(short, long)]
        arg: String,
    },
}

// In main()
Commands::NewCommand { arg } => {
    // Handle the command
    println!("Executing with: {}", arg);
    Ok(())
}
```

### Adding Arguments to Existing Commands

```rust
Query {
    query: String,
    output: String,
    // Add new argument
    #[arg(short, long, default_value = "100")]
    limit: usize,
}
```

## Storage Initialization

Commands need storage. Current pattern:

```rust
#[tokio::main]
async fn main() -> cai_core::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Query { query, output } => {
            // Initialize in-memory for now
            let storage = std::sync::Arc::new(cai_storage::MemoryStorage::new());

            // TODO: Use persistent storage based on config
            // let storage = std::sync::Arc::new(
            //     cai_storage::SqliteStorage::new("cai.db").await?
            // );

            // Execute query with storage
            // ...
        }
        // ...
    }
}
```

## Error Handling

User-friendly error messages:

```rust
use colored::Colorize;

// In command handlers
Commands::Query { query, output } => {
    match execute_query(&storage, &query).await {
        Ok(results) => {
            println!("{}", "Success".green());
            // Display results
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            std::process::exit(1);
        }
    }
}
```

## Output Formatting

### Table Output (Default)

```rust
use cai_output::TableFormatter;
use colored::*;

println!("{}", "Query Results:".green().bold());
let formatter = TableFormatter::new();
let output = formatter.format(&results)?;
println!("{}", output);
```

### JSON Output

```rust
use cai_output::JsonFormatter;

let formatter = JsonFormatter::new();
let output = formatter.format(&results)?;
println!("{}", output);
```

### Conditional Formatting

```rust
let output: String = match output.as_str() {
    "json" => JsonFormatter::new().format(&results)?,
    "csv" => CsvFormatter::new().format(&results)?,
    "table" => TableFormatter::new().format(&results)?,
    _ => return Err(Error::Message(format!("Unknown output format: {}", output))),
};
```

## Configuration

### Future: Config File

Plan to support `~/.cai/config.toml`:

```toml
[storage]
type = "sqlite"
path = "~/.cai/data.db"

[output]
default_format = "table"
max_rows = 1000

[ingest]
claude_path = "~/.claude/conversations"
codex_path = "~/.codex/history.jsonl"
```

### Environment Variables

```rust
use clap::Parser;

#[derive(Parser, Clone)]
struct Cli {
    #[arg(short, long, env = "CAI_OUTPUT")]
    output: Option<String>,
}
```

## Testing CLI

### Unit Tests

Test command parsing:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_query_command() {
        let cli = Cli::try_parse_from(["cai", "query", "SELECT *"]).unwrap();
        matches!(cli.command, Commands::Query { .. });
    }

    #[test]
    fn test_parse_with_output_flag() {
        let cli = Cli::try_parse_from(["cai", "query", "SELECT *", "--output", "json"]).unwrap();
        if let Commands::Query { output, .. } = cli.command {
            assert_eq!(output, "json");
        }
    }
}
```

### Integration Tests

Test end-to-end flows in `tests/cli_tests.rs`:

```rust
use std::process::Command;

#[test]
fn test_query_command() {
    let output = Command::new("./target/debug/cai")
        .args(["query", "SELECT * FROM history LIMIT 1"])
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
}
```

## Common Patterns

### Async Command Execution

```rust
Commands::Query { query, output } => {
    tokio::spawn(async move {
        // Long-running operation
        execute_query(&storage, &query).await
    });
    Ok(())
}
```

### Progress Indicators

```rust
use indicatif::ProgressBar;

Commands::Ingest { source, path } => {
    let progress = ProgressBar::new_spinner();
    progress.set_message(format!("Ingesting from {}...", source));

    // Do work...

    progress.finish_with_message("Done!");
    Ok(())
}
```

### Multi-Source Ingestion

```rust
Commands::Ingest { source, path } => {
    let parser = match source.as_str() {
        "claude" => cai_ingest::ClaudeParser::new(),
        "codex" => cai_ingest::CodexParser::new(),
        "git" => cai_ingest::GitParser::new(),
        _ => return Err(Error::Message(format!("Unknown source: {}", source))),
    };

    let path = path.unwrap_or_else(|| default_path(&source));
    let entries = parser.parse(&path)?;
    // ...
}
```

## Debugging

### Enable Logging

```bash
RUST_LOG=cai_cli=debug cai query "SELECT *"
```

### Trace All Crates

```bash
RUST_LOG=cai_core=trace,cai_storage=trace,cai_query=trace cai query "SELECT *"
```

### Verbose Mode

Add verbose flag:

```rust
#[derive(Parser, Clone)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

// In main()
let log_level = match cli.verbose {
    0 => tracing::Level::WARN,
    1 => tracing::Level::INFO,
    2 => tracing::Level::DEBUG,
    _ => tracing::Level::TRACE,
};
```

## Release Checklist

Before releasing cai-cli:

1. **Test all commands** - Manual testing of each command
2. **Help text** - Verify `cai --help` and `cai command --help`
3. **Error messages** - Ensure user-friendly errors
4. **Dependencies** - Check for unused dependencies
5. **Binary size** - Consider `cargo bloat` for size analysis

## Dependencies

`cai-cli` depends on:
- `cai-core` - Shared types
- `cai-ingest` - Data ingestion
- `cai-query` - Query engine
- `cai-storage` - Storage backends
- `cai-output` - Output formatters
- `cai-tui` - Terminal UI
- `cai-web` - Web dashboard

Avoid adding heavy CLI libraries. Use `clap` for parsing, `colored` for colors.

## Getting Help

- See crate `README.md` for usage examples
- Check `src/main.rs` for command implementations
- Review clap documentation for advanced parsing
