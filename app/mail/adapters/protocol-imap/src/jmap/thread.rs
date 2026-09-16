use mail_api::Change;
use mail_kernel::{Account, MessageState};
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

pub(super) fn changes(
    service: &MailService,
    token: &str,
    account: &Account,
    args: &Value,
) -> Result<Value, &'static str> {
    let since = args["sinceState"]
        .as_str()
        .ok_or("invalidArguments")?
        .parse()
        .map_err(|_| "cannotCalculateChanges")?;
    let changes = super::changes::history(service, token, account, since)?;
    let mut old: BTreeMap<_, _> = account
        .messages
        .iter()
        .map(|m| (m.id.clone(), m.state()))
        .collect();
    for change in changes.iter().rev() {
        if let Some(before) = &change.before {
            old.insert(change.id.clone(), before.clone());
        } else {
            old.remove(&change.id);
        }
    }
    let mut threads = Threads::new();
    for message in old.values() {
        insert(&mut threads, message);
    }
    let mut result = vec![];
    // Update only groups touched by each atomic commit; do not reconstruct
    // every mailbox or read message bodies for each historical revision.
    for batch in changes.chunk_by(|a, b| a.revision == b.revision) {
        let affected: BTreeSet<_> = batch
            .iter()
            .flat_map(|c| [&c.before, &c.after])
            .flatten()
            .map(|m| thread_id(m).to_owned())
            .collect();
        let before: BTreeMap<_, _> = affected
            .iter()
            .map(|id| (id.clone(), threads.get(id).cloned()))
            .collect();
        for change in batch {
            if let Some(old) = &change.before {
                let id = thread_id(old);
                if let Some(members) = threads.get_mut(id) {
                    members.remove(&(old.received_at, old.id.clone()));
                    if members.is_empty() {
                        threads.remove(id);
                    }
                }
            }
            if let Some(new) = &change.after {
                insert(&mut threads, new);
            }
        }
        for (id, before) in before {
            let after = threads.get(&id).cloned();
            if before != after {
                result.push(Change {
                    revision: batch[0].revision,
                    id,
                    before,
                    after,
                });
            }
        }
    }
    super::changes::response(account, args, &result)
}
