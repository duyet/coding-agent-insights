# cai-tui - Claude Code Instructions

## Crate Purpose

`cai-tui` provides an interactive terminal UI for browsing CAI data:

- Query interface with history navigation
- Scrollable results table with sorting
- Real-time status updates
- Keyboard shortcuts (q=quit, /=search, etc.)

## Architecture

### Module Structure

```
src/
├── lib.rs     - Public API, run() function
├── app.rs     - App struct, state management
├── ui.rs      - Terminal UI rendering (init, draw, restore)
└── event.rs   - EventHandler, key bindings
```

### App State

```rust
pub struct App<S: Storage + ?Sized> {
    pub mode: Mode,              // Normal, Query, Search, Detail, Help
    pub state: AppState,         // Running, Quitting
    pub entries: Vec<Entry>,     // Query results
    pub selected: usize,         // Selected entry index
    pub scroll: usize,           // Scroll offset
    pub sort_column: Column,     // Current sort column
    pub sort_order: SortOrder,   // Asc/Desc
    pub status_message: String,  // Status bar message
    // ... more fields
}
```

### Modes

- **Normal** - Viewing results table
- **Query** - Inputting SQL query
- **Search** - Searching within results
- **Detail** - Viewing selected entry
- **Help** - Showing help screen

## Common Tasks

### Adding a New Keyboard Shortcut

1. Define key in `event.rs::handle_key()`:
```rust
Event::Key(key) => match key.code {
    KeyCode::Char('k') => {
        app.move_up();
    }
    // ... add your key here
}
```

2. Implement handler in `app.rs`:
```rust
impl<S: Storage> App<S> {
    pub fn my_action(&mut self) {
        self.status_message = "Action performed".to_string();
        self.status_color = Color::Green;
        self.update_status_timestamp();
    }
}
```

### Adding a New Widget

1. Add widget drawing to `ui.rs::draw()`:
```rust
fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(1),  // Status
        ])
        .split(f.size());

    draw_header(f, chunks[0], app);
    draw_my_widget(f, chunks[1], app);  // Your widget
    draw_status(f, chunks[2], app);
}
```

2. Create widget function:
```rust
fn draw_my_widget<B: Backend>(f: &mut Frame<B>, area: Rect, app: &mut App) {
    let widget = Paragraph::new("My Widget Content")
        .block(Block::default()
            .title("My Widget")
            .borders(Borders::ALL));

    f.render_widget(widget, area);
}
```

### Customizing Layout

Edit `ui.rs::draw()` layout constraints:

```rust
let chunks = Layout::default()
    .direction(Direction::Horizontal)  // Change direction
    .constraints([
        Constraint::Percentage(30),  // Left panel
        Constraint::Percentage(70),  // Right panel
    ])
    .split(f.size());
```

## Testing Patterns

### Unit Tests for App Logic

```rust
#[test]
fn test_sort_entries() {
    let storage = Arc::new(MockStorage::new());
    let mut app = App::new(storage);

    app.entries = vec![ /* test entries */ ];
    app.sort_column = Column::Timestamp;
    app.sort_order = SortOrder::Desc;
    app.sort_entries();

    assert!(app.entries[0].timestamp > app.entries[1].timestamp);
}
```

### Event Handling Tests

```rust
#[test]
fn test_handle_key_quit() {
    let mut app = App::new(storage);
    app.handle_key(Event::Key(KeyCode::Char('q')));
    assert_eq!(app.state, AppState::Quitting);
}
```

## Keyboard Shortcuts Reference

| Key | Action |
|-----|--------|
| q / Ctrl+C | Quit |
| ↑ / k | Move up |
| ↓ / j | Move down |
| Enter | View details / submit query |
| Esc | Go back / cancel |
| / | Search |
| n | Next search result |
| N | Previous search result |
| ? | Help |

## Event Flow

```
EventHandler::poll()
    ↓
Event::Key(KeyCode)
    ↓
App::handle_key()
    ↓
State update + status_message
    ↓
ui.rs::draw() renders new state
```

## Dependencies

- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal
- `tokio` - Async runtime
- `cai-storage` - Storage trait

## Performance Notes

- Re-render on every key press
- Use lazy evaluation for expensive queries
- Scroll offset limits visible items
- History is kept in memory

## Getting Help

- See `README.md` for keyboard shortcuts
- Check `app.rs` for state management
- Review `ui.rs` for rendering patterns
- Look at `event.rs` for input handling
