use crate::{Decryption, Error, Message};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionChange {
    pub target: String,
    pub key: String,
    pub active: bool,
    #[serde(default)]
    pub removes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReactionSummary {
    pub key: String,
    pub senders: Vec<String>,
    #[serde(default)]
    pub events: BTreeMap<String, Vec<String>>,
}

pub fn reaction_key(key: &str) -> Result<&str, Error> {
    if key.is_empty() || key.len() > 64 || key.trim() != key || key.chars().any(char::is_control) {
        return Err(Error::Invalid(
            "reaction must be 1–64 bytes without control characters".into(),
        ));
    }
    Ok(key)
}

fn reaction_event_id(id: &str) -> bool {
    id.starts_with('$')
        && id.len() <= 255
        && !id.chars().any(|c| c.is_whitespace() || c.is_control())
}

impl ReactionChange {
    pub fn validate(&self) -> Result<(), Error> {
        reaction_key(&self.key)?;
        if !reaction_event_id(&self.target)
            || self.removes.len() > 256
            || self.removes.iter().any(|id| !reaction_event_id(id))
            || (self.active && !self.removes.is_empty())
        {
            return Err(Error::Invalid(
                "invalid reaction target or observed removals".into(),
            ));
        }
        Ok(())
    }
}

pub(super) fn reactions(messages: &[Message]) -> BTreeMap<String, Vec<ReactionSummary>> {
    // Event IDs identify additions; encrypted controls remove only explicitly
    // observed additions by their own author, target and key. Timestamps cannot
    // establish causality, and input order changes during history pagination.
    type Observed<'a> = (BTreeSet<&'a str>, BTreeSet<&'a str>);
    let mut changes = BTreeMap::<(&str, &str, &str), Observed<'_>>::new();
    for message in messages {
        if message.redacted || message.decryption != Decryption::Decrypted {
            continue;
        }
        let Some(change) = &message.reaction else {
            continue;
        };
        if change.validate().is_err() || !reaction_event_id(&message.id) {
            continue;
        }
        let (adds, removed) = changes
            .entry((&change.target, &change.key, &message.sender))
            .or_default();
        if change.active {
            adds.insert(&message.id);
        } else {
            removed.extend(change.removes.iter().map(String::as_str));
        }
    }
    let mut groups = BTreeMap::<String, BTreeMap<String, BTreeMap<String, Vec<String>>>>::new();
    for ((target, key, sender), (adds, removed)) in changes {
        let active: Vec<_> = adds
            .difference(&removed)
            .map(|id| (*id).to_owned())
            .collect();
        if !active.is_empty() {
            groups
                .entry(target.into())
                .or_default()
                .entry(key.into())
                .or_default()
                .insert(sender.into(), active);
        }
    }
    groups
        .into_iter()
        .map(|(target, keys)| {
            (
                target,
                keys.into_iter()
                    .map(|(key, events)| ReactionSummary {
                        key,
                        senders: events.keys().cloned().collect(),
                        events,
                    })
                    .collect(),
            )
        })
        .collect()
}

pub fn visible_messages(messages: &[Message]) -> Vec<Message> {
    let grouped = reactions(messages);
    let mut edits = std::collections::BTreeMap::<(&str, &str), &Message>::new();
    for message in messages {
        if let Some(target) = message.edit_of.as_deref()
            && !message.redacted
            && message.decryption == Decryption::Decrypted
        {
            let edit = edits.entry((target, &message.sender)).or_insert(message);
            if (message.timestamp, &message.id) > (edit.timestamp, &edit.id) {
                *edit = message;
            }
        }
    }
    messages
        .iter()
        .filter(|m| m.edit_of.is_none() && m.reaction.is_none())
        .cloned()
        .map(|mut message| {
            message.reactions = if !message.redacted && message.decryption == Decryption::Decrypted
            {
                grouped.get(&message.id).cloned().unwrap_or_default()
            } else {
                Vec::new()
            };
            if !message.redacted
                && message.decryption == Decryption::Decrypted
                && let Some(edit) = edits.get(&(message.id.as_str(), message.sender.as_str()))
            {
                message.body.clone_from(&edit.body);
                message.attachment.clone_from(&edit.attachment);
            }
            message
        })
        .collect()
}

/// Search only locally decrypted, current projections. Callers load additional
/// encrypted history explicitly; no query or plaintext is sent to a search server.
pub fn search_messages(messages: &[Message], query: &str) -> Result<Vec<Message>, Error> {
    if query.len() > 256 || query.chars().any(char::is_control) {
        return Err(Error::Invalid(
            "search must not exceed 256 bytes or contain control characters".into(),
        ));
    }
    let query = query.to_lowercase();
    let terms: Vec<_> = query.split_whitespace().collect();
    if terms.is_empty() {
        return Ok(Vec::new());
    }
    Ok(visible_messages(messages)
        .into_iter()
        .filter(|message| {
            if message.redacted || message.decryption != Decryption::Decrypted {
                return false;
            }
            let body = message.body.to_lowercase();
            terms.iter().all(|term| body.contains(term))
        })
        .collect())
}
