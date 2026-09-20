#![forbid(unsafe_code)]

use mail_kernel::{Account, Command, Error, HistoryEntry};
mod blob_store;
mod change_feed;
mod queue;
mod submission;
mod submission_store;
pub use blob_store::{BlobStore, COMMAND_RESERVATION_SECS, UPLOAD_RESERVATION_SECS};
pub use change_feed::{AuditRow, ChangeFeed, Cursor, Dirty, FeedRead, Resume};
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

/// One key-only range read over a mailbox: `(uid, id)` in UID order plus the
/// mailbox record's counters and watermarks at the same snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MailboxSelection {
    pub revision: u64,
    pub uid_validity: u32,
    pub uid_next: u32,
    pub highest_modseq: u64,
    pub uids: Vec<(u32, String)>,
}

/// How a mutation relates to the revision its caller observed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precondition {
    /// Re-apply on the current state when another writer got there first;
    /// a re-applied removal of a record that no longer exists is a no-op.
    Observed(u64),
    /// Client-conditional (`UNCHANGEDSINCE`, `ifInState`): `Conflict` on mismatch.
    Require(u64),
}

/// Result of one committed batch: the new revision, created record ids in
/// creation order and `(mailbox, uid)` allocations in append order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Execution {
    pub revision: u64,
    pub ids: Vec<String>,
    pub allocations: Vec<(String, u32)>,
}

/// History rows in `(since, revision]`, whole revisions only. When `since`
/// is below `floor` the rows are empty and callers fall back per protocol
/// (RFC 7162 §3.2.5.2; JMAP `cannotCalculateChanges`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryPage {
    pub since: u64,
    pub revision: u64,
    pub floor: u64,
    pub has_more: bool,
    pub rows: Vec<(u64, HistoryEntry)>,
}
impl HistoryPage {
    pub fn below_floor(&self) -> bool {
        self.since < self.floor
    }
}

/// Before/after pair of one record inside one committed revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change<T> {
    pub revision: u64,
    pub id: String,
    pub before: Option<T>,
    pub after: Option<T>,
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

/// Account, mailbox and message records with per-account revisions. The
/// kernel owns every rule; the adapter loads the bounded working set a batch
/// needs, persists its effects in one transaction and allocates under them.
pub trait MetadataStore: Send + Sync {
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error>;
    /// Full projection: account, mailbox and every message record. Bodies are
    /// fetched independently through `blob`.
    fn account(&self, id: &str) -> Result<Account, Error>;
    /// At most 256 message records and the account revision in one snapshot.
    /// Missing ids are omitted; cost is bounded by the selection, not the account.
    fn messages(&self, account: &str, ids: &[String]) -> Result<MessageSelection, Error>;
    /// Sequence↔UID map of one mailbox from a single key-only range read.
    fn mailbox_uids(&self, account: &str, mailbox: &str) -> Result<MailboxSelection, Error>;
    fn resolve(&self, address: &str) -> Result<String, Error>;
    /// One atomic batch. Conflict retry is the adapter's; only `Require`
    /// surfaces `Conflict`. `Busy` is retryable by the caller within its deadline.
    fn execute(
        &self,
        id: &str,
        precondition: Precondition,
        commands: Vec<Command>,
    ) -> Result<Execution, Error>;
    /// A delivery key is bound to the content and timestamp. Replaying a
    /// committed key does not append again, even after the email was deleted.
    /// An exact Message-ID/References match already in the target mailbox or
    /// Junk is suppressed as a duplicate.
    fn deliver_once(
        &self,
        account: &str,
        key: &str,
        raw: &[u8],
        received_at: i64,
    ) -> Result<(), Error>;
    /// JMAP upload: the body persists under a 24 h upload reservation and is
    /// addressed as `b<hash>`; `blob` resolves that id or a message id to the
    /// version this account links or holds.
    fn put_blob(&self, account: &str, raw: &[u8]) -> Result<String, Error>;
    fn blob(&self, account: &str, id: &str) -> Result<Vec<u8>, Error>;
    /// History rows after `since`, at most `limit` rows without splitting a
    /// revision. Cost is proportional to the changes since, never the account.
    fn history(&self, account: &str, since: u64, limit: usize) -> Result<HistoryPage, Error>;
    /// Advance `history_floor` under the policy, never above the lowest cursor
    /// of an enabled consumer; a held account is `retention-blocked`.
    fn compact_history(
        &self,
        account: &str,
        now: i64,
        policy: mail_kernel::RetentionPolicy,
        cursors: &[(Consumer, u64)],
    ) -> Result<mail_kernel::Retention, Error>;
}

/// `ChangeFeed` consumers are compiled in; a cell enables a subset by
/// configuration. An enabled consumer's cursor bounds history compaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Consumer {
    FoundryRecords,
}
impl Consumer {
    pub const ALL: [Consumer; 1] = [Consumer::FoundryRecords];
    pub fn name(self) -> &'static str {
        match self {
            Self::FoundryRecords => "foundry-records",
        }
    }
}

pub trait Store: MetadataStore + SubmissionStore + BlobStore + ChangeFeed {}
impl<T: MetadataStore + SubmissionStore + BlobStore + ChangeFeed + ?Sized> Store for T {}
