//! CAI Web - Local web interface
//!
//! Provides a local web server with REST API and dashboard for exploring
//! AI coding history.
//!
//! # Features
//!
//! - REST API for queries
//! - Interactive HTML/JS dashboard
//! - WebSocket for real-time updates
//! - Static file serving
//!
//! # Example
//!
//! ```rust,no_run
//! use cai_web::run;
//! use cai_storage::MemoryStorage;
//!
//! # async fn example() -> cai_core::Result<()> {
//! let storage = MemoryStorage::new();
//! run(storage, 3000).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs, unused_crate_dependencies)]

pub use cai_core::Result;

mod api;
mod server;
mod handlers;

pub use server::run;

use cai_storage::Storage;

/// Web server configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Port to listen on
    pub port: u16,
    /// Host to bind to
    pub host: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
        }
    }
}

/// Shared application state
#[derive(Clone)]
pub struct AppState<S>
where
    S: Storage + ?Sized,
{
    /// Storage backend
    pub storage: std::sync::Arc<S>,
}
