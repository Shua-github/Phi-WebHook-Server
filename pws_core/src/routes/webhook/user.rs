use std::sync::Arc;

use crate::types::{AppState, AppUtils, KVStorage, KVTable};

use super::WebhookPayload;

pub async fn handle_user_update_login_create<U: AppUtils, KV: KVStorage>(
    payload: &WebhookPayload,
    state: &Arc<AppState<U, KV>>,
) {
    let openid = &payload.user.openid;
    let user = state.kv.open_table("user").await;
    user.put(openid, &payload.user.nickname.as_bytes()).await;
}
