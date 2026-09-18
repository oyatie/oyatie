use super::{Account, Error, valid_name};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Mailbox record: properties, the per-mailbox UID counter, HIGHESTMODSEQ and
/// the MESSAGES/UNSEEN/SIZE counters maintained inside every mutation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mailbox {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub sort_order: u32,
    #[serde(default = "subscribed")]
    pub is_subscribed: bool,
    pub uid_next: u32,
    pub uid_validity: u32,
    #[serde(default)]
    pub created_revision: u64,
    #[serde(default)]
    pub highest_modseq: u64,
    #[serde(default)]
    pub total_emails: usize,
    #[serde(default)]
    pub unread_emails: usize,
    #[serde(default)]
    pub size_bytes: usize,
}
fn subscribed() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MailboxProperties {
    pub name: String,
    pub parent_id: Option<String>,
    pub role: Option<String>,
    pub sort_order: u32,
    pub is_subscribed: bool,
}
impl MailboxProperties {
    pub fn named(name: String) -> Self {
        Self {
            name,
            parent_id: None,
            role: None,
            sort_order: 0,
            is_subscribed: true,
        }
    }
}
impl Mailbox {
    pub fn new(id: &str, name: &str, role: Option<&str>, uid_validity: u32) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            role: role.map(Into::into),
            parent_id: None,
            sort_order: 0,
            is_subscribed: true,
            uid_next: 1,
            uid_validity,
            created_revision: 0,
            highest_modseq: 0,
            total_emails: 0,
            unread_emails: 0,
            size_bytes: 0,
        }
    }
    pub fn properties(&self) -> MailboxProperties {
        MailboxProperties {
            name: self.name.clone(),
            parent_id: self.parent_id.clone(),
            role: self.role.clone(),
            sort_order: self.sort_order,
            is_subscribed: self.is_subscribed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailboxState {
    pub mailbox: Mailbox,
    pub total_emails: usize,
    pub unread_emails: usize,
    #[serde(default)]
    pub total_threads: Option<usize>,
    #[serde(default)]
    pub unread_threads: Option<usize>,
}

impl Account {
    /// Counters come from the mailbox record; thread counters need the loaded
    /// messages and are reported only when the full projection is present.
    pub fn mailbox_states(&self) -> Vec<MailboxState> {
        let mut threads = BTreeMap::new();
        for message in &self.messages {
            let unread = !message.seen();
            for id in message.mailboxes.keys() {
                let (all, unseen) = threads
                    .entry(id.as_str())
                    .or_insert((BTreeSet::new(), BTreeSet::new()));
                all.insert(message.thread_id());
                if unread {
                    unseen.insert(message.thread_id());
                }
            }
        }
        self.mailboxes
            .iter()
            .map(|m| {
                let (total_threads, unread_threads) = threads
                    .get(m.id.as_str())
                    .map(|(all, unseen)| (all.len(), unseen.len()))
                    .unwrap_or_default();
                MailboxState {
                    mailbox: m.clone(),
                    total_emails: m.total_emails,
                    unread_emails: m.unread_emails,
                    total_threads: Some(total_threads),
                    unread_threads: Some(unread_threads),
                }
            })
            .collect()
    }

    pub(super) fn set_mailbox(
        &mut self,
        id: Option<String>,
        properties: MailboxProperties,
        revision: u64,
    ) -> Result<(), Error> {
        if !valid_name(&properties.name) || properties.name.contains('/') {
            return Err(Error::Invalid);
        }
        let existing = id
            .as_ref()
            .map(|id| {
                self.mailboxes
                    .iter()
                    .position(|m| &m.id == id)
                    .ok_or(Error::NotFound)
            })
            .transpose()?;
        if id.as_deref() == Some(self.inbox())
            && (properties.name != "INBOX"
                || properties.parent_id.is_some()
                || properties.role.as_deref() != Some("inbox"))
        {
            return Err(Error::Forbidden);
        }
        // Another root mailbox may not take INBOX's reserved name; INBOX itself
        // keeps its name, so updating INBOX never collides with its siblings.
        if self.mailboxes.iter().any(|m| {
            Some(&m.id) != id.as_ref()
                && m.parent_id == properties.parent_id
                && (m.name == properties.name
                    || (properties.parent_id.is_none()
                        && properties.name.eq_ignore_ascii_case("inbox")
                        && m.name.eq_ignore_ascii_case("inbox")))
        }) {
            return Err(Error::Conflict);
        }
        if let Some(role) = &properties.role {
            if ![
                "inbox",
                "archive",
                "drafts",
                "sent",
                "junk",
                "trash",
                "important",
            ]
            .contains(&role.as_str())
                && !(role.starts_with("x-")
                    && role.len() <= 255
                    && role.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
            {
                return Err(Error::Invalid);
            }
            if self
                .mailboxes
                .iter()
                .any(|m| Some(&m.id) != id.as_ref() && m.role.as_ref() == Some(role))
            {
                return Err(Error::Conflict);
            }
        }
        let mut parent = properties.parent_id.as_deref();
        let mut depth = 0;
        while let Some(parent_id) = parent {
            if id.as_deref() == Some(parent_id) || depth > self.mailboxes.len() {
                return Err(Error::Invalid);
            }
            parent = self
                .mailboxes
                .iter()
                .find(|m| m.id == parent_id)
                .ok_or(Error::NotFound)?
                .parent_id
                .as_deref();
            depth += 1;
        }
        let mut mailbox = match existing {
            Some(index) => self.mailboxes[index].clone(),
            None => Mailbox {
                // A new mailbox starts at the account's HIGHESTMODSEQ so an
                // empty selection reports the same watermark as its siblings.
                highest_modseq: self.mail_modseq,
                ..Mailbox::new(
                    &format!("m{revision}"),
                    "",
                    None,
                    revision
                        .checked_add(1)
                        .and_then(|n| u32::try_from(n).ok())
                        .ok_or(Error::OverQuota)?,
                )
            },
        };
        mailbox.name = properties.name;
        mailbox.parent_id = properties.parent_id;
        mailbox.role = properties.role;
        mailbox.sort_order = properties.sort_order;
        mailbox.is_subscribed = properties.is_subscribed;
        if let Some(index) = existing {
            self.mailboxes[index] = mailbox;
        } else {
            self.mailboxes.push(mailbox);
        }
        Ok(())
    }

    /// Emptiness comes from the counters; with `remove_emails` the working set
    /// must hold the mailbox's members (`Scope::Mailbox`).
    pub(super) fn delete_mailbox(&mut self, id: &str, remove_emails: bool) -> Result<(), Error> {
        if id == self.inbox() {
            return Err(Error::Forbidden);
        }
        let index = self
            .mailboxes
            .iter()
            .position(|m| m.id == id)
            .ok_or(Error::NotFound)?;
        if self
            .mailboxes
            .iter()
            .any(|m| m.parent_id.as_deref() == Some(id))
        {
            return Err(Error::Conflict);
        }
        if !remove_emails && self.mailboxes[index].total_emails > 0 {
            return Err(Error::Conflict);
        }
        if remove_emails {
            let mut position = 0;
            while position < self.messages.len() {
                let before = self.messages.len();
                if self.messages[position].mailboxes.contains_key(id) {
                    self.detach(position, id);
                }
                if self.messages.len() == before {
                    position += 1;
                }
            }
        }
        self.mailboxes.remove(index);
        Ok(())
    }

    pub fn mailbox_path(&self, id: &str) -> Option<String> {
        let mut parts = vec![];
        let mut current = Some(id);
        while let Some(id) = current {
            if parts.len() > self.mailboxes.len() {
                return None;
            }
            let mailbox = self.mailboxes.iter().find(|m| m.id == id)?;
            parts.push(mailbox.name.as_str());
            current = mailbox.parent_id.as_deref();
        }
        parts.reverse();
        Some(parts.join("/"))
    }
}
