use crate::utils::decrypt;
use axum::Json;
use bitvec::prelude::{BitSlice, Lsb0};
use phi_save_codec::game_key::{field::GameKey, serde::SerializableGameKey};
use phi_save_codec::game_progress::{field::GameProgress, serde::SerializableGameProgress};
use phi_save_codec::game_record::{field::GameRecord, serde::SerializableGameRecord};
use phi_save_codec::user::{field::User, serde::SerializableUser};
use serde_json::json;
use shua_struct::field::BinaryField;
use std::io::Cursor;
use std::io::Read;
use std::sync::Arc;
use zip::ZipArchive;

const SAVE_LIST: &[&str] = &["gameKey", "gameProgress", "gameRecord", "user"];

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::types::LogLevel;
use crate::types::{AppState, AppUtils, KVStorage, KVTable};

#[derive(Default)]
struct Zip {
    game_progress: Vec<u8>,
    game_record: Vec<u8>,
    user: Vec<u8>,
    game_key: Vec<u8>,
}
fn unzip(save: Cursor<Vec<u8>>) -> Result<Zip, (String, StatusCode)> {
    let mut zi = match ZipArchive::new(save) {
        Ok(z) => z,
        Err(e) => {
            return Err((
                format!("Failed to open zip archive: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let mut zip = Zip::default();

    for &file_name in SAVE_LIST {
        let mut file = match zi.by_name(file_name) {
            Ok(f) => f,
            Err(e) => {
                return Err((
                    format!("Failed to read file {}: {}", file_name, e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        };

        let mut buf = Vec::new();
        if file.read_to_end(&mut buf).is_err() {
            return Err((
                format!("Failed to read file {} content", file_name),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        // 删除头
        buf.drain(0..1);

        // 解密
        let data = match decrypt(&buf) {
            Ok(v) => v,
            Err(e) => {
                return Err((
                    format!("Failed to decrypt file {}: {}", file_name, e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        };

        match file_name {
            "gameProgress" => zip.game_progress = data,
            "gameRecord" => zip.game_record = data,
            "user" => zip.user = data,
            "gameKey" => zip.game_key = data,
            _ => {}
        }
    }

    Ok(zip)
}

pub async fn handler<U: AppUtils, KV: KVStorage>(
    State(state): State<Arc<AppState<U, KV>>>,
    Path(open_id): Path<String>,
) -> axum::response::Response {
    let save: Cursor<Vec<u8>> = match state.kv.open_table("save").await.get(&open_id).await {
        Some(v) => Cursor::new(v),
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let zip = match unzip(save) {
        Ok(z) => z,
        Err((msg, code)) => {
            state.utils.logger(LogLevel::ERROR, &msg);
            return code.into_response();
        }
    };

    // Parse gameKey
    let game_key_json = if !zip.game_key.is_empty() {
        let bits = BitSlice::<u8, Lsb0>::from_slice(&zip.game_key);
        match GameKey::parse(bits, &None) {
            Ok((item, _)) => Some(SerializableGameKey::from(item)),
            Err(e) => {
                state
                    .utils
                    .logger(LogLevel::ERROR, &format!("Failed to parse gameKey: {}", e));
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    } else {
        None
    };

    // Parse gameProgress
    let game_progress_json = if !zip.game_progress.is_empty() {
        let bits = BitSlice::<u8, Lsb0>::from_slice(&zip.game_progress);
        match GameProgress::parse(bits, &None) {
            Ok((item, _)) => Some(SerializableGameProgress::from(item)),
            Err(e) => {
                state.utils.logger(
                    LogLevel::ERROR,
                    &format!("Failed to parse gameProgress: {}", e),
                );
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    } else {
        None
    };

    // Parse gameRecord
    let game_record_json = if !zip.game_record.is_empty() {
        let bits = BitSlice::<u8, Lsb0>::from_slice(&zip.game_record);
        match GameRecord::parse(bits, &None) {
            Ok((item, _)) => Some(SerializableGameRecord::from(item)),
            Err(e) => {
                state.utils.logger(
                    LogLevel::ERROR,
                    &format!("Failed to parse gameRecord: {}", e),
                );
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    } else {
        None
    };

    // Parse user
    let user_json = if !zip.user.is_empty() {
        let bits = BitSlice::<u8, Lsb0>::from_slice(&zip.user);
        match User::parse(bits, &None) {
            Ok((item, _)) => Some(SerializableUser::from(item)),
            Err(e) => {
                state
                    .utils
                    .logger(LogLevel::ERROR, &format!("Failed to parse user: {}", e));
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    } else {
        None
    };

    let response = json!({
        "gameKey": game_key_json,
        "gameProgress": game_progress_json,
        "gameRecord": game_record_json,
        "user": user_json
    });

    Json(response).into_response()
}
