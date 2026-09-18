pub(super) use super::state::{Selection, synchronize};
use super::syntax::sequence_set;
use mail_kernel::{Account, Command, Message};
use mail_service::{Budget, MailService};

pub(super) fn selected_command(
    service: &MailService,
    token: &str,
    account: &Account,
    parts: &[String],
    selection: &mut Option<Selection>,
    budget: &Budget,
    output: &mut super::response::Output,
) -> Result<Option<String>, &'static str> {
    let selected = selection.as_mut().ok_or("BAD")?;
    let verb = parts[1].to_ascii_uppercase();

    let uid = verb == "UID";
    let offset = if uid { 3 } else { 2 };
    let operation = if uid {
        parts.get(2).ok_or("BAD")?.to_ascii_uppercase()
    } else {
        verb
    };
    // A bare `UID EXPUNGE` addresses every message, like Stalwart.
    let bare_expunge = uid && operation == "EXPUNGE" && parts.len() == offset;
    if parts.len() <= offset && !bare_expunge {
        return Err("BAD");
    }
    let mailbox = &selected.mailbox;
    let by_id: std::collections::BTreeMap<_, _> = account
        .messages
        .iter()
        .map(|m| (m.id.as_str(), m))
        .collect();
    let messages: Vec<_> = selected
        .ids
        .iter()
        .enumerate()
        .filter_map(|(i, (id, uid))| {
            by_id
                .get(id.as_str())
                .filter(|m| m.uid_in(mailbox) == Some(*uid))
                .map(|m| (i, *m))
        })
        .collect();
    if matches!(operation.as_str(), "SEARCH" | "SORT" | "THREAD") {
        return super::search::execute(service, token, account, selected, parts, &messages, output);
    }
    let largest = if uid {
        selected.largest_uid(account)
    } else {
        selected.ids.len() as u32
    };
    let saved = !bare_expunge && parts[offset] == "$";
    let set = if saved {
        vec![]
    } else if bare_expunge {
        vec![(1, u32::MAX)]
    } else {
        sequence_set(&parts[offset], largest).ok_or("BAD")?
    };
    let chosen: Vec<_> = messages
        .iter()
        .copied()
        .filter(|(i, m)| {
            if saved {
                return selected
                    .saved
                    .contains(&(m.id.clone(), m.uid_in(mailbox).unwrap_or_default()));
            }
            set.iter().any(|(a, b)| {
                let n = if uid {
                    m.uid_in(mailbox).unwrap_or_default()
                } else {
                    (*i + 1) as u32
                };
                n >= *a && n <= *b
            })
        })
        .collect();
    if operation == "EXPUNGE" && uid {
        if parts.len() > offset + 1 || selected.readonly {
            return Err("NO");
        }
        let commands = chosen
            .iter()
            .filter(|(_, m)| m.keywords.iter().any(|k| k == "$deleted"))
            .map(|(_, m)| {
                let mailboxes: Vec<_> = m
                    .mailboxes
                    .keys()
                    .filter(|id| *id != mailbox)
                    .cloned()
                    .collect();
                if mailboxes.is_empty() {
                    Command::Destroy { id: m.id.clone() }
                } else {
                    Command::SetMailboxes {
                        id: m.id.clone(),
                        mailboxes,
                    }
                }
            })
            .collect();
        let (_, updated) = super::retry::commit(service, token, account, commands, budget)?;
        synchronize(&updated, parts, selection, output);
        return Ok(None);
    }
    let call = super::retry::Call {
        service,
        token,
        account,
        budget,
    };
    if matches!(operation.as_str(), "COPY" | "MOVE") {
        return super::transfer::execute(&call, parts, selection, chosen, output);
    }
    if operation == "FETCH" {
        return super::fetch::execute(&call, selected, chosen, &set, parts, output);
    }
    if operation == "STORE" {
        return super::store::execute(&call, selected, chosen, parts, output);
    }
    Err("BAD")
}

pub(super) fn messages_in<'a>(account: &'a Account, mailbox: &str) -> Vec<&'a Message> {
    let mut messages: Vec<_> = account
        .messages
        .iter()
        .filter(|m| m.uid_in(mailbox).is_some())
        .collect();
    messages.sort_by_key(|m| m.uid_in(mailbox).unwrap_or_default());
    messages
}
