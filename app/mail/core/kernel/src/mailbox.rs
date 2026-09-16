use super::{Account, Error, valid_name};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    // Older journal snapshots do not carry thread counters; preserve that absence.
    #[serde(default)]
    pub total_threads: Option<usize>,
    #[serde(default)]
    pub unread_threads: Option<usize>,
}

impl Account {
    pub fn mailbox_states(&self) -> Vec<MailboxState> {
        let mut counts = BTreeMap::new();
        for message in &self.messages {
            let unread = !message.keywords.iter().any(|k| k == "$seen");
            for id in message.mailboxes.keys() {
                let (total, unseen, threads, unread_threads) = counts
                    .entry(id.as_str())
                    .or_insert((0, 0, BTreeSet::new(), BTreeSet::new()));
                *total += 1;
                *unseen += usize::from(unread);
                threads.insert(message.thread_id());
                if unread {
                    unread_threads.insert(message.thread_id());
                }
            }
        }
        self.mailboxes
            .iter()
            .map(|m| {
                let (total_emails, unread_emails, total_threads, unread_threads) = counts
                    .get(m.id.as_str())
                    .map(|(total, unread, threads, unread_threads)| {
                        (*total, *unread, threads.len(), unread_threads.len())
                    })
                    .unwrap_or_default();
                MailboxState {
                    mailbox: m.clone(),
                    total_emails,
                    unread_emails,
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
        if self.mailboxes.iter().any(|m| {
            Some(&m.id) != id.as_ref()
                && m.parent_id == properties.parent_id
                && (m.name == properties.name
                    || (properties.parent_id.is_none()
                        && properties.name.eq_ignore_ascii_case("inbox")))
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
        let (uid_next, uid_validity) = match existing {
            Some(index) => (
                self.mailboxes[index].uid_next,
                self.mailboxes[index].uid_validity,
            ),
            None => (
                1,
                revision
                    .checked_add(1)
                    .and_then(|n| u32::try_from(n).ok())
                    .ok_or(Error::OverQuota)?,
            ),
        };
        let mailbox = Mailbox {
            id: id.unwrap_or_else(|| format!("m{revision}")),
            name: properties.name,
            parent_id: properties.parent_id,
            role: properties.role,
            sort_order: properties.sort_order,
            is_subscribed: properties.is_subscribed,
            uid_next,
            uid_validity,
        };
        if let Some(index) = existing {
            self.mailboxes[index] = mailbox;
        } else {
            self.mailboxes.push(mailbox);
        }
        Ok(())
    }

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
        if !remove_emails && self.messages.iter().any(|m| m.mailboxes.contains_key(id)) {
            return Err(Error::Conflict);
        }
        if remove_emails {
            for message in &mut self.messages {
                message.mailboxes.remove(id);
            }
            self.messages.retain(|m| !m.mailboxes.is_empty());
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
