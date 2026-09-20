use super::{Account, Error, Message};
use std::collections::BTreeMap;

/// One row of `history/{revision}`. `Added`/`Removed` describe a link between
/// a message record and a mailbox; the transaction that removes the last link
/// deletes the record and emits only `Removed`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HistoryEntry {
    Added {
        id: String,
        mailbox: String,
        uid: u32,
    },
    Removed {
        id: String,
        mailbox: String,
        uid: u32,
        /// Thread the record belonged to, so consumers can classify a thread
        /// whose last member vanished without reading the deleted record.
        thread: String,
    },
    Flags {
        id: String,
    },
    Thread {
        id: String,
    },
    Mailbox {
        id: String,
    },
}

impl HistoryEntry {
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Added { id, .. }
            | Self::Removed { id, .. }
            | Self::Flags { id }
            | Self::Thread { id } => Some(id),
            Self::Mailbox { .. } => None,
        }
    }
}

/// What one committed batch changed, in a form an adapter persists without
/// re-deriving any rule: the new revision, created record ids in creation
/// order, `(mailbox, uid)` allocations in append order, history rows and the
/// records whose last link was removed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effects {
    pub revision: u64,
    pub ids: Vec<String>,
    pub allocations: Vec<(String, u32)>,
    pub history: Vec<HistoryEntry>,
    pub deleted: Vec<String>,
    pub mail_changed: bool,
}

/// Compaction keeps at most `max_rows` history rows or `max_age_secs` of
/// history per account, and never advances past the lowest cursor of a
/// consumer enabled in this cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetentionPolicy {
    pub max_age_secs: i64,
    pub max_rows: u64,
}
impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_age_secs: 90 * 24 * 60 * 60,
            max_rows: 100_000,
        }
    }
}

/// Outcome of one compaction pass over an account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Retention {
    Advanced {
        floor: u64,
    },
    /// The policy wanted `wanted` but a consumer cursor held the floor.
    Blocked {
        floor: u64,
        wanted: u64,
        consumer: String,
    },
}

impl RetentionPolicy {
    /// `candidate` is the highest revision the age and row limits allow;
    /// `cursors` are the enabled consumers' positions.
    pub fn floor(&self, current: u64, candidate: u64, cursors: &[(&str, u64)]) -> Retention {
        let wanted = candidate.max(current);
        match cursors
            .iter()
            .filter(|(_, c)| *c < wanted)
            .min_by_key(|(_, c)| *c)
        {
            Some((consumer, cursor)) => Retention::Blocked {
                floor: (*cursor).max(current),
                wanted,
                consumer: (*consumer).to_owned(),
            },
            None => Retention::Advanced { floor: wanted },
        }
    }
}

fn ordinal(id: &str) -> u64 {
    id.trim_start_matches(|c: char| c.is_ascii_alphabetic())
        .parse()
        .unwrap_or(u64::MAX)
}

impl Account {
    /// Stamp one atomic batch: every changed message, and every mailbox one
    /// was linked to or unlinked from, takes the commit revision as MODSEQ.
    pub fn commit(&mut self, before: &Account) -> Result<Effects, Error> {
        if self.id != before.id || self.revision < before.revision {
            return Err(Error::Conflict);
        }
        let revision = self.revision;
        let previous: BTreeMap<_, _> = before.messages.iter().map(|m| (&m.id, m)).collect();
        let mut effects = Effects {
            revision,
            ..Effects::default()
        };
        let mut changed = vec![false; self.messages.len()];
        let mut touched = Vec::new();
        for (index, message) in self.messages.iter().enumerate() {
            match previous.get(&message.id) {
                None => {
                    changed[index] = true;
                    effects.ids.push(message.id.clone());
                    for (mailbox, uid) in &message.mailboxes {
                        touched.push(mailbox.clone());
                        effects.history.push(HistoryEntry::Added {
                            id: message.id.clone(),
                            mailbox: mailbox.clone(),
                            uid: *uid,
                        });
                    }
                }
                Some(old) => {
                    changed[index] = diff(old, message, &mut effects.history, &mut touched);
                    if changed[index] {
                        touched.extend(message.mailboxes.keys().cloned());
                    }
                }
            }
        }
        for old in before
            .messages
            .iter()
            .filter(|m| !self.messages.iter().any(|current| current.id == m.id))
        {
            effects.deleted.push(old.id.clone());
            for (mailbox, uid) in &old.mailboxes {
                touched.push(mailbox.clone());
                effects.history.push(HistoryEntry::Removed {
                    id: old.id.clone(),
                    mailbox: mailbox.clone(),
                    uid: *uid,
                    thread: old.thread_id().to_owned(),
                });
            }
        }
        effects.ids.sort_by_key(|id| ordinal(id));
        effects.allocations = effects
            .history
            .iter()
            .filter_map(|entry| match entry {
                HistoryEntry::Added { mailbox, uid, .. } => Some((mailbox.clone(), *uid)),
                _ => None,
            })
            .collect();
        effects.allocations.sort();
        effects.mail_changed = changed.iter().any(|c| *c) || !effects.deleted.is_empty();
        for mailbox in &mut self.mailboxes {
            let old = before.mailboxes.iter().find(|m| m.id == mailbox.id);
            if old.is_none() {
                mailbox.created_revision = revision;
            }
            if old.is_none_or(|old| old.properties() != mailbox.properties()) {
                effects.history.push(HistoryEntry::Mailbox {
                    id: mailbox.id.clone(),
                });
            }
            if touched.contains(&mailbox.id) {
                mailbox.highest_modseq = revision;
            }
        }
        for old in &before.mailboxes {
            if !self.mailboxes.iter().any(|m| m.id == old.id) {
                effects
                    .history
                    .push(HistoryEntry::Mailbox { id: old.id.clone() });
            }
        }
        if effects.mail_changed {
            if revision == before.revision {
                return Err(Error::Conflict);
            }
            self.mail_modseq = revision;
        }
        for (message, changed) in self.messages.iter_mut().zip(changed) {
            if changed {
                message.modseq = revision;
                if !previous.contains_key(&message.id) {
                    message.created_revision = revision;
                }
            }
        }
        Ok(effects)
    }
}

/// Record link, flag and thread changes of a retained message; returns
/// whether the record changed at all.
fn diff(
    old: &Message,
    new: &Message,
    history: &mut Vec<HistoryEntry>,
    touched: &mut Vec<String>,
) -> bool {
    let mut changed = false;
    for (mailbox, uid) in &old.mailboxes {
        if new.mailboxes.get(mailbox) != Some(uid) {
            changed = true;
            touched.push(mailbox.clone());
            history.push(HistoryEntry::Removed {
                id: old.id.clone(),
                mailbox: mailbox.clone(),
                uid: *uid,
                thread: old.thread_id().to_owned(),
            });
        }
    }
    for (mailbox, uid) in &new.mailboxes {
        if old.mailboxes.get(mailbox) != Some(uid) {
            changed = true;
            touched.push(mailbox.clone());
            history.push(HistoryEntry::Added {
                id: new.id.clone(),
                mailbox: mailbox.clone(),
                uid: *uid,
            });
        }
    }
    if old.keywords != new.keywords {
        changed = true;
        history.push(HistoryEntry::Flags { id: new.id.clone() });
    }
    if old.thread != new.thread
        || old.email_identity != new.email_identity
        || old.thread_identity != new.thread_identity
    {
        changed = true;
        history.push(HistoryEntry::Thread { id: new.id.clone() });
    }
    changed || old.size != new.size || old.received_at != new.received_at
}
