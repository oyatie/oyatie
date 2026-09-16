use super::{compatibility, content, storage};
use mail_kernel::{Account, Error};
use mail_parser::{HeaderForm, HeaderValue, MessageParser};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub(super) struct References {
    subject: Vec<u8>,
    keys: BTreeSet<Vec<u8>>,
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
    if let Some(parsed) = parsed {
        for name in ["Message-ID", "References", "In-Reply-To"] {
            for value in parsed.header_as(name, HeaderForm::MessageIds) {
                let ids = match value {
                    HeaderValue::Text(id) => vec![id],
                    HeaderValue::TextList(ids) => ids,
                    _ => continue,
                };
                for id in ids {
                    keys.insert(Sha256::digest(id.as_bytes()).to_vec());
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
    })
}

/// Backfill old accounts only inside their next successful write. Opening or
/// reading an old database does not change its snapshots or message bodies.
pub(super) fn prepare(db: &Connection, account: &mut Account) -> Result<(), Error> {
    let indexed: bool = db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM thread_indexed WHERE account=?1)",
            [&account.id],
            |r| r.get(0),
        )
        .map_err(storage)?;
    if indexed {
        return Ok(());
    }
    for message in &mut account.messages {
        if message.thread_identity.is_none() {
            message.thread_identity = Some(message.id.clone());
        }
    }
    db.execute("DELETE FROM thread_members WHERE account=?1", [&account.id])
        .map_err(storage)?;
    db.execute(
        "DELETE FROM thread_references WHERE account=?1",
        [&account.id],
    )
    .map_err(storage)?;
    let (_, bodies) = compatibility::decode_all(&content::load(db, &account.id)?)?;
    let mut bodies: std::collections::BTreeMap<_, _> = bodies.into_iter().collect();
    let ids: Vec<_> = account.messages.iter().map(|m| m.id.clone()).collect();
    for id in ids {
        let raw = match bodies.remove(&id) {
            Some(raw) => raw,
            None => content::get(db, &account.id, &id)?,
        };
        link(db, account, &id, references(&raw)?)?;
    }
    seal(db, &account.id)
}

pub(super) fn seal(db: &Connection, account: &str) -> Result<(), Error> {
    db.execute(
        "INSERT OR IGNORE INTO thread_indexed(account) VALUES(?1)",
        [account],
    )
    .map_err(storage)?;
    Ok(())
}

/// Match normalized subjects AND shared Message-ID/References/In-Reply-To.
/// Subject equality alone never groups unrelated mail. Referenced parents need
/// not have arrived yet; a later bridging message can merge existing groups.
pub(super) fn link(
    db: &Connection,
    account: &mut Account,
    id: &str,
    refs: References,
) -> Result<(), Error> {
    let mut threads = BTreeSet::new();
    let mut lookup = db.prepare("SELECT DISTINCT m.thread FROM thread_references r JOIN thread_members m ON m.account=r.account AND m.message=r.message WHERE r.account=?1 AND r.reference=?2 AND m.subject=?3").map_err(storage)?;
    for key in &refs.keys {
        let found = lookup
            .query_map(params![account.id, key, refs.subject], |r| {
                r.get::<_, String>(0)
            })
            .map_err(storage)?;
        for thread in found {
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
    let immutable_thread = account
        .messages
        .iter()
        .find(|m| m.id != id && m.thread_id() == thread)
        .map(|m| m.thread_identity())
        .unwrap_or(id)
        .to_owned();
    for old in threads.iter().filter(|old| **old != thread) {
        db.execute(
            "UPDATE thread_members SET thread=?3 WHERE account=?1 AND thread=?2",
            params![account.id, old, thread],
        )
        .map_err(storage)?;
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
    Ok(())
}
