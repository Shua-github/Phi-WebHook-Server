use async_trait::async_trait;
use axum::response::IntoResponse;
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use blake2s_simd::{Params, State};
use pws_core::types::{AppUtils, LogLevel};
use reqwest::{Client, StatusCode};

fn sign(key: Vec<u8>, data: &[u8]) -> String {
    let mut state: State = Params::new().hash_length(16).key(&key).to_state();

    state.update(data);
    let hash = state.finalize();
    URL_SAFE.encode(hash.as_bytes())
}

pub struct ServerUtils {
    file_url_template: String,
    client: Client,
    sign_key: Vec<u8>,
    log_level: LogLevel,
}

impl ServerUtils {
    pub fn new(file_url_template: String, sign_key: Vec<u8>, log_level: LogLevel) -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
        Self {
            file_url_template,
            client,
            sign_key,
            log_level,
        }
    }

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
impl AppUtils for ServerUtils {
    async fn get_file(&self, file_obj_id: &str) -> Vec<u8> {
        let url = self.file_url_template.replace("{file_obj_id}", file_obj_id);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .expect("Failed to get file");
        resp.bytes().await.expect("Failed to read bytes").to_vec()
    }

    fn sign(&self, data: &[u8]) -> String {
        sign(self.sign_key.clone(), data)
    }

    fn logger(&self, level: LogLevel, msg: &str) {
        if self.log_level as u8 <= level as u8 {
            println!("[{}] {}", Self::get_level_str(level), msg);
        }
    }
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found :(")
}
