mod all;
mod curated;
mod utils;

use axum::Router;
use axum::routing::get;
use std::sync::Arc;

use crate::types::{AppState, AppUtils, KVStorage};

pub fn router<U: AppUtils, KV: KVStorage>(state: Arc<AppState<U, KV>>) -> Router {
    Router::new()
        .route("/{open_id}/all", get(all::handler))
        .route("/{open_id}/curated", get(curated::handler))
        .with_state(state.clone())
}
