//! Expression evaluation - stub implementation

use crate::error::QueryResult;
use cai_core::Entry;

/// Evaluate a WHERE expression against an entry (stub)
///
/// This is a placeholder for future full sqlparser-rust integration.
/// Currently, WHERE clause filtering is handled by `apply_where_filter()`
/// in the executor using simpler string matching.
#[allow(dead_code)]
pub fn eval_where(_expr: &sqlparser::ast::Expr, _entry: &Entry) -> QueryResult<bool> {
    // Stub: always return true for now
    Ok(true)
}
