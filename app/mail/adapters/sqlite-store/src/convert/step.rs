//! Versioned steps between released schemas. Each step is one transaction
//! under the converter's lock after a verified backup; a step never rewrites
//! account data, so it completes in seconds and `converting` is not needed.
use super::lock::ConvertError;
use crate::{schema, storage};
use mail_kernel::Error;
use rusqlite::{Connection, params};

/// Whether a complete file at `version` has a step to this binary's schema.
pub fn has_step(version: u64) -> bool {
    version == 3
}

/// 3 → 4: the owner epoch replaces the random lease token on both queues.
/// Live leases are released; receipts make re-delivery safe and a stale
/// worker's settlement now fails on the epoch it never had. A retained
/// failure keeps its job's epoch so a retry continues the sequence. The
/// `token` column stays, unused, so the step is a pure addition. Also
/// applied to a legacy file, whose queue tables predate the column.
pub(super) fn add_epoch_columns(db: &Connection) -> Result<(), Error> {
    for table in ["delivery_jobs", "outbound_jobs", "failed_delivery_jobs"] {
        let present: bool = db
            .query_row(
                &format!(
                    "SELECT EXISTS(SELECT 1 FROM pragma_table_info('{table}') WHERE name='epoch')"
                ),
                [],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if !present {
            db.execute_batch(&format!(
                "ALTER TABLE {table} ADD COLUMN epoch INTEGER NOT NULL DEFAULT 0;"
            ))
            .map_err(storage)?;
            if table != "failed_delivery_jobs" {
                db.execute_batch(&format!(
                    "UPDATE {table} SET lease_until=0,next_attempt=min(next_attempt,unixepoch()) WHERE lease_until>unixepoch();"
                ))
                .map_err(storage)?;
            }
        }
    }
    Ok(())
}

/// Apply every step from `version` to `SCHEMA_VERSION`, recording each as a
/// complete `schema_version` row with the backup it was taken against.
pub fn advance(
    db: &Connection,
    version: u64,
    backup: &str,
    digest: &str,
    operator: &str,
) -> Result<(), ConvertError> {
    let mut at = version;
    while at < schema::SCHEMA_VERSION {
        let tx = db.unchecked_transaction().map_err(storage)?;
        match at {
            3 => add_epoch_columns(&tx)?,
            _ => {
                return Err(ConvertError::NotLegacy(schema::SchemaState::Complete {
                    version: at,
                }));
            }
        }
        tx.execute(
            "INSERT INTO schema_version(version,state,backup_path,backup_sha256,converted_at_utc,operator) VALUES(?1,'complete',?2,?3,strftime('%Y-%m-%dT%H:%M:%SZ','now'),?4)",
            params![at + 1, backup, digest, operator],
        )
        .map_err(storage)?;
        tx.commit().map_err(storage)?;
        at += 1;
    }
    Ok(())
}
