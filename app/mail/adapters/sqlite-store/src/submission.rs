use super::{SqliteStore, access, outbound, queue, storage};
use mail_api::{DeliveryOutcome, DeliveryTarget, OutboundLease, QueuedMessage, SubmissionQueue};
use mail_kernel::{Error, MAX_MESSAGE_BYTES, valid_address};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use std::collections::{BTreeMap, BTreeSet};

impl SubmissionQueue for SqliteStore {
    fn enqueue_submission(
        &self,
        account: &str,
        sender: &str,
        recipients: &[String],
        raw: &[u8],
    ) -> Result<String, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let id = enqueue_tx(&tx, account, sender, recipients, raw, true)?;
        tx.commit().map_err(storage)?;
        Ok(id)
    }

    fn claim_outbound(&self, limit: usize) -> Result<Vec<OutboundLease>, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut query = tx.prepare("SELECT message,account,recipient FROM outbound_jobs WHERE next_attempt<=unixepoch() AND lease_until<=unixepoch() ORDER BY next_attempt,message,recipient LIMIT ?1").map_err(storage)?;
        let jobs = query
            .query_map([limit.min(100)], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage)?;
        drop(query);
        let mut leases = vec![];
        for (message, account, recipient) in jobs {
            let (token,attempt) = tx.query_row("UPDATE outbound_jobs SET token=lower(hex(randomblob(16))),lease_until=unixepoch()+120,next_attempt=unixepoch()+120,attempt=attempt+1 WHERE message=?1 AND recipient=?2 RETURNING token,attempt",
                params![message,recipient], |r| Ok((r.get(0)?,r.get(1)?))).map_err(storage)?;
            tx.execute(
                "UPDATE submission_schedule SET claimed=1 WHERE message=?1",
                [&message],
            )
            .map_err(storage)?;
            leases.push(OutboundLease {
                message,
                account,
                recipient,
                token,
                attempt,
            });
        }
        tx.commit().map_err(storage)?;
        Ok(leases)
    }

    fn outbound_message(&self, lease: &OutboundLease) -> Result<QueuedMessage, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        db.query_row("SELECT m.sender,m.content,m.received_at,coalesce((SELECT send_at FROM submission_schedule WHERE message=m.id),m.received_at) FROM submitted_messages m JOIN outbound_jobs j ON j.message=m.id AND j.account=m.account WHERE j.message=?1 AND j.account=?2 AND j.recipient=?3 AND j.token=?4 AND j.lease_until>unixepoch()",
            params![lease.message,lease.account,lease.recipient,lease.token], |r| Ok(QueuedMessage { sender:r.get(0)?,raw:r.get(1)?,received_at:r.get(2)?,retry_at:r.get(3)? }))
            .optional().map_err(storage)?.ok_or(Error::Conflict)
    }

    fn renew_outbound(&self, lease: &OutboundLease) -> Result<(), Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let changed = db.execute("UPDATE outbound_jobs SET lease_until=unixepoch()+120,next_attempt=unixepoch()+120 WHERE message=?1 AND account=?2 AND recipient=?3 AND token=?4 AND lease_until>unixepoch()", params![lease.message, lease.account, lease.recipient, lease.token]).map_err(storage)?;
        if changed == 1 {
            Ok(())
        } else {
            Err(Error::Conflict)
        }
    }

    fn finish_outbound(
        &self,
        lease: &OutboundLease,
        outcome: DeliveryOutcome,
    ) -> Result<(), Error> {
        outbound::finish(self, lease, outcome)
    }
}

pub(super) fn enqueue_tx(
    tx: &rusqlite::Transaction<'_>,
    account: &str,
    sender: &str,
    recipients: &[String],
    raw: &[u8],
    allow_remote: bool,
) -> Result<String, Error> {
    if !valid_address(sender)
        || recipients.is_empty()
        || recipients.len() > 100
        || recipients.iter().any(|r| !valid_address(r))
    {
        return Err(Error::Invalid);
    }
    if raw.len() > MAX_MESSAGE_BYTES {
        return Err(Error::OverQuota);
    }
    super::threads::references(raw)?;
    let owner = access::load(tx, account)?;
    if !owner.address.eq_ignore_ascii_case(sender) {
        return Err(Error::Forbidden);
    }
    let mut local = BTreeMap::new();
    let mut remote = BTreeSet::new();
    for address in recipients {
        let local_account: Option<String> = tx
            .query_row(
                "SELECT id FROM accounts WHERE address=?1",
                [address.to_ascii_lowercase()],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?;
        if let Some(id) = local_account {
            local.insert(id, address.clone());
        } else {
            let (local, domain) = address.rsplit_once('@').ok_or(Error::Invalid)?;
            remote.insert(format!("{local}@{}", domain.to_ascii_lowercase()));
        }
    }
    if !allow_remote && !remote.is_empty() {
        return Err(Error::Forbidden);
    }
    let (count, bytes): (usize, usize) = tx.query_row(
            "SELECT count(*),coalesce(sum(size),0) FROM (SELECT m.size FROM outbound_jobs j JOIN submitted_messages m ON m.id=j.message WHERE j.account=?1 UNION ALL SELECT m.size FROM delivery_jobs j JOIN queued_messages m ON m.id=j.message WHERE j.account=?1 UNION ALL SELECT m.size FROM failed_delivery_jobs j JOIN failed_delivery_messages m ON m.id=j.message WHERE j.account=?1)",
            [account], |r| Ok((r.get(0)?,r.get(1)?))).map_err(storage)?;
    let charged = remote.len() + usize::from(local.contains_key(account));
    if !remote.is_empty()
        && (count.saturating_add(charged) > 1000
            || raw
                .len()
                .checked_mul(charged)
                .and_then(|n| bytes.checked_add(n))
                .is_none_or(|n| n > owner.quota_bytes))
    {
        return Err(Error::OverQuota);
    }
    let id = if local.is_empty() {
        tx.query_row("SELECT lower(hex(randomblob(16)))", [], |r| {
            r.get::<_, String>(0)
        })
        .map_err(storage)?
    } else {
        let targets = local
            .into_iter()
            .map(|(account, address)| DeliveryTarget { account, address })
            .collect::<Vec<_>>();
        queue::enqueue_tx(tx, sender, &targets, raw)?
    };
    if !remote.is_empty() {
        // Independent content keeps older local workers from deleting remote
        // payloads during a rolling downgrade. The acceptance remains atomic.
        tx.execute(
            "INSERT INTO submitted_messages VALUES(?1,?2,?3,?4,?5,unixepoch())",
            params![id, account, sender, raw, raw.len()],
        )
        .map_err(storage)?;
        for recipient in remote {
            tx.execute("INSERT INTO outbound_jobs(message,account,recipient,next_attempt) VALUES(?1,?2,?3,unixepoch())",params![id,account,recipient]).map_err(storage)?;
        }
    }
    Ok(id)
}
