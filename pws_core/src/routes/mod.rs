mod info;
mod webhook;

use crate::types::{AppState, AppUtils, KVStorage};
use axum::Router;
use std::sync::Arc;

pub fn router<U: AppUtils, KV: KVStorage>(state: Arc<AppState<U, KV>>) -> Router {
    Router::new()
        .nest("/webhook", webhook::router(state.clone()))
        .nest("/info", info::router(state))
}
