use super::{SqliteStore, access, storage};
use mail_api::{DeliveryOutcome, OutboundLease};
use mail_kernel::Error;
use rusqlite::{OptionalExtension, TransactionBehavior, params};

pub(super) fn finish(
    store: &SqliteStore,
    lease: &OutboundLease,
    outcome: DeliveryOutcome,
) -> Result<(), Error> {
    match outcome {
        DeliveryOutcome::Temporary(code) if !(400..500).contains(&code) => {
            return Err(Error::Invalid);
        }
        DeliveryOutcome::Permanent(code) if !(500..600).contains(&code) => {
            return Err(Error::Invalid);
        }
        _ => {}
    }
    let mut db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    let (expired,attempt): (bool,u32) = tx.query_row(
        "SELECT coalesce((SELECT send_at FROM submission_schedule WHERE message=m.id),m.received_at)<=unixepoch()-432000,j.attempt FROM submitted_messages m JOIN outbound_jobs j ON j.message=m.id AND j.account=m.account WHERE j.message=?1 AND j.account=?2 AND j.recipient=?3 AND j.epoch=?4 AND j.lease_until>unixepoch()",
        params![lease.message,lease.account,lease.recipient,lease.epoch], |r| Ok((r.get(0)?,r.get(1)?)))
        .optional().map_err(storage)?.ok_or(Error::Conflict)?;
    let failed = matches!(outcome, DeliveryOutcome::Permanent(_))
        || expired && outcome != DeliveryOutcome::Delivered;
    if failed {
        let code = match outcome {
            DeliveryOutcome::Permanent(code) => code,
            _ => 554,
        };
        notice(&tx, lease, code, expired)?;
    }
    if failed || outcome == DeliveryOutcome::Delivered {
        // Every settlement names the epoch: a claim between the read above
        // and this write belongs to another owner and is left alone.
        let changed = tx
            .execute(
                "DELETE FROM outbound_jobs WHERE message=?1 AND recipient=?2 AND epoch=?3",
                params![lease.message, lease.recipient, lease.epoch],
            )
            .map_err(storage)?;
        if changed != 1 {
            return Err(Error::Conflict);
        }
        tx.execute("DELETE FROM submitted_messages WHERE id=?1 AND NOT EXISTS(SELECT 1 FROM outbound_jobs WHERE message=?1)",[&lease.message]).map_err(storage)?;
    } else if let DeliveryOutcome::Temporary(code) = outcome {
        let delay = mail_api::retry::delay_secs(attempt, entropy(&lease.message, attempt));
        let changed = tx.execute("UPDATE outbound_jobs SET lease_until=0,next_attempt=unixepoch()+?3,last_code=?4 WHERE message=?1 AND recipient=?2 AND epoch=?5",
            params![lease.message,lease.recipient,delay,code,lease.epoch]).map_err(storage)?;
        if changed != 1 {
            return Err(Error::Conflict);
        }
    }
    let (delivered, reply) = match outcome {
        DeliveryOutcome::Delivered => (mail_kernel::SubmissionDelivered::Yes, "250".to_owned()),
        DeliveryOutcome::Temporary(code) if !failed => {
            (mail_kernel::SubmissionDelivered::Queued, code.to_string())
        }
        DeliveryOutcome::Temporary(_) => (mail_kernel::SubmissionDelivered::No, "554".to_owned()),
        DeliveryOutcome::Permanent(code) => {
            (mail_kernel::SubmissionDelivered::No, code.to_string())
        }
    };
    super::submission_history::finish(&tx, &lease.message, &lease.recipient, delivered, &reply)?;
    tx.commit().map_err(storage)
}

fn notice(
    tx: &rusqlite::Transaction<'_>,
    lease: &OutboundLease,
    code: u16,
    expired: bool,
) -> Result<(), Error> {
    let account = access::load(tx, &lease.account)?;
    let id: String = tx
        .query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))
        .map_err(storage)?;
    let status = if expired { "5.4.7" } else { "5.0.0" };
    let time: i64 = tx
        .query_row("SELECT unixepoch()", [], |r| r.get(0))
        .map_err(storage)?;
    let date = mail_builder::headers::date::Date::new(time).to_rfc822();
    let raw = format!(
        "From: Mail Delivery System <MAILER-DAEMON@localhost>\r\nTo: {}\r\nDate: {date}\r\nMessage-ID: <{id}@localhost>\r\nAuto-Submitted: auto-generated\r\nSubject: Delivery failure\r\nMIME-Version: 1.0\r\nContent-Type: multipart/report; report-type=delivery-status; boundary=\"{id}\"\r\n\r\n--{id}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nDelivery to {} failed.\r\n\r\n--{id}\r\nContent-Type: message/delivery-status\r\n\r\nReporting-MTA: dns; localhost\r\nOriginal-Envelope-Id: {}\r\n\r\nFinal-Recipient: rfc822; {}\r\nAction: failed\r\nStatus: {status}\r\nDiagnostic-Code: smtp; {code}\r\n\r\n--{id}--\r\n",
        account.address, lease.recipient, lease.message, lease.recipient
    );
    // Notices are control traffic: queue saturation must not lose the failure.
    // Each notice replaces one bounded outbound job and is never relayed.
    tx.execute(
        "INSERT INTO queued_messages VALUES(?1,'',?2,?3,?4)",
        params![id, raw.as_bytes(), raw.len(), time],
    )
    .map_err(storage)?;
    tx.execute(
        "INSERT INTO delivery_jobs(message,account,address,next_attempt) VALUES(?1,?2,?3,?4)",
        params![id, account.id, account.address, time],
    )
    .map_err(storage)?;
    Ok(())
}

/// Jitter source for a retry delay: stable per job and attempt, so two
/// workers that settle the same attempt agree, and spread across jobs.
pub(super) fn entropy(message: &str, attempt: u32) -> u64 {
    message.bytes().fold(
        u64::from(attempt).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3),
    )
}
