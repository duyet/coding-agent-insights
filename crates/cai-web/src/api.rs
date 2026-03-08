//! REST API types and router

use axum::{
    routing::get,
    Router,
};
use super::handlers;

/// Statistics response
#[derive(Debug, serde::Serialize)]
pub struct StatsResponse {
    /// Total number of entries
    pub total: usize,
    /// Count by source
    pub by_source: std::collections::HashMap<String, usize>,
    /// Date range (min, max) as RFC3339 strings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_range: Option<(String, String)>,
}

/// Create API router
pub fn router() -> Router<super::AppState> {
    Router::new()
        .route("/query", get(handlers::query_handler))
        .route("/stats", get(handlers::stats_handler))
        .route("/export/json", get(handlers::export_json_handler))
        .route("/export/csv", get(handlers::export_csv_handler))
        .route("/ws", get(handlers::websocket_handler))
}
