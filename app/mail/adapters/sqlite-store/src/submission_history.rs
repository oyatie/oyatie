use super::{access, storage};
use mail_api::{Change, SubmissionChanges};
use mail_kernel::{Error, SubmissionRecord, UndoStatus};
use rusqlite::{Connection, OptionalExtension, params};

pub(super) fn revision(db: &Connection, account: &str) -> Result<u64, Error> {
    access::load(db, account)?;
    db.query_row(
        "SELECT revision FROM submission_heads WHERE account=?1",
        [account],
        |r| r.get(0),
    )
    .optional()
    .map_err(storage)
    .map(|v| v.unwrap_or(0))
}

pub(super) fn floor(db: &Connection, account: &str) -> Result<u64, Error> {
    db.query_row(
        "SELECT floor FROM submission_heads WHERE account=?1",
        [account],
        |r| r.get(0),
    )
    .optional()
    .map_err(storage)
    .map(|v| v.unwrap_or(0))
}

pub(super) fn current(db: &Connection, account: &str, id: &str) -> Result<SubmissionRecord, Error> {
    let state: String = db.query_row("SELECT state FROM submission_versions WHERE account=?1 AND id=?2 AND until_revision IS NULL AND state IS NOT NULL", params![account,id], |r| r.get(0))
        .optional().map_err(storage)?.ok_or(Error::NotFound)?;
    decode(&state)
}

pub(super) fn decode(state: &str) -> Result<SubmissionRecord, Error> {
    crate::count_row();
    serde_json::from_str(state).map_err(|_| Error::Unavailable)
}

pub(super) fn status(value: UndoStatus) -> &'static str {
    match value {
        UndoStatus::Pending => "pending",
        UndoStatus::Final => "final",
        UndoStatus::Canceled => "canceled",
    }
}

pub(super) fn write(
    db: &Connection,
    account: &str,
    record: &SubmissionRecord,
    destroy: bool,
) -> Result<u64, Error> {
    let revision = revision(db, account)?
        .checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .ok_or(Error::OverQuota)?;
    let state = if destroy {
        None
    } else {
        Some(serde_json::to_string(record).map_err(|_| Error::Unavailable)?)
    };
    db.execute("UPDATE submission_versions SET until_revision=?3 WHERE account=?1 AND id=?2 AND until_revision IS NULL", params![account,record.id,revision]).map_err(storage)?;
    db.execute("INSERT INTO submission_versions(account,id,revision,state,identity_id,email_id,thread_id,send_at,undo) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![account,record.id,revision,state,record.identity_id,record.email_id,record.thread_id,record.send_at,status(record.undo_status)]).map_err(storage)?;
    db.execute("INSERT INTO submission_heads(account,revision) VALUES(?1,?2) ON CONFLICT(account) DO UPDATE SET revision=excluded.revision", params![account,revision]).map_err(storage)?;
    if revision.saturating_sub(floor(db, account)?) > 20000 {
        let floor = revision - 10000;
        db.execute("DELETE FROM submission_versions WHERE account=?1 AND (until_revision<=?2 OR (state IS NULL AND revision<=?2))",params![account,floor]).map_err(storage)?;
        db.execute(
            "UPDATE submission_heads SET floor=?2 WHERE account=?1",
            params![account, floor],
        )
        .map_err(storage)?;
    }
    Ok(revision)
}

pub(super) fn changes(
    db: &Connection,
    account: &str,
    since: u64,
    limit: usize,
) -> Result<SubmissionChanges, Error> {
    if limit == 0 || limit > 10000 {
        return Err(Error::Invalid);
    }
    let latest = revision(db, account)?;
    if since > latest || since < floor(db, account)? {
        return Err(Error::Conflict);
    }
    let mut stmt = db.prepare("SELECT v.revision,v.id,p.state,v.state FROM submission_versions v LEFT JOIN submission_versions p ON p.account=v.account AND p.id=v.id AND p.until_revision=v.revision WHERE v.account=?1 AND v.revision>?2 ORDER BY v.revision LIMIT ?3").map_err(storage)?;
    let rows = stmt
        .query_map(params![account, since, limit], |r| {
            Ok((
                r.get::<_, u64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(storage)?;
    let mut changes = Vec::new();
    let mut last = since;
    for row in rows {
        let (revision, id, before, after) = row.map_err(storage)?;
        if last.checked_add(1) != Some(revision) {
            return Err(Error::Conflict);
        }
        changes.push(Change {
            revision,
            id,
            before: before.as_deref().map(decode).transpose()?,
            after: after.as_deref().map(decode).transpose()?,
        });
        last = revision;
    }
    if last < latest && changes.len() < limit {
        return Err(Error::Conflict);
    }
    Ok(SubmissionChanges {
        revision: last,
        has_more: last < latest,
        changes,
    })
}

pub(super) fn finish(
    db: &Connection,
    message: &str,
    recipient: &str,
    delivered: mail_kernel::SubmissionDelivered,
    reply: &str,
) -> Result<(), Error> {
    let account: Option<String> = db
        .query_row(
            "SELECT account FROM submission_schedule WHERE message=?1",
            [message],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?;
    let Some(account) = account else {
        return Ok(());
    };
    let pending: bool = db.query_row("SELECT EXISTS(SELECT 1 FROM delivery_jobs WHERE message=?1) OR EXISTS(SELECT 1 FROM outbound_jobs WHERE message=?1)", [message], |r| r.get(0)).map_err(storage)?;
    match current(db, &account, message) {
        Ok(mut record) if record.undo_status == UndoStatus::Pending => {
            let previous = record.clone();
            if pending {
                if let Some(status) = record.delivery_status.get_mut(recipient) {
                    status.delivered = delivered;
                    status.smtp_reply = reply.into();
                }
            } else {
                record.undo_status = UndoStatus::Final;
                reset(&mut record);
            }
            if record != previous {
                write(db, &account, &record, false)?;
            }
        }
        Err(Error::NotFound) | Ok(_) => {}
        Err(e) => return Err(e),
    }
    if !pending {
        db.execute(
            "DELETE FROM submission_schedule WHERE message=?1",
            [message],
        )
        .map_err(storage)?;
    }
    Ok(())
}

pub(super) fn reset(record: &mut SubmissionRecord) {
    for status in record.delivery_status.values_mut() {
        status.delivered = mail_kernel::SubmissionDelivered::Unknown;
        status.smtp_reply = "250 2.1.5 Queued".into();
    }
}
