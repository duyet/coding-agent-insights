//! Error types for data ingestion

use thiserror::Error;

/// Ingest-specific errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IngestError {
    /// No conversation files found
    #[error("No conversation files found in {0}")]
    NoFilesFound(String),

    /// Invalid file format
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    /// Git operation failed
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),

    /// Path not found
    #[error("Path not found: {0}")]
    PathNotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
