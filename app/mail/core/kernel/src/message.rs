use super::{Account, Error, MAX_MESSAGE_BYTES, valid_keywords};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub id: String,
    /// Revision of the commit that last changed this record (RFC 7162 MODSEQ).
    #[serde(default)]
    pub modseq: u64,
    /// Revision of the commit that created this record.
    #[serde(default)]
    pub created_revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_identity: Option<String>,
    pub mailboxes: BTreeMap<String, u32>,
    pub size: usize,
    pub keywords: Vec<String>,
    #[serde(default)]
    pub received_at: i64,
}
impl Message {
    pub fn email_identity(&self) -> &str {
        self.email_identity.as_deref().unwrap_or(&self.id)
    }
    pub fn thread_identity(&self) -> &str {
        self.thread_identity.as_deref().unwrap_or(&self.id)
    }
    pub fn thread_id(&self) -> &str {
        self.thread.as_deref().unwrap_or(&self.id)
    }
    pub fn seen(&self) -> bool {
        self.keywords.iter().any(|k| k == "$seen")
    }
    pub fn state(&self) -> MessageState {
        MessageState {
            id: self.id.clone(),
            modseq: self.modseq,
            uids: self.mailboxes.clone(),
            thread: self.thread.clone(),
            mailboxes: self.mailboxes.keys().cloned().collect(),
            keywords: self.keywords.clone(),
            received_at: self.received_at,
        }
    }
    pub fn uid_in(&self, mailbox: &str) -> Option<u32> {
        self.mailboxes.get(mailbox).copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageState {
    pub id: String,
    #[serde(default)]
    pub modseq: u64,
    #[serde(default)]
    pub uids: BTreeMap<String, u32>,
    #[serde(default)]
    pub thread: Option<String>,
    pub mailboxes: Vec<String>,
    pub keywords: Vec<String>,
    pub received_at: i64,
}
impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("id", &self.id)
            .field("bytes", &self.size)
            .finish_non_exhaustive()
    }
}

impl Account {
    pub(super) fn index_of(&self, id: &str) -> Result<usize, Error> {
        self.messages
            .iter()
            .position(|m| m.id == id)
            .ok_or(Error::NotFound)
    }

    fn charge(&self, added: usize) -> Result<(), Error> {
        if self
            .used_bytes
            .checked_add(added)
            .is_none_or(|n| n > self.quota_bytes)
        {
            return Err(Error::OverQuota);
        }
        Ok(())
    }

    /// Allocate the next UID of every listed mailbox inside the mutation.
    /// Nothing is consumed when the batch fails to commit.
    fn allocate_uids(&mut self, ids: &[String]) -> Result<BTreeMap<String, u32>, Error> {
        let unique: BTreeSet<_> = ids.iter().collect();
        if unique.len() != ids.len() {
            return Err(Error::Invalid);
        }
        let mut result = BTreeMap::new();
        for id in ids {
            let folder = self
                .mailboxes
                .iter()
                .find(|m| m.id == *id)
                .ok_or(Error::NotFound)?;
            folder.uid_next.checked_add(1).ok_or(Error::OverQuota)?;
            result.insert(id.clone(), folder.uid_next);
        }
        for folder in &mut self.mailboxes {
            if result.contains_key(&folder.id) {
                folder.uid_next += 1;
            }
        }
        Ok(result)
    }

    /// Link a loaded message into mailboxes, keeping their counters current.
    pub fn attach(&mut self, index: usize, links: BTreeMap<String, u32>) {
        let (size, unseen) = {
            let message = &self.messages[index];
            (message.size, !message.seen())
        };
        for (mailbox, uid) in links {
            if let Some(folder) = self.mailboxes.iter_mut().find(|m| m.id == mailbox) {
                folder.total_emails += 1;
                folder.unread_emails += usize::from(unseen);
                folder.size_bytes += size;
            }
            self.messages[index].mailboxes.insert(mailbox, uid);
        }
    }

    /// Unlink; removing the last link deletes the record and its quota charge.
    pub(super) fn detach(&mut self, index: usize, mailbox: &str) {
        let (size, unseen) = {
            let message = &self.messages[index];
            (message.size, !message.seen())
        };
        if self.messages[index].mailboxes.remove(mailbox).is_some()
            && let Some(folder) = self.mailboxes.iter_mut().find(|m| m.id == mailbox)
        {
            folder.total_emails -= 1;
            folder.unread_emails -= usize::from(unseen);
            folder.size_bytes -= size;
        }
        if self.messages[index].mailboxes.is_empty() {
            self.messages.remove(index);
            self.used_bytes -= size;
        }
    }

    pub(super) fn transfer(
        &mut self,
        id: &str,
        mailbox: &str,
        remove_from: Option<&str>,
        revision: u64,
    ) -> Result<(), Error> {
        let index = self.index_of(id)?;
        let source = &self.messages[index];
        if remove_from.is_some_and(|id| !source.mailboxes.contains_key(id)) {
            return Err(Error::NotFound);
        }
        let removes_original = remove_from.is_some() && source.mailboxes.len() == 1;
        self.charge(if removes_original { 0 } else { source.size })?;
        let mut copy = source.clone();
        copy.email_identity = Some(source.email_identity().to_owned());
        copy.thread_identity = Some(source.thread_identity().to_owned());
        copy.thread = Some(source.thread_id().to_owned());
        copy.id = format!("e{revision}");
        copy.mailboxes = BTreeMap::new();
        let links = self.allocate_uids(&[mailbox.to_owned()])?;
        if let Some(folder) = remove_from {
            self.detach(index, folder);
        }
        self.used_bytes += copy.size;
        self.messages.push(copy);
        let last = self.messages.len() - 1;
        self.attach(last, links);
        Ok(())
    }

    pub(super) fn append(
        &mut self,
        mailboxes: Vec<String>,
        raw: Vec<u8>,
        mut keywords: Vec<String>,
        received_at: i64,
        revision: u64,
    ) -> Result<(), Error> {
        if mailboxes.is_empty() || !valid_keywords(&keywords) {
            return Err(Error::Invalid);
        }
        if raw.len() > MAX_MESSAGE_BYTES {
            return Err(Error::OverQuota);
        }
        self.charge(raw.len())?;
        let links = self.allocate_uids(&mailboxes)?;
        keywords.sort();
        keywords.dedup();
        self.used_bytes += raw.len();
        self.messages.push(Message {
            id: format!("e{revision}"),
            modseq: 0,
            created_revision: 0,
            thread: None,
            email_identity: None,
            thread_identity: None,
            mailboxes: BTreeMap::new(),
            size: raw.len(),
            keywords,
            received_at,
        });
        let last = self.messages.len() - 1;
        self.attach(last, links);
        Ok(())
    }

    pub(super) fn set_mailboxes(&mut self, id: &str, ids: &[String]) -> Result<(), Error> {
        if ids.is_empty() || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
            return Err(Error::Invalid);
        }
        let index = self.index_of(id)?;
        let existing = &self.messages[index].mailboxes;
        let additions: Vec<_> = ids
            .iter()
            .filter(|id| !existing.contains_key(*id))
            .cloned()
            .collect();
        let removals: Vec<_> = existing
            .keys()
            .filter(|id| !ids.contains(id))
            .cloned()
            .collect();
        let links = self.allocate_uids(&additions)?;
        self.attach(index, links);
        for mailbox in removals {
            self.detach(index, &mailbox);
        }
        Ok(())
    }
}
