use serde::Deserialize;
use serde_json;
use std::sync::Arc;

use crate::types::{AppState, AppUtils, KVStorage, KVTable, LogLevel};

use super::WebhookPayload;

#[derive(Deserialize, Debug)]
struct Data {
    file_object_id: String,
    #[allow(dead_code)]
    summary: String,
}

pub async fn handle_save<U: AppUtils, KV: KVStorage>(
    payload: &WebhookPayload,
    state: &Arc<AppState<U, KV>>,
) {
    let data: Data = match serde_json::from_value(payload.data.clone()) {
        Ok(d) => d,
        Err(e) => {
            state
                .utils
                .logger(LogLevel::ERROR, &format!("Failed to parse data: {}", e));
            return;
        }
    };

    let openid = &payload.user.openid;
    let save = state.kv.open_table("save").await;
    let file_data = state.utils.get_file(&data.file_object_id);
    let user = state.kv.open_table("user").await;
    save.put(openid, &file_data.await).await;
    user.put(openid, &payload.user.nickname.as_bytes()).await;
}
