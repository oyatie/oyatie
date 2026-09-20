//! Test doubles for the contract suite: wrappers over any `MetadataStore +
//! SubmissionStore`, a row-read counter, and a builder for `41ce37e61`
//! databases. Compiled into this crate's own tests through the `contract`
//! feature and a self dev-dependency, so the merge verdict runs the gates.
//!
//! Row reads are counted per thread: every store operation runs
//! synchronously on the caller's thread under the connection mutex, so a
//! test's counter is unaffected by other tests in the same process.

/// Forward every `MetadataStore` method to `self.inner` after `self.before`.
macro_rules! metadata_store {
    ($ty:ident) => {
        impl<T: MetadataStore + SubmissionStore + BlobStore + ChangeFeed> MetadataStore for $ty<T> {
            fn account_info(&self, id: &str) -> Result<AccountInfo, Error> {
                self.before("account_info")?;
                self.inner.account_info(id)
            }
            fn account(&self, id: &str) -> Result<Account, Error> {
                self.before("account")?;
                self.inner.account(id)
            }
            fn messages(&self, account: &str, ids: &[String]) -> Result<MessageSelection, Error> {
                self.before("messages")?;
                self.inner.messages(account, ids)
            }
            fn mailbox_uids(
                &self,
                account: &str,
                mailbox: &str,
            ) -> Result<MailboxSelection, Error> {
                self.before("mailbox_uids")?;
                self.inner.mailbox_uids(account, mailbox)
            }
            fn resolve(&self, address: &str) -> Result<String, Error> {
                self.before("resolve")?;
                self.inner.resolve(address)
            }
            fn execute(
                &self,
                id: &str,
                precondition: Precondition,
                commands: Vec<Command>,
            ) -> Result<Execution, Error> {
                self.before("execute")?;
                self.inner.execute(id, precondition, commands)
            }
            fn deliver_once(
                &self,
                account: &str,
                key: &str,
                raw: &[u8],
                received_at: i64,
            ) -> Result<(), Error> {
                self.before("deliver_once")?;
                self.inner.deliver_once(account, key, raw, received_at)
            }
            fn put_blob(&self, account: &str, raw: &[u8]) -> Result<String, Error> {
                self.before("put_blob")?;
                self.inner.put_blob(account, raw)
            }
            fn blob(&self, account: &str, id: &str) -> Result<Vec<u8>, Error> {
                self.before("blob")?;
                self.inner.blob(account, id)
            }
            fn history(
                &self,
                account: &str,
                since: u64,
                limit: usize,
            ) -> Result<HistoryPage, Error> {
                self.before("history")?;
                self.inner.history(account, since, limit)
            }
            fn compact_history(
                &self,
                account: &str,
                now: i64,
                policy: RetentionPolicy,
                cursors: &[(Consumer, u64)],
            ) -> Result<Retention, Error> {
                self.before("compact_history")?;
                self.inner.compact_history(account, now, policy, cursors)
            }
        }
    };
}

/// Forward every `SubmissionStore` method to `self.inner` after `self.before`.
macro_rules! submission_store {
    ($ty:ident) => {
        impl<T: MetadataStore + SubmissionStore + BlobStore + ChangeFeed> SubmissionStore
            for $ty<T>
        {
            fn submissions(
                &self,
                account: &str,
                ids: Option<&[String]>,
            ) -> Result<SubmissionSelection, Error> {
                self.before("submissions")?;
                self.inner.submissions(account, ids)
            }
            fn accept_submission(
                &self,
                account: &str,
                revision: u64,
                acceptance: SubmissionAcceptance,
            ) -> Result<SubmissionSelection, Error> {
                self.before("accept_submission")?;
                self.inner.accept_submission(account, revision, acceptance)
            }
            fn cancel_submission(
                &self,
                account: &str,
                revision: u64,
                id: &str,
            ) -> Result<SubmissionSelection, SubmissionFailure> {
                self.before("cancel_submission")?;
                self.inner.cancel_submission(account, revision, id)
            }
            fn destroy_submission(
                &self,
                account: &str,
                revision: u64,
                id: &str,
            ) -> Result<u64, Error> {
                self.before("destroy_submission")?;
                self.inner.destroy_submission(account, revision, id)
            }
            fn query_submissions(
                &self,
                account: &str,
                revision: Option<u64>,
                query: &SubmissionQuery,
            ) -> Result<SubmissionPage, SubmissionFailure> {
                self.before("query_submissions")?;
                self.inner.query_submissions(account, revision, query)
            }
            fn submission_changes(
                &self,
                account: &str,
                since: u64,
                limit: usize,
            ) -> Result<SubmissionChanges, Error> {
                self.before("submission_changes")?;
                self.inner.submission_changes(account, since, limit)
            }
        }
    };
}

/// Forward every `BlobStore` method to `self.inner` after `self.before`.
macro_rules! blob_store {
    ($ty:ident) => {
        impl<T: MetadataStore + SubmissionStore + BlobStore + ChangeFeed> BlobStore for $ty<T> {
            fn persist(
                &self,
                account: &str,
                scope: &str,
                raw: &[u8],
                ttl_secs: i64,
            ) -> Result<BlobRef, Error> {
                self.before("persist")?;
                self.inner.persist(account, scope, raw, ttl_secs)
            }
            fn renew(&self, account: &str, scope: &str, ttl_secs: i64) -> Result<(), Error> {
                self.before("renew")?;
                self.inner.renew(account, scope, ttl_secs)
            }
            fn read(&self, account: &str, blob: &BlobRef) -> Result<Vec<u8>, Error> {
                self.before("read")?;
                self.inner.read(account, blob)
            }
            fn orphan_sweep(&self, now: i64, limit: usize) -> Result<usize, Error> {
                self.before("orphan_sweep")?;
                self.inner.orphan_sweep(now, limit)
            }
        }
    };
}

/// Forward every `ChangeFeed` method to `self.inner` after `self.before`.
macro_rules! change_feed {
    ($ty:ident) => {
        impl<T: MetadataStore + SubmissionStore + BlobStore + ChangeFeed> ChangeFeed for $ty<T> {
            fn dirty(
                &self,
                c: Consumer,
                r: &mut Resume,
                now: i64,
                per: usize,
                limit: usize,
            ) -> Result<Vec<Dirty>, Error> {
                self.before("dirty")?;
                self.inner.dirty(c, r, now, per, limit)
            }
            fn changes(&self, c: Consumer, a: &str, limit: usize) -> Result<FeedRead, Error> {
                self.before("changes")?;
                self.inner.changes(c, a, limit)
            }
            fn acknowledge(
                &self,
                c: Consumer,
                a: &str,
                rev: u64,
                now: i64,
                delay: i64,
            ) -> Result<(), Error> {
                self.before("acknowledge")?;
                self.inner.acknowledge(c, a, rev, now, delay)
            }
            fn poison(&self, c: Consumer, a: &str, reason: &str) -> Result<(), Error> {
                self.before("poison")?;
                self.inner.poison(c, a, reason)
            }
            fn poisoned(&self, c: Consumer) -> Result<Vec<(Dirty, String)>, Error> {
                self.before("poisoned")?;
                self.inner.poisoned(c)
            }
            fn cursors(&self, a: &str, e: &[Consumer]) -> Result<Vec<(Consumer, u64)>, Error> {
                self.before("cursors")?;
                self.inner.cursors(a, e)
            }
            fn reconcile(&self, c: Consumer, op: &str, reason: &str) -> Result<u64, Error> {
                self.before("reconcile")?;
                self.inner.reconcile(c, op, reason)
            }
            fn dead_letter(
                &self,
                c: Consumer,
                a: &str,
                rev: u64,
                op: &str,
                reason: &str,
            ) -> Result<(), Error> {
                self.before("dead_letter")?;
                self.inner.dead_letter(c, a, rev, op, reason)
            }
            fn retire_cursor(
                &self,
                c: Consumer,
                t: Option<&str>,
                op: &str,
                reason: &str,
            ) -> Result<u64, Error> {
                self.before("retire_cursor")?;
                self.inner.retire_cursor(c, t, op, reason)
            }
            fn audit(&self, limit: usize) -> Result<Vec<AuditRow>, Error> {
                self.before("audit")?;
                self.inner.audit(limit)
            }
        }
    };
}

mod broken;
pub mod converted;
mod counting;
mod faulty;
pub mod legacy;
mod legacy_ddl;
mod legacy_rows;

pub use broken::Broken;
pub use converted::converted_store;
pub use counting::Counting;
pub use faulty::Faulty;
pub use legacy_ddl::{
    LEGACY_ACCESS, LEGACY_DDL, LEGACY_EVENT_CURSORS, LEGACY_EVENTS_COLUMN, LEGACY_QUEUE,
    LEGACY_SUBMISSION, LEGACY_THREADS,
};

use std::cell::Cell;

thread_local! {
    static ROWS: Cell<u64> = const { Cell::new(0) };
}

pub(crate) fn count_row() {
    ROWS.with(|rows| rows.set(rows.get() + 1));
}

/// Record rows (account, mailbox, message, link, uid, history) this thread's
/// store operations mapped since `reset_rows`.
pub fn rows_read() -> u64 {
    ROWS.with(Cell::get)
}

pub fn reset_rows() {
    ROWS.with(|rows| rows.set(0));
}
