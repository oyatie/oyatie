use super::{SqliteProjectionStore, access_error, storage};
use foundry_projection_draft::ProjectionStoreError;
use foundry_store_local_draft::{LocalStoreGuard, StoreOpen};
use rusqlite::Connection;
use std::path::Path;

impl SqliteProjectionStore {
    /// Open (creating if absent) the projection at `path`.
    pub fn open(path: &Path) -> Result<Self, ProjectionStoreError> {
        let access = LocalStoreGuard::shared(path, StoreOpen::Create).map_err(access_error)?;
        Self::open_with_access(path, access)
    }

    pub fn open_with_access(
        path: &Path,
        access: LocalStoreGuard,
    ) -> Result<Self, ProjectionStoreError> {
        access.validate(path).map_err(access_error)?;
        let connection = Connection::open_with_flags(
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
        connection
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = FULL;
                 CREATE TABLE IF NOT EXISTS projection_heads (
                     tenant_id       TEXT    NOT NULL PRIMARY KEY,
                     applied_ordinal INTEGER NOT NULL
                 ) WITHOUT ROWID;
                 CREATE TABLE IF NOT EXISTS projection_entries (
                     tenant_id   TEXT    NOT NULL,
                     ordinal     INTEGER NOT NULL,
                     entry_bytes BLOB    NOT NULL,
                     PRIMARY KEY (tenant_id, ordinal)
                 ) WITHOUT ROWID;
                 CREATE TABLE IF NOT EXISTS projection_objects (
                     tenant_id    TEXT NOT NULL,
                     object_ref   TEXT NOT NULL,
                     entity_type  TEXT NOT NULL,
                     object_bytes BLOB NOT NULL,
                     PRIMARY KEY (tenant_id, object_ref)
                 ) WITHOUT ROWID;
                 CREATE INDEX IF NOT EXISTS projection_objects_type
                     ON projection_objects (tenant_id, entity_type, object_ref);
                 CREATE TABLE IF NOT EXISTS projection_property_index (
                     tenant_id   TEXT NOT NULL,
                     object_ref  TEXT NOT NULL,
                     entity_type TEXT NOT NULL,
                     property    TEXT NOT NULL,
                     value_kind  TEXT NOT NULL,
                     int_value   INTEGER,
                     real_value  REAL,
                     text_value  TEXT,
                     PRIMARY KEY (tenant_id, object_ref, property)
                 ) WITHOUT ROWID;
                 CREATE INDEX IF NOT EXISTS projection_property_kind
                     ON projection_property_index
                     (tenant_id, entity_type, property, value_kind, object_ref);
                 CREATE TABLE IF NOT EXISTS projection_links (
                     tenant_id  TEXT NOT NULL,
                     from_ref   TEXT NOT NULL,
                     link_type  TEXT NOT NULL,
                     to_ref     TEXT NOT NULL,
                     observed_at_epoch_ms INTEGER NOT NULL,
                     PRIMARY KEY (tenant_id, from_ref, link_type, to_ref)
                 ) WITHOUT ROWID;
                 CREATE INDEX IF NOT EXISTS projection_links_inbound
                     ON projection_links (tenant_id, to_ref, link_type, from_ref);
                 CREATE TABLE IF NOT EXISTS projection_poisons (
                     tenant_id TEXT    NOT NULL,
                     ordinal   INTEGER NOT NULL,
                     reason    TEXT    NOT NULL,
                     PRIMARY KEY (tenant_id, ordinal)
                 ) WITHOUT ROWID;",
            )
            .map_err(storage)?;
        access.validate(path).map_err(access_error)?;
        Ok(Self { connection, access })
    }
}
