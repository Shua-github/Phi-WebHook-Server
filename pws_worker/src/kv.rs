use async_trait::async_trait;
use pws_core::types::{KVStorage, KVTable};
use worker::*;

use crate::utils::UnsafeSend;

#[derive(Clone)]
pub struct WorkerKVTable {
    pub table: KvStore,
}

#[derive(Clone)]
pub struct WorkerKVStorage {
    pub env: Env,
}

#[async_trait]
impl KVStorage for WorkerKVStorage {
    type Table = WorkerKVTable;

    async fn open_table(&self, table: &str) -> Self::Table {
        WorkerKVTable {
            table: self.env.kv(table).expect("无效表"),
        }
    }
}

#[async_trait]
impl KVTable for WorkerKVTable {
    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        UnsafeSend(async move { self.table.get(key).bytes().await.unwrap() }).await
    }

    async fn put(&self, key: &str, value: &[u8]) {
        UnsafeSend(async move {
            self.table
                .put_bytes(key, value)
                .unwrap()
                .execute()
                .await
                .unwrap();
        })
        .await
    }

    async fn delete(&self, key: &str) {
        UnsafeSend(async move {
            self.table.delete(key).await.unwrap();
        })
        .await
    }
}
