use axum::{routing::post, Json, Router};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct AttestationRequest {
    node_id: String,
    latest_hash: String,
    signature: String,
}

#[derive(Serialize)]
struct AttestationResponse {
    accepted: bool,
    reason: Option<String>,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/attest", post(attest));

    let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    println!("Control running on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn attest(
    Json(req): Json<AttestationRequest>,
) -> Json<AttestationResponse> {
    // Decode public key
    let pubkey_bytes = match hex::decode(&req.node_id) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Json(AttestationResponse {
                accepted: false,
                reason: Some("Invalid node_id encoding".into()),
            })
        }
    };

    if pubkey_bytes.len() != 32 {
        return Json(AttestationResponse {
            accepted: false,
            reason: Some("Invalid node_id length".into()),
        });
    }

    let verifying_key =
        match VerifyingKey::from_bytes(&pubkey_bytes.try_into().unwrap()) {
            Ok(key) => key,
            Err(_) => {
                return Json(AttestationResponse {
                    accepted: false,
                    reason: Some("Invalid public key".into()),
                })
            }
        };

    // Decode signature
    let sig_bytes = match hex::decode(&req.signature) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Json(AttestationResponse {
                accepted: false,
                reason: Some("Invalid signature encoding".into()),
            })
        }
    };

    if sig_bytes.len() != 64 {
        return Json(AttestationResponse {
            accepted: false,
            reason: Some("Invalid signature encoding".into()),
        });
    }

    let signature = match Signature::from_slice(&sig_bytes) {
        Ok(sig) => sig,
        Err(_) => {
            return Json(AttestationResponse {
                accepted: false,
                reason: Some("Invalid signature encoding".into()),
            })
        }
    };

    // Hash message exactly like agent
    let mut hasher = Sha256::new();
    hasher.update(req.latest_hash.as_bytes());
    let message = hasher.finalize();

    // Verify
    match verifying_key.verify(&message, &signature) {
        Ok(_) => Json(AttestationResponse {
            accepted: true,
            reason: None,
        }),
        Err(_) => Json(AttestationResponse {
            accepted: false,
            reason: Some("Signature verification failed".into()),
        }),
    }
}
