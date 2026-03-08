//! Event handling for TUI
//!
//! Provides keyboard and event loop handling for the terminal UI.

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;

/// Terminal event
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Key press event
    Key(KeyEvent),
    /// Tick event (periodic)
    Tick,
}

/// Event handler for terminal events
pub struct EventHandler {
    /// Event sender channel (kept to hold channel open)
    _sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel
    receiver: mpsc::UnboundedReceiver<Event>,
    /// Event handler task handle
    _handle: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler
    ///
    /// # Arguments
    ///
    /// * `tick_rate` - Milliseconds between tick events
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sender_clone = sender.clone();
        let _handle = tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(Duration::from_millis(tick_rate));
            loop {
                // Wait for next tick or event
                tokio::select! {
                    _ = tick_interval.tick() => {
                        if sender_clone.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Self {
            _sender: sender,
            receiver,
            _handle,
        }
    }

    /// Get the next event
    ///
    /// Returns events in priority order:
    /// 1. Key events (highest priority)
    /// 2. Tick events (lowest priority)
    pub async fn next(&mut self) -> Event {

        // Check for key events with timeout
        let key_event = tokio::task::spawn_blocking(move || {
            if event::poll(Duration::from_millis(10)).ok()? {
                if let CrosstermEvent::Key(key) = event::read().ok()? {
                    // Only handle key press events, ignore repeat/release
                    if key.kind == KeyEventKind::Press {
                        return Some(Some(Event::Key(key)));
                    }
                }
            }
            Some(None)
        })
        .await
        .ok()
        .flatten();

        if let Some(Some(event)) = key_event {
            return event;
        }

        // Wait for next tick
        self.receiver.recv().await.unwrap_or(Event::Tick)
    }
}
