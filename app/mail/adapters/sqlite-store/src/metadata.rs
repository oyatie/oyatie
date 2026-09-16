use super::{SqliteStore, load, storage};
use mail_api::MessageSelection;
use mail_kernel::{Account, Error, Message};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn revision(db: &Connection, account: &str) -> Result<Option<u64>, Error> {
    db.query_row(
        "SELECT revision FROM message_index_state WHERE account=?1",
        [account],
        |r| r.get(0),
    )
    .optional()
    .map_err(storage)
}

pub(super) fn record(
    db: &Connection,
    before: &Account,
    after: &Account,
    indexed: bool,
) -> Result<(), Error> {
    let old: BTreeMap<_, _> = before.messages.iter().map(|m| (&m.id, m)).collect();
    let retained: BTreeSet<_> = after.messages.iter().map(|m| &m.id).collect();
    if !indexed {
        db.execute("DELETE FROM message_metadata WHERE account=?1", [&after.id])
            .map_err(storage)?;
    } else {
        for id in old.keys().filter(|id| !retained.contains(**id)) {
            db.execute(
                "DELETE FROM message_metadata WHERE account=?1 AND id=?2",
                params![after.id, id],
            )
            .map_err(storage)?;
        }
    }
    for message in &after.messages {
        if !indexed || old.get(&message.id).is_none_or(|old| **old != *message) {
            let state = serde_json::to_string(message).map_err(|_| Error::Unavailable)?;
            db.execute("INSERT INTO message_metadata(account,id,state) VALUES(?1,?2,?3) ON CONFLICT(account,id) DO UPDATE SET state=excluded.state", params![after.id,message.id,state]).map_err(storage)?;
        }
    }
    db.execute("INSERT INTO message_index_state(account,revision) VALUES(?1,?2) ON CONFLICT(account) DO UPDATE SET revision=excluded.revision", params![after.id,after.revision]).map_err(storage)?;
    Ok(())
}

pub(super) fn selected(
    store: &SqliteStore,
    account: &str,
    ids: &[String],
) -> Result<MessageSelection, Error> {
    if ids.len() > 256 {
        return Err(Error::OverQuota);
    }
    let ids: BTreeSet<_> = ids.iter().collect();
    let mut db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let tx = db.transaction().map_err(storage)?;
    let Some(revision) = revision(&tx, account)? else {
        // Compatibility read only. The next successful write builds the index;
        // opening an old database never rewrites its authoritative snapshot.
        let old = load(&tx, account)?;
        return Ok(MessageSelection {
            revision: old.revision,
            messages: old
                .messages
                .into_iter()
                .filter(|m| ids.contains(&m.id))
                .collect(),
        });
    };
    let mut query = tx
        .prepare("SELECT state FROM message_metadata WHERE account=?1 AND id=?2")
        .map_err(storage)?;
    let mut messages = vec![];
    for id in ids {
        let state: Option<String> = query
            .query_row(params![account, id], |r| r.get(0))
            .optional()
            .map_err(storage)?;
        if let Some(state) = state {
            let message: Message = serde_json::from_str(&state).map_err(|_| Error::Unavailable)?;
            if message.id != *id {
                return Err(Error::Unavailable);
            }
            messages.push(message);
        }
    }
    Ok(MessageSelection { revision, messages })
}
