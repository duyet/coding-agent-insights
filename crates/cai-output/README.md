# cai-output

Output formatters for CAI entries in multiple formats.

## Overview

`cai-output` provides flexible output formatting for entries, supporting JSON, JSONL, CSV, tables, and more.

## Formatters

### JSON Formatter

```rust
use cai_output::JsonFormatter;
use cai_core::Entry;

let formatter = JsonFormatter::default();
formatter.format(&entries, &mut writer)?;
```

### JSONL Formatter (JSON Lines)

```rust
use cai_output::JsonlFormatter;

let formatter = JsonlFormatter::default();
formatter.format(&entries, &mut writer)?;
```

### CSV Formatter

```rust
use cai_output::CsvFormatter;

let formatter = CsvFormatter::default();
formatter.format(&entries, &mut writer)?;
```

### Table Formatter

```rust
use cai_output::TableFormatter;

let formatter = TableFormatter::default();
formatter.format(&entries, &mut writer)?;
```

### AI Formatter

Compact format optimized for AI consumption:

```rust
use cai_output::AiFormatter;

let formatter = AiFormatter::default();
formatter.format(&entries, &mut writer)?;
```

### Stats Formatter

Summary statistics:

```rust
use cai_output::StatsFormatter;

let formatter = StatsFormatter::default();
formatter.format(&entries, &mut writer)?;
```

## Configuration

Formatters support configuration:

```rust
use cai_output::{Formatter, FormatterConfig};

let mut config = FormatterConfig {
    max_width: 120,
    colorize: true,
    truncate: 100,
    limit: 50,
};

formatter.set_config(config);
```

## Streaming

Format entries one at a time:

```rust
formatter.format_one(&entry, &mut writer)?;
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cai-output = { path = "../cai-output" }
```

## Design Decisions

- **Generic over Write**: Works with any writer
- **Streaming support**: Handle large datasets efficiently
- **Configurable**: Customizable truncation and limits

## Testing

```bash
cargo test -p cai-output
```

## License

MIT OR Apache-2.0
