use super::Command;

/// The message records a command needs loaded into the working set. Every
/// other rule (quota, counters, UID allocation) reads the account and mailbox
/// records, so a mutation never loads the whole account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope<'a> {
    None,
    Message(&'a str),
    /// Every record linked to the mailbox.
    Mailbox(&'a str),
    /// Records linked to the mailbox that carry the keyword.
    Flagged(&'a str, &'static str),
}

impl Command {
    pub fn scope(&self) -> Scope<'_> {
        match self {
            Command::Transfer { id, .. }
            | Command::SetMailboxes { id, .. }
            | Command::Keywords { id, .. }
            | Command::Destroy { id } => Scope::Message(id),
            Command::Expunge { mailbox } => Scope::Flagged(mailbox, "$deleted"),
            Command::RemoveMailbox {
                id,
                remove_emails: true,
            } => Scope::Mailbox(id),
            _ => Scope::None,
        }
    }
}
