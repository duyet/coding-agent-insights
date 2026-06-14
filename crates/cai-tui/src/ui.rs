use crate::{
    app::{Action, App as AppState, Column, Mode, Theme},
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

type Term = Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn init_terminal() -> Result<Term, std::io::Error> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn restore_terminal(mut terminal: Term) -> Result<(), std::io::Error> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

pub async fn run_app<S>(
    terminal: &mut Term,
    app: &mut Arc<RwLock<AppState<S>>>,
    mut event_handler: EventHandler,
) -> cai_core::Result<()>
where
    S: cai_storage::Storage,
{
    let theme = Theme::default();

    {
        let mut a = app.write().await;
        a.execute_query("SELECT * FROM entries").await;
    }

    loop {
        {
            let a = app.read().await;
            if a.state == crate::AppState::Quitting {
                return Ok(());
            }
        }

        terminal.draw(|f| {
            let a = app.try_read();
            if let Ok(a) = a {
                ui(f, &a, &theme);
            }
        })?;

        let event = event_handler.next().await;

        let action = match event {
            crate::Event::Key(key) => {
                let mut a = app.write().await;
                handle_key_event(&mut a, key, &theme)
            }
            crate::Event::Tick => {
                let mut a = app.write().await;
                if a.should_clear_status() {
                    a.reset_status();
                }
                Action::None
            }
        };

        match action {
            Action::ExecuteQuery(query) => {
                let mut a = app.write().await;
                a.execute_query(&query).await;
            }
            Action::ClearSearch => {
                let mut a = app.write().await;
                a.clear_search().await;
            }
            Action::None => {}
        }
    }
}

fn handle_key_event<S>(app: &mut AppState<S>, key: KeyEvent, _theme: &Theme) -> Action
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

fn handle_normal_mode<S>(app: &mut AppState<S>, key: KeyEvent) -> Action
where
    S: cai_storage::Storage,
{
    match key.code {
        KeyCode::Char('q') => {
            app.state = crate::AppState::Quitting;
            Action::None
        }
        KeyCode::Char('i') => {
            app.mode = Mode::Query;
            app.set_status(
                "Enter SQL, Esc to cancel, Enter to execute".into(),
                Color::Cyan,
            );
            Action::None
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Search;
            app.search_input.clear();
            app.set_status("Type to filter entries, Esc to cancel".into(), Color::Cyan);
            Action::None
        }
        KeyCode::Char('?') => {
            app.mode = Mode::Help;
            app.help_scroll = 0;
            app.set_status("Help — Esc or q to close".into(), Color::Cyan);
            Action::None
        }
        KeyCode::Enter => {
            if app.selected_entry().is_some() {
                app.mode = Mode::Detail;
                app.detail_scroll_reset();
                app.set_status("Esc or q to close, arrows to scroll".into(), Color::Cyan);
            }
            Action::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
            Action::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next(20);
            Action::None
        }
        KeyCode::Char('t') => {
            app.toggle_sort(Column::Timestamp);
            Action::None
        }
        KeyCode::Char('s') => {
            app.toggle_sort(Column::Source);
            Action::None
        }
        KeyCode::Char('p') => {
            app.toggle_sort(Column::Prompt);
            Action::None
        }
        KeyCode::Char('r') => {
            let query = app.query_input.clone();
            Action::ExecuteQuery(query)
        }
        KeyCode::Esc => {
            app.reset_status();
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_query_mode<S>(app: &mut AppState<S>, key: KeyEvent) -> Action
where
    S: cai_storage::Storage,
{
    match key.code {
        KeyCode::Enter => {
            if !app.query_input.is_empty() {
                let query = app.query_input.clone();
                app.query_input.clear();
                app.mode = Mode::Normal;
                return Action::ExecuteQuery(query);
            }
            app.mode = Mode::Normal;
            Action::None
        }
        KeyCode::Esc => {
            app.query_input.clear();
            app.history_index = None;
            app.mode = Mode::Normal;
            app.reset_status();
            Action::None
        }
        KeyCode::Up => {
            app.history_previous();
            Action::None
        }
        KeyCode::Down => {
            app.history_next();
            Action::None
        }
        KeyCode::Char(c) => {
            app.query_input.push(c);
            Action::None
        }
        KeyCode::Backspace => {
            app.query_input.pop();
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_search_mode<S>(app: &mut AppState<S>, key: KeyEvent) -> Action
where
    S: cai_storage::Storage,
{
    match key.code {
        KeyCode::Enter => {
            app.search();
            app.mode = Mode::Normal;
            Action::None
        }
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.reset_status();
            Action::ClearSearch
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
            Action::None
        }
        KeyCode::Backspace => {
            app.search_input.pop();
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_detail_mode<S>(_app: &mut AppState<S>, key: KeyEvent) -> Action
where
    S: cai_storage::Storage,
{
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            _app.mode = Mode::Normal;
            _app.reset_status();
            Action::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            _app.detail_scroll_up();
            Action::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            _app.detail_scroll_down();
            Action::None
        }
        _ => Action::None,
    }
}

fn handle_help_mode<S>(_app: &mut AppState<S>, key: KeyEvent) -> Action
where
    S: cai_storage::Storage,
{
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            _app.mode = Mode::Normal;
            _app.reset_status();
            Action::None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            _app.help_scroll_up();
            Action::None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            _app.help_scroll_down();
            Action::None
        }
        _ => Action::None,
    }
}

fn ui<S>(f: &mut Frame, app: &AppState<S>, theme: &Theme)
where
    S: cai_storage::Storage,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(f.area());

    render_main(f, app, chunks[0], theme);
    render_status(f, app, chunks[1], theme);

    match app.mode {
        Mode::Query => render_query_input(f, app, theme),
        Mode::Search => render_search_input(f, app, theme),
        Mode::Detail => render_detail_view(f, app, theme),
        Mode::Help => render_help_screen(f, app, theme),
        Mode::Normal => {}
    }
}

fn render_main<S>(f: &mut Frame, app: &AppState<S>, area: Rect, theme: &Theme)
where
    S: cai_storage::Storage,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(area);

    let sort_label = format!("{:?}", app.sort_column);
    let order_icon = match app.sort_order {
        crate::SortOrder::Asc => " ▲",
        crate::SortOrder::Desc => " ▼",
    };

    let header = vec![Line::from(vec![
        Span::styled(
            "CAI",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" · "),
        Span::styled(
            format!("{} entries", app.entries.len()),
            Style::default().fg(theme.secondary),
        ),
        Span::raw(" · "),
        Span::styled(
            format!("sorted by {}{}", sort_label.to_lowercase(), order_icon),
            Style::default().fg(theme.accent),
        ),
    ])];

    let header = Paragraph::new(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        )
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    if app.entries.is_empty() {
        render_empty_state(f, app, chunks[1], theme);
    } else {
        render_results_table(f, app, chunks[1], theme);
    }
}

fn render_empty_state<S>(f: &mut Frame, _app: &AppState<S>, area: Rect, theme: &Theme)
where
    S: cai_storage::Storage,
{
    let msg = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "No entries found",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  i  Enter query mode",
            Style::default().fg(theme.dim),
        )]),
        Line::from(vec![Span::styled(
            "  r  Refresh from storage",
            Style::default().fg(theme.dim),
        )]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    )
    .alignment(Alignment::Center);
    f.render_widget(msg, area);
}

fn render_results_table<S>(f: &mut Frame, app: &AppState<S>, area: Rect, theme: &Theme)
where
    S: cai_storage::Storage,
{
    let sort_col_name = format!("{:?}", app.sort_column);
    let header_cells = ["Timestamp", "Source", "Prompt"].iter().map(|h| {
        let is_active = *h == sort_col_name;
        let style = if is_active {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.dim)
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
            let is_selected = i == app.selected;
            let bg = if is_selected {
                theme.highlight
            } else if i % 2 == 0 {
                theme.row_even
            } else if theme.row_odd != Color::Reset {
                theme.row_odd
            } else {
                Color::Reset
            };
            let row_style = Style::default().bg(bg);
            let cells = vec![
                Cell::from(format_timestamp(entry.timestamp)),
                Cell::from(Span::styled(
                    format!("{:?}", entry.source),
                    Style::default().fg(theme.secondary),
                )),
                Cell::from(truncate_string(&entry.prompt, 60)),
            ];
            Row::new(cells).style(row_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );

    let mut table_state = TableState::default();
    table_state.select(Some(app.selected.saturating_sub(app.scroll)));

    f.render_stateful_widget(table, area, &mut table_state);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
    let mut scrollbar_state = ScrollbarState::new(app.entries.len()).position(app.scroll);
    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin::new(0, 1)),
        &mut scrollbar_state,
    );
}

fn render_status<S>(f: &mut Frame, app: &AppState<S>, area: Rect, theme: &Theme)
where
    S: cai_storage::Storage,
{
    fn active_label<'a>(label: &'a str, color: Color, is_active: bool) -> Span<'a> {
        if is_active {
            Span::styled(
                label,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(label, Style::default().fg(Color::Gray))
        }
    }

    fn key_hint<'a>(keys: &'a str, label: &'a str, accent: Color, dim: Color) -> Vec<Span<'a>> {
        vec![
            Span::styled(keys, Style::default().fg(accent)),
            Span::styled(label, Style::default().fg(dim)),
        ]
    }

    let dim = theme.dim;
    let accent = theme.accent;

    let mode_label: Vec<Span> = match app.mode {
        Mode::Normal => {
            let mut sp = vec![active_label(
                " NORMAL ",
                theme.primary,
                app.mode == Mode::Normal,
            )];
            sp.push(Span::raw("│"));
            sp.extend(key_hint(" i", "query ", accent, dim));
            sp.extend(key_hint(" /", "search ", accent, dim));
            sp.extend(key_hint(" ?", "help ", accent, dim));
            sp.extend(key_hint(" r", "refresh ", accent, dim));
            sp.extend(key_hint(" q", "quit", accent, dim));
            sp
        }
        Mode::Query => {
            let mut sp = vec![active_label(" QUERY ", Color::Cyan, true)];
            if !app.query_input.is_empty() {
                sp.push(Span::raw("│ "));
                sp.push(Span::raw(&app.query_input));
            }
            sp
        }
        Mode::Search => {
            let mut sp = vec![active_label(" SEARCH ", Color::Magenta, true)];
            if !app.search_input.is_empty() {
                sp.push(Span::raw("│ "));
                sp.push(Span::raw(&app.search_input));
            }
            sp
        }
        Mode::Detail => {
            let mut sp = vec![active_label(" DETAIL ", Color::Blue, true)];
            sp.push(Span::raw("│"));
            sp.extend(key_hint(" ↑↓", "scroll ", accent, dim));
            sp.extend(key_hint(" Esc", "back", accent, dim));
            sp
        }
        Mode::Help => {
            let mut sp = vec![active_label(" HELP ", Color::Yellow, true)];
            sp.push(Span::raw("│"));
            sp.extend(key_hint(" ↑↓", "scroll ", accent, dim));
            sp.extend(key_hint(" Esc", "close", accent, dim));
            sp
        }
    };

    let status_bar = Paragraph::new(Line::from(mode_label))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.status_border)),
        )
        .alignment(Alignment::Left);

    f.render_widget(status_bar, area);
}

fn render_query_input<S>(f: &mut Frame, app: &AppState<S>, theme: &Theme)
where
    S: cai_storage::Storage,
{
    let area = centered_rect(60, 3, f.area());
    f.render_widget(Clear, area);

    let input = Paragraph::new(app.query_input.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary))
                .title(" Query "),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(input, area);

    let cursor_x = area.x + app.query_input.len() as u16 + 1;
    let cursor_y = area.y + 1;
    if cursor_x < area.right() && cursor_y < area.bottom() {
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn render_search_input<S>(f: &mut Frame, app: &AppState<S>, theme: &Theme)
where
    S: cai_storage::Storage,
{
    let area = centered_rect(60, 3, f.area());
    f.render_widget(Clear, area);

    let input = Paragraph::new(app.search_input.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary))
                .title(" Search "),
        )
        .wrap(Wrap { trim: false });
    f.render_widget(input, area);

    let cursor_x = area.x + app.search_input.len() as u16 + 1;
    let cursor_y = area.y + 1;
    if cursor_x < area.right() && cursor_y < area.bottom() {
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

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

fn format_timestamp(ts: chrono::DateTime<chrono::Utc>) -> String {
    ts.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn render_detail_view<S>(f: &mut Frame, app: &AppState<S>, theme: &Theme)
where
    S: cai_storage::Storage,
{
    let area = centered_rect(80, 70, f.area());
    f.render_widget(Clear, area);

    if let Some(entry) = app.selected_entry() {
        let meta_style = Style::default().fg(theme.secondary);
        let label_style = Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD);

        let mut lines = vec![
            Line::from(vec![Span::styled("ID: ", meta_style), Span::raw(&entry.id)]),
            Line::from(vec![
                Span::styled("Source: ", meta_style),
                Span::raw(format!("{:?}", entry.source)),
            ]),
            Line::from(vec![
                Span::styled("Timestamp: ", meta_style),
                Span::raw(format_timestamp(entry.timestamp)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("Prompt:", label_style)]),
            Line::from(""),
        ];

        for line in word_wrap(&entry.prompt, 76) {
            lines.push(Line::from(vec![Span::raw("  "), Span::raw(line)]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled("Response:", label_style)]));
        lines.push(Line::from(""));

        for line in word_wrap(&entry.response, 76) {
            lines.push(Line::from(vec![Span::raw("  "), Span::raw(line)]));
        }

        if entry.metadata.file_path.is_some() || entry.metadata.language.is_some() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled("Metadata:", meta_style)]));
            if let Some(ref file) = entry.metadata.file_path {
                lines.push(Line::from(vec![Span::raw("  File: "), Span::raw(file)]));
            }
            if let Some(ref lang) = entry.metadata.language {
                lines.push(Line::from(vec![Span::raw("  Language: "), Span::raw(lang)]));
            }
            if let Some(ref repo) = entry.metadata.repo_url {
                lines.push(Line::from(vec![Span::raw("  Repo: "), Span::raw(repo)]));
            }
        }

        let paragraph = Paragraph::new(lines.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.secondary))
                    .title(" Entry Details "),
            )
            .scroll((app.detail_scroll as u16, 0))
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);

        if lines.len() > area.height as usize {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::new(lines.len()).position(app.detail_scroll);
            f.render_stateful_widget(
                scrollbar,
                area.inner(Margin::new(0, 1)),
                &mut scrollbar_state,
            );
        }
    } else {
        let no_entry = Paragraph::new("No entry selected").block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary))
                .title(" Entry Details "),
        );
        f.render_widget(no_entry, area);
    }
}

fn render_help_screen<S>(f: &mut Frame, app: &AppState<S>, theme: &Theme)
where
    S: cai_storage::Storage,
{
    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    fn section_line<'a>(label: &'a str, secondary: Color) -> Line<'a> {
        Line::from(vec![Span::styled(
            label,
            Style::default().fg(secondary).add_modifier(Modifier::BOLD),
        )])
    }
    fn cmd_line<'a>(keys: &'a str, desc: &'a str, accent: Color, dim: Color) -> Line<'a> {
        Line::from(vec![
            Span::styled(format!("  {:<12}", keys), Style::default().fg(accent)),
            Span::styled(desc, Style::default().fg(dim)),
        ])
    }

    let help_text = vec![
        Line::from(vec![Span::styled(
            " CAI TUI — Keyboard Shortcuts ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        section_line("Normal Mode", theme.secondary),
        cmd_line("i", "Enter query mode", theme.accent, theme.dim),
        cmd_line("/", "Search / filter", theme.accent, theme.dim),
        cmd_line("?", "Help screen", theme.accent, theme.dim),
        cmd_line(
            "Enter",
            "View selected entry details",
            theme.accent,
            theme.dim,
        ),
        cmd_line("↑/k, ↓/j", "Navigate entries", theme.accent, theme.dim),
        cmd_line("t", "Sort by timestamp", theme.accent, theme.dim),
        cmd_line("s", "Sort by source", theme.accent, theme.dim),
        cmd_line("p", "Sort by prompt", theme.accent, theme.dim),
        cmd_line("r", "Refresh data", theme.accent, theme.dim),
        cmd_line("q", "Quit", theme.accent, theme.dim),
        Line::from(""),
        section_line("Query Mode", theme.secondary),
        cmd_line("Enter", "Execute query", theme.accent, theme.dim),
        cmd_line("Esc", "Cancel", theme.accent, theme.dim),
        cmd_line("↑/↓", "History navigation", theme.accent, theme.dim),
        Line::from(""),
        section_line("Search Mode", theme.secondary),
        cmd_line("Enter", "Apply filter", theme.accent, theme.dim),
        cmd_line("Esc", "Clear and cancel", theme.accent, theme.dim),
        Line::from(""),
        section_line("Detail View", theme.secondary),
        cmd_line("Esc/q", "Close", theme.accent, theme.dim),
        cmd_line("↑/↓", "Scroll content", theme.accent, theme.dim),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Press Esc or q to close ",
            Style::default().fg(theme.accent),
        )]),
    ];

    let paragraph = Paragraph::new(help_text.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.secondary))
                .title(" Help "),
        )
        .scroll((app.help_scroll as u16, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);

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

    let mut result = Vec::new();
    for line in lines {
        if line.len() <= max_width {
            result.push(line);
        } else {
            for chunk in line.as_bytes().chunks(max_width) {
                result.push(String::from_utf8_lossy(chunk).to_string());
            }
        }
    }

    result
}
