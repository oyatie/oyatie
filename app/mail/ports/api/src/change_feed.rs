use crate::{Consumer, HistoryPage};
use mail_kernel::Error;

/// A consumer's position in one account's history: the revision it has
/// acknowledged. Absent ⇒ the consumer starts at the account's tail.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cursor(pub u64);

/// One `(consumer, tenant, account)` with unacknowledged changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dirty {
    pub tenant: String,
    pub account: String,
    /// Earliest UTC second the scheduler may hand it out again.
    pub not_before: i64,
}

/// Where a node resumes its round-robin over tenants for one consumer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Resume {
    pub after_tenant: Option<String>,
}

/// History after the cursor, or the reason it cannot be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedRead {
    Changes {
        cursor: Cursor,
        page: HistoryPage,
    },
    /// The cursor is below the account's `history_floor`: the consumer was
    /// disabled, compaction advanced, and it was re-enabled. Never acked;
    /// released only by `retire_cursor`.
    BelowFloor {
        cursor: Cursor,
        floor: u64,
    },
}

/// A durable operator action: reason, UTC time and `user@host`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditRow {
    pub id: u64,
    pub kind: String,
    pub reason: String,
    pub at_utc: String,
    pub operator: String,
    pub detail: String,
}

/// Replaces `Events`: cursor and dirty key per `(consumer, tenant, account)`,
/// the dirty key written in the change transaction and cleared only in a
/// transaction that reads the account's tail. Poison is retained, never
/// skipped or falsely acknowledged.
pub trait ChangeFeed: Send + Sync {
    /// Dirty accounts for `consumer`, round-robin over tenants from
    /// `resume` (advanced in place), at most `per_tenant` per tenant and
    /// `limit` in total; poisoned entries and `not_before > now` excluded.
    fn dirty(
        &self,
        consumer: Consumer,
        resume: &mut Resume,
        now: i64,
        per_tenant: usize,
        limit: usize,
    ) -> Result<Vec<Dirty>, Error>;
    /// Changes after the consumer's cursor for one account.
    fn changes(&self, consumer: Consumer, account: &str, limit: usize) -> Result<FeedRead, Error>;
    /// Advance the cursor to `revision` (never backwards); the dirty key is
    /// cleared iff `revision` is the account's tail, read in the same
    /// transaction, else `not_before` is set to `now + delay_secs`.
    fn acknowledge(
        &self,
        consumer: Consumer,
        account: &str,
        revision: u64,
        now: i64,
        delay_secs: i64,
    ) -> Result<(), Error>;
    /// Mark `(consumer, account)` poison with `reason`; excluded from `dirty`
    /// until `retire_cursor` or `dead_letter` releases it.
    fn poison(&self, consumer: Consumer, account: &str, reason: &str) -> Result<(), Error>;
    fn poisoned(&self, consumer: Consumer) -> Result<Vec<(Dirty, String)>, Error>;
    /// Cursors of every enabled consumer for `account`, the compaction bound.
    fn cursors(&self, account: &str, enabled: &[Consumer]) -> Result<Vec<(Consumer, u64)>, Error>;

    /// Operator actions, each an audited durable row.
    /// Re-mark accounts whose cursor is behind their tail; returns how many.
    fn reconcile(&self, consumer: Consumer, operator: &str, reason: &str) -> Result<u64, Error>;
    /// Record a poisoned change as dead-lettered and release the account.
    fn dead_letter(
        &self,
        consumer: Consumer,
        account: &str,
        revision: u64,
        operator: &str,
        reason: &str,
    ) -> Result<(), Error>;
    /// Delete `consumer`'s cursors (all tenants or one), so absent ⇒ tail
    /// applies and every account it held back is released.
    fn retire_cursor(
        &self,
        consumer: Consumer,
        tenant: Option<&str>,
        operator: &str,
        reason: &str,
    ) -> Result<u64, Error>;
    fn audit(&self, limit: usize) -> Result<Vec<AuditRow>, Error>;
}
