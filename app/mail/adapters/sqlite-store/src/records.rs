//! Row ↔ kernel record mapping. Every function reads or writes exactly the
//! rows named by its arguments; nothing here scans an account.
use crate::storage;
use mail_kernel::{Account, Error, Mailbox, Message, Scope};
use rusqlite::{Connection, OptionalExtension, Row, params};
use std::collections::{BTreeMap, BTreeSet};

const MAILBOX_COLUMNS: &str = "id,name,role,parent_id,sort_order,is_subscribed,uid_next,uid_validity,created_revision,highest_modseq,total_emails,unread_emails,size_bytes";
const MESSAGE_COLUMNS: &str =
    "id,modseq,created_revision,thread,email_identity,thread_identity,size,received_at,keywords";

/// Account record with its mailboxes and no messages.
pub(crate) fn header(db: &Connection, id: &str) -> Result<Account, Error> {
    let mut account = db
        .query_row(
            "SELECT id,tenant,owner,address,revision,mail_modseq,history_floor,identity,identity_revision,vacation,vacation_revision,quota_bytes,used_bytes FROM accounts WHERE id=?1",
            [id],
            |r| {
                Ok(Account {
                    id: r.get(0)?,
                    tenant: r.get(1)?,
                    owner: r.get(2)?,
                    address: r.get(3)?,
                    revision: r.get(4)?,
                    mail_modseq: r.get(5)?,
                    history_floor: r.get(6)?,
                    identity: serde_json::from_str(&r.get::<_, String>(7)?).unwrap_or_default(),
                    identity_revision: r.get(8)?,
                    vacation: serde_json::from_str(&r.get::<_, String>(9)?).unwrap_or_default(),
                    vacation_revision: r.get(10)?,
                    quota_bytes: r.get(11)?,
                    used_bytes: r.get(12)?,
                    mailboxes: vec![],
                    messages: vec![],
                })
            },
        )
        .optional()
        .map_err(storage)?
        .ok_or(Error::NotFound)?;
    let mut query = db
        .prepare(&format!(
            "SELECT {MAILBOX_COLUMNS} FROM mailboxes WHERE account=?1 ORDER BY rowid"
        ))
        .map_err(storage)?;
    account.mailboxes = query
        .query_map([id], mailbox)
        .map_err(storage)?
        .collect::<Result<_, _>>()
        .map_err(storage)?;
    Ok(account)
}

fn mailbox(r: &Row<'_>) -> rusqlite::Result<Mailbox> {
    Ok(Mailbox {
        id: r.get(0)?,
        name: r.get(1)?,
        role: r.get(2)?,
        parent_id: r.get(3)?,
        sort_order: r.get(4)?,
        is_subscribed: r.get(5)?,
        uid_next: r.get(6)?,
        uid_validity: r.get(7)?,
        created_revision: r.get(8)?,
        highest_modseq: r.get(9)?,
        total_emails: r.get(10)?,
        unread_emails: r.get(11)?,
        size_bytes: r.get(12)?,
    })
}

fn message(r: &Row<'_>) -> rusqlite::Result<Message> {
    let id: String = r.get(0)?;
    let thread: String = r.get(3)?;
    let keywords: String = r.get(8)?;
    Ok(Message {
        thread: (thread != id).then_some(thread),
        id,
        modseq: r.get(1)?,
        created_revision: r.get(2)?,
        email_identity: r.get(4)?,
        thread_identity: r.get(5)?,
        size: r.get(6)?,
        received_at: r.get(7)?,
        keywords: keywords
            .split(' ')
            .filter(|k| !k.is_empty())
            .map(Into::into)
            .collect(),
        mailboxes: BTreeMap::new(),
    })
}

/// Attach mailbox links to already loaded message records.
fn link(db: &Connection, account: &str, messages: &mut [Message]) -> Result<(), Error> {
    let mut query = db
        .prepare("SELECT mailbox,uid FROM message_mailboxes WHERE account=?1 AND message=?2")
        .map_err(storage)?;
    for message in messages {
        let rows = query
            .query_map(params![account, message.id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?))
            })
            .map_err(storage)?;
        for row in rows {
            let (mailbox, uid) = row.map_err(storage)?;
            message.mailboxes.insert(mailbox, uid);
        }
    }
    Ok(())
}

fn query_messages(
    db: &Connection,
    account: &str,
    sql: &str,
    args: &[&dyn rusqlite::ToSql],
) -> Result<Vec<Message>, Error> {
    let mut query = db.prepare(sql).map_err(storage)?;
    let mut messages: Vec<Message> = query
        .query_map(args, message)
        .map_err(storage)?
        .collect::<Result<_, _>>()
        .map_err(storage)?;
    link(db, account, &mut messages)?;
    Ok(messages)
}

pub(crate) fn by_ids(
    db: &Connection,
    account: &str,
    ids: &BTreeSet<&str>,
) -> Result<Vec<Message>, Error> {
    let mut messages = Vec::with_capacity(ids.len());
    for id in ids {
        messages.extend(query_messages(
            db,
            account,
            &format!("SELECT {MESSAGE_COLUMNS} FROM messages WHERE account=?1 AND id=?2"),
            &[&account, id],
        )?);
    }
    Ok(messages)
}

/// Load the union of the scopes into the working set, each record once.
pub(crate) fn working_set(
    db: &Connection,
    account: &mut Account,
    scopes: &[Scope<'_>],
) -> Result<(), Error> {
    let mut ids = BTreeSet::new();
    for scope in scopes {
        match scope {
            Scope::None => {}
            Scope::Message(id) => {
                ids.insert(*id);
            }
            Scope::Mailbox(mailbox) => {
                for message in query_messages(
                    db,
                    &account.id,
                    &format!(
                        "SELECT {MESSAGE_COLUMNS} FROM messages m WHERE m.account=?1 AND m.id IN (SELECT message FROM message_mailboxes WHERE account=?1 AND mailbox=?2)"
                    ),
                    &[&account.id, mailbox],
                )? {
                    push(account, message);
                }
            }
            Scope::Flagged(mailbox, keyword) => {
                let pattern = format!("% {keyword} %");
                for message in query_messages(
                    db,
                    &account.id,
                    &format!(
                        "SELECT {MESSAGE_COLUMNS} FROM messages m WHERE m.account=?1 AND (' '||m.keywords||' ') LIKE ?3 AND m.id IN (SELECT message FROM message_mailboxes WHERE account=?1 AND mailbox=?2)"
                    ),
                    &[&account.id, mailbox, &pattern],
                )? {
                    push(account, message);
                }
            }
        }
    }
    for id in &ids {
        if !account.messages.iter().any(|m| m.id == *id) {
            for message in by_ids(db, &account.id, &BTreeSet::from([*id]))? {
                push(account, message);
            }
        }
    }
    Ok(())
}

fn push(account: &mut Account, message: Message) {
    if !account.messages.iter().any(|m| m.id == message.id) {
        account.messages.push(message);
    }
}

/// Full projection: every message record, in UID order of first appearance
/// (creation order), as the protocol layer expects today.
pub(crate) fn projection(db: &Connection, id: &str) -> Result<Account, Error> {
    let mut account = header(db, id)?;
    account.messages = query_messages(
        db,
        id,
        &format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages WHERE account=?1 ORDER BY CAST(substr(id,2) AS INTEGER)"
        ),
        &[&id],
    )?;
    Ok(account)
}

pub(crate) fn upsert_header(db: &Connection, account: &Account) -> Result<(), Error> {
    db.execute(
        "INSERT INTO accounts(id,address,tenant,owner,revision,mail_modseq,history_floor,identity,identity_revision,vacation,vacation_revision,quota_bytes,used_bytes)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
         ON CONFLICT(id) DO UPDATE SET revision=excluded.revision,mail_modseq=excluded.mail_modseq,history_floor=excluded.history_floor,
            identity=excluded.identity,identity_revision=excluded.identity_revision,vacation=excluded.vacation,
            vacation_revision=excluded.vacation_revision,quota_bytes=excluded.quota_bytes,used_bytes=excluded.used_bytes",
        params![
            account.id,
            account.address.to_ascii_lowercase(),
            account.tenant,
            account.owner,
            account.revision,
            account.mail_modseq,
            account.history_floor,
            serde_json::to_string(&account.identity).map_err(|_| Error::Invalid)?,
            account.identity_revision,
            serde_json::to_string(&account.vacation).map_err(|_| Error::Invalid)?,
            account.vacation_revision,
            account.quota_bytes,
            account.used_bytes
        ],
    )
    .map_err(storage)?;
    Ok(())
}

pub(crate) fn upsert_mailbox(db: &Connection, account: &str, m: &Mailbox) -> Result<(), Error> {
    db.execute(
        &format!("INSERT OR REPLACE INTO mailboxes(account,{MAILBOX_COLUMNS}) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)"),
        params![account, m.id, m.name, m.role, m.parent_id, m.sort_order, m.is_subscribed, m.uid_next, m.uid_validity, m.created_revision, m.highest_modseq, m.total_emails, m.unread_emails, m.size_bytes],
    )
    .map_err(storage)?;
    Ok(())
}

pub(crate) fn upsert_message(db: &Connection, account: &str, m: &Message) -> Result<(), Error> {
    db.execute(
        &format!("INSERT OR REPLACE INTO messages(account,{MESSAGE_COLUMNS}) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)"),
        params![account, m.id, m.modseq, m.created_revision, m.thread_id(), m.email_identity, m.thread_identity, m.size, m.received_at, m.keywords.join(" ")],
    )
    .map_err(storage)?;
    db.execute(
        "DELETE FROM message_mailboxes WHERE account=?1 AND message=?2",
        params![account, m.id],
    )
    .map_err(storage)?;
    for (mailbox, uid) in &m.mailboxes {
        db.execute(
            "INSERT INTO message_mailboxes(account,mailbox,uid,message) VALUES(?1,?2,?3,?4)",
            params![account, mailbox, uid, m.id],
        )
        .map_err(storage)?;
    }
    Ok(())
}

pub(crate) fn delete_message(db: &Connection, account: &str, id: &str) -> Result<(), Error> {
    for table in ["message_mailboxes", "thread_members", "thread_references"] {
        db.execute(
            &format!("DELETE FROM {table} WHERE account=?1 AND message=?2"),
            params![account, id],
        )
        .map_err(storage)?;
    }
    for table in ["messages", "message_bodies"] {
        db.execute(
            &format!("DELETE FROM {table} WHERE account=?1 AND id=?2"),
            params![account, id],
        )
        .map_err(storage)?;
    }
    Ok(())
}
