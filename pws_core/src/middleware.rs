use std::sync::Arc;

use crate::types::{AppState, AppUtils, KVStorage};
use crate::utils::constant_time_eq;
use axum::body::to_bytes;
use axum::extract::State;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

pub async fn sign_check<U, KV>(
    State(state): State<Arc<AppState<U, KV>>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode>
where
    U: AppUtils,
    KV: KVStorage,
{
    let (parts, body) = req.into_parts();

    let sign_header = parts
        .headers
        .get("X-Sign")
        .and_then(|v: &axum::http::HeaderValue| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let bytes = to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sign_local = state.utils.sign(&bytes);

    if !constant_time_eq(sign_local.as_bytes(), sign_header.as_bytes()) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let req = Request::from_parts(parts, bytes.into());

    Ok(next.run(req).await)
}
