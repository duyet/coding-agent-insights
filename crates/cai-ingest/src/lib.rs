//! CAI Ingest - Data ingestion from various sources

#![warn(missing_docs, unused_crate_dependencies)]
pub use cai_core::Result;

// Tokio is used for async functions in this crate (via #[tokio::test] in tests)
// This import suppresses the unused_crate_dependencies lint
#[allow(unused_imports)]
use tokio as _;

mod claude;
mod codex;
mod error;
mod git;
mod ingest;

pub use claude::ClaudeParser;
pub use codex::CodexParser;
pub use error::IngestError;
pub use git::GitScanner;
pub use ingest::{IngestConfig, Ingestor};
