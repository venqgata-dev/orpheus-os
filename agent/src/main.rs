mod identity;
mod config;
mod policy_engine;
mod audit;

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};

use log::info;
use serde::{Deserialize, Serialize};

use tokio::{net::TcpListener, signal, task, time};

use policy_engine::PolicyEngine;
use audit::{AuditLogger, AuditRecord};

#[derive(Clone)]
struct AppState {
    node_id: String,
    node_name: String,
    environment: String,

    policy_engine: Arc<PolicyEngine>,
    audit: Arc<AuditLogger>,
}

#[derive(Serialize)]
struct VerifyResponse {
    valid: bool,
}

#[derive(Serialize)]
struct InfoResponse {
    node_id: String,
    node_name: String,
    environment: String,
    status: String,
}

#[derive(Deserialize)]
struct ValidateRequest {
    environment: String,
    payload_size: usize,
}

#[derive(Serialize)]
struct ValidateResponse {
    allowed: bool,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    env_logger::init();

    info!("Starting Orpheus Agent v0.1...");

    let node = identity::load_or_create_identity()?;
    let config = config::load_config()?;

    let policy_engine = Arc::new(
        PolicyEngine::new(config.policy.clone())
    );

    let audit = Arc::new(
        AuditLogger::new(node.signing_key().clone())
    );

    let state = Arc::new(AppState {
        node_id: node.id(),
        node_name: config.node_name.clone(),
        environment: config.environment.clone(),
        policy_engine,
        audit,
    });

    // Background heartbeat task
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
        .route("/validate", post(validate))
        .route("/audit/recent", get(audit_recent))
        .route("/audit/verify-signatures", get(audit_verify_signatures))
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

async fn validate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ValidateRequest>,
) -> Json<ValidateResponse> {

    let mut allowed = true;
    let mut message = "Validation successful".to_string();

    if let Err(e) = state.policy_engine.validate_environment(&req.environment) {
        allowed = false;
        message = e;
    }

    if allowed {
        if let Err(e) = state.policy_engine.validate_payload_size(req.payload_size) {
            allowed = false;
            message = e;
        }
    }

    state.audit.log_event(
        &req.environment,
        req.payload_size,
        allowed,
        &message,
    );

    Json(ValidateResponse { allowed, message })
}

async fn audit_recent(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<AuditRecord>> {

    Json(state.audit.get_recent(20))
}

async fn audit_verify_signatures(
    State(state): State<Arc<AppState>>,
) -> Json<VerifyResponse> {

    Json(VerifyResponse {
        valid: state.audit.verify_signatures(),
    })
}
