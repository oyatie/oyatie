use super::{Account, Error, MAX_MESSAGE_BYTES, valid_keywords};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub id: String,
    #[serde(default = "super::modseq::initial")]
    pub modseq: u64,
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
    #[serde(default = "super::modseq::initial")]
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
    pub(super) fn transfer(
        &mut self,
        id: &str,
        mailbox: &str,
        remove_from: Option<&str>,
        revision: u64,
    ) -> Result<(), Error> {
        let source = self
            .messages
            .iter()
            .find(|m| m.id == id)
            .ok_or(Error::NotFound)?;
        if remove_from.is_some_and(|id| !source.mailboxes.contains_key(id)) {
            return Err(Error::NotFound);
        }
        let removes_original = remove_from.is_some() && source.mailboxes.len() == 1;
        let added = if removes_original { 0 } else { source.size };
        if self
            .messages
            .iter()
            .try_fold(added, |n, m| n.checked_add(m.size))
            .is_none_or(|size| size > self.quota_bytes)
        {
            return Err(Error::OverQuota);
        }
        let mut copy = source.clone();
        copy.email_identity = Some(source.email_identity().to_owned());
        copy.thread_identity = Some(source.thread_identity().to_owned());
        copy.thread = Some(source.thread_id().to_owned());
        copy.id = format!("e{revision}");
        copy.mailboxes = self.allocate_uids(&[mailbox.to_owned()])?;
        if removes_original {
            self.messages.retain(|m| m.id != id);
        } else if let Some(folder) = remove_from
            && let Some(source) = self.messages.iter_mut().find(|m| m.id == id)
        {
            source.mailboxes.remove(folder);
        }
        self.messages.push(copy);
        Ok(())
    }

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
        if raw.len() > MAX_MESSAGE_BYTES
            || self
                .messages
                .iter()
                .try_fold(raw.len(), |sum, m| sum.checked_add(m.size))
                .is_none_or(|n| n > self.quota_bytes)
        {
            return Err(Error::OverQuota);
        }
        let mailboxes = self.allocate_uids(&mailboxes)?;
        keywords.sort();
        keywords.dedup();
        self.messages.push(Message {
            id: format!("e{revision}"),
            modseq: super::modseq::initial(),
            thread: None,
            email_identity: None,
            thread_identity: None,
            mailboxes,
            size: raw.len(),
            keywords,
            received_at,
        });
        Ok(())
    }

    pub(super) fn set_mailboxes(&mut self, id: &str, ids: &[String]) -> Result<(), Error> {
        if ids.is_empty() || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
            return Err(Error::Invalid);
        }
        let index = self
            .messages
            .iter()
            .position(|m| m.id == id)
            .ok_or(Error::NotFound)?;
        let existing = &self.messages[index].mailboxes;
        let additions: Vec<_> = ids
            .iter()
            .filter(|id| !existing.contains_key(*id))
            .cloned()
            .collect();
        let mut next = existing
            .iter()
            .filter(|(id, _)| ids.contains(id))
            .map(|(id, uid)| (id.clone(), *uid))
            .collect::<BTreeMap<_, _>>();
        next.extend(self.allocate_uids(&additions)?);
        self.messages[index].mailboxes = next;
        Ok(())
    }
}
