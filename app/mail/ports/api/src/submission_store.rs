use crate::Change;
use mail_kernel::{Error, SubmissionQuery, SubmissionRecord};

pub struct SubmissionSelection {
    pub revision: u64,
    pub records: Vec<SubmissionRecord>,
}

pub struct SubmissionPage {
    pub revision: u64,
    pub ids: Vec<String>,
    pub position: usize,
    pub total: usize,
}

pub struct SubmissionChanges {
    pub revision: u64,
    pub has_more: bool,
    pub changes: Vec<Change<SubmissionRecord>>,
}

pub struct SubmissionAcceptance {
    pub record: SubmissionRecord,
    pub email_revision: u64,
    pub raw: Vec<u8>,
    pub allow_remote: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmissionFailure {
    Storage(Error),
    CannotUnsend,
    AnchorNotFound,
}
impl From<Error> for SubmissionFailure {
    fn from(error: Error) -> Self {
        Self::Storage(error)
    }
}

/// Submission metadata has an independent revision and lifetime from email and
/// delivery payloads. Acceptance and all recipient jobs commit atomically.
pub trait SubmissionStore: Send + Sync {
    /// At most 256 records. None requests all and refuses overflow.
    fn submissions(
        &self,
        account: &str,
        ids: Option<&[String]>,
    ) -> Result<SubmissionSelection, Error>;
    fn accept_submission(
        &self,
        account: &str,
        revision: u64,
        acceptance: SubmissionAcceptance,
    ) -> Result<SubmissionSelection, Error>;
    /// Cancellation refuses any job ever claimed: lease expiry cannot prove
    /// that an external SMTP receiver did not accept an ambiguous delivery.
    fn cancel_submission(
        &self,
        account: &str,
        revision: u64,
        id: &str,
    ) -> Result<SubmissionSelection, SubmissionFailure>;
    /// Destroy removes history's current object; delivery continues.
    fn destroy_submission(&self, account: &str, revision: u64, id: &str) -> Result<u64, Error>;
    /// Indexed filtering and paging in one snapshot; limit at most 10,000 for
    /// bounded query-change reconciliation (wire query caps at 256).
    fn query_submissions(
        &self,
        account: &str,
        revision: Option<u64>,
        query: &SubmissionQuery,
    ) -> Result<SubmissionPage, SubmissionFailure>;
    /// Ordered committed changes, at most 10,000, without splitting revisions.
    fn submission_changes(
        &self,
        account: &str,
        since: u64,
        limit: usize,
    ) -> Result<SubmissionChanges, Error>;
}
