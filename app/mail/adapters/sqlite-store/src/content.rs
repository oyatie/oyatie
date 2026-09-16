use super::{compatibility, storage};
use mail_kernel::{Account, Command, Error, MAX_MESSAGE_BYTES};
use rusqlite::{Connection, OptionalExtension, params};

pub(super) fn apply(db: &Connection, account: &mut Account, command: Command) -> Result<(), Error> {
    super::threads::prepare(db, account)?;
    let transfer = match &command {
        Command::Transfer { id, .. } => Some(id.clone()),
        _ => None,
    };
    let previous = matches!(
        command,
        Command::Transfer {
            remove_from: Some(_),
            ..
        } | Command::Destroy { .. }
            | Command::Expunge { .. }
            | Command::RemoveMailbox {
                remove_emails: true,
                ..
            }
    )
    .then(|| {
        account
            .messages
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>()
    });
    let thread = if let Command::Append { raw, .. } = &command {
        if raw.len() > MAX_MESSAGE_BYTES {
            return Err(Error::OverQuota);
        }
        let next = account.revision.checked_add(1).ok_or(Error::OverQuota)?;
        db.execute(
            "INSERT INTO message_bodies(account,id,content) VALUES(?1,?2,?3)",
            params![account.id, format!("e{next}"), raw],
        )
        .map_err(storage)?;
        Some(super::threads::references(raw)?)
    } else {
        None
    };
    account.apply(command)?;
    if let Some(source) = transfer {
        super::transfer::copy(db, &account.id, &source, &format!("e{}", account.revision))?;
    }
    if let Some(previous) = previous {
        let retained: std::collections::BTreeSet<_> =
            account.messages.iter().map(|m| &m.id).collect();
        for id in previous.iter().filter(|id| !retained.contains(id)) {
            db.execute(
                "DELETE FROM thread_members WHERE account=?1 AND message=?2",
                params![account.id, id],
            )
            .map_err(storage)?;
            db.execute(
                "DELETE FROM thread_references WHERE account=?1 AND message=?2",
                params![account.id, id],
            )
            .map_err(storage)?;
        }
    }
    if let Some(thread) = thread {
        super::threads::link(db, account, &format!("e{}", account.revision), thread)?;
    }
    Ok(())
}

pub(super) fn load(db: &Connection, id: &str) -> Result<String, Error> {
    db.query_row("SELECT state FROM accounts WHERE id=?1", [id], |r| r.get(0))
        .optional()
        .map_err(storage)?
        .ok_or(Error::NotFound)
}

pub(super) fn get(db: &Connection, account: &str, id: &str) -> Result<Vec<u8>, Error> {
    if let Some(raw) = db
        .query_row(
            "SELECT content FROM message_bodies WHERE account=?1 AND id=?2",
            params![account, id],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?
    {
        return Ok(raw);
    }
    // Legacy snapshots remain readable until their first successful mutation.
    compatibility::decode_all(&load(db, account)?)?
        .1
        .into_iter()
        .find(|(message, _)| message == id)
        .map(|(_, raw)| raw)
        .ok_or(Error::NotFound)
}

pub(super) fn prepare_save(db: &Connection, account: &mut Account) -> Result<Account, Error> {
    let (before, legacy) = compatibility::decode_all(&load(db, &account.id)?)?;
    account.complete_batch(&before)?;
    for (id, raw) in legacy {
        db.execute(
            "INSERT INTO message_bodies(account,id,content) VALUES(?1,?2,?3)",
            params![account.id, id, raw],
        )
        .map_err(storage)?;
    }
    let retained: std::collections::BTreeSet<_> = account.messages.iter().map(|m| &m.id).collect();
    let mut query = db
        .prepare("SELECT id FROM message_bodies WHERE account=?1")
        .map_err(storage)?;
    let stored = query
        .query_map([&account.id], |r| r.get::<_, String>(0))
        .map_err(storage)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage)?;
    for removed in stored.iter().filter(|id| !retained.contains(id)) {
        db.execute(
            "DELETE FROM message_bodies WHERE account=?1 AND id=?2",
            params![account.id, removed],
        )
        .map_err(storage)?;
    }
    Ok(before)
}
