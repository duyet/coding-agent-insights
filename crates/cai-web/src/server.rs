//! Web server module

use super::{api, AppState, Config};
use axum::{
    routing::get,
    Router,
};
use cai_core::Result;
use cai_storage::Storage;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

/// Run the web server
pub async fn run<S>(storage: std::sync::Arc<S>, config: Config) -> Result<()>
where
    S: Storage + Send + Sync + 'static,
{
    let state = AppState { storage: storage as std::sync::Arc<dyn Storage + Send + Sync> };

    let app = Router::new()
        .route("/", get(super::handlers::dashboard_handler))
        .nest("/api", api::router())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;

    info!("CAI Web Dashboard listening on http://{}", addr);
    info!("Open your browser to explore your AI coding history");

    axum::serve(listener, app).await?;

    Ok(())
}
