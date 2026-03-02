use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

mod config;
mod core;
mod identity;
mod logger;
mod middleware;
mod policy;

use core::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init();

    log::info!("Starting Orpheus Agent v0.1...");

    let identity = identity::load_or_create_identity()?;
    let config = config::load_config()?;

    let state = Arc::new(AppState::new(identity.clone(), config.clone()));

    log::info!("Loaded existing node identity.");
    log::info!("Node ID: {}", identity.id());
    log::info!("HTTP server running on http://127.0.0.1:8080");

    // Heartbeat task
    let heartbeat_state = state.clone();
    tokio::spawn(async move {
        loop {
            log::info!(
                "Heartbeat: node={} env={}",
                heartbeat_state.identity.id(),
                heartbeat_state.config.environment
            );
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    // ---- ROUTER ----
    let app = Router::new()
        .route("/health", get(health))
        .route("/info", get(info))
        .route("/execute", post(execute))
        .with_state(state.clone())
        .layer(
            axum::middleware::from_fn_with_state(
                state.clone(),
                middleware::policy_middleware,
            ),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    tokio::select! {
        _ = axum::serve(listener, app) => {},
        _ = shutdown_signal() => {
            log::info!("Shutdown signal received.");
        }
    }

    log::info!("Orpheus Agent shut down cleanly.");
    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn info(State(state): State<Arc<AppState>>) -> String {
    serde_json::json!({
        "node_id": state.identity.id(),
        "node_name": state.config.node_name,
        "environment": state.config.environment
    })
    .to_string()
}

async fn execute() -> &'static str {
    "Execution allowed"
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
}
