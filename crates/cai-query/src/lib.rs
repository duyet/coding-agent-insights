//! CAI Query - SQL parser and query engine
//!
//! Provides SQL-like query interface for CAI entries with support for:
//! - SELECT with WHERE, ORDER BY, LIMIT, GROUP BY
//! - Aggregate functions: COUNT, SUM, AVG, MIN, MAX
//! - Date functions: date_range, time_bucket
//!
//! # Example
//!
//! ```rust,no_run
//! use cai_query::{QueryEngine, sql};
//! use cai_storage::MemoryStorage;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = MemoryStorage::new();
//! let engine = QueryEngine::new(storage);
//!
//! // Simple SELECT
//! let _results = engine.execute("SELECT * FROM entries LIMIT 10").await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs, unused_crate_dependencies)]

pub use cai_core::Result;

mod error;
mod parser;
mod executor;
mod functions;
mod eval;

pub use error::{QueryError, QueryResult};
pub use executor::QueryEngine;
pub use parser::parse;
pub use functions::{date_range, time_bucket};

/// Convenience function to execute a SQL query
///
/// # Example
///
/// ```rust,no_run
/// use cai_query::sql;
/// use cai_storage::MemoryStorage;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let storage = MemoryStorage::new();
/// let _results = sql("SELECT * FROM entries", storage).await?;
/// # Ok(())
/// # }
/// ```
pub async fn sql<S>(query: &str, storage: S) -> QueryResult<Vec<cai_core::Entry>>
where
    S: cai_storage::Storage + 'static,
{
    QueryEngine::new(storage).execute(query).await
}
