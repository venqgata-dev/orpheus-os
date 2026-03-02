use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

use crate::{
    core::AppState,
    policy::{evaluate_policy, PolicyCheckRequest},
};

pub async fn policy_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let (parts, body) = request.into_parts();

    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let payload: PolicyCheckRequest = match serde_json::from_slice(&body_bytes) {
        Ok(data) => data,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let result = evaluate_policy(&state.config.policy, &payload);

    if !result.allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(result),
        )
            .into_response();
    }

    let request = Request::from_parts(parts, Body::from(body_bytes));

    next.run(request).await
}
