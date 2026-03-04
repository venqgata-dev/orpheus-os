use axum::{routing::get, routing::post, Json, Router};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

mod audit;
use audit::{AuditLogger, RootResponse};

#[derive(Deserialize)]
struct ValidateRequest {
    environment: String,
    payload_size: u64,
}

#[derive(Serialize)]
struct ValidateResponse {
    allowed: bool,
    message: String,
}

#[tokio::main]
async fn main() {
    // Generate node key
    let signing_key = SigningKey::generate(&mut OsRng);

    let audit_logger = Arc::new(AuditLogger::new(signing_key));

    let app = Router::new()
        .route("/validate", post(validate))
        .route("/audit/root", get(get_root))
        .with_state(audit_logger);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("HTTP server running on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn validate(
    axum::extract::State(audit): axum::extract::State<Arc<AuditLogger>>,
    Json(req): Json<ValidateRequest>,
) -> Json<ValidateResponse> {
    let payload = format!("{}:{}", req.environment, req.payload_size);

    audit.record_event(&payload);

    Json(ValidateResponse {
        allowed: true,
        message: "Validation successful".into(),
    })
}

async fn get_root(
    axum::extract::State(audit): axum::extract::State<Arc<AuditLogger>>,
) -> Json<RootResponse> {
    Json(audit.get_root())
}
