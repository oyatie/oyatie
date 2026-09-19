//! The converter connection: `locking_mode=EXCLUSIVE` before its first
//! access, a short busy timeout, refusal on `SQLITE_BUSY`, and a checkpoint on
//! close so no `-wal` remains beside the converted file.
use super::{account, verify, verify::BackupCheck};
use crate::{schema, storage};
use mail_kernel::Error;
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum ConvertError {
    /// Another connection has touched the database: stop `serve` first.
    Busy,
    Backup(BackupCheck),
    /// The database is already at (or above) this binary's version.
    NotLegacy(schema::SchemaState),
    /// A resumed conversion names a different backup than the one recorded.
    BackupMismatch {
        recorded: String,
    },
    Storage(Error),
}
impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Busy => write!(f, "database is in use (SQLITE_BUSY): stop `serve` first"),
            Self::Backup(check) => write!(f, "backup refused: {check}"),
            Self::NotLegacy(state) => write!(f, "nothing to convert: {state:?}"),
            Self::BackupMismatch { recorded } => {
                write!(
                    f,
                    "conversion in progress was started with backup {recorded}"
                )
            }
            Self::Storage(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for ConvertError {}
impl From<Error> for ConvertError {
    fn from(error: Error) -> Self {
        match error {
            Error::Busy => Self::Busy,
            other => Self::Storage(other),
        }
    }
}

/// Audited outcome of a conversion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conversion {
    pub accounts: u64,
    pub elapsed: Duration,
    pub backup_path: PathBuf,
    pub backup_sha256: String,
}

pub struct Converter {
    db: Connection,
    database: PathBuf,
}

impl Converter {
    /// Open with the exclusive lock held for the connection's lifetime; a
    /// database another connection has touched yields `Busy`.
    pub fn open(database: &Path) -> Result<Self, ConvertError> {
        let db = Connection::open(database).map_err(storage)?;
        db.busy_timeout(Duration::from_millis(250))
            .map_err(storage)?;
        db.pragma_update(None, "locking_mode", "EXCLUSIVE")
            .map_err(storage)?;
        // First access: takes and keeps the lock, or reports contention.
        db.execute_batch("BEGIN IMMEDIATE; COMMIT;")
            .map_err(storage)?;
        Ok(Self {
            db,
            database: database.to_path_buf(),
        })
    }

    /// Produce a consistent single-file backup under the lock (`VACUUM INTO`).
    pub fn backup_into(&self, path: &Path) -> Result<(), ConvertError> {
        if path.exists() {
            return Err(ConvertError::Backup(BackupCheck::Shape));
        }
        self.db
            .execute("VACUUM INTO ?1", [path.to_string_lossy().as_ref()])
            .map_err(storage)?;
        Ok(())
    }

    /// Verify the backup, then convert every account in its own transaction;
    /// `schema_version` flips to `complete` last and records the audit row.
    pub fn convert(self, backup: &Path, operator: &str) -> Result<Conversion, ConvertError> {
        let started = Instant::now();
        let state = schema::inspect(&self.db)?;
        match &state {
            schema::SchemaState::Legacy => {}
            schema::SchemaState::Converting { .. } => {
                let (recorded, recorded_digest): (String, String) = self
                    .db
                    .query_row(
                        "SELECT coalesce(backup_path,''),coalesce(backup_sha256,'') FROM schema_version",
                        [],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .map_err(storage)?;
                // The recorded backup, by path and by content: a resumed
                // conversion may not be pointed at another file.
                let same = std::fs::canonicalize(&recorded)
                    .ok()
                    .is_some_and(|r| std::fs::canonicalize(backup).ok() == Some(r));
                let digest = std::fs::read(backup)
                    .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
                    .unwrap_or_default();
                if !same || digest != recorded_digest {
                    return Err(ConvertError::BackupMismatch { recorded });
                }
            }
            complete => return Err(ConvertError::NotLegacy(complete.clone())),
        }
        verify::backup(&self.db, &self.database, backup)?.map_err(ConvertError::Backup)?;
        let digest = format!(
            "{:x}",
            Sha256::digest(std::fs::read(backup).map_err(|_| Error::Unavailable)?)
        );
        if state == schema::SchemaState::Legacy {
            self.begin(backup, &digest)?;
        }
        let pending = account::pending(&self.db)?;
        let accounts = pending.len() as u64;
        for id in pending {
            let indexed: bool = self
                .db
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM legacy_thread_indexed WHERE account=?1)",
                    [&id],
                    |r| r.get(0),
                )
                .map_err(storage)?;
            let tx = self.db.unchecked_transaction().map_err(storage)?;
            account::convert(&tx, &id, indexed)?;
            tx.commit().map_err(storage)?;
        }
        let tx = self.db.unchecked_transaction().map_err(storage)?;
        tx.execute_batch("DROP TABLE legacy_accounts; DROP TABLE legacy_thread_indexed; DROP TABLE IF EXISTS message_bodies; DROP TABLE IF EXISTS blobs;")
            .map_err(storage)?;
        tx.execute(
            "UPDATE schema_version SET state='complete',converted_at_utc=strftime('%Y-%m-%dT%H:%M:%SZ','now'),operator=?1 WHERE version=?2",
            params![operator, schema::SCHEMA_VERSION],
        )
        .map_err(storage)?;
        tx.commit().map_err(storage)?;
        self.db
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(storage)?;
        Ok(Conversion {
            accounts,
            elapsed: started.elapsed(),
            backup_path: backup.to_path_buf(),
            backup_sha256: digest,
        })
    }

    /// DDL, legacy renames and the `converting` marker in one transaction.
    fn begin(&self, backup: &Path, digest: &str) -> Result<(), Error> {
        let mut ddl = String::new();
        for (kind, name) in account::DROPPED {
            if *name != "thread_indexed" {
                ddl.push_str(&format!("DROP {kind} IF EXISTS {name};"));
            }
        }
        ddl.push_str(
            "DROP TABLE IF EXISTS history_commits;
             DROP INDEX IF EXISTS account_credentials;
             ALTER TABLE accounts RENAME TO legacy_accounts;
             CREATE TABLE IF NOT EXISTS thread_indexed(account TEXT PRIMARY KEY);
             ALTER TABLE thread_indexed RENAME TO legacy_thread_indexed;",
        );
        ddl.push_str(schema::DDL);
        ddl.push_str(crate::blob::DDL);
        let tx = self.db.unchecked_transaction().map_err(storage)?;
        tx.execute_batch(&ddl).map_err(storage)?;
        tx.execute(
            "INSERT INTO schema_version(version,state,backup_path,backup_sha256) VALUES(?1,'converting',?2,?3)",
            params![schema::SCHEMA_VERSION, backup.to_string_lossy(), digest],
        )
        .map_err(storage)?;
        tx.commit().map_err(storage)
    }
}
