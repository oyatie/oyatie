use super::{SqliteStore, storage};
use mail_api::Change;
use mail_kernel::{Account, Error};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::{BTreeMap, BTreeSet};

pub(super) enum Kind {
    Message,
    MessageAfter,
    Mailbox,
}
impl Kind {
    fn tables(&self) -> (&'static str, &'static str) {
        match self {
            Self::Message | Self::MessageAfter => ("history_commits", "message_changes"),
            Self::Mailbox => ("mailbox_commits", "mailbox_changes"),
        }
    }
}

pub(super) fn record(db: &Connection, old: &Account, new: &Account) -> Result<(), Error> {
    let before = old
        .messages
        .iter()
        .map(|m| (m.id.clone(), m.state()))
        .collect();
    let after = new
        .messages
        .iter()
        .map(|m| (m.id.clone(), m.state()))
        .collect();
    record_kind(db, old, new, Kind::Message, before, after)?;
    let before = old
        .mailbox_states()
        .into_iter()
        .map(|m| (m.mailbox.id.clone(), m))
        .collect();
    let after = new
        .mailbox_states()
        .into_iter()
        .map(|m| (m.mailbox.id.clone(), m))
        .collect();
    record_kind(db, old, new, Kind::Mailbox, before, after)
}

fn record_kind<T: Serialize + PartialEq>(
    db: &Connection,
    old: &Account,
    new: &Account,
    kind: Kind,
    before: BTreeMap<String, T>,
    after: BTreeMap<String, T>,
) -> Result<(), Error> {
    let (commits, changes) = kind.tables();
    let insert = format!(
        "INSERT INTO {changes}(account,revision,id,before_state,after_state) VALUES(?1,?2,?3,?4,?5)"
    );
    for id in before.keys().chain(after.keys()).collect::<BTreeSet<_>>() {
        let old = before.get(id);
        let next = after.get(id);
        if old != next {
            let old = old
                .map(serde_json::to_string)
                .transpose()
                .map_err(|_| Error::Unavailable)?;
            let next = next
                .map(serde_json::to_string)
                .transpose()
                .map_err(|_| Error::Unavailable)?;
            db.execute(&insert, params![new.id, new.revision, id, old, next])
                .map_err(storage)?;
        }
    }
    db.execute(
        &format!("INSERT INTO {commits}(account,previous,revision) VALUES(?1,?2,?3)"),
        params![new.id, old.revision, new.revision],
    )
    .map_err(storage)?;
    Ok(())
}

pub(super) fn read<T: DeserializeOwned>(
    store: &SqliteStore,
    account: &str,
    since: u64,
    until: u64,
    kind: Kind,
) -> Result<Vec<Change<T>>, Error> {
    if since > until {
        return Err(Error::Conflict);
    }
    let mut db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let tx = db.transaction().map_err(storage)?;
    let current: u64 = tx
        .query_row(
            "SELECT json_extract(state,'$.revision') FROM accounts WHERE id=?1",
            [account],
            |r| r.get(0),
        )
        .map_err(storage)?;
    if until > current {
        return Err(Error::Conflict);
    }
    let (commits, changes) = kind.tables();
    let since = if matches!(kind, Kind::MessageAfter) && since < until {
        let previous: Option<u64> = tx.query_row(
            "SELECT previous FROM history_commits WHERE account=?1 AND revision>?2 AND revision<=?3 ORDER BY revision LIMIT 1",
            params![account, since, until], |row| row.get(0),
        ).optional().map_err(storage)?;
        match previous {
            Some(previous) if previous <= since => previous,
            _ => return Err(Error::Conflict),
        }
    } else {
        since
    };
    let mut commits = tx.prepare(&format!("SELECT previous,revision FROM {commits} WHERE account=?1 AND revision>?2 AND revision<=?3 ORDER BY revision LIMIT 10001")).map_err(storage)?;
    let mut rows = commits
        .query(params![account, since, until])
        .map_err(storage)?;
    let mut position = since;
    let mut count = 0;
    while let Some(row) = rows.next().map_err(storage)? {
        let previous: u64 = row.get(0).map_err(storage)?;
        if previous != position || count == 10000 {
            return Err(Error::Conflict);
        }
        position = row.get(1).map_err(storage)?;
        count += 1;
    }
    if position != until {
        return Err(Error::Conflict);
    }
    let mut query = tx.prepare(&format!("SELECT revision,id,before_state,after_state FROM {changes} WHERE account=?1 AND revision>?2 AND revision<=?3 ORDER BY revision,id LIMIT 10001")).map_err(storage)?;
    let mut rows = query
        .query(params![account, since, until])
        .map_err(storage)?;
    let mut result = vec![];
    while let Some(row) = rows.next().map_err(storage)? {
        if result.len() == 10000 {
            return Err(Error::Conflict);
        }
        let decode = |column| -> Result<Option<T>, Error> {
            let value: Option<String> = row.get(column).map_err(storage)?;
            value
                .map(|v| serde_json::from_str(&v))
                .transpose()
                .map_err(|_| Error::Unavailable)
        };
        result.push(Change {
            revision: row.get(0).map_err(storage)?,
            id: row.get(1).map_err(storage)?,
            before: decode(2)?,
            after: decode(3)?,
        });
    }
    Ok(result)
}
