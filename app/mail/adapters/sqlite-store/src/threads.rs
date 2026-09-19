use super::storage;
use mail_kernel::{Account, Error, HistoryEntry};
use mail_parser::{HeaderForm, HeaderValue, MessageParser};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub(super) struct References {
    pub(super) subject: Vec<u8>,
    pub(super) keys: BTreeSet<Vec<u8>>,
    /// Hash of the first `Message-ID`, the `(account, reference_hash)` key
    /// duplicate suppression looks up.
    pub(super) message_id: Option<Vec<u8>>,
}

pub(super) fn references(raw: &[u8]) -> Result<References, Error> {
    let parsed = MessageParser::default().parse_headers(raw);
    let subject = parsed
        .as_ref()
        .and_then(|m| m.thread_name())
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let mut keys = BTreeSet::new();
    let mut message_id = None;
    if let Some(parsed) = parsed {
        for name in ["Message-ID", "References", "In-Reply-To"] {
            for value in parsed.header_as(name, HeaderForm::MessageIds) {
                let ids = match value {
                    HeaderValue::Text(id) => vec![id],
                    HeaderValue::TextList(ids) => ids,
                    _ => continue,
                };
                for id in ids {
                    let key = Sha256::digest(id.as_bytes()).to_vec();
                    if name == "Message-ID" && message_id.is_none() {
                        message_id = Some(key.clone());
                    }
                    keys.insert(key);
                    if keys.len() > 1000 {
                        // A fixed content limit cannot recover when queue space
                        // becomes available; callers must not retry this payload.
                        return Err(Error::Invalid);
                    }
                }
            }
        }
    }
    Ok(References {
        subject: Sha256::digest(subject.as_bytes()).to_vec(),
        keys,
        message_id,
    })
}

/// SMTP-source duplicate suppression: a record whose Message-ID/References
/// set equals the incoming one and which is linked to the target mailbox or
/// the Junk mailbox. One indexed lookup on `(account, reference_hash)`.
pub(super) fn duplicate(
    db: &Connection,
    account: &Account,
    target: &str,
    refs: &References,
) -> Result<bool, Error> {
    let Some(message_id) = &refs.message_id else {
        return Ok(false);
    };
    let junk = account
        .mailboxes
        .iter()
        .find(|m| m.role.as_deref() == Some("junk"))
        .map(|m| m.id.as_str());
    let mut lookup = db
        .prepare("SELECT message FROM thread_references WHERE account=?1 AND reference=?2")
        .map_err(storage)?;
    let candidates = lookup
        .query_map(params![account.id, message_id], |r| r.get::<_, String>(0))
        .map_err(storage)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage)?;
    for candidate in candidates {
        crate::count_row();
        let linked: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM message_mailboxes WHERE account=?1 AND message=?2 AND (mailbox=?3 OR mailbox=?4))",
                params![account.id, candidate, target, junk],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if !linked {
            continue;
        }
        let mut query = db
            .prepare("SELECT reference FROM thread_references WHERE account=?1 AND message=?2")
            .map_err(storage)?;
        let keys = query
            .query_map(params![account.id, candidate], |r| r.get::<_, Vec<u8>>(0))
            .map_err(storage)?
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(storage)?;
        if keys == refs.keys {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Match normalized subjects AND shared Message-ID/References/In-Reply-To.
/// Subject equality alone never groups unrelated mail. Referenced parents need
/// not have arrived yet; a later bridging message can merge existing groups.
/// Members outside the working set are re-threaded by key; their ids are
/// returned so the commit can stamp them with the final revision.
pub(super) fn link(
    db: &Connection,
    account: &mut Account,
    id: &str,
    refs: References,
) -> Result<Vec<String>, Error> {
    let mut outside = Vec::new();
    let mut threads = BTreeSet::new();
    let mut lookup = db.prepare("SELECT DISTINCT m.thread FROM thread_references r JOIN thread_members m ON m.account=r.account AND m.message=r.message WHERE r.account=?1 AND r.reference=?2 AND m.subject=?3").map_err(storage)?;
    for key in &refs.keys {
        let found = lookup
            .query_map(params![account.id, key, refs.subject], |r| {
                r.get::<_, String>(0)
            })
            .map_err(storage)?;
        for thread in found {
            crate::count_row();
            threads.insert(thread.map_err(storage)?);
        }
    }
    let previous = account
        .messages
        .iter()
        .find(|m| m.id == id)
        .ok_or(Error::Unavailable)?
        .thread_id();
    let thread = threads
        .iter()
        .next()
        .map_or(previous, String::as_str)
        .to_owned();
    // A member in the working set (same batch, not yet persisted) precedes
    // the persisted rows; both carry the thread's immutable identity.
    let immutable_thread: String = account
        .messages
        .iter()
        .find(|m| m.id != id && m.thread_id() == thread)
        .map(|m| m.thread_identity().to_owned())
        .or_else(|| {
            db.query_row(
                "SELECT coalesce(thread_identity,id) FROM messages WHERE account=?1 AND thread=?2 AND id!=?3 ORDER BY CAST(substr(id,2) AS INTEGER) LIMIT 1",
                params![account.id, thread, id],
                |r| r.get(0),
            )
            .ok()
        })
        .unwrap_or_else(|| id.to_owned());
    for old in threads.iter().filter(|old| **old != thread) {
        db.execute(
            "UPDATE thread_members SET thread=?3 WHERE account=?1 AND thread=?2",
            params![account.id, old, thread],
        )
        .map_err(storage)?;
        // Records outside the working set: bounded by the merged thread.
        let mut members = db
            .prepare("SELECT id FROM messages WHERE account=?1 AND thread=?2")
            .map_err(storage)?;
        let ids = members
            .query_map(params![account.id, old], |r| r.get::<_, String>(0))
            .map_err(storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage)?;
        for member in ids {
            crate::count_row();
            if account.messages.iter().any(|m| m.id == member) {
                continue;
            }
            db.execute(
                "UPDATE messages SET thread=?3 WHERE account=?1 AND id=?2",
                params![account.id, member, thread],
            )
            .map_err(storage)?;
            outside.push(member);
        }
    }
    for message in &mut account.messages {
        if message.id == id && message.thread_identity.is_none() {
            message.thread_identity = Some(immutable_thread.clone());
        }
        if message.id == id || threads.contains(message.thread_id()) {
            // The original singleton ID remains the backwards-compatible value.
            message.thread = (thread != message.id).then(|| thread.clone());
        }
    }
    db.execute(
        "INSERT INTO thread_members(account,message,subject,thread) VALUES(?1,?2,?3,?4)",
        params![account.id, id, refs.subject, thread],
    )
    .map_err(storage)?;
    for key in refs.keys {
        db.execute(
            "INSERT INTO thread_references(account,message,reference) VALUES(?1,?2,?3)",
            params![account.id, id, key],
        )
        .map_err(storage)?;
    }
    Ok(outside)
}

/// Drop a record's thread keys so no later lookup can join its thread.
pub(super) fn forget(db: &Connection, account: &str, id: &str) -> Result<(), Error> {
    for table in ["thread_members", "thread_references"] {
        db.execute(
            &format!("DELETE FROM {table} WHERE account=?1 AND message=?2"),
            params![account, id],
        )
        .map_err(storage)?;
    }
    Ok(())
}

/// Stamp re-threaded records outside the working set with the commit revision.
pub(super) fn stamp(
    db: &Connection,
    account: &mut Account,
    outside: &[String],
    revision: u64,
    history: &mut Vec<HistoryEntry>,
) -> Result<(), Error> {
    let mut links = db
        .prepare("SELECT mailbox FROM message_mailboxes WHERE account=?1 AND message=?2")
        .map_err(storage)?;
    for id in outside {
        db.execute(
            "UPDATE messages SET modseq=?3 WHERE account=?1 AND id=?2",
            params![account.id, id, revision],
        )
        .map_err(storage)?;
        let mailboxes = links
            .query_map(params![account.id, id], |r| r.get::<_, String>(0))
            .map_err(storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage)?;
        for mailbox in account
            .mailboxes
            .iter_mut()
            .filter(|m| mailboxes.contains(&m.id))
        {
            mailbox.highest_modseq = revision;
        }
        history.push(HistoryEntry::Thread { id: id.clone() });
    }
    Ok(())
}
