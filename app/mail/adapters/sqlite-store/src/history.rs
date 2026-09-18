//! `history/{revision}` rows and their consumer-bounded compaction.
use crate::storage;
use mail_api::{Consumer, HistoryPage};
use mail_kernel::{Error, HistoryEntry, Retention, RetentionPolicy};
use rusqlite::{Connection, OptionalExtension, params};

pub(crate) fn record(
    db: &Connection,
    account: &str,
    revision: u64,
    rows: &[HistoryEntry],
) -> Result<(), Error> {
    db.execute(
        "INSERT INTO history_commits(account,revision,committed_at) VALUES(?1,?2,unixepoch())",
        params![account, revision],
    )
    .map_err(storage)?;
    let mut insert = db
        .prepare("INSERT INTO history(account,revision,seq,kind,id,mailbox,uid,thread) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)")
        .map_err(storage)?;
    for (seq, entry) in rows.iter().enumerate() {
        let (kind, id, mailbox, uid, thread): (
            &str,
            &str,
            Option<&str>,
            Option<u32>,
            Option<&str>,
        ) = match entry {
            HistoryEntry::Added { id, mailbox, uid } => {
                ("added", id, Some(mailbox), Some(*uid), None)
            }
            HistoryEntry::Removed {
                id,
                mailbox,
                uid,
                thread,
            } => ("removed", id, Some(mailbox), Some(*uid), Some(thread)),
            HistoryEntry::Flags { id } => ("flags", id, None, None, None),
            HistoryEntry::Thread { id } => ("thread", id, None, None, None),
            HistoryEntry::Mailbox { id } => ("mailbox", id, None, None, None),
        };
        insert
            .execute(params![
                account, revision, seq as u64, kind, id, mailbox, uid, thread
            ])
            .map_err(storage)?;
    }
    Ok(())
}

/// Rows in `(since, revision]`, whole revisions only, at most `limit` rows.
pub(crate) fn page(
    db: &Connection,
    account: &str,
    since: u64,
    limit: usize,
) -> Result<HistoryPage, Error> {
    let (revision, floor): (u64, u64) = db
        .query_row(
            "SELECT revision,history_floor FROM accounts WHERE id=?1",
            [account],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(storage)?
        .ok_or(Error::NotFound)?;
    if since > revision {
        return Err(Error::Conflict);
    }
    let mut page = HistoryPage {
        since,
        revision,
        floor,
        has_more: false,
        rows: vec![],
    };
    if since < floor {
        return Ok(page);
    }
    let limit = limit.clamp(1, 10_000);
    let mut query = db
        .prepare("SELECT revision,kind,id,mailbox,uid,thread FROM history WHERE account=?1 AND revision>?2 ORDER BY revision,seq LIMIT ?3")
        .map_err(storage)?;
    let mut rows = query
        .query(params![account, since, (limit + 1) as u64])
        .map_err(storage)?;
    let mut collected = Vec::new();
    while let Some(row) = rows.next().map_err(storage)? {
        let revision: u64 = row.get(0).map_err(storage)?;
        let kind: String = row.get(1).map_err(storage)?;
        let id: String = row.get(2).map_err(storage)?;
        let mailbox: Option<String> = row.get(3).map_err(storage)?;
        let uid: Option<u32> = row.get(4).map_err(storage)?;
        let thread: Option<String> = row.get(5).map_err(storage)?;
        let entry = match kind.as_str() {
            "added" => HistoryEntry::Added {
                id,
                mailbox: mailbox.ok_or(Error::Unavailable)?,
                uid: uid.ok_or(Error::Unavailable)?,
            },
            "removed" => HistoryEntry::Removed {
                id,
                mailbox: mailbox.ok_or(Error::Unavailable)?,
                uid: uid.ok_or(Error::Unavailable)?,
                thread: thread.ok_or(Error::Unavailable)?,
            },
            "flags" => HistoryEntry::Flags { id },
            "thread" => HistoryEntry::Thread { id },
            "mailbox" => HistoryEntry::Mailbox { id },
            _ => return Err(Error::Unavailable),
        };
        collected.push((revision, entry));
    }
    if collected.len() > limit {
        // Drop the incomplete last revision; report the last complete one.
        let last = collected[limit].0;
        collected.retain(|(revision, _)| *revision < last);
        page.has_more = true;
        page.revision = collected.last().map(|(r, _)| *r).unwrap_or(since);
        if collected.is_empty() {
            return Err(Error::OverQuota);
        }
    }
    page.rows = collected;
    Ok(page)
}

/// Advance the floor to the highest revision the policy allows, bounded by
/// enabled consumers' cursors; delete rows at or below the new floor.
pub(crate) fn compact(
    db: &Connection,
    account: &str,
    now: i64,
    policy: RetentionPolicy,
    cursors: &[(Consumer, u64)],
) -> Result<Retention, Error> {
    let (revision, floor): (u64, u64) = db
        .query_row(
            "SELECT revision,history_floor FROM accounts WHERE id=?1",
            [account],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(storage)?
        .ok_or(Error::NotFound)?;
    let by_age: Option<u64> = db
        .query_row(
            "SELECT max(revision) FROM history_commits WHERE account=?1 AND committed_at<?2",
            params![account, now.saturating_sub(policy.max_age_secs)],
            |r| r.get(0),
        )
        .map_err(storage)?;
    let by_rows: Option<u64> = db
        .query_row(
            "SELECT min(revision) FROM (SELECT revision FROM history WHERE account=?1 ORDER BY revision DESC,seq DESC LIMIT 1 OFFSET ?2)",
            params![account, policy.max_rows],
            |r| r.get(0),
        )
        .map_err(storage)?;
    let candidate = by_age.unwrap_or(0).max(by_rows.unwrap_or(0)).min(revision);
    let named: Vec<(&str, u64)> = cursors.iter().map(|(c, n)| (c.name(), *n)).collect();
    let retention = policy.floor(floor, candidate, &named);
    let next = match &retention {
        Retention::Advanced { floor } | Retention::Blocked { floor, .. } => *floor,
    };
    if next > floor {
        db.execute(
            "UPDATE accounts SET history_floor=?2 WHERE id=?1",
            params![account, next],
        )
        .map_err(storage)?;
        for table in ["history", "history_commits"] {
            db.execute(
                &format!("DELETE FROM {table} WHERE account=?1 AND revision<=?2"),
                params![account, next],
            )
            .map_err(storage)?;
        }
    }
    Ok(retention)
}
