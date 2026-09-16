use super::{Account, Command, Error};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn initial() -> u64 {
    1
}

pub(super) struct Change {
    sequence: u64,
    messages: BTreeSet<String>,
}

impl Account {
    /// Assign one visible modification sequence to an atomic commit. Adapters
    /// call this after all message and thread changes, before persisting state.
    pub fn complete_batch(&mut self, before: &Account) -> Result<(), Error> {
        if self.id != before.id || self.revision < before.revision {
            return Err(Error::Conflict);
        }
        let previous: BTreeMap<_, _> = before.messages.iter().map(|m| (&m.id, m)).collect();
        let changed: Vec<_> = self
            .messages
            .iter()
            .map(|message| {
                previous
                    .get(&message.id)
                    .is_none_or(|old| !same_message(old, message))
            })
            .collect();
        let has_changes =
            self.messages.len() != before.messages.len() || changed.iter().any(|changed| *changed);
        let sequence = if has_changes {
            let sequence = self.revision.checked_add(1).ok_or(Error::OverQuota)?;
            if sequence <= before.mail_modseq || self.revision == before.revision {
                return Err(Error::OverQuota);
            }
            sequence
        } else {
            before.mail_modseq
        };
        // No fallible work remains: cancelled edits restore the original stamp.
        for (message, changed) in self.messages.iter_mut().zip(changed) {
            message.modseq = if changed {
                sequence
            } else {
                previous[&message.id].modseq
            };
        }
        self.mail_modseq = sequence;
        Ok(())
    }

    pub(super) fn prepare_modseq(
        &self,
        command: &Command,
        revision: u64,
    ) -> Result<Option<Change>, Error> {
        let mut messages = BTreeSet::new();
        match command {
            Command::Append { .. } => {
                messages.insert(format!("e{revision}"));
            }
            Command::Transfer {
                id, remove_from, ..
            } => {
                messages.insert(format!("e{revision}"));
                if remove_from.is_some() {
                    messages.insert(id.clone());
                }
            }
            Command::Destroy { id } => {
                messages.insert(id.clone());
            }
            Command::Keywords { id, keywords } => {
                if let Some(message) = self.messages.iter().find(|m| &m.id == id)
                    && message.keywords.iter().collect::<BTreeSet<_>>()
                        != keywords.iter().collect::<BTreeSet<_>>()
                {
                    messages.insert(id.clone());
                }
            }
            Command::SetMailboxes { id, mailboxes } => {
                if let Some(message) = self.messages.iter().find(|m| &m.id == id)
                    && message.mailboxes.keys().collect::<BTreeSet<_>>()
                        != mailboxes.iter().collect::<BTreeSet<_>>()
                {
                    messages.insert(id.clone());
                }
            }
            Command::Expunge { mailbox } => {
                messages.extend(
                    self.messages
                        .iter()
                        .filter(|message| {
                            message.mailboxes.contains_key(mailbox)
                                && message.keywords.iter().any(|k| k == "$deleted")
                        })
                        .map(|message| message.id.clone()),
                );
            }
            Command::RemoveMailbox {
                id,
                remove_emails: true,
            } => {
                messages.extend(
                    self.messages
                        .iter()
                        .filter(|message| message.mailboxes.contains_key(id))
                        .map(|message| message.id.clone()),
                );
            }
            _ => {}
        }
        if messages.is_empty() {
            return Ok(None);
        }
        // Revision remains the CAS/journal cursor. The offset reserves the
        // positive message modseqs and allows consumers to recover that cursor.
        let sequence = revision.checked_add(1).ok_or(Error::OverQuota)?;
        if sequence <= self.mail_modseq {
            return Err(Error::OverQuota);
        }
        Ok(Some(Change { sequence, messages }))
    }

    pub(super) fn commit_modseq(&mut self, change: Option<Change>) {
        if let Some(change) = change {
            self.mail_modseq = change.sequence;
            for message in &mut self.messages {
                if change.messages.contains(&message.id) {
                    message.modseq = change.sequence;
                }
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for Account {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Stored {
            id: String,
            tenant: String,
            owner: String,
            address: String,
            revision: u64,
            #[serde(default, deserialize_with = "present_sequence")]
            mail_modseq: Option<u64>,
            #[serde(default)]
            identity: super::IdentitySettings,
            #[serde(default)]
            identity_revision: u64,
            #[serde(default)]
            vacation: super::VacationSettings,
            #[serde(default)]
            vacation_revision: u64,
            quota_bytes: usize,
            mailboxes: Vec<super::Mailbox>,
            messages: Vec<super::Message>,
        }
        let stored = Stored::deserialize(deserializer)?;
        // Before modseq support, empty accounts had no message log. Existing
        // messages need a positive baseline when their older snapshot is read.
        let mail_modseq = stored
            .mail_modseq
            .unwrap_or_else(|| stored.messages.iter().map(|m| m.modseq).max().unwrap_or(0));
        Ok(Self {
            id: stored.id,
            tenant: stored.tenant,
            owner: stored.owner,
            address: stored.address,
            revision: stored.revision,
            mail_modseq,
            identity: stored.identity,
            identity_revision: stored.identity_revision,
            vacation: stored.vacation,
            vacation_revision: stored.vacation_revision,
            quota_bytes: stored.quota_bytes,
            mailboxes: stored.mailboxes,
            messages: stored.messages,
        })
    }
}

fn present_sequence<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u64>, D::Error> {
    <u64 as serde::Deserialize>::deserialize(deserializer).map(Some)
}

fn same_message(a: &super::Message, b: &super::Message) -> bool {
    a.id == b.id
        && a.thread == b.thread
        && a.email_identity == b.email_identity
        && a.thread_identity == b.thread_identity
        && a.mailboxes == b.mailboxes
        && a.size == b.size
        && a.keywords == b.keywords
        && a.received_at == b.received_at
}
