use std::sync::Arc;

use redb::{Database, ReadableTable, TableDefinition};

use crate::error::{Error, Result};

const VTL_CONFIG: TableDefinition<&str, &[u8]> = TableDefinition::new("vtl_config");
const USERS: TableDefinition<&str, &[u8]> = TableDefinition::new("users");

#[derive(Clone)]
pub struct Store {
    db: Arc<Database>,
}

impl Store {
    pub fn new(path: &str) -> Result<Self> {
        let db = Database::create(path)?;

        // Ensure tables exist.
        let txn = db.begin_write()?;
        {
            txn.open_table(VTL_CONFIG)?;
            txn.open_table(USERS)?;
        }
        txn.commit()?;

        Ok(Self { db: Arc::new(db) })
    }

    pub async fn config_get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let db = self.db.clone();
        let key = key.to_owned();
        tokio::task::spawn_blocking(move || {
            let txn = db.begin_read()?;
            let table = txn.open_table(VTL_CONFIG)?;
            match table.get(key.as_str())? {
                Some(v) => Ok(Some(v.value().to_vec())),
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Other(format!("spawn_blocking failed: {e}")))?
    }

    pub async fn config_set(&self, key: &str, value: &[u8]) -> Result<()> {
        let db = self.db.clone();
        let key = key.to_owned();
        let value = value.to_vec();
        tokio::task::spawn_blocking(move || {
            let txn = db.begin_write()?;
            {
                let mut table = txn.open_table(VTL_CONFIG)?;
                table.insert(key.as_str(), value.as_slice())?;
            }
            txn.commit()?;
            Ok(())
        })
        .await
        .map_err(|e| Error::Other(format!("spawn_blocking failed: {e}")))?
    }

    pub async fn config_list(&self) -> Result<Vec<(String, Vec<u8>)>> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let txn = db.begin_read()?;
            let table = txn.open_table(VTL_CONFIG)?;
            let mut entries = Vec::new();
            for item in table.iter()? {
                let (k, v) = item?;
                entries.push((k.value().to_owned(), v.value().to_vec()));
            }
            Ok(entries)
        })
        .await
        .map_err(|e| Error::Other(format!("spawn_blocking failed: {e}")))?
    }
}
