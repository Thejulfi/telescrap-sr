use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::path::Path;

use crate::controller::encounter_store::{EncounterRecord, StoreEncounters};
use crate::core::encounter::Encounter;

const ENCOUNTERS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("encounters");

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum StorageError {
    Redb(Box<dyn std::error::Error + Send + Sync>),
    Json(serde_json::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Redb(e) => write!(f, "Database error: {e}"),
            StorageError::Json(e) => write!(f, "Serialization error: {e}"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<serde_json::Error> for StorageError {
    fn from(e: serde_json::Error) -> Self {
        StorageError::Json(e)
    }
}

macro_rules! impl_from_redb_error {
    ($($t:ty),*) => {
        $(impl From<$t> for StorageError {
            fn from(e: $t) -> Self {
                StorageError::Redb(Box::new(e))
            }
        })*
    };
}

impl_from_redb_error!(
    redb::DatabaseError,
    redb::TransactionError,
    redb::TableError,
    redb::CommitError,
    redb::StorageError
);

// ---------------------------------------------------------------------------
// Persisted record
// ---------------------------------------------------------------------------

fn make_stable_id(title: &str, date: &str) -> String {
    format!("{title}|{date}")
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

pub struct EncounterStore {
    db: Database,
}

impl EncounterStore {
    /// Opens (or creates) the database at `path` and ensures the table exists.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = Database::create(path)?;
        let txn = db.begin_write()?;
        txn.open_table(ENCOUNTERS_TABLE)?;
        txn.commit()?;
        Ok(Self { db })
    }
}

impl StoreEncounters for EncounterStore {
    fn upsert(&self, encounter: &Encounter) -> Result<(), String> {
        (|| -> Result<(), StorageError> {
            let stable_id = make_stable_id(&encounter.title, &encounter.date);
            let txn = self.db.begin_write()?;
            {
                let mut table = txn.open_table(ENCOUNTERS_TABLE)?;

                if let Some(resale_link) = &encounter.resale_link {
                    let record = EncounterRecord {
                        stable_id: stable_id.clone(),
                        title: encounter.title.clone(),
                        date: encounter.date.clone(),
                        club_type: format!("{:?}", encounter.club_type),
                        resale_link: resale_link.clone(),
                        resale_active: true,
                    };
                    let json = serde_json::to_string(&record)?;
                    table.insert(stable_id.as_str(), json.as_str())?;
                } else {
                    let existing_json = table
                        .get(stable_id.as_str())?
                        .map(|guard| guard.value().to_owned());

                    if let Some(existing_str) = existing_json {
                        let mut record: EncounterRecord = serde_json::from_str(&existing_str)?;
                        if record.resale_active {
                            record.resale_active = false;
                            let json = serde_json::to_string(&record)?;
                            table.insert(stable_id.as_str(), json.as_str())?;
                        }
                    }
                }
            }
            txn.commit()?;
            Ok(())
        })()
        .map_err(|e| e.to_string())
    }

    fn get_active_resale_links(&self) -> Result<Vec<EncounterRecord>, String> {
        (|| -> Result<Vec<EncounterRecord>, StorageError> {
            let txn = self.db.begin_read()?;
            let table = txn.open_table(ENCOUNTERS_TABLE)?;
            let mut result = Vec::new();
            for entry in table.iter()? {
                let item = entry?;
                if let Ok(record) = serde_json::from_str::<EncounterRecord>(item.1.value()) {
                    if record.resale_active {
                        result.push(record);
                    }
                }
            }
            Ok(result)
        })()
        .map_err(|e| e.to_string())
    }

    fn get_by_stable_id(&self, title: &str, date: &str) -> Result<Option<EncounterRecord>, String> {
        (|| -> Result<Option<EncounterRecord>, StorageError> {
            let stable_id = make_stable_id(title, date);
            let txn = self.db.begin_read()?;
            let table = txn.open_table(ENCOUNTERS_TABLE)?;
            match table.get(stable_id.as_str())? {
                Some(guard) => {
                    let record: EncounterRecord = serde_json::from_str(guard.value())?;
                    Ok(Some(record))
                }
                None => Ok(None),
            }
        })()
        .map_err(|e| e.to_string())
    }

    fn get_all(&self) -> Result<Vec<EncounterRecord>, String> {
        (|| -> Result<Vec<EncounterRecord>, StorageError> {
            let txn = self.db.begin_read()?;
            let table = txn.open_table(ENCOUNTERS_TABLE)?;
            let mut result = Vec::new();
            for entry in table.iter()? {
                let item = entry?;
                if let Ok(record) = serde_json::from_str::<EncounterRecord>(item.1.value()) {
                    result.push(record);
                }
            }
            Ok(result)
        })()
        .map_err(|e| e.to_string())
    }
}
