use crate::core::SharedDaemon;
use axum::{
    Extension, Json, Router,
    routing::{get, post},
};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Serialize)]
struct StatusResponse {
    status: String,
}

pub async fn start_web_server(
    daemon: SharedDaemon,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .nest_service("/", ServeDir::new("ui"))
        .route("/health", get(|| async { "OK" }))
        .route("/status", get(get_status))
        .route("/threads", get(get_threads))
        .route("/pause", post(pause_factory))
        .route("/resume", post(resume_factory))
        .layer(CorsLayer::permissive())
        .layer(Extension(daemon));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(
        "🌐 Web Viewer (The Colosseum) is listening on http://{}",
        addr
    );

    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_status(Extension(daemon): Extension<SharedDaemon>) -> Json<StatusResponse> {
    let status = format!("{:?}", *daemon.status_rx.borrow());
    Json(StatusResponse { status })
}

async fn get_threads(
    Extension(daemon): Extension<SharedDaemon>,
) -> Json<Vec<crate::core::AgentThread>> {
    Json(daemon.get_threads())
}

async fn pause_factory(Extension(daemon): Extension<SharedDaemon>) -> Json<StatusResponse> {
    let _ = daemon.pause();
    Json(StatusResponse {
        status: "HOLD".to_string(),
    })
}

async fn resume_factory(Extension(daemon): Extension<SharedDaemon>) -> Json<StatusResponse> {
    let _ = daemon.resume();
    Json(StatusResponse {
        status: "RUNNING".to_string(),
    })
}
