//! CAI TUI - Terminal UI
//!
//! Interactive terminal interface for querying and exploring AI coding history.
//!
//! # Features
//!
//! - Query input with history navigation
//! - Scrollable results table with sorting
//! - Real-time status updates
//! - Keyboard shortcuts (q=quit, /=search, etc.)
//!
//! # Example
//!
//! ```rust,no_run
//! use cai_tui::run;
//!
//! // Create a storage implementation (MemoryStorage, SqliteStorage, etc.)
//! // and pass it to the run function
//! // let storage = ...;
//! // run(storage).await?;
//! ```

#![warn(missing_docs, unused_crate_dependencies)]

pub use cai_core::Result;

mod app;
mod event;
mod ui;

pub use app::{App, AppState, Column, Mode, SortOrder};
pub use event::{Event, EventHandler};

use cai_storage::Storage;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Run the TUI application
///
/// # Errors
///
/// Returns an error if the terminal setup fails or a runtime error occurs
pub async fn run<S>(storage: Arc<S>) -> Result<()>
where
    S: Storage + 'static,
{
    // Initialize terminal
    let mut terminal = ui::init_terminal()?;

    // Create application
    let app = App::new(storage);
    let mut app = Arc::new(RwLock::new(app));

    // Create event handler
    let event_handler = EventHandler::new(100);

    // Run application
    let res = ui::run_app(&mut terminal, &mut app, event_handler).await;

    // Restore terminal
    ui::restore_terminal(terminal)?;

    // Convert error type
    res.map_err(|e| cai_core::Error::Message(e.to_string()))
}
