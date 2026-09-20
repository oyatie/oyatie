use super::changes::Kind;
use mail_kernel::{Account, HistoryEntry, MessageState};
use mail_service::MailService;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type Members = BTreeSet<(i64, String)>;
type Threads = BTreeMap<String, Members>;

fn thread_id(message: &MessageState) -> &str {
    message.thread.as_deref().unwrap_or(&message.id)
}

fn insert(threads: &mut Threads, message: &MessageState) {
    threads
        .entry(thread_id(message).into())
        .or_default()
        .insert((message.received_at, message.id.clone()));
}

pub(super) fn get(account: &Account, args: &Value) -> Result<Value, &'static str> {
    let ids = super::method::get_ids(args)?;
    if let Some(properties) = args.get("properties").filter(|v| !v.is_null())
        && properties
            .as_array()
            .ok_or("invalidArguments")?
            .iter()
            .any(|v| v != "id" && v != "emailIds")
    {
        return Err("invalidArguments");
    }
    let mut threads = Threads::new();
    for message in &account.messages {
        if ids
            .as_ref()
            .is_none_or(|ids| ids.contains(&message.thread_id()))
        {
            insert(&mut threads, &message.state());
        }
    }
    if threads.len() > 256 {
        return Err("tooManyObjects");
    }
    super::method::get_result(&account.id, account.revision, args, threads.into_iter().map(|(id, members)| {
        json!({"id":id,"emailIds":members.into_iter().map(|(_,id)| id).collect::<Vec<_>>()})
    }).collect())
}

/// A thread changes when its membership does: a message created in it, a
/// member gone or re-threaded. Flag changes leave the `Thread` object alone.
pub(super) fn changes(
    service: &MailService,
    token: &str,
    account: &Account,
    args: &Value,
) -> Result<Value, &'static str> {
    let since = super::changes::since(args, "sinceState")?;
    let window = super::changes::window(service, token, account, since)?;
    let messages: BTreeMap<_, _> = account
        .messages
        .iter()
        .map(|m| (m.id.as_str(), m))
        .collect();
    let thread_of = |id: &str| messages.get(id).map(|m| m.thread_id().to_owned());
    let ids = |entry: &HistoryEntry| match entry {
        HistoryEntry::Added { id, .. } => messages
            .get(id.as_str())
            .filter(|m| m.created_revision > since)
            .map(|m| vec![m.thread_id().to_owned()])
            .unwrap_or_default(),
        // The row does not name the thread the message left. A thread is
        // identified by its first message, so the message's own id is the
        // candidate for a thread that emptied when it was re-threaded.
        HistoryEntry::Thread { id } => {
            let mut ids = vec![id.clone()];
            ids.extend(thread_of(id));
            ids
        }
        HistoryEntry::Removed { id, thread, .. } => {
            if thread_of(id).is_some_and(|current| current == *thread) {
                vec![]
            } else {
                vec![thread.clone()]
            }
        }
        HistoryEntry::Flags { .. } | HistoryEntry::Mailbox { .. } => vec![],
    };
    // Members per thread: (all, created after `since`).
    let mut members: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for message in &account.messages {
        let entry = members.entry(message.thread_id()).or_default();
        entry.0 += 1;
        entry.1 += usize::from(message.created_revision > since);
    }
    let kind = |id: &str| match members.get(id) {
        None => Some(Kind::Destroyed),
        Some((all, new)) if all == new => Some(Kind::Created),
        Some(_) => Some(Kind::Updated),
    };
    window.response(account, args, &ids, &kind)
}
