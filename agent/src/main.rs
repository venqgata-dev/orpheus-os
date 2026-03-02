mod identity;
mod config;

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use axum::{routing::get, Router, extract::State, response::Json};
use log::info;
use serde::Serialize;
use tokio::{net::TcpListener, signal, task, time};

#[derive(Clone)]
struct AppState {
    node_id: String,
    node_name: String,
    environment: String,
}

#[derive(Serialize)]
struct InfoResponse {
    node_id: String,
    node_name: String,
    environment: String,
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    info!("Starting Orpheus Agent v0.1...");

    let node = identity::load_or_create_identity()?;
    let config = config::load_config()?;

    let state = Arc::new(AppState {
        node_id: node.id(),
        node_name: config.node_name.clone(),
        environment: config.environment.clone(),
    });

    // Spawn heartbeat background task
    let heartbeat_state = state.clone();
    task::spawn(async move {
        loop {
            info!(
                "Heartbeat | node={} | env={}",
                heartbeat_state.node_id,
                heartbeat_state.environment
            );
            time::sleep(Duration::from_secs(10)).await;
        }
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/info", get(info_endpoint))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    info!("HTTP server running on http://127.0.0.1:8080");

    tokio::select! {
        _ = axum::serve(listener, app) => {},
        _ = signal::ctrl_c() => {
            info!("Shutdown signal received.");
        }
    }

    info!("Orpheus Agent shutting down gracefully.");
    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn ready() -> &'static str {
    "READY"
}

async fn info_endpoint(
    State(state): State<Arc<AppState>>,
) -> Json<InfoResponse> {
    Json(InfoResponse {
        node_id: state.node_id.clone(),
        node_name: state.node_name.clone(),
        environment: state.environment.clone(),
        status: "running".to_string(),
    })
}
