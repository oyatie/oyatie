//! Counts store operations by method name and record rows read, so a gate
//! can assert that an operation's cost is bounded by its selection and not by
//! the account.
use super::{reset_rows, rows_read};
use mail_api::{
    AccountInfo, AuditRow, BlobStore, ChangeFeed, Consumer, Dirty, Execution, FeedRead,
    HistoryPage, MailboxSelection, MessageSelection, MetadataStore, Precondition, Resume,
    SubmissionAcceptance, SubmissionChanges, SubmissionFailure, SubmissionPage,
    SubmissionSelection, SubmissionStore,
};
use mail_kernel::{Account, BlobRef, Command, Error, Retention, RetentionPolicy, SubmissionQuery};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub struct Counting<T> {
    inner: T,
    calls: Mutex<BTreeMap<&'static str, u64>>,
}

impl<T> Counting<T> {
    /// Wrap a store; the row counter of the calling thread is reset so
    /// `reads` starts from zero.
    pub fn new(inner: T) -> Self {
        reset_rows();
        Self {
            inner,
            calls: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Calls of one trait method since the last `reset`.
    pub fn calls(&self, method: &str) -> u64 {
        self.calls
            .lock()
            .map(|calls| calls.get(method).copied().unwrap_or(0))
            .unwrap_or(0)
    }

    /// Calls of every trait method since the last `reset`.
    pub fn total_calls(&self) -> u64 {
        self.calls
            .lock()
            .map(|calls| calls.values().sum())
            .unwrap_or(0)
    }

    /// Record rows the wrapped store mapped on this thread since the last
    /// `reset` (or construction).
    pub fn reads(&self) -> u64 {
        rows_read()
    }

    /// Zero both the call counters and this thread's row counter.
    pub fn reset(&self) {
        if let Ok(mut calls) = self.calls.lock() {
            calls.clear();
        }
        reset_rows();
    }

    fn before(&self, method: &'static str) -> Result<(), Error> {
        if let Ok(mut calls) = self.calls.lock() {
            *calls.entry(method).or_insert(0) += 1;
        }
        Ok(())
    }
}

metadata_store!(Counting);
submission_store!(Counting);
blob_store!(Counting);
change_feed!(Counting);
