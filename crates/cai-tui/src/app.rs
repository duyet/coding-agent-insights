//! TUI Application state and logic

use cai_core::Entry;
use cai_storage::Storage;
use ratatui::style::Color;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Normal mode - viewing results
    Normal,
    /// Query input mode
    Query,
    /// Search mode
    Search,
    /// Detail view - showing selected entry
    Detail,
    /// Help screen
    Help,
}

/// Application state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    /// Running normally
    Running,
    /// Should quit
    Quitting,
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

/// Sortable column
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    /// Timestamp column
    Timestamp,
    /// Source column
    Source,
    /// Prompt column
    Prompt,
}

/// Main TUI application
pub struct App<S>
where
    S: Storage + ?Sized,
{
    /// Storage backend
    storage: Arc<S>,
    /// Current application mode
    pub mode: Mode,
    /// Application state
    pub state: AppState,
    /// Current query input
    pub query_input: String,
    /// Current search input
    pub search_input: String,
    /// Query results
    pub entries: Vec<Entry>,
    /// Selected entry index
    pub selected: usize,
    /// Scroll offset
    pub scroll: usize,
    /// Current sort column
    pub sort_column: Column,
    /// Current sort order
    pub sort_order: SortOrder,
    /// Status message
    pub status_message: String,
    /// Status message color
    pub status_color: Color,
    /// Status message timestamp (for auto-clear)
    pub status_timestamp: u64,
    /// Query history
    pub history: Vec<String>,
    /// History index (for navigation)
    pub history_index: Option<usize>,
    /// Detail view scroll offset
    pub detail_scroll: usize,
    /// Help scroll offset
    pub help_scroll: usize,
}

impl<S> App<S>
where
    S: Storage,
{
    /// Create a new application
    pub fn new(storage: Arc<S>) -> Self {
        Self {
            storage,
            mode: Mode::Normal,
            state: AppState::Running,
            query_input: String::new(),
            search_input: String::new(),
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
            sort_column: Column::Timestamp,
            sort_order: SortOrder::Desc,
            status_message: "Press '/' to search, 'i' to enter query mode, 'q' to quit".to_string(),
            status_color: Color::Gray,
            status_timestamp: now(),
            history: Vec::new(),
            history_index: None,
            detail_scroll: 0,
            help_scroll: 0,
        }
    }

    /// Execute a query and update entries
    pub async fn execute_query(&mut self, query: &str) {
        // Add to history
        if !query.is_empty() {
            self.history.push(query.to_string());
            self.history_index = None;
        }

        // Parse and execute query (simplified - actual SQL parsing would be in cai-query)
        // For now, just get all entries
        match self.storage.query(None).await {
            Ok(entries) => {
                self.entries = entries;
                self.selected = 0;
                self.scroll = 0;
                self.sort_entries();
                self.set_status(
                    format!("Query returned {} results", self.entries.len()),
                    Color::Green,
                );
            }
            Err(e) => {
                self.set_status(format!("Query error: {}", e), Color::Red);
            }
        }
    }

    /// Search entries
    pub fn search(&mut self) {
        if self.search_input.is_empty() {
            return;
        }

        let query = self.search_input.to_lowercase();
        self.entries.retain(|entry| {
            entry.prompt.to_lowercase().contains(&query)
                || entry.response.to_lowercase().contains(&query)
                || format!("{:?}", entry.source).to_lowercase().contains(&query)
        });

        self.selected = 0;
        self.scroll = 0;
        self.set_status(
            format!("Found {} results", self.entries.len()),
            Color::Green,
        );
    }

    /// Clear search and reload all entries
    pub async fn clear_search(&mut self) {
        self.search_input.clear();
        self.execute_query("").await;
    }

    /// Set status message
    pub fn set_status(&mut self, msg: String, color: Color) {
        self.status_message = msg;
        self.status_color = color;
        self.status_timestamp = now();
    }

    /// Check if status message should be cleared (after 5 seconds)
    pub fn should_clear_status(&self) -> bool {
        now() - self.status_timestamp > 5
    }

    /// Clear status message to default
    pub fn reset_status(&mut self) {
        self.status_message =
            "Press '/' to search, 'i' to enter query mode, 'q' to quit".to_string();
        self.status_color = Color::Gray;
    }

    /// Select previous entry
    pub fn select_previous(&mut self) {
        if !self.entries.is_empty() && self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll {
                self.scroll = self.selected;
            }
        }
    }

    /// Select next entry
    pub fn select_next(&mut self, height: usize) {
        if !self.entries.is_empty() {
            self.selected = self.selected.saturating_add(1).min(self.entries.len() - 1);
            // Calculate visible area height (minus header and padding)
            let visible_height = height.saturating_sub(4);
            if self.selected >= self.scroll + visible_height && visible_height > 0 {
                self.scroll = self.selected - visible_height + 1;
            }
        }
    }

    /// Sort entries by current column
    pub fn sort_entries(&mut self) {
        match self.sort_column {
            Column::Timestamp => {
                self.entries.sort_by(|a, b| {
                    if self.sort_order == SortOrder::Asc {
                        a.timestamp.cmp(&b.timestamp)
                    } else {
                        b.timestamp.cmp(&a.timestamp)
                    }
                });
            }
            Column::Source => {
                self.entries.sort_by(|a, b| {
                    let source_cmp = format!("{:?}", a.source).cmp(&format!("{:?}", b.source));
                    if self.sort_order == SortOrder::Asc {
                        source_cmp
                    } else {
                        source_cmp.reverse()
                    }
                });
            }
            Column::Prompt => {
                self.entries.sort_by(|a, b| {
                    let prompt_cmp = a.prompt.cmp(&b.prompt);
                    if self.sort_order == SortOrder::Asc {
                        prompt_cmp
                    } else {
                        prompt_cmp.reverse()
                    }
                });
            }
        }
    }

    /// Toggle sort order for current column
    pub fn toggle_sort(&mut self, column: Column) {
        if self.sort_column == column {
            self.sort_order = match self.sort_order {
                SortOrder::Asc => SortOrder::Desc,
                SortOrder::Desc => SortOrder::Asc,
            };
        } else {
            self.sort_column = column;
            self.sort_order = SortOrder::Asc;
        }
        self.sort_entries();
    }

    /// Navigate history backwards
    pub fn history_previous(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.history_index = Some(self.history.len() - 1);
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
            }
            _ => {}
        }

        if let Some(idx) = self.history_index {
            self.query_input = self.history[idx].clone();
        }
    }

    /// Navigate history forwards
    pub fn history_next(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            Some(idx) if idx < self.history.len() - 1 => {
                self.history_index = Some(idx + 1);
                if let Some(idx) = self.history_index {
                    self.query_input = self.history[idx].clone();
                }
            }
            Some(_) => {
                self.history_index = None;
                self.query_input.clear();
            }
            None => {}
        }
    }

    /// Get selected entry
    pub fn selected_entry(&self) -> Option<&Entry> {
        self.entries.get(self.selected)
    }

    /// Get row style based on selection
    pub fn row_style(&self, index: usize) -> ratatui::style::Style {
        use ratatui::style::Style;
        if index == self.selected {
            Style::default().bg(ratatui::style::Color::DarkGray)
        } else {
            Style::default()
        }
    }

    /// Scroll detail view down
    pub fn detail_scroll_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    /// Scroll detail view up
    pub fn detail_scroll_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }

    /// Reset detail scroll
    pub fn detail_scroll_reset(&mut self) {
        self.detail_scroll = 0;
    }

    /// Scroll help view down
    pub fn help_scroll_down(&mut self) {
        self.help_scroll = self.help_scroll.saturating_add(1);
    }

    /// Scroll help view up
    pub fn help_scroll_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }
}

/// Get current timestamp in seconds
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
