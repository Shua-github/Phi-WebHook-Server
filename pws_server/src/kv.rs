use async_trait::async_trait;
use pws_core::types::{KVStorage, KVTable};
use redb::{Database, ReadableDatabase, TableDefinition};
use std::sync::Arc;

#[derive(Clone)]
pub struct RedbKVTable {
    db: Arc<Database>,
    table_name: String,
}

impl RedbKVTable {
    pub fn new(db: Arc<Database>, table_name: String) -> Self {
        Self { db, table_name }
    }
}

pub struct RedbKVStorage {
    db: Arc<Database>,
}

impl RedbKVStorage {
    pub fn new(path: String) -> Result<Self, redb::Error> {
        let db = Arc::new(Database::create(path)?);
        Ok(Self { db })
    }
}

#[async_trait]
impl KVStorage for RedbKVStorage {
    type Table = RedbKVTable;

    async fn open_table(&self, table: &str) -> Self::Table {
        RedbKVTable::new(self.db.clone(), table.to_string())
    }
}

#[async_trait]
impl KVTable for RedbKVTable {
    async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let table: TableDefinition<&str, Vec<u8>> = TableDefinition::new(&self.table_name);
        let read_txn = self.db.begin_read().expect("Failed to begin read");
        let table = read_txn.open_table(table).expect("Failed to open table");
        let value = table.get(key);
        match value {
            Ok(Some(v)) => Some(v.value().clone()),
            _ => None,
        }
    }

    async fn put(&self, key: &str, value: &[u8]) {
        let table: TableDefinition<&str, Vec<u8>> = TableDefinition::new(&self.table_name);
        let write_txn = self.db.begin_write().expect("Failed to begin write");
        {
            let mut table = write_txn.open_table(table).expect("Failed to open table");
            table.insert(key, value.to_vec()).expect("Failed to insert");
        }
        write_txn.commit().expect("Failed to commit");
    }

    async fn delete(&self, key: &str) {
        let table: TableDefinition<&str, Vec<u8>> = TableDefinition::new(&self.table_name);
        let write_txn = self.db.begin_write().expect("Failed to begin write");
        {
            let mut table = write_txn.open_table(table).expect("Failed to open table");
            table.remove(key).expect("Failed to remove");
        }
        write_txn.commit().expect("Failed to commit");
    }
}
