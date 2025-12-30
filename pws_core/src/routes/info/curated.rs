use super::utils::{parse_save, unzip};
use axum::Json;
use phi_save_codec::game_progress::serde::SerializableMoney;
use phi_save_codec::game_record::serde::SerializableGameRecord;
use phi_save_codec::user::serde::SerializableUser;
use serde::Serialize;
use std::io::Cursor;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::types::LogLevel;
use crate::types::{AppState, AppUtils, KVStorage, KVTable};

#[derive(Serialize)]
struct Curated {
    nickname: String,
    user: SerializableUser,
    money: SerializableMoney,
    device_name: String,
    record: SerializableGameRecord,
}

pub async fn handler<U: AppUtils, KV: KVStorage>(
    State(state): State<Arc<AppState<U, KV>>>,
    Path(open_id): Path<String>,
) -> axum::response::Response {
    let save: Cursor<Vec<u8>> = match state.kv.open_table("save").await.get(&open_id).await {
        Some(v) => Cursor::new(v),
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let nickname = match state.kv.open_table("user").await.get(&open_id).await {
        Some(v) => String::from_utf8_lossy(&v).into_owned(),
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let zip = match unzip(save) {
        Ok(z) => z,
        Err(msg) => {
            state.utils.logger(LogLevel::ERROR, &msg);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let save = match parse_save(zip) {
        Ok(z) => z,
        Err(msg) => {
            state.utils.logger(LogLevel::ERROR, &msg);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let curated = Curated {
        nickname: nickname,
        device_name: save.settings.device_name,
        money: save.game_progress.money,
        record: save.game_record,
        user: save.user,
    };
    Json(curated).into_response()
}
