//! Query error types

use cai_core::Error as CoreError;

/// Query-specific result type
pub type QueryResult<T> = std::result::Result<T, QueryError>;

/// Query engine errors
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    /// SQL parsing error
    #[error("SQL parse error: {0}")]
    ParseError(String),

    /// Invalid table name
    #[error("Invalid table name: {0}. Expected 'entries'")]
    InvalidTable(String),

    /// Invalid column name
    #[error("Invalid column: {0}")]
    InvalidColumn(String),

    /// Invalid function
    #[error("Invalid function: {0}")]
    InvalidFunction(String),

    /// Invalid operator
    #[error("Invalid operator: {0}")]
    InvalidOperator(String),

    /// Type mismatch
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Core error
    #[error("Core error: {0}")]
    Core(#[from] CoreError),

    /// Not supported
    #[error("Not supported: {0}")]
    NotSupported(String),
}

impl From<sqlparser::parser::ParserError> for QueryError {
    fn from(err: sqlparser::parser::ParserError) -> Self {
        QueryError::ParseError(err.to_string())
    }
}
