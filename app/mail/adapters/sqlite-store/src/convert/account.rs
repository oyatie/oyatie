//! One account, one transaction: decode the legacy row, rebuild the kernel
//! records with counters and watermarks, backfill threads when the legacy
//! index was absent, write the relational rows, delete the legacy row.
use super::legacy::LegacyAccount;
use crate::{records, storage, threads};
use mail_kernel::{Account, Error, Mailbox, Message};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::BTreeMap;

/// Legacy tables and triggers that have no relational successor.
pub(super) const DROPPED: &[(&str, &str)] = &[
    ("TRIGGER", "account_access_insert"),
    ("TRIGGER", "account_access_update"),
    ("TRIGGER", "account_access_delete"),
    ("TRIGGER", "message_index_invalidated"),
    ("TRIGGER", "thread_index_invalidated"),
    ("TRIGGER", "message_index_deleted"),
    ("TRIGGER", "thread_content_delete"),
    ("TABLE", "account_access"),
    ("TABLE", "message_changes"),
    ("TABLE", "mailbox_commits"),
    ("TABLE", "mailbox_changes"),
    ("TABLE", "message_index_state"),
    ("TABLE", "message_metadata"),
    ("TABLE", "thread_indexed"),
];

pub(super) fn pending(db: &Connection) -> Result<Vec<String>, Error> {
    let mut query = db
        .prepare("SELECT id FROM legacy_accounts ORDER BY id")
        .map_err(storage)?;
    query
        .query_map([], |r| r.get(0))
        .map_err(storage)?
        .collect::<Result<_, _>>()
        .map_err(storage)
}

/// The conversion is itself one commit: the revision advances once so every
/// legacy MODSEQ (at most `revision + 1`) fits under the new watermark, and
/// the floor is the legacy revision because the legacy journal is not carried.
pub(super) fn convert(db: &Connection, id: &str, indexed: bool) -> Result<(), Error> {
    let (address, token, state): (String, Option<Vec<u8>>, String) = db
        .query_row(
            "SELECT address,token,state FROM legacy_accounts WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(storage)?
        .ok_or(Error::NotFound)?;
    let legacy = LegacyAccount::decode(&state)?;
    if legacy.id != id {
        return Err(Error::Unavailable);
    }
    let revision = legacy.revision.checked_add(1).ok_or(Error::OverQuota)?;
    let mut account = Account {
        id: legacy.id.clone(),
        tenant: legacy.tenant.clone(),
        owner: legacy.owner.clone(),
        address,
        revision,
        mail_modseq: 0,
        history_floor: legacy.revision,
        identity: legacy.identity.clone(),
        identity_revision: legacy.identity_revision,
        vacation: legacy.vacation.clone(),
        vacation_revision: legacy.vacation_revision,
        quota_bytes: legacy.quota_bytes,
        used_bytes: 0,
        mailboxes: legacy
            .mailboxes
            .iter()
            .map(|m| Mailbox {
                id: m.id.clone(),
                name: m.name.clone(),
                role: m.role.clone(),
                parent_id: m.parent_id.clone(),
                sort_order: m.sort_order,
                is_subscribed: m.is_subscribed,
                uid_next: m.uid_next,
                uid_validity: m.uid_validity,
                ..Mailbox::new("", "", None, 0)
            })
            .collect(),
        messages: vec![],
    };
    let mut bodies: Vec<(String, Vec<u8>)> = vec![];
    for legacy in &legacy.messages {
        let (size, raw) = legacy.content()?;
        if let Some(raw) = raw {
            bodies.push((legacy.id.clone(), raw.to_vec()));
        }
        let modseq = legacy.modseq.unwrap_or(1).min(revision);
        let links = legacy.links()?;
        let message = Message {
            id: legacy.id.clone(),
            modseq,
            created_revision: 0,
            thread: legacy.thread.clone(),
            email_identity: legacy.email_identity.clone(),
            thread_identity: legacy.thread_identity.clone(),
            mailboxes: BTreeMap::new(),
            size,
            keywords: legacy.keywords.clone(),
            received_at: legacy.received_at,
        };
        account.used_bytes += size;
        account.mail_modseq = account.mail_modseq.max(modseq);
        account.messages.push(message);
        let index = account.messages.len() - 1;
        account.attach(index, links);
    }
    for mailbox in &mut account.mailboxes {
        mailbox.highest_modseq = account
            .messages
            .iter()
            .filter(|m| m.mailboxes.contains_key(&mailbox.id))
            .map(|m| m.modseq)
            .max()
            .unwrap_or(0);
    }
    records::upsert_header(db, &account)?;
    db.execute(
        "UPDATE accounts SET token=?2 WHERE id=?1",
        params![account.id, token],
    )
    .map_err(storage)?;
    for (message, raw) in &bodies {
        db.execute(
            "INSERT OR IGNORE INTO message_bodies(account,id,content) VALUES(?1,?2,?3)",
            params![account.id, message, raw],
        )
        .map_err(storage)?;
    }
    for message in &account.messages {
        records::upsert_message(db, &account.id, message)?;
    }
    if !indexed {
        backfill_threads(db, &mut account)?;
        for message in &account.messages {
            records::upsert_message(db, &account.id, message)?;
        }
    }
    for mailbox in &account.mailboxes {
        records::upsert_mailbox(db, &account.id, mailbox)?;
    }
    db.execute(
        "INSERT INTO history_commits(account,revision,committed_at) VALUES(?1,?2,unixepoch())",
        params![account.id, revision],
    )
    .map_err(storage)?;
    db.execute("DELETE FROM legacy_accounts WHERE id=?1", [id])
        .map_err(storage)?;
    Ok(())
}

/// Accounts the legacy store never thread-indexed get the same backfill the
/// retired lazy path ran on their first write.
fn backfill_threads(db: &Connection, account: &mut Account) -> Result<(), Error> {
    for message in &mut account.messages {
        if message.thread_identity.is_none() {
            message.thread_identity = Some(message.id.clone());
        }
    }
    for table in ["thread_members", "thread_references"] {
        db.execute(
            &format!("DELETE FROM {table} WHERE account=?1"),
            [&account.id],
        )
        .map_err(storage)?;
    }
    let ids: Vec<_> = account.messages.iter().map(|m| m.id.clone()).collect();
    for id in ids {
        let raw = crate::content::get(db, &account.id, &id)?;
        threads::link(db, account, &id, threads::references(&raw)?)?;
    }
    Ok(())
}
