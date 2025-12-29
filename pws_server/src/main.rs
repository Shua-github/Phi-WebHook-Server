mod kv;
mod types;
mod utils;

use std::sync::Arc;
use tokio::signal;

use pws_core::routes::router;
use pws_core::types::AppState;
use std::fs;

use crate::kv::RedbKVStorage;
use crate::utils::{ServerUtils, handler_404};

#[tokio::main]
async fn main() {
    let config_data: types::Config =
        serde_json::from_str(&fs::read_to_string("./config.json").unwrap()).unwrap();

    let utils = ServerUtils::new(
        config_data.file_url_template.clone(),
        config_data.sign_key.as_bytes().to_vec(),
        config_data.log_level,
    );

    let kv = RedbKVStorage::new(config_data.kv_storage_path.clone()).unwrap();

    let state = Arc::new(AppState { utils, kv });

    let app = router(state).fallback(handler_404);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind TCP listener");

    let server = axum::serve(listener, app);

    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        println!("Shutting down...");
    };

    tokio::select! {
        result = server => {
            if let Err(err) = result {
                eprintln!("Server error: {}", err);
            }
        },
        _ = shutdown_signal => {},
    }
}
