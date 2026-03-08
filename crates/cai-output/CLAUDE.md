# cai-output - Claude Code Instructions

## Crate Purpose

`cai-output` provides flexible output formatting for CAI entries:

- Formatter trait for pluggable output formats
- Built-in formatters: JSON, JSONL, CSV, Table, AI, Stats
- Streaming support for large datasets
- Configurable truncation and limits

## Architecture

### Module Structure

```
src/
├── lib.rs        - Public exports
├── formatter.rs  - Formatter trait, FormatterConfig
└── formats.rs    - Built-in formatter implementations
```

### Formatter Trait

Core abstraction:

```rust
pub trait Formatter: Send + Sync {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()>;
    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()>;
    fn config(&self) -> &FormatterConfig;
    fn set_config(&mut self, config: FormatterConfig);
}
```

### FormatterConfig

```rust
pub struct FormatterConfig {
    pub max_width: usize,    // Table width (0 = no limit)
    pub colorize: bool,       // Enable colors
    pub truncate: usize,      // Truncate length (0 = no truncation)
    pub limit: usize,         // Entry limit (0 = all)
}
```

## Common Tasks

### Creating a Custom Formatter

Implement the `Formatter` trait:

```rust
use cai_output::{Formatter, FormatterConfig};
use cai_core::Entry;
use std::io::Write;

pub struct MyFormatter {
    config: FormatterConfig,
}

impl MyFormatter {
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
        }
    }
}

impl Formatter for MyFormatter {
    fn format<W: Write>(&self, entries: &[Entry], writer: &mut W) -> Result<()> {
        for entry in entries {
            self.format_one(entry, writer)?;
        }
        Ok(())
    }

    fn format_one<W: Write>(&self, entry: &Entry, writer: &mut W) -> Result<()> {
        writeln!(writer, "{}: {}", entry.id, entry.prompt)?;
        Ok(())
    }

    fn config(&self) -> &FormatterConfig {
        &self.config
    }

    fn set_config(&mut self, config: FormatterConfig) {
        self.config = config;
    }
}
```

### Adding Streaming Support

Use `format_one()` for streaming:

```rust
let formatter = JsonlFormatter::default();

// Stream entries one at a time
for entry in entry_stream {
    formatter.format_one(&entry, &mut writer)?;
}
```

### Handling Truncation

Use the `Truncate` helper:

```rust
use cai_output::formatter::FormatterConfig;

let config = FormatterConfig {
    truncate: 100,
    ..Default::default()
};

let truncated = config.truncate_text(&very_long_string, 100);
// Returns "first 97 chars..."
```

## Built-in Formatters

### JsonFormatter
- Outputs array of JSON objects
- Pretty-printed with indentation
- Full entry serialization

### JsonlFormatter
- One JSON object per line
- Streaming-friendly
- No outer array

### CsvFormatter
- Comma-separated values
- Header row
- Escaped special characters

### TableFormatter
- Terminal table layout
- Auto-sizing columns
- Color support

### AiFormatter
- Compact format for AI consumption
- Includes all metadata
- Structured text format

### StatsFormatter
- Summary statistics
- Entry counts by source
- Date ranges

## Testing Patterns

### Test Formatter Output

```rust
#[test]
fn test_json_formatter() {
    let formatter = JsonFormatter::default();
    let entries = vec![test_entry()];

    let mut buffer = Vec::new();
    formatter.format(&entries, &mut buffer).unwrap();

    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("\"id\":"));
}
```

### Test Truncation

```rust
#[test]
fn test_truncation() {
    let config = FormatterConfig {
        truncate: 10,
        ..Default::default()
    };

    let result = config.truncate_text("hello world test", 10);
    assert_eq!(result, "hello w...");
}
```

## Configuration Best Practices

1. **CLI defaults**: Use `FormatterConfig::default()` for CLI
2. **Library usage**: Allow users to pass custom config
3. **Testing**: Use deterministic configs (no colors)
4. **Production**: Set reasonable limits for large outputs

## Dependencies

- `cai-core` - Entry types
- Minimal external dependencies
- `std::io::Write` for output

## Getting Help

- See `README.md` for usage examples
- Check `formats.rs` for implementation patterns
- Review tests for formatter behavior
