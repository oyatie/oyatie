//! A store that deliberately violates the `MetadataStore` contract, so the
//! suite is shown to go red — on gates 2 through 6, and no other:
//!
//! - `execute` downgrades `Precondition::Require` to `Observed`: a stale
//!   client-conditional batch is re-applied instead of returning `Conflict`.
//! - `history` reports `floor: 0`, so a page below the real floor looks
//!   complete instead of telling the caller to fall back.
//! - `messages` loads the whole account to answer a selection: the right
//!   records, at a cost proportional to the account.
//! - `mailbox_uids` likewise derives the UID map from the full projection.
//! - `compact_history` ignores consumer cursors: never `Retention::Blocked`.
//! - `deliver_once` ignores receipts: a replayed delivery lands twice.
//! - `dirty` ignores the resume position; `orphan_sweep` sweeps nothing
//!   (`broken_ports.rs`).
use mail_api::{
    AccountInfo, BlobStore, ChangeFeed, Consumer, Execution, HistoryPage, MailboxSelection,
    MessageSelection, MetadataStore, Precondition, SubmissionAcceptance, SubmissionChanges,
    SubmissionFailure, SubmissionPage, SubmissionSelection, SubmissionStore,
};
use mail_kernel::{Account, Command, Error, Retention, RetentionPolicy, SubmissionQuery};

pub struct Broken<T> {
    pub(super) inner: T,
    replays: std::sync::atomic::AtomicU64,
}

impl<T> Broken<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            replays: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    fn before(&self, _method: &'static str) -> Result<(), Error> {
        Ok(())
    }
}

impl<T: MetadataStore + SubmissionStore + BlobStore + ChangeFeed> MetadataStore for Broken<T> {
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error> {
        self.inner.account_info(id)
    }
    fn account(&self, id: &str) -> Result<Account, Error> {
        self.inner.account(id)
    }
    fn messages(&self, account: &str, ids: &[String]) -> Result<MessageSelection, Error> {
        // The whole account is read (and counted); the right ids come back.
        let account = self.inner.account(account)?;
        let mut messages = account.messages;
        messages.retain(|m| ids.contains(&m.id));
        Ok(MessageSelection {
            revision: account.revision,
            messages,
        })
    }
    fn mailbox_uids(&self, account: &str, mailbox: &str) -> Result<MailboxSelection, Error> {
        let account = self.inner.account(account)?;
        let record = account
            .mailboxes
            .iter()
            .find(|m| m.id == mailbox)
            .ok_or(Error::NotFound)?;
        let mut uids: Vec<(u32, String)> = account
            .messages
            .iter()
            .filter_map(|m| m.uid_in(mailbox).map(|uid| (uid, m.id.clone())))
            .collect();
        uids.sort();
        Ok(MailboxSelection {
            revision: account.revision,
            uid_validity: record.uid_validity,
            uid_next: record.uid_next,
            highest_modseq: record.highest_modseq,
            uids,
        })
    }
    fn resolve(&self, address: &str) -> Result<String, Error> {
        self.inner.resolve(address)
    }
    fn execute(
        &self,
        id: &str,
        precondition: Precondition,
        commands: Vec<Command>,
    ) -> Result<Execution, Error> {
        let precondition = match precondition {
            Precondition::Require(revision) => Precondition::Observed(revision),
            observed => observed,
        };
        self.inner.execute(id, precondition, commands)
    }
    fn deliver_once(
        &self,
        account: &str,
        key: &str,
        raw: &[u8],
        received_at: i64,
    ) -> Result<(), Error> {
        // A fresh key every time: the receipt never matches, so a replay
        // delivers the message again.
        let n = self
            .replays
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.inner
            .deliver_once(account, &format!("{key}#{n}"), raw, received_at)
    }
    fn put_blob(&self, account: &str, raw: &[u8]) -> Result<String, Error> {
        self.inner.put_blob(account, raw)
    }
    fn blob(&self, account: &str, id: &str) -> Result<Vec<u8>, Error> {
        self.inner.blob(account, id)
    }
    fn history(&self, account: &str, since: u64, limit: usize) -> Result<HistoryPage, Error> {
        let mut page = self.inner.history(account, since, limit)?;
        page.floor = 0;
        Ok(page)
    }
    fn compact_history(
        &self,
        account: &str,
        now: i64,
        policy: RetentionPolicy,
        _cursors: &[(Consumer, u64)],
    ) -> Result<Retention, Error> {
        self.inner.compact_history(account, now, policy, &[])
    }
}

submission_store!(Broken);
delivery_queue!(Broken);
submission_queue!(Broken);
