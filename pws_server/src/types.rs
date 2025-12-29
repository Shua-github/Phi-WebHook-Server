use pws_core::types::LogLevel;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub log_level: LogLevel,
    pub kv_storage_path: String,
    pub sign_key: String,
    pub file_url_template: String,
}
