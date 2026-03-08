//! Terminal UI rendering and setup
//!
//! Provides terminal initialization, rendering, and restoration.

use crate::{
    app::{App as AppState, Column, Mode},
    event::EventHandler,
};
use crossterm::{
    event::{KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Wrap,
    },
    Frame, Terminal,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Terminal type alias
type Term = Terminal<CrosstermBackend<std::io::Stdout>>;

/// Initialize the terminal for TUI
///
/// # Errors
///
/// Returns an error if terminal initialization fails
pub fn init_terminal() -> Result<Term, std::io::Error> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore terminal to original state
///
/// # Errors
///
/// Returns an error if terminal restoration fails
pub fn restore_terminal(mut terminal: Term) -> Result<(), std::io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Run the main application loop
pub async fn run_app<S>(
    terminal: &mut Term,
    app: &mut Arc<RwLock<AppState<S>>>,
    mut event_handler: EventHandler,
) -> cai_core::Result<()>
where
    S: cai_storage::Storage,
{
    // Load initial data
    {
        let mut a = app.write().await;
        a.execute_query("SELECT * FROM entries").await;
    }

    loop {
        // Check if should quit
        {
            let a = app.read().await;
            if a.state == crate::AppState::Quitting {
                return Ok(());
            }
        }

        // Draw UI
        terminal.draw(|f| {
            let rt = tokio::runtime::Handle::current();
            let a = rt.block_on(app.read());
            ui(f, &a);
        })?;

        // Handle events
        let event = event_handler.next().await;

        let mut a = app.write().await;

        match event {
            crate::Event::Key(key) => {
                handle_key_event(&mut a, key);
            }
            crate::Event::Tick => {
                // Auto-clear status message
                if a.should_clear_status() {
                    a.reset_status();
                }
            }
        }
    }
}

/// Handle keyboard events
fn handle_key_event<S>(app: &mut AppState<S>, key: KeyEvent)
where
    S: cai_storage::Storage,
{
    match app.mode {
        Mode::Query => handle_query_mode(app, key),
        Mode::Search => handle_search_mode(app, key),
        Mode::Normal => handle_normal_mode(app, key),
        Mode::Detail => handle_detail_mode(app, key),
        Mode::Help => handle_help_mode(app, key),
    }
}

/// Handle key events in normal mode
fn handle_normal_mode<S>(app: &mut AppState<S>, key: KeyEvent)
where
    S: cai_storage::Storage,
{
    match key.code {
        KeyCode::Char('q') => {
            app.state = crate::AppState::Quitting;
        }
        KeyCode::Char('i') => {
            app.mode = Mode::Query;
            app.set_status("Query mode: Enter SQL query, Esc to cancel, Enter to execute".to_string(), Color::Cyan);
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Search;
            app.search_input.clear();
            app.set_status("Search mode: Type to filter, Esc to cancel".to_string(), Color::Cyan);
        }
        KeyCode::Char('?') => {
            app.mode = Mode::Help;
            app.help_scroll = 0;
            app.set_status("Help: Press Esc or q to close".to_string(), Color::Cyan);
        }
        KeyCode::Enter => {
            if app.selected_entry().is_some() {
                app.mode = Mode::Detail;
                app.detail_scroll_reset();
                app.set_status("Detail view: Press Esc or q to close, arrows to scroll".to_string(), Color::Cyan);
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next(20); // Default height, will be refined
        }
        KeyCode::Char('t') => {
            app.toggle_sort(Column::Timestamp);
        }
        KeyCode::Char('s') => {
            app.toggle_sort(Column::Source);
        }
        KeyCode::Char('p') => {
            app.toggle_sort(Column::Prompt);
        }
        KeyCode::Char('r') => {
            // Refresh data
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(rt) = rt {
                let query = app.query_input.clone();
                rt.block_on(app.execute_query(&query));
            }
        }
        KeyCode::Esc => {
            app.reset_status();
        }
        _ => {}
    }
}

/// Handle key events in query mode
fn handle_query_mode<S>(app: &mut AppState<S>, key: KeyEvent)
where
    S: cai_storage::Storage,
{
    match key.code {
        KeyCode::Enter => {
            if !app.query_input.is_empty() {
                let rt = tokio::runtime::Handle::try_current();
                if let Ok(rt) = rt {
                    let query = app.query_input.clone();
                    rt.block_on(app.execute_query(&query));
                    app.query_input.clear();
                }
            }
            app.mode = Mode::Normal;
        }
        KeyCode::Esc => {
            app.query_input.clear();
            app.history_index = None;
            app.mode = Mode::Normal;
            app.reset_status();
        }
        KeyCode::Up => {
            app.history_previous();
        }
        KeyCode::Down => {
            app.history_next();
        }
        KeyCode::Char(c) => {
            app.query_input.push(c);
        }
        KeyCode::Backspace => {
            app.query_input.pop();
        }
        _ => {}
    }
}

/// Handle key events in search mode
fn handle_search_mode<S>(app: &mut AppState<S>, key: KeyEvent)
where
    S: cai_storage::Storage,
{
    match key.code {
        KeyCode::Enter => {
            app.search();
            app.mode = Mode::Normal;
        }
        KeyCode::Esc => {
            let rt = tokio::runtime::Handle::try_current();
            if let Ok(rt) = rt {
                rt.block_on(app.clear_search());
            }
            app.mode = Mode::Normal;
            app.reset_status();
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
        }
        KeyCode::Backspace => {
            app.search_input.pop();
        }
        _ => {}
    }
}

/// Handle key events in detail mode
fn handle_detail_mode<S>(app: &mut AppState<S>, key: KeyEvent)
where
    S: cai_storage::Storage,
{
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = Mode::Normal;
            app.reset_status();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.detail_scroll_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.detail_scroll_down();
        }
        _ => {}
    }
}

/// Handle key events in help mode
fn handle_help_mode<S>(app: &mut AppState<S>, key: KeyEvent)
where
    S: cai_storage::Storage,
{
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = Mode::Normal;
            app.reset_status();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.help_scroll_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.help_scroll_down();
        }
        _ => {}
    }
}

/// Draw the UI
fn ui<S>(f: &mut Frame, app: &AppState<S>)
where
    S: cai_storage::Storage,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.area());

    // Main content area
    render_main(f, app, chunks[0]);

    // Status bar
    render_status(f, app, chunks[1]);

    // Draw overlays based on mode
    match app.mode {
        Mode::Query => render_query_input(f, app),
        Mode::Search => render_search_input(f, app),
        Mode::Detail => render_detail_view(f, app),
        Mode::Help => render_help_screen(f, app),
        Mode::Normal => {}
    }
}

/// Render main content area
fn render_main<S>(f: &mut Frame, app: &AppState<S>, area: Rect)
where
    S: cai_storage::Storage,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(area);

    // Header
    let header = vec![Line::from(vec![
        Span::styled("CAI", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" - "),
        Span::styled(format!("{} entries", app.entries.len()), Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
        Span::styled(
            format!("Sort: {:?} ({:?})", app.sort_column, app.sort_order),
            Style::default().fg(Color::Yellow),
        ),
    ])];

    let header = Paragraph::new(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center);

    f.render_widget(header, chunks[0]);

    // Results table
    render_results_table(f, app, chunks[1]);
}

/// Render results table
fn render_results_table<S>(f: &mut Frame, app: &AppState<S>, area: Rect)
where
    S: cai_storage::Storage,
{
    let header_cells = ["Timestamp", "Source", "Prompt"]
        .iter()
        .map(|h| {
            let style = if *h == format!("{:?}", app.sort_column) {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            Cell::from(*h).style(style)
        });

    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = app
        .entries
        .iter()
        .enumerate()
        .skip(app.scroll)
        .take(area.height.saturating_sub(3) as usize)
        .map(|(i, entry)| {
            let cells = vec![
                Cell::from(format_timestamp(entry.timestamp)),
                Cell::from(format!("{:?}", entry.source)),
                Cell::from(truncate_string(&entry.prompt, 60)),
            ];
            Row::new(cells).style(app.row_style(i))
        })
        .collect();

    let table = Table::new(rows, [Constraint::Length(20), Constraint::Length(10), Constraint::Min(0)])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut table_state = TableState::default();
    table_state.select(Some(app.selected.saturating_sub(app.scroll)));

    f.render_stateful_widget(table, area, &mut table_state);

    // Render scrollbar
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state = ScrollbarState::new(app.entries.len()).position(app.scroll);
    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin::new(0, 1)),
        &mut scrollbar_state,
    );
}

/// Render status bar
fn render_status<S>(f: &mut Frame, app: &AppState<S>, area: Rect)
where
    S: cai_storage::Storage,
{
    let status = vec![Line::from(vec![
        Span::styled(
            match app.mode {
                Mode::Normal => "NORMAL",
                Mode::Query => "QUERY",
                Mode::Search => "SEARCH",
                Mode::Detail => "DETAIL",
                Mode::Help => "HELP",
            },
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(&app.status_message, Style::default().fg(app.status_color)),
    ])];

    let status_bar = Paragraph::new(status)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Left);

    f.render_widget(status_bar, area);
}

/// Render query input overlay
fn render_query_input<S>(f: &mut Frame, app: &AppState<S>)
where
    S: cai_storage::Storage,
{
    let area = centered_rect(60, 3, f.area());

    f.render_widget(Clear, area);

    let input = Paragraph::new(app.query_input.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Query (SQL)"),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(input, area);
    let cursor_x = area.x + app.query_input.len() as u16 + 1;
    let cursor_y = area.y + 1;
    // Ensure cursor is within bounds
    if cursor_x < area.right() && cursor_y < area.bottom() {
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

/// Render search input overlay
fn render_search_input<S>(f: &mut Frame, app: &AppState<S>)
where
    S: cai_storage::Storage,
{
    let area = centered_rect(60, 3, f.area());

    f.render_widget(Clear, area);

    let input = Paragraph::new(app.search_input.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Search"),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(input, area);
    let cursor_x = area.x + app.search_input.len() as u16 + 1;
    let cursor_y = area.y + 1;
    // Ensure cursor is within bounds
    if cursor_x < area.right() && cursor_y < area.bottom() {
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

/// Helper to create centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

/// Format timestamp for display
fn format_timestamp(ts: chrono::DateTime<chrono::Utc>) -> String {
    ts.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Truncate string to max length
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Render detail view overlay
fn render_detail_view<S>(f: &mut Frame, app: &AppState<S>)
where
    S: cai_storage::Storage,
{
    let area = centered_rect(80, 70, f.area());

    f.render_widget(Clear, area);

    if let Some(entry) = app.selected_entry() {
        let content = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Cyan)),
                Span::raw(&entry.id),
            ]),
            Line::from(vec![
                Span::styled("Source: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{:?}", entry.source)),
            ]),
            Line::from(vec![
                Span::styled("Timestamp: ", Style::default().fg(Color::Cyan)),
                Span::raw(format_timestamp(entry.timestamp)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Prompt:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
        ];

        let mut full_content = content.clone();

        // Add prompt content (split into lines)
        for line in word_wrap(&entry.prompt, 76) {
            full_content.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(line),
            ]));
        }

        full_content.push(Line::from(""));
        full_content.push(Line::from(vec![
            Span::styled("Response:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        full_content.push(Line::from(""));

        // Add response content
        for line in word_wrap(&entry.response, 76) {
            full_content.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(line),
            ]));
        }

        // Add metadata if present
        if entry.metadata.file_path.is_some() || entry.metadata.language.is_some() {
            full_content.push(Line::from(""));
            full_content.push(Line::from(vec![
                Span::styled("Metadata:", Style::default().fg(Color::Cyan)),
            ]));

            if let Some(ref file) = entry.metadata.file_path {
                full_content.push(Line::from(vec![
                    Span::raw("  File: "),
                    Span::raw(file),
                ]));
            }

            if let Some(ref lang) = entry.metadata.language {
                full_content.push(Line::from(vec![
                    Span::raw("  Language: "),
                    Span::raw(lang),
                ]));
            }

            if let Some(ref repo) = entry.metadata.repo_url {
                full_content.push(Line::from(vec![
                    Span::raw("  Repo: "),
                    Span::raw(repo),
                ]));
            }
        }

        let paragraph = Paragraph::new(full_content.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title("Entry Details"),
            )
            .scroll((app.detail_scroll as u16, 0))
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);

        // Render scrollbar if content overflows
        if full_content.len() > area.height as usize {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::new(full_content.len()).position(app.detail_scroll);
            f.render_stateful_widget(
                scrollbar,
                area.inner(Margin::new(0, 1)),
                &mut scrollbar_state,
            );
        }
    } else {
        let no_entry = Paragraph::new("No entry selected")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title("Entry Details"),
            );
        f.render_widget(no_entry, area);
    }
}

/// Render help screen overlay
fn render_help_screen<S>(f: &mut Frame, app: &AppState<S>)
where
    S: cai_storage::Storage,
{
    let area = centered_rect(80, 80, f.area());

    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![
            Span::styled("CAI TUI - Keyboard Shortcuts", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Normal Mode:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  q          - Quit application"),
        Line::from("  i          - Enter query mode"),
        Line::from("  /          - Enter search mode"),
        Line::from("  ?          - Show this help screen"),
        Line::from("  Enter      - View selected entry details"),
        Line::from("  Up/Down    - Navigate entries"),
        Line::from("  j/k        - Navigate entries (vim-style)"),
        Line::from("  t          - Sort by timestamp"),
        Line::from("  s          - Sort by source"),
        Line::from("  p          - Sort by prompt"),
        Line::from("  r          - Refresh data"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Query Mode:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Enter      - Execute query"),
        Line::from("  Esc        - Cancel"),
        Line::from("  Up/Down    - Navigate history"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Search Mode:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Enter      - Apply search"),
        Line::from("  Esc        - Cancel"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Detail View:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Esc/q      - Close detail view"),
        Line::from("  Up/Down    - Scroll content"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Help Screen:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Esc/q      - Close help"),
        Line::from("  Up/Down    - Scroll help"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press Esc or q to close this help screen", Style::default().fg(Color::Yellow)),
        ]),
    ];

    let paragraph = Paragraph::new(help_text.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Help"),
        )
        .scroll((app.help_scroll as u16, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);

    // Render scrollbar if content overflows
    if help_text.len() > area.height as usize {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut scrollbar_state = ScrollbarState::new(help_text.len()).position(app.help_scroll);
        f.render_stateful_widget(
            scrollbar,
            area.inner(Margin::new(0, 1)),
            &mut scrollbar_state,
        );
    }
}

/// Simple word wrap for text
fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_length = 0;

    for word in text.split_whitespace() {
        let word_len = word.len();

        if current_length == 0 {
            current_line = word.to_string();
            current_length = word_len;
        } else if current_length + 1 + word_len <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
            current_length += 1 + word_len;
        } else {
            lines.push(current_line);
            current_line = word.to_string();
            current_length = word_len;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Handle long words that exceed max_width
    let mut result = Vec::new();
    for line in lines {
        if line.len() <= max_width {
            result.push(line);
        } else {
            // Split long words
            for chunk in line.as_bytes().chunks(max_width) {
                result.push(String::from_utf8_lossy(chunk).to_string());
            }
        }
    }

    result
}
