use super::{SqliteStore, storage};
use mail_api::{DeliveryLease, DeliveryQueue, DeliveryTarget, QueuedMessage};
use mail_kernel::{Error, MAX_MESSAGE_BYTES, valid_address, valid_identifier};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use std::collections::BTreeMap;

impl DeliveryQueue for SqliteStore {
    fn failed_deliveries(
        &self,
        account: &str,
        limit: usize,
    ) -> Result<Vec<mail_api::DeliveryFailure>, Error> {
        super::failures::list(self, account, limit)
    }
    fn retry_failed_delivery(&self, account: &str, message: &str) -> Result<(), Error> {
        super::failures::retry(self, account, message)
    }

    fn enqueue(
        &self,
        sender: &str,
        recipients: &[DeliveryTarget],
        raw: &[u8],
    ) -> Result<String, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let id = enqueue_tx(&tx, sender, recipients, raw)?;
        tx.commit().map_err(storage)?;
        Ok(id)
    }

    fn claim(&self, limit: usize) -> Result<Vec<DeliveryLease>, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut query = tx.prepare("SELECT message,account FROM delivery_jobs WHERE next_attempt<=unixepoch() AND lease_until<=unixepoch() ORDER BY next_attempt,message,account LIMIT ?1").map_err(storage)?;
        let jobs = query
            .query_map([limit.min(100)], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage)?;
        drop(query);
        let mut leases = vec![];
        for (message, account) in jobs {
            let (epoch, attempt) = tx.query_row("UPDATE delivery_jobs SET epoch=epoch+1,lease_until=unixepoch()+?3,next_attempt=unixepoch()+?3,attempt=attempt+1 WHERE message=?1 AND account=?2 RETURNING epoch,attempt",
                params![message,account,mail_api::QUEUE_LEASE_SECS], |r| Ok((r.get(0)?,r.get(1)?))).map_err(storage)?;
            tx.execute(
                "UPDATE submission_schedule SET claimed=1 WHERE message=?1",
                [&message],
            )
            .map_err(storage)?;
            leases.push(DeliveryLease {
                message,
                account,
                epoch,
                attempt,
            });
        }
        tx.commit().map_err(storage)?;
        Ok(leases)
    }

    fn queued_message(&self, lease: &DeliveryLease) -> Result<QueuedMessage, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        db.query_row("SELECT m.sender,m.content,m.received_at,coalesce((SELECT send_at FROM submission_schedule WHERE message=m.id),m.received_at) FROM queued_messages m JOIN delivery_jobs j ON j.message=m.id WHERE j.message=?1 AND j.account=?2 AND j.epoch=?3 AND j.lease_until>unixepoch()",
            params![lease.message,lease.account,lease.epoch], |r| Ok(QueuedMessage { sender:r.get(0)?,raw:r.get(1)?,received_at:r.get(2)?,retry_at:r.get(3)? }))
            .optional().map_err(storage)?.ok_or(Error::Conflict)
    }

    fn finish(
        &self,
        lease: &DeliveryLease,
        outcome: Result<Option<String>, Error>,
    ) -> Result<(), Error> {
        let outcome = outcome.map(|_| ());
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let (attempt, expired, recipient): (u32, bool, String) = tx.query_row("SELECT j.attempt,coalesce((SELECT send_at FROM submission_schedule WHERE message=m.id),m.received_at)<=unixepoch()-432000,j.address FROM delivery_jobs j JOIN queued_messages m ON m.id=j.message WHERE j.message=?1 AND j.account=?2 AND j.epoch=?3 AND j.lease_until>unixepoch()", params![lease.message,lease.account,lease.epoch], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?.ok_or(Error::Conflict)?;
        let terminal = match outcome {
            Err(error) => {
                expired
                    || attempt >= mail_api::retry::MAX_ATTEMPTS
                    || mail_api::retry::classify(error) == mail_api::retry::Class::Terminal
            }
            Ok(()) => false,
        };
        if terminal && let Err(error) = outcome {
            super::failures::retain(&tx, lease, error)?;
        }
        let changed = match outcome {
            Err(error) if !terminal => tx.execute("UPDATE delivery_jobs SET lease_until=0,next_attempt=unixepoch()+?4,last_error=?5 WHERE message=?1 AND account=?2 AND epoch=?3 AND lease_until>unixepoch()",
                params![lease.message,lease.account,lease.epoch, mail_api::retry::delay_secs(attempt, super::outbound::entropy(&lease.message, attempt)),format!("{error:?}")]),
            _ => tx.execute("DELETE FROM delivery_jobs WHERE message=?1 AND account=?2 AND epoch=?3 AND lease_until>unixepoch()", params![lease.message,lease.account,lease.epoch]),
        }.map_err(storage)?;
        if changed != 1 {
            return Err(Error::Conflict);
        }
        let delivered = if outcome.is_ok() {
            mail_kernel::SubmissionDelivered::Yes
        } else if terminal {
            mail_kernel::SubmissionDelivered::No
        } else {
            mail_kernel::SubmissionDelivered::Queued
        };
        super::submission_history::finish(
            &tx,
            &lease.message,
            &recipient,
            delivered,
            if outcome.is_ok() {
                "250"
            } else if terminal {
                "550"
            } else {
                "451"
            },
        )?;
        tx.execute("DELETE FROM queued_messages WHERE id=?1 AND NOT EXISTS(SELECT 1 FROM delivery_jobs WHERE message=?1)", [&lease.message]).map_err(storage)?;
        tx.commit().map_err(storage)
    }
}

pub(super) fn enqueue_tx(
    tx: &rusqlite::Connection,
    sender: &str,
    recipients: &[DeliveryTarget],
    raw: &[u8],
) -> Result<String, Error> {
    if (!sender.is_empty() && !valid_address(sender))
        || recipients.is_empty()
        || recipients.len() > 100
    {
        return Err(Error::Invalid);
    }
    if raw.len() > MAX_MESSAGE_BYTES {
        return Err(Error::OverQuota);
    }
    super::threads::references(raw)?;
    let mut targets = BTreeMap::new();
    for target in recipients {
        if !valid_identifier(&target.account) || !valid_address(&target.address) {
            return Err(Error::Invalid);
        }
        if let Some(previous) = targets.insert(&target.account, &target.address)
            && !previous.eq_ignore_ascii_case(&target.address)
        {
            return Err(Error::Forbidden);
        }
    }
    for (account, address) in &targets {
        admit(tx, account, address, raw.len())?;
    }
    let id: String = tx
        .query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))
        .map_err(storage)?;
    tx.execute("INSERT INTO queued_messages(id,sender,content,size,received_at) VALUES(?1,?2,?3,?4,unixepoch())",
            params![id,sender,raw,raw.len()]).map_err(storage)?;
    for (account, address) in targets {
        tx.execute("INSERT INTO delivery_jobs(message,account,address,next_attempt) VALUES(?1,?2,?3,unixepoch())",
                params![id,account,address]).map_err(storage)?;
    }
    Ok(id)
}

pub(super) fn admit(
    tx: &rusqlite::Connection,
    account: &str,
    address: &str,
    size: usize,
) -> Result<(), Error> {
    let (actual, quota): (String, usize) = tx
        .query_row(
            "SELECT address,quota_bytes FROM accounts WHERE id=?1",
            [account],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(storage)?
        .ok_or(Error::NotFound)?;
    if !actual.eq_ignore_ascii_case(address) {
        return Err(Error::Forbidden);
    }
    let (count, bytes): (usize, usize) = tx.query_row(
                "SELECT count(*),coalesce(sum(size),0) FROM (SELECT m.size FROM delivery_jobs j JOIN queued_messages m ON m.id=j.message WHERE j.account=?1 UNION ALL SELECT m.size FROM failed_delivery_jobs j JOIN failed_delivery_messages m ON m.id=j.message WHERE j.account=?1)",
                [account], |r| Ok((r.get(0)?,r.get(1)?)),
            ).map_err(storage)?;
    let used: usize = tx
        .query_row(
            "SELECT used_bytes FROM accounts WHERE id=?1",
            [account],
            |r| r.get(0),
        )
        .map_err(storage)?;
    if count >= 1000
        || bytes
            .checked_add(size)
            .and_then(|n| n.checked_add(used))
            .is_none_or(|n| n > quota)
    {
        return Err(Error::OverQuota);
    }
    Ok(())
}

/// The SQLite tier is one process on one file: its connection mutex and
/// `IMMEDIATE` transactions are the fence, so node and batch leases are
/// no-ops here. The hosted tier holds real rows.
impl mail_api::NodeLease for SqliteStore {
    fn heartbeat(&self, _: &str, _: u64, _: i64) -> Result<(), Error> {
        Ok(())
    }
}
