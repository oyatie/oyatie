//! Injects one failure: the next matching call returns the planned error
//! without reaching the wrapped store, so a caller's rollback and retry paths
//! can be exercised deterministically.
use mail_api::{
    AccountInfo, Consumer, Execution, HistoryPage, MailboxSelection, MessageSelection,
    MetadataStore, Precondition, SubmissionAcceptance, SubmissionChanges, SubmissionFailure,
    SubmissionPage, SubmissionSelection, SubmissionStore,
};
use mail_kernel::{Account, Command, Error, Retention, RetentionPolicy, SubmissionQuery};
use std::sync::Mutex;

struct Plan {
    /// `None` fails whichever trait method is called next.
    method: Option<&'static str>,
    error: Error,
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
        self.arm(None, error);
    }

    /// The next call of the named trait method (`"execute"`, `"history"`,
    /// ...) fails with `error`; other methods pass through untouched.
    pub fn fail_at(&self, method: &'static str, error: Error) {
        self.arm(Some(method), error);
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

    fn arm(&self, method: Option<&'static str>, error: Error) {
        if let Ok(mut plan) = self.plan.lock() {
            *plan = Some(Plan { method, error });
        }
    }

    fn before(&self, method: &'static str) -> Result<(), Error> {
        let mut plan = self.plan.lock().map_err(|_| Error::Unavailable)?;
        let fires = plan
            .as_ref()
            .is_some_and(|p| p.method.is_none_or(|m| m == method));
        if fires && let Some(Plan { error, .. }) = plan.take() {
            return Err(error);
        }
        Ok(())
    }
}

metadata_store!(Faulty);
submission_store!(Faulty);
