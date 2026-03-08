//! Built-in SQL functions - stub implementation

use crate::error::{QueryError, QueryResult};
use chrono::{DateTime, Utc};

/// Date range filter function
pub fn date_range(_start: DateTime<Utc>, _end: DateTime<Utc>) -> QueryResult<bool> {
    Ok(true)
}

/// Time bucket function for grouping
pub fn time_bucket(_timestamp: DateTime<Utc>, _bucket: &str) -> QueryResult<String> {
    Ok("2024-01-01".to_string())
}
