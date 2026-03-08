# cai-tui

Interactive terminal UI for CAI.

## Overview

`cai-tui` provides a feature-rich terminal user interface for browsing and querying CAI data interactively.

## Features

- **Dashboard**: Overview of recent entries
- **Table View**: Paginated entry table
- **Search**: Filter entries by text
- **Detail View**: Full entry details
- **Keyboard Navigation**: Intuitive controls

## Usage

### Standalone

```bash
cai tui
```

### In Code

```rust
use cai_storage::MemoryStorage;
use cai_tui::run;

let storage = std::sync::Arc::new(MemoryStorage::new());
run(storage).await?;
```

## Keyboard Controls

- `q` / `Ctrl+C` - Quit
- `↑` / `k` - Move up
- `↓` / `j` - Move down
- `Enter` - View details
- `Esc` - Go back
- `/` - Search
- `n` - Next search result
- `N` - Previous search result

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
cai-tui = { path = "../cai-tui" }
```

## Design Decisions

- **ratatui**: Cross-platform TUI framework
- **crossterm**: Terminal handling
- **Event-driven**: Async event processing

## Testing

```bash
cargo test -p cai-tui
```

## License

MIT OR Apache-2.0
