//! HTTP request handlers

use super::api::StatsResponse;
use super::AppState;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Query as AxumQuery, State},
    response::{Html, IntoResponse, Json, Response},
};
use cai_core::{Result, Source};
use futures::stream::StreamExt;
use serde::Deserialize;

/// Dashboard HTML handler
pub async fn dashboard_handler() -> Html<String> {
    Html(include_str!("dashboard.html").to_string())
}

/// Query parameters for API requests
#[derive(Debug, Deserialize)]
pub struct QueryParams {
    /// SQL query string
    pub sql: String,
}

/// Query API handler
pub async fn query_handler(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<QueryParams>,
) -> Response {
    let engine = cai_query::QueryEngine::from_arc(state.storage.clone());

    match engine.execute(&params.sql).await {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => {
            tracing::error!("Query error: {:?}", e);
            Json(serde_json::json!({
                "error": e.to_string(),
                "sql": params.sql
            }))
            .into_response()
        }
    }
}

/// Statistics API handler
pub async fn stats_handler(State(state): State<AppState>) -> Json<StatsResponse> {
    let all_entries = state.storage.query(None).await.unwrap_or_default();

    let mut by_source = std::collections::HashMap::new();
    for entry in &all_entries {
        let source_name = match &entry.source {
            Source::Claude => "Claude",
            Source::Codex => "Codex",
            Source::Git => "Git",
            Source::Other(name) => name.as_str(),
            _ => "Other",
        };
        *by_source.entry(source_name.to_string()).or_insert(0) += 1;
    }

    let date_range = if all_entries.is_empty() {
        None
    } else {
        let timestamps: Vec<_> = all_entries.iter().map(|e| e.timestamp).collect();
        let min = timestamps.iter().min().unwrap();
        let max = timestamps.iter().max().unwrap();
        Some((min.to_rfc3339(), max.to_rfc3339()))
    };

    Json(StatsResponse {
        total: all_entries.len(),
        by_source,
        date_range,
    })
}

/// WebSocket handler for real-time updates
pub async fn websocket_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // Send connected message
    if socket
        .send(Message::Text(
            serde_json::json!({"type": "connected"}).to_string().into(),
        ))
        .await
        .is_err()
    {
        return;
    }

    while let Some(result) = socket.next().await {
        match result {
            Ok(Message::Text(text)) => {
                if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                    match msg.r#type.as_str() {
                        "stats" => {
                            if let Ok(stats) = get_stats(&state.storage).await {
                                let response = serde_json::json!({
                                    "type": "stats",
                                    "total": stats.total,
                                    "by_source": stats.by_source,
                                    "date_range": stats.date_range
                                });
                                let _ = socket.send(Message::Text(response.to_string().into())).await;
                            }
                        }
                        "query" => {
                            if let Some(query) = msg.query {
                                let engine = cai_query::QueryEngine::from_arc(state.storage.clone());
                                match engine.execute(&query).await {
                                    Ok(entries) => {
                                        let _ = socket
                                            .send(Message::Text(
                                                serde_json::json!({
                                                    "type": "query_result",
                                                    "entries": entries
                                                })
                                                .to_string()
                                                .into(),
                                            ))
                                            .await;
                                    }
                                    Err(e) => {
                                        let _ = socket
                                            .send(Message::Text(
                                                serde_json::json!({
                                                    "type": "error",
                                                    "message": e.to_string()
                                                })
                                                .to_string()
                                                .into(),
                                            ))
                                            .await;
                                    }
                                }
                            }
                        }
                        "ping" => {
                            let _ = socket
                                .send(Message::Text(
                                    serde_json::json!({"type": "pong"}).to_string().into(),
                                ))
                                .await;
                        }
                        _ => {}
                    }
                }
            }
            Ok(Message::Close(_)) => {
                break;
            }
            Err(_) => {
                break;
            }
            _ => {}
        }
    }
}

/// Get statistics
async fn get_stats(storage: &std::sync::Arc<dyn cai_storage::Storage + Send + Sync>) -> Result<StatsResponse> {
    let all_entries = storage.query(None).await?;

    let mut by_source = std::collections::HashMap::new();
    for entry in &all_entries {
        let source_name = match &entry.source {
            Source::Claude => "Claude",
            Source::Codex => "Codex",
            Source::Git => "Git",
            Source::Other(name) => name.as_str(),
            _ => "Other",
        };
        *by_source.entry(source_name.to_string()).or_insert(0) += 1;
    }

    let date_range = if all_entries.is_empty() {
        None
    } else {
        let timestamps: Vec<_> = all_entries.iter().map(|e| e.timestamp).collect();
        let min = timestamps.iter().min().unwrap();
        let max = timestamps.iter().max().unwrap();
        Some((min.to_rfc3339(), max.to_rfc3339()))
    };

    Ok(StatsResponse {
        total: all_entries.len(),
        by_source,
        date_range,
    })
}

/// WebSocket message from client
#[derive(Debug, Deserialize)]
struct WsMessage {
    #[serde(rename = "type")]
    r#type: String,
    query: Option<String>,
}

/// Export as JSON handler
pub async fn export_json_handler(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<QueryParams>,
) -> Response {
    let engine = cai_query::QueryEngine::from_arc(state.storage.clone());

    match engine.execute(&params.sql).await {
        Ok(entries) => {
            let json = serde_json::to_string_pretty(&entries).unwrap_or_default();
            (
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                json,
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Query error: {:?}", e);
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

/// Export as CSV handler
pub async fn export_csv_handler(
    State(state): State<AppState>,
    AxumQuery(params): AxumQuery<QueryParams>,
) -> Response {
    let engine = cai_query::QueryEngine::from_arc(state.storage.clone());

    match engine.execute(&params.sql).await {
        Ok(entries) => {
            let mut csv = String::from("id,timestamp,source,prompt,response\n");
            for entry in &entries {
                let prompt = entry.prompt.replace('"', "\"\"").replace('\n', " ");
                let response = entry.response.replace('"', "\"\"").replace('\n', " ");
                csv.push_str(&format!(
                    "{},{},{:?},\"{}\",\"{}\"\n",
                    entry.id, entry.timestamp, entry.source, prompt, response
                ));
            }
            (
                [(axum::http::header::CONTENT_TYPE, "text/csv")],
                csv,
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Query error: {:?}", e);
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
