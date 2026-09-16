#![forbid(unsafe_code)]

use mail_kernel::{Account, Command, Error};
mod queue;
mod submission;
mod submission_store;
pub use queue::{DeliveryFailure, DeliveryLease, DeliveryQueue, DeliveryTarget, QueuedMessage};
pub use submission::{DeliveryOutcome, MailTransport, OutboundLease, SubmissionQueue};
pub use submission_store::{
    SubmissionAcceptance, SubmissionChanges, SubmissionFailure, SubmissionPage,
    SubmissionSelection, SubmissionStore,
};

/// Bounded account context. Authorization never requires mailbox contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountInfo {
    pub id: String,
    pub tenant: String,
    pub owner: String,
    pub address: String,
    pub quota_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageSelection {
    pub revision: u64,
    pub messages: Vec<mail_kernel::Message>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Principal {
    pub tenant: String,
    pub subject: String,
    pub account: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Read,
    Write,
    Deliver,
    Submit,
}

/// Authentication adapters bind identity from verified credentials, never from
/// caller-supplied tenant or account identifiers.
pub trait Identity: Send + Sync {
    fn authenticate(&self, token: &str) -> Result<Principal, Error>;
}

/// Refusal and unavailability both prevent access. There is no implicit allow.
pub trait Policy: Send + Sync {
    fn authorize(
        &self,
        principal: &Principal,
        action: Action,
        account: &AccountInfo,
    ) -> Result<(), Error>;
}

/// Atomic revision checking prevents concurrent protocol sessions losing writes.
/// Each account commits independently with its event outbox.
pub trait Store: SubmissionStore {
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error>;
    /// Metadata only; message content is fetched independently through `blob`.
    fn account(&self, id: &str) -> Result<Account, Error>;
    /// Read at most 256 selected message metadata records and their account
    /// revision in one consistent snapshot. Missing IDs are omitted.
    fn messages(&self, account: &str, ids: &[String]) -> Result<MessageSelection, Error>;
    fn resolve(&self, address: &str) -> Result<String, Error>;
    fn execute(&self, id: &str, revision: u64, commands: Vec<Command>) -> Result<Account, Error>;
    /// A delivery key is bound to the content and timestamp. Replaying a committed
    /// key does not append again, even after the original email was deleted.
    fn deliver_once(
        &self,
        account: &str,
        key: &str,
        raw: &[u8],
        received_at: i64,
    ) -> Result<(), Error>;
    /// Temporary blobs are account-scoped, quota-limited, and retained for at
    /// least 24 hours after upload or copy. Message blobs live with the message.
    fn put_blob(&self, account: &str, raw: &[u8]) -> Result<String, Error>;
    fn blob(&self, account: &str, id: &str) -> Result<Vec<u8>, Error>;
    /// Returns only committed metadata changes in (since, until]. A gap or an
    /// unrecognized state is Conflict, never a successful incomplete history.
    fn message_changes(
        &self,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<MessageChange>, Error>;
    /// Committed message changes after a numeric revision threshold. Unlike
    /// exact-state changes, the threshold may fall inside an atomic batch;
    /// that batch is included whole. Missing retained history is Conflict.
    fn message_changes_after(
        &self,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<MessageChange>, Error>;
    fn mailbox_changes(
        &self,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<MailboxChange>, Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change<T> {
    pub revision: u64,
    pub id: String,
    pub before: Option<T>,
    pub after: Option<T>,
}
pub type MessageChange = Change<mail_kernel::MessageState>;
pub type MailboxChange = Change<mail_kernel::MailboxState>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub sequence: u64,
    pub tenant: String,
    pub account: String,
    pub revision: u64,
    pub observed_at_ms: u64,
}

/// An adapter publishes durable events with (account, revision) as the stable
/// deduplication key. No message content or credential belongs in this event.
pub trait Events: Send + Sync {
    fn pending(&self, consumer: &str, limit: usize) -> Result<Vec<Event>, Error>;
    /// Advance only across the next event. Repeated acknowledgements are safe;
    /// a consumer cannot skip an event or change another consumer's position.
    fn acknowledge(&self, consumer: &str, sequence: u64) -> Result<(), Error>;
}
