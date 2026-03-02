use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PolicyCheckRequest {
    pub environment: String,
    pub payload_size: usize,
}

#[derive(Serialize)]
pub struct PolicyCheckResponse {
    pub allowed: bool,
    pub reason: String,
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub environment: String,
    pub prompt: String,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub allowed: bool,
    pub reason: String,
    pub response: Option<String>,
}

pub fn evaluate_policy(
    config: &crate::config::PolicyConfig,
    request: &PolicyCheckRequest,
) -> PolicyCheckResponse {

    if !config.allow_environments.contains(&request.environment) {
        return PolicyCheckResponse {
            allowed: false,
            reason: format!("Environment '{}' not allowed", request.environment),
        };
    }

    if request.payload_size > config.max_payload_size {
        return PolicyCheckResponse {
            allowed: false,
            reason: "Payload size exceeds limit".to_string(),
        };
    }

    PolicyCheckResponse {
        allowed: true,
        reason: "Policy check passed".to_string(),
    }
}
