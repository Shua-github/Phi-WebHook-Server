use async_trait::async_trait;
use serde::Deserialize;

#[async_trait]
pub trait KVStorage: Send + Sync + 'static {
    type Table: KVTable;
    async fn open_table(&self, table: &str) -> Self::Table;
}

#[async_trait]
pub trait KVTable: Send + Sync {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    async fn put(&self, key: &str, value: &[u8]);
    async fn delete(&self, key: &str);
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    #[serde(other)]
    UNKNOWN,
}

#[async_trait]
pub trait AppUtils: Send + Sync + 'static {
    async fn get_file(&self, file_obj_id: &str) -> Vec<u8>;
    fn sign(&self, data: &[u8]) -> String;
    fn logger(&self, level: LogLevel, msg: &str);
}

pub struct AppState<U: AppUtils, KV: KVStorage> {
    pub utils: U,
    pub kv: KV,
}
