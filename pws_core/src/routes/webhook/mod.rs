mod save;
mod user;

use axum::extract::State;
use axum::middleware::from_fn_with_state;
use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::middleware::sign_check;
use crate::types::{AppState, AppUtils, KVStorage, LogLevel};

#[derive(Deserialize, Debug)]
pub struct WebhookPayload {
    pub meta: Meta,
    pub user: User,
    pub data: Value,
}

#[derive(Deserialize, Debug)]
pub struct Meta {
    #[serde(rename = "type")]
    pub r#type: String,
    pub action: String,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub openid: String,
    #[allow(dead_code)]
    pub session_token: String,
    pub nickname: String,
}

pub async fn webhook_handler<U: AppUtils, KV: KVStorage>(
    State(state): State<Arc<AppState<U, KV>>>,
    Json(payload): Json<WebhookPayload>,
) -> impl IntoResponse {
    match (payload.meta.r#type.as_str(), payload.meta.action.as_str()) {
        ("save", _) => {
            save::handle_save(&payload, &state).await;
        }

        ("user", "update" | "login" | "create") => {
            user::handle_user_update_login_create(&payload, &state).await;
        }

        (t, a) => {
            state.utils.logger(
                LogLevel::WARN,
                &format!(
                    "Unhandled webhook: type={}, action={}, payload={:?}",
                    t, a, payload
                ),
            );
        }
    }

    StatusCode::OK
}

pub fn router<U: AppUtils, KV: KVStorage>(state: Arc<AppState<U, KV>>) -> Router {
    Router::new()
        .route("/tcs", post(webhook_handler))
        .with_state(state.clone())
        .route_layer(from_fn_with_state(state.clone(), sign_check))
}
