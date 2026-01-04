use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;
use pws_core::types::{AppUtils, LogLevel};
use worker::{Fetch, Url, wasm_bindgen::JsValue, web_sys::console};

use crate::sign::sign;

pub struct UnsafeSend<F>(pub F);

unsafe impl<F> Send for UnsafeSend<F> {}

impl<F: Future> Future for UnsafeSend<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let inner = self.map_unchecked_mut(|s| &mut s.0);
            inner.poll(cx)
        }
    }
}

pub struct WorkerUtils {
    pub file_url_template: String,
    pub sign_key: Vec<u8>,
    pub log_level: LogLevel,
}

impl WorkerUtils {
    fn get_level_str(log_level: LogLevel) -> &'static str {
        match log_level {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
            LogLevel::UNKNOWN => "UNKNOWN",
        }
    }
}

#[async_trait]
impl AppUtils for WorkerUtils {
    async fn get_file(&self, file_obj_id: &str) -> Vec<u8> {
        let url = self.file_url_template.replace("{file_obj_id}", file_obj_id);
        let url = Url::parse(&url).expect("url error");
        UnsafeSend(async move {
            Fetch::Url(url)
                .send()
                .await
                .expect("Failed to get file")
                .bytes()
                .await
                .expect("Failed to read bytes")
        })
        .await
    }

    fn sign(&self, data: &[u8]) -> String {
        sign(&self.sign_key, data)
    }

    fn logger(&self, level: LogLevel, msg: &str) {
        if self.log_level as u8 <= level as u8 {
            console::log_1(&JsValue::from_str(&format!(
                "[{}] {}",
                Self::get_level_str(level),
                msg
            )));
        }
    }
}
