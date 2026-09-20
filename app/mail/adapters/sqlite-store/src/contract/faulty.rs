//! Injects one failure: the next matching call returns the planned error
//! without reaching the wrapped store, so a caller's rollback and retry paths
//! can be exercised deterministically.
use mail_api::{
    AccountInfo, AuditRow, BlobStore, ChangeFeed, Consumer, Dirty, Execution, FeedRead,
    HistoryPage, MailboxSelection, MessageSelection, MetadataStore, Precondition, Resume,
    SubmissionAcceptance, SubmissionChanges, SubmissionFailure, SubmissionPage,
    SubmissionSelection, SubmissionStore,
};
use mail_kernel::{Account, BlobRef, Command, Error, Retention, RetentionPolicy, SubmissionQuery};
use std::sync::Mutex;

struct Plan {
    /// `None` fails whichever trait method is called next.
    method: Option<&'static str>,
    error: Error,
    /// Calls left to fail before the plan is spent.
    remaining: u32,
}

pub struct Faulty<T> {
    inner: T,
    plan: Mutex<Option<Plan>>,
}

impl<T> Faulty<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            plan: Mutex::new(None),
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// The next call of any trait method fails with `error`.
    pub fn fail_next(&self, error: Error) {
        self.arm(None, error, 1);
    }

    /// The next call of the named trait method (`"execute"`, `"history"`,
    /// ...) fails with `error`; other methods pass through untouched.
    pub fn fail_at(&self, method: &'static str, error: Error) {
        self.arm(Some(method), error, 1);
    }

    /// The next `times` calls of the named method fail with `error`: a
    /// lease holder that keeps answering `Busy` until it commits.
    pub fn fail_times(&self, method: &'static str, error: Error, times: u32) {
        self.arm(Some(method), error, times.max(1));
    }

    /// Whether a planned failure is still waiting for its call.
    pub fn armed(&self) -> bool {
        self.plan.lock().map(|plan| plan.is_some()).unwrap_or(false)
    }

    /// Drop a planned failure that has not fired.
    pub fn disarm(&self) {
        if let Ok(mut plan) = self.plan.lock() {
            *plan = None;
        }
    }

    fn arm(&self, method: Option<&'static str>, error: Error, remaining: u32) {
        if let Ok(mut plan) = self.plan.lock() {
            *plan = Some(Plan {
                method,
                error,
                remaining,
            });
        }
    }

    fn before(&self, method: &'static str) -> Result<(), Error> {
        let mut plan = self.plan.lock().map_err(|_| Error::Unavailable)?;
        let fires = plan
            .as_ref()
            .is_some_and(|p| p.method.is_none_or(|m| m == method));
        if fires && let Some(armed) = plan.as_mut() {
            let error = armed.error;
            armed.remaining -= 1;
            if armed.remaining == 0 {
                *plan = None;
            }
            return Err(error);
        }
        Ok(())
    }
}

metadata_store!(Faulty);
submission_store!(Faulty);
blob_store!(Faulty);
change_feed!(Faulty);
