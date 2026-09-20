use super::{SqliteStore, queue, storage};
use mail_api::{DeliveryFailure, DeliveryLease};
use mail_kernel::Error;
use rusqlite::{OptionalExtension, TransactionBehavior, params};

pub(super) fn retain(
    tx: &rusqlite::Transaction<'_>,
    lease: &DeliveryLease,
    reason: Error,
) -> Result<(), Error> {
    // Independent retained content survives older workers' queued-message cleanup.
    tx.execute(
        "INSERT OR IGNORE INTO failed_delivery_messages SELECT * FROM queued_messages WHERE id=?1",
        [&lease.message],
    )
    .map_err(storage)?;
    // The epoch is retained: a retry continues the job's owner sequence.
    tx.execute("INSERT INTO failed_delivery_jobs(message,account,address,failed_at,reason,epoch) SELECT message,account,address,unixepoch(),?3,epoch FROM delivery_jobs WHERE message=?1 AND account=?2", params![lease.message,lease.account,format!("{reason:?}")]).map_err(storage)?;
    Ok(())
}

pub(super) fn list(
    store: &SqliteStore,
    account: &str,
    limit: usize,
) -> Result<Vec<DeliveryFailure>, Error> {
    let db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let mut query = db.prepare("SELECT message,account,failed_at,reason FROM failed_delivery_jobs WHERE account=?1 ORDER BY failed_at,message LIMIT ?2").map_err(storage)?;
    query
        .query_map(params![account, limit.min(1000)], |r| {
            Ok(DeliveryFailure {
                message: r.get(0)?,
                account: r.get(1)?,
                failed_at: r.get(2)?,
                reason: r.get(3)?,
            })
        })
        .map_err(storage)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage)
}

pub(super) fn retry(store: &SqliteStore, account: &str, message: &str) -> Result<(), Error> {
    let mut db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    let (address, size, epoch): (String, usize, u64) = tx.query_row("SELECT j.address,m.size,j.epoch FROM failed_delivery_jobs j JOIN failed_delivery_messages m ON m.id=j.message WHERE j.message=?1 AND j.account=?2", params![message,account], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?.ok_or(Error::NotFound)?;
    tx.execute(
        "DELETE FROM failed_delivery_jobs WHERE message=?1 AND account=?2",
        params![message, account],
    )
    .map_err(storage)?;
    queue::admit(&tx, account, &address, size)?;
    let mismatch: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM queued_messages q JOIN failed_delivery_messages f ON q.id=f.id WHERE q.id=?1 AND (q.sender!=f.sender OR q.content!=f.content OR q.received_at!=f.received_at))", [message], |r| r.get(0)).map_err(storage)?;
    if mismatch {
        return Err(Error::Conflict);
    }
    tx.execute(
        "INSERT OR IGNORE INTO queued_messages SELECT * FROM failed_delivery_messages WHERE id=?1",
        [message],
    )
    .map_err(storage)?;
    tx.execute("INSERT INTO delivery_jobs(message,account,address,next_attempt,epoch) VALUES(?1,?2,?3,unixepoch(),?4)", params![message,account,address,epoch]).map_err(storage)?;
    tx.execute("DELETE FROM failed_delivery_messages WHERE id=?1 AND NOT EXISTS(SELECT 1 FROM failed_delivery_jobs WHERE message=?1)", [message]).map_err(storage)?;
    tx.commit().map_err(storage)
}
