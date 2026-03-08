//! Expression evaluation - stub implementation

use crate::error::{QueryError, QueryResult};
use cai_core::Entry;

/// Evaluate a WHERE expression against an entry (stub)
pub fn eval_where(_expr: &sqlparser::ast::Expr, _entry: &Entry) -> QueryResult<bool> {
    // Stub: always return true for now
    Ok(true)
}
