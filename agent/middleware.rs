use axum::{
    body::{Body, to_bytes},
    extract::{State, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use log::info;

use crate::{
    policy::PolicyCheckRequest,
    audit::PolicyAuditEvent,
    AppState,
};

pub async fn policy_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {

    let (mut parts, body) = request.into_parts();

    let bytes = to_bytes(body, usize::MAX)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid body".to_string()))?;

    let parsed: PolicyCheckRequest = serde_json::from_slice(&bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()))?;

    let result = crate::policy::evaluate_policy(&state.config.policy, &parsed);

    // 🔷 Create audit event
    let audit_event = PolicyAuditEvent::new(
        state.identity.id(),
        parsed.environment.clone(),
        parsed.payload_size,
        result.allowed,
        result.reason.clone(),
    );

    // Structured log output
    info!(
        "POLICY_AUDIT {}",
        serde_json::to_string(&audit_event).unwrap()
    );

    if !result.allowed {
        return Err((StatusCode::FORBIDDEN, result.reason));
    }

    let request = Request::from_parts(parts, Body::from(bytes));

    Ok(next.run(request).await)
}
