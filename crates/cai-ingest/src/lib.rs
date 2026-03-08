//! CAI Ingest - Data ingestion from various sources

#![warn(missing_docs, unused_crate_dependencies)]

pub use cai_core::Result;

mod claude;
mod codex;
mod git;
mod error;
mod ingest;

pub use claude::ClaudeParser;
pub use codex::CodexParser;
pub use git::GitScanner;
pub use error::IngestError;
pub use ingest::{Ingestor, IngestConfig};
