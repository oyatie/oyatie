#![forbid(unsafe_code)]

use serde::Serialize;

pub const MAX_MESSAGE_BYTES: usize = 25 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    NotFound,
    Forbidden,
    Conflict,
    OverQuota,
    Unavailable,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mail operation failed: {self:?}")
    }
}
impl std::error::Error for Error {}

mod identity;
pub use identity::{IdentityAddress, IdentitySettings};
mod vacation;
mod vacation_reply;
pub use vacation::VacationSettings;
pub use vacation_reply::VacationReply;
mod submission;
pub use submission::{
    EnvelopeAddress, SubmissionDelivered, SubmissionDeliveryStatus, SubmissionEnvelope,
    SubmissionFilter, SubmissionQuery, SubmissionRecord, SubmissionSortField, UndoStatus,
};

mod mailbox;
pub use mailbox::{Mailbox, MailboxProperties, MailboxState};

mod message;
mod modseq;
pub use message::{Message, MessageState};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Account {
    pub id: String,
    pub tenant: String,
    pub owner: String,
    pub address: String,
    pub revision: u64,
    pub mail_modseq: u64,
    pub identity: IdentitySettings,
    pub identity_revision: u64,
    pub vacation: VacationSettings,
    pub vacation_revision: u64,
    pub quota_bytes: usize,
    pub mailboxes: Vec<Mailbox>,
    pub messages: Vec<Message>,
}

pub enum Command {
    SetIdentity {
        settings: IdentitySettings,
    },
    SetVacation {
        settings: VacationSettings,
    },
    SetMailbox {
        id: Option<String>,
        properties: MailboxProperties,
    },
    RemoveMailbox {
        id: String,
        remove_emails: bool,
    },
    CreateMailbox {
        name: String,
    },
    RenameMailbox {
        id: String,
        name: String,
    },
    DeleteMailbox {
        id: String,
    },
    Append {
        mailboxes: Vec<String>,
        received_at: i64,
        raw: Vec<u8>,
        keywords: Vec<String>,
    },
    Transfer {
        id: String,
        mailbox: String,
        remove_from: Option<String>,
    },
    SetMailboxes {
        id: String,
        mailboxes: Vec<String>,
    },
    Keywords {
        id: String,
        keywords: Vec<String>,
    },
    Destroy {
        id: String,
    },
    Expunge {
        mailbox: String,
    },
}

pub fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.:@".contains(&b))
}

pub fn valid_address(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && local.len() <= 64
        && domain.len() <= 253
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
        && local
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_`{|}~".contains(&b))
        && domain.contains('.')
        && domain.split('.').all(|part| {
            !part.is_empty()
                && part.len() <= 63
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

fn valid_name(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 255 && !value.chars().any(char::is_control)
}

pub fn valid_keywords(values: &[String]) -> bool {
    values.len() <= 100
        && values.iter().all(|v| {
            !v.is_empty()
                && v.len() <= 255
                && v.bytes()
                    .all(|b| b.is_ascii_graphic() && !b"(){%*\"\\]".contains(&b))
        })
}

impl Account {
    pub fn new(id: &str, tenant: &str, owner: &str, address: &str) -> Result<Self, Error> {
        if ![id, tenant, owner].iter().all(|v| valid_identifier(v)) || !valid_address(address) {
            return Err(Error::Invalid);
        }
        Ok(Self {
            id: id.into(),
            tenant: tenant.into(),
            owner: owner.into(),
            address: address.into(),
            revision: 0,
            mail_modseq: 0,
            identity: IdentitySettings::default(),
            identity_revision: 0,
            vacation: VacationSettings::default(),
            vacation_revision: 0,
            quota_bytes: 1024 * 1024 * 1024,
            mailboxes: vec![Mailbox {
                id: "inbox".into(),
                name: "INBOX".into(),
                role: Some("inbox".into()),
                parent_id: None,
                sort_order: 0,
                is_subscribed: true,
                uid_next: 1,
                uid_validity: 1,
            }],
            messages: vec![],
        })
    }

    pub fn inbox(&self) -> &str {
        "inbox"
    }

    /// A failed command leaves the account unchanged. Persistent adapters commit
    /// the resulting revision and its event in the same transaction.
    pub fn apply(&mut self, command: Command) -> Result<(), Error> {
        let next = self.revision.checked_add(1).ok_or(Error::OverQuota)?;
        let change = self.prepare_modseq(&command, next)?;
        match command {
            Command::SetIdentity { settings } => self.set_identity(settings)?,
            Command::SetVacation { settings } => self.set_vacation(settings)?,
            Command::SetMailbox { id, properties } => self.set_mailbox(id, properties, next)?,
            Command::RemoveMailbox { id, remove_emails } => {
                self.delete_mailbox(&id, remove_emails)?
            }
            Command::CreateMailbox { name } => {
                self.set_mailbox(None, MailboxProperties::named(name), next)?;
            }
            Command::RenameMailbox { id, name } => {
                if id == self.inbox() {
                    return Err(Error::Forbidden);
                }
                let mut properties = self
                    .mailboxes
                    .iter()
                    .find(|m| m.id == id)
                    .ok_or(Error::NotFound)?
                    .properties();
                properties.name = name;
                self.set_mailbox(Some(id), properties, next)?;
            }
            Command::DeleteMailbox { id } => {
                self.delete_mailbox(&id, false)?;
            }
            Command::Append {
                mailboxes,
                raw,
                keywords,
                received_at,
            } => {
                self.append(mailboxes, raw, keywords, received_at, next)?;
            }
            Command::Transfer {
                id,
                mailbox,
                remove_from,
            } => {
                self.transfer(&id, &mailbox, remove_from.as_deref(), next)?;
            }
            Command::SetMailboxes { id, mailboxes } => {
                self.set_mailboxes(&id, &mailboxes)?;
            }
            Command::Keywords { id, mut keywords } => {
                if !valid_keywords(&keywords) {
                    return Err(Error::Invalid);
                }
                let message = self
                    .messages
                    .iter_mut()
                    .find(|m| m.id == id)
                    .ok_or(Error::NotFound)?;
                keywords.sort();
                keywords.dedup();
                message.keywords = keywords;
            }
            Command::Destroy { id } => {
                let index = self
                    .messages
                    .iter()
                    .position(|m| m.id == id)
                    .ok_or(Error::NotFound)?;
                self.messages.remove(index);
            }
            Command::Expunge { mailbox } => {
                if !self.mailboxes.iter().any(|m| m.id == mailbox) {
                    return Err(Error::NotFound);
                }
                for message in &mut self.messages {
                    if message.keywords.iter().any(|k| k == "$deleted") {
                        message.mailboxes.remove(&mailbox);
                    }
                }
                self.messages.retain(|m| !m.mailboxes.is_empty());
            }
        }
        self.commit_modseq(change);
        self.revision = next;
        Ok(())
    }
}
