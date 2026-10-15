//! Bound SQLite model metadata; production activation remains a facade concern.
#![forbid(unsafe_code)]

mod codec;
mod load;
#[cfg(test)]
mod tests;
mod write;

use foundry_model_api::*;
use foundry_store_local_draft::{LocalStoreGuard, StoreOpen};
use rusqlite::{Connection, OpenFlags, params};
use std::path::Path;

pub struct SqliteModelStore {
    connection: Connection,
    access: LocalStoreGuard,
    binding: ModelBinding,
    baseline: ModelBaseline,
}

const SCHEMA: &str = "
CREATE TABLE ontology_model_binding (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    binding BLOB NOT NULL, baseline BLOB NOT NULL,
    head INTEGER NOT NULL CHECK (head > 0),
    next_draft INTEGER NOT NULL CHECK (next_draft > 0)
);
CREATE TABLE ontology_model_draft (
    id INTEGER PRIMARY KEY CHECK (id > 0), record BLOB NOT NULL
);
CREATE TABLE ontology_model_publication (
    generation INTEGER PRIMARY KEY CHECK (generation > 0),
    draft_id INTEGER UNIQUE REFERENCES ontology_model_draft(id), record BLOB NOT NULL,
    CHECK ((generation = 1 AND draft_id IS NULL) OR (generation > 1 AND draft_id IS NOT NULL))
);";

impl SqliteModelStore {
    /// Explicit bootstrap only; never a recovery path for established state.
    pub fn initialize(
        path: &Path,
        binding: ModelBinding,
        baseline: ModelBaseline,
    ) -> Result<Self, ModelError> {
        let ledger = ModelLedger::initialize(binding.clone(), baseline.clone())?;
        let access = LocalStoreGuard::shared(path, StoreOpen::Create).map_err(access_error)?;
        access.validate(path).map_err(access_error)?;
        let mut connection = Connection::open_with_flags(
            access.path(),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(storage)?;
        access
            .validate(Path::new(connection.path().ok_or_else(|| {
                access_error(foundry_store_local_draft::StoreAccessError::IdentityChanged)
            })?))
            .map_err(access_error)?;
        configure(&connection)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage)?;
        match model_table_count(&transaction)? {
            0 => {}
            3 => return Err(ModelError::AlreadyInitialized),
            _ => return Err(ModelError::CorruptState),
        }
        transaction.execute_batch(SCHEMA).map_err(storage)?;
        #[cfg(test)]
        tests::initialization_fault("schema")?;
        transaction
            .execute(
                "INSERT INTO ontology_model_binding VALUES (1, 1, ?1, ?2, 1, 1)",
                params![encode(&binding)?, encode(&baseline)?],
            )
            .map_err(storage)?;
        #[cfg(test)]
        tests::initialization_fault("binding")?;
        transaction
            .execute(
                "INSERT INTO ontology_model_publication VALUES (1, NULL, ?1)",
                params![encode(&ledger.history_record().publications[0])?],
            )
            .map_err(storage)?;
        #[cfg(test)]
        tests::initialization_fault("publication")?;
        #[cfg(test)]
        tests::initialization_fault("commit")?;
        transaction.commit().map_err(storage)?;
        Ok(Self {
            connection,
            access,
            binding,
            baseline,
        })
    }

    pub fn open_existing(
        path: &Path,
        binding: ModelBinding,
        baseline: ModelBaseline,
    ) -> Result<Self, ModelError> {
        binding.validate()?;
        baseline.check_bounds()?;
        let access = LocalStoreGuard::shared(path, StoreOpen::Existing).map_err(|error| {
            if !path.exists() {
                ModelError::Uninitialized
            } else {
                access_error(error)
            }
        })?;
        Self::open_existing_with_access(path, binding, baseline, access)
    }

    pub fn open_existing_with_access(
        path: &Path,
        binding: ModelBinding,
        baseline: ModelBaseline,
        access: LocalStoreGuard,
    ) -> Result<Self, ModelError> {
        access.validate(path).map_err(access_error)?;
        binding.validate()?;
        baseline.check_bounds()?;
        let connection = Connection::open_with_flags(
            access.path(),
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|error| {
            if !path.exists() {
                ModelError::Uninitialized
            } else {
                storage(error)
            }
        })?;
        access
            .validate(Path::new(connection.path().ok_or_else(|| {
                access_error(foundry_store_local_draft::StoreAccessError::IdentityChanged)
            })?))
            .map_err(access_error)?;
        configure(&connection)?;
        match model_table_count(&connection)? {
            0 => return Err(ModelError::Uninitialized),
            3 => {}
            _ => return Err(ModelError::CorruptState),
        }
        let store = Self {
            connection,
            access,
            binding,
            baseline,
        };
        store.read(|connection, state| load::audit_drafts(connection, &state.history))?;
        Ok(store)
    }

    fn read<T>(
        &self,
        operation: impl FnOnce(&Connection, &load::State) -> Result<T, ModelError>,
    ) -> Result<T, ModelError> {
        let transaction = self.connection.unchecked_transaction().map_err(storage)?;
        let state = load::state(&transaction, &self.binding, &self.baseline)?;
        let result = operation(&transaction, &state)?;
        transaction.commit().map_err(storage)?;
        Ok(result)
    }
}

fn configure(connection: &Connection) -> Result<(), ModelError> {
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(storage)?;
    connection
        .execute_batch(
            "PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL; PRAGMA foreign_keys = ON;",
        )
        .map_err(storage)
}

fn model_table_count(connection: &Connection) -> Result<usize, ModelError> {
    connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN (
        'ontology_model_binding', 'ontology_model_publication', 'ontology_model_draft')",
            [],
            |row| row.get(0),
        )
        .map_err(storage)
}

pub(crate) fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, ModelError> {
    serde_json::to_vec(value).map_err(|_| ModelError::CorruptState)
}

pub(crate) fn storage(error: rusqlite::Error) -> ModelError {
    ModelError::Storage {
        detail: error.to_string(),
    }
}

fn access_error(error: foundry_store_local_draft::StoreAccessError) -> ModelError {
    ModelError::Storage {
        detail: error.to_string(),
    }
}
