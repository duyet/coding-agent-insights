//! HTTP request handlers stub

use cai_core::Result;
use axum::response::Html;

/// Query handler
pub async fn query_handler() -> Result<Html<String>> {
    Ok(Html("<html><body>CAI Dashboard</body></html>".to_string()))
}
