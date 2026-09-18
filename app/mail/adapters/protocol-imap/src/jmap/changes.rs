//! JMAP `*/changes` (RFC 8620 §5.2) and the delta behind `*/queryChanges`
//! (§5.6), both computed from the store's history log: rows name what a
//! commit touched, the current projection says what each id is now.
use mail_kernel::{Account, Error, HistoryEntry};
use mail_service::MailService;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Created,
    Updated,
    Destroyed,
}

/// History rows after `since`, none newer than the projection they are
/// classified against.
pub(super) struct Window {
    pub since: u64,
    pub rows: Vec<(u64, HistoryEntry)>,
    /// The state a client reaches once every row is consumed.
    pub end: u64,
    /// Revisions beyond `end` exist (page limit or a concurrent commit).
    pub more: bool,
}

pub(super) fn since(args: &Value, key: &str) -> Result<u64, &'static str> {
    args[key]
        .as_str()
        .ok_or("invalidArguments")?
        .parse()
        .map_err(|_| "cannotCalculateChanges")
}

pub(super) fn window(
    service: &MailService,
    token: &str,
    account: &Account,
    since: u64,
) -> Result<Window, &'static str> {
    if since > account.revision {
        return Err("cannotCalculateChanges");
    }
    let page = service
        .history(token, &account.id, since, 10_000)
        .map_err(|e| match e {
            Error::Conflict => "cannotCalculateChanges",
            _ => super::method::error(e),
        })?;
    if page.below_floor() {
        return Err("cannotCalculateChanges");
    }
    let lag = page.rows.iter().any(|(r, _)| *r > account.revision);
    let mut rows = page.rows;
    rows.retain(|(r, _)| *r <= account.revision);
    Ok(Window {
        since,
        rows,
        end: if page.has_more {
            page.revision.min(account.revision)
        } else {
            account.revision
        },
        more: page.has_more || lag,
    })
}

impl Window {
    /// Message ids whose first row here is a link creation: a record born
    /// and gone within the window is reported as neither (Stalwart parity).
    fn born(&self) -> BTreeSet<&str> {
        let mut seen = BTreeSet::new();
        let mut born = BTreeSet::new();
        for (_, entry) in &self.rows {
            if let Some(id) = entry.message()
                && seen.insert(id)
                && matches!(entry, HistoryEntry::Added { .. })
            {
                born.insert(id);
            }
        }
        born
    }

    /// Classified ids, whole revisions only; `None` from `kind` omits an
    /// id. Returns the ids, the state reached and whether more follows.
    pub fn classify(
        &self,
        max: usize,
        ids: &dyn Fn(&HistoryEntry) -> Vec<String>,
        kind: &dyn Fn(&str) -> Option<Kind>,
    ) -> Result<(BTreeMap<String, Kind>, u64, bool), &'static str> {
        let mut changed = BTreeMap::new();
        let mut last = self.since;
        for group in self.rows.chunk_by(|a, b| a.0 == b.0) {
            let mut next = changed.clone();
            for (_, entry) in group {
                for id in ids(entry) {
                    if let Some(kind) = kind(&id) {
                        next.insert(id, kind);
                    }
                }
            }
            if next.len() > max {
                // An atomic commit cannot be split into invented states.
                if last == self.since {
                    return Err("cannotCalculateChanges");
                }
                return Ok((changed, last, true));
            }
            changed = next;
            last = group[0].0;
        }
        Ok((changed, self.end, self.more))
    }

    pub fn response(
        &self,
        account: &Account,
        args: &Value,
        ids: &dyn Fn(&HistoryEntry) -> Vec<String>,
        kind: &dyn Fn(&str) -> Option<Kind>,
    ) -> Result<Value, &'static str> {
        let (changed, state, more) = self.classify(limit(args)?, ids, kind)?;
        let list = |wanted: Kind| -> Vec<&str> {
            changed
                .iter()
                .filter(|(_, kind)| **kind == wanted)
                .map(|(id, _)| id.as_str())
                .collect()
        };
        Ok(
            json!({"accountId":account.id,"oldState":args["sinceState"],"newState":state.to_string(),
            "hasMoreChanges":more,"created":list(Kind::Created),"updated":list(Kind::Updated),
            "destroyed":list(Kind::Destroyed)}),
        )
    }

    /// `Email` classification: ids of every message row.
    pub fn emails<'a>(
        &'a self,
        account: &'a Account,
    ) -> (
        impl Fn(&HistoryEntry) -> Vec<String> + 'a,
        impl Fn(&str) -> Option<Kind> + 'a,
    ) {
        let born = self.born();
        let messages: BTreeMap<_, _> = account
            .messages
            .iter()
            .map(|m| (m.id.as_str(), m))
            .collect();
        let since = self.since;
        let ids = |entry: &HistoryEntry| entry.message().map(str::to_owned).into_iter().collect();
        let kind = move |id: &str| match messages.get(id) {
            Some(m) if m.created_revision > since => Some(Kind::Created),
            Some(_) => Some(Kind::Updated),
            None if born.contains(id) => None,
            None => Some(Kind::Destroyed),
        };
        (ids, kind)
    }

    /// `Mailbox` classification: property changes plus every mailbox whose
    /// counters moved (links added or removed, flags of its messages).
    pub fn mailboxes<'a>(
        &'a self,
        account: &'a Account,
    ) -> (
        impl Fn(&HistoryEntry) -> Vec<String> + 'a,
        impl Fn(&str) -> Option<Kind> + 'a,
    ) {
        let since = self.since;
        let ids = |entry: &HistoryEntry| match entry {
            HistoryEntry::Mailbox { id } => vec![id.clone()],
            HistoryEntry::Added { mailbox, .. } | HistoryEntry::Removed { mailbox, .. } => {
                vec![mailbox.clone()]
            }
            HistoryEntry::Flags { id } => account
                .messages
                .iter()
                .find(|m| m.id == *id)
                .map(|m| m.mailboxes.keys().cloned().collect())
                .unwrap_or_default(),
            HistoryEntry::Thread { .. } => vec![],
        };
        let kind = move |id: &str| match account.mailboxes.iter().find(|m| m.id == id) {
            Some(m) if m.created_revision > since => Some(Kind::Created),
            Some(_) => Some(Kind::Updated),
            None => Some(Kind::Destroyed),
        };
        (ids, kind)
    }
}

pub(super) fn limit(args: &Value) -> Result<usize, &'static str> {
    if args["maxChanges"].is_null() {
        return Ok(10000);
    }
    let n = args["maxChanges"]
        .as_u64()
        .filter(|n| *n > 0)
        .ok_or("invalidArguments")?;
    Ok(n.min(10000) as usize)
}

pub(super) fn changes(
    service: &MailService,
    token: &str,
    account: &Account,
    args: &Value,
) -> Result<Value, &'static str> {
    let window = window(service, token, account, since(args, "sinceState")?)?;
    let (ids, kind) = window.emails(account);
    window.response(account, args, &ids, &kind)
}

pub(super) fn mailbox(
    service: &MailService,
    token: &str,
    account: &Account,
    args: &Value,
) -> Result<Value, &'static str> {
    let window = window(service, token, account, since(args, "sinceState")?)?;
    let (ids, kind) = window.mailboxes(account);
    let mut value = window.response(account, args, &ids, &kind)?;
    value["updatedProperties"] = Value::Null;
    Ok(value)
}

/// §5.6 delta over `current` (the query result now): a created id is added
/// where it sits, a destroyed one removed, an updated one removed and added
/// again so a client re-places it. Errors when the window is incomplete or
/// the delta exceeds `max`.
pub(super) fn delta(
    window: &Window,
    ids: &dyn Fn(&HistoryEntry) -> Vec<String>,
    kind: &dyn Fn(&str) -> Option<Kind>,
    current: &[String],
    max: usize,
) -> Result<(Vec<String>, Vec<Value>), &'static str> {
    let (changed, _, more) = window.classify(usize::MAX, ids, kind)?;
    if more {
        return Err("tooManyChanges");
    }
    let removed: Vec<String> = changed
        .iter()
        .filter(|(_, kind)| **kind != Kind::Created)
        .map(|(id, _)| id.clone())
        .collect();
    let added: Vec<Value> = current
        .iter()
        .enumerate()
        .filter(|(_, id)| changed.contains_key(*id))
        .map(|(index, id)| json!({"id":id,"index":index}))
        .collect();
    if removed.len() + added.len() > max {
        return Err("tooManyChanges");
    }
    Ok((removed, added))
}
