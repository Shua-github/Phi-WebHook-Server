mod kv;
mod sign;
mod utils;

use std::sync::Arc;

use pws_core::routes::router;
use pws_core::types::{AppState, LogLevel};
use serde::Deserialize;
use serde::de::value::Error as DeError;
use serde::de::value::StrDeserializer;
use tower_service::Service;
use worker::*;

use crate::{kv::WorkerKVStorage, utils::WorkerUtils};

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: worker::Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    let fut = env
        .var("FILE_URL_TEMPLATE")
        .expect("模板获取失败")
        .to_string();

    let sign_key = env
        .secret("SIGN_KEY")
        .expect("签名KEY获取失败")
        .to_string()
        .as_bytes()
        .to_vec();

    let log_level_str = env.var("LOG_LEVEL").expect("日志等级获取失败").to_string();
    let deserializer = StrDeserializer::<DeError>::new(&log_level_str);
    let log_level: LogLevel = LogLevel::deserialize(deserializer).expect("日志等级解析失败");

    let utils = WorkerUtils {
        file_url_template: fut,
        log_level,
        sign_key,
    };
    let kv = WorkerKVStorage { env: env.clone() };
    let state = Arc::new(AppState { utils, kv });
    Ok(router(state).call(req).await?)
}
