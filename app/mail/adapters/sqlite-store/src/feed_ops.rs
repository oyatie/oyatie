//! `ChangeFeed` operator actions: every one writes an audited durable row.
use super::feed::{audited, tail};
use super::{SqliteStore, storage};
use mail_api::{AuditRow, Consumer};
use mail_kernel::Error;
use rusqlite::{OptionalExtension, TransactionBehavior, params};

impl SqliteStore {
    /// Every provisioned account id, for the cell's compaction pass.
    pub fn account_ids(&self) -> Result<Vec<String>, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let mut ids = db
            .prepare("SELECT id FROM accounts ORDER BY id")
            .map_err(storage)?;
        ids.query_map([], |r| r.get(0))
            .map_err(storage)?
            .collect::<Result<_, _>>()
            .map_err(storage)
    }
}

pub(super) fn reconcile(
    store: &SqliteStore,
    consumer: Consumer,
    operator: &str,
    reason: &str,
) -> Result<u64, Error> {
    let mut db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    let marked = tx.execute(
        "INSERT INTO feed_dirty(consumer,tenant,account) SELECT ?1,a.tenant,a.id FROM accounts a JOIN feed_cursors c ON c.consumer=?1 AND c.account=a.id WHERE c.revision<a.revision ON CONFLICT(consumer,account) DO NOTHING",
        [consumer.name()],
    )
    .map_err(storage)? as u64;
    audited(
        &tx,
        "reconcile",
        reason,
        operator,
        &format!("consumer={} re-marked={marked}", consumer.name()),
    )?;
    tx.commit().map_err(storage)?;
    Ok(marked)
}

pub(super) fn dead_letter(
    store: &SqliteStore,
    consumer: Consumer,
    account: &str,
    revision: u64,
    operator: &str,
    reason: &str,
) -> Result<(), Error> {
    let mut db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    let (tenant, tail, floor) = tail(&tx, account)?;
    if revision > tail || revision < floor {
        return Err(Error::Conflict);
    }
    // Only a poisoned change is dead-lettered; a live key is the consumer's.
    let poisoned: Option<String> = tx
        .query_row(
            "SELECT poison FROM feed_dirty WHERE consumer=?1 AND account=?2",
            params![consumer.name(), account],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?
        .flatten();
    if poisoned.is_none() {
        return Err(Error::Conflict);
    }
    tx.execute(
        "INSERT INTO feed_cursors(consumer,account,revision) VALUES(?1,?2,?3) ON CONFLICT(consumer,account) DO UPDATE SET revision=max(revision,excluded.revision)",
        params![consumer.name(), account, revision],
    )
    .map_err(storage)?;
    tx.execute(
        "UPDATE feed_dirty SET poison=NULL,not_before=0 WHERE consumer=?1 AND account=?2",
        params![consumer.name(), account],
    )
    .map_err(storage)?;
    audited(
        &tx,
        "dead-letter",
        reason,
        operator,
        &format!(
            "consumer={} tenant={tenant} account={account} revision={revision}",
            consumer.name()
        ),
    )?;
    tx.commit().map_err(storage)
}

pub(super) fn retire_cursor(
    store: &SqliteStore,
    consumer: Consumer,
    tenant: Option<&str>,
    operator: &str,
    reason: &str,
) -> Result<u64, Error> {
    let mut db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    let retired = match tenant {
        Some(tenant) => tx.execute(
            "DELETE FROM feed_cursors WHERE consumer=?1 AND account IN (SELECT id FROM accounts WHERE tenant=?2)",
            params![consumer.name(), tenant],
        ),
        None => tx.execute("DELETE FROM feed_cursors WHERE consumer=?1", [consumer.name()]),
    }
    .map_err(storage)? as u64;
    match tenant {
        Some(tenant) => tx.execute(
            "DELETE FROM feed_dirty WHERE consumer=?1 AND tenant=?2",
            params![consumer.name(), tenant],
        ),
        None => tx.execute(
            "DELETE FROM feed_dirty WHERE consumer=?1",
            [consumer.name()],
        ),
    }
    .map_err(storage)?;
    audited(
        &tx,
        "cursor-retired",
        reason,
        operator,
        &format!(
            "consumer={} tenant={} retired={retired}",
            consumer.name(),
            tenant.unwrap_or("*")
        ),
    )?;
    tx.commit().map_err(storage)?;
    Ok(retired)
}

pub(super) fn audit(store: &SqliteStore, limit: usize) -> Result<Vec<AuditRow>, Error> {
    let db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let mut query = db
        .prepare(
            "SELECT id,kind,reason,at_utc,operator,detail FROM audit ORDER BY id DESC LIMIT ?1",
        )
        .map_err(storage)?;
    query
        .query_map([limit.min(1000) as i64], |r| {
            Ok(AuditRow {
                id: r.get(0)?,
                kind: r.get(1)?,
                reason: r.get(2)?,
                at_utc: r.get(3)?,
                operator: r.get(4)?,
                detail: r.get(5)?,
            })
        })
        .map_err(storage)?
        .collect::<Result<_, _>>()
        .map_err(storage)
}
