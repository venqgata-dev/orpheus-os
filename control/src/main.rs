use axum::{
    routing::{get, post},
    Router,
    Json,
};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
struct Heartbeat {
    node: String,
    version: String,
    signature: String,
}

async fn health() -> &'static str {
    "OK"
}

async fn receive_heartbeat(Json(payload): Json<Heartbeat>) -> &'static str {
    println!(
        "🔥 Received heartbeat | node: {} | version: {} | signature: {}",
        payload.node, payload.version, payload.signature
    );

    "Heartbeat received"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health))
        .route("/heartbeat", post(receive_heartbeat));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("🚀 Control server running on http://{}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app,
    )
    .await
    .unwrap();
}
