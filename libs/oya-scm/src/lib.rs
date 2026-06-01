//! # oya-scm
//!
//! SCM-agnostic trait surface for the Oyatie CI stack (ADR-0517).
//!
//! This kernel crate defines the `Scm` trait and its neutral types. It has no
//! I/O dependencies. Adapter crates (`oya-scm-forgejo`, and future adapters)
//! implement `Scm` against specific VCS hosts.
//!
//! ## Design intent
//!
//! The CI kernels (ci-webhook-gateway, ci-tide, ci-controller) MUST NOT call
//! Forgejo APIs directly. They depend on this crate and receive a `dyn Scm`
//! (or `impl Scm`) at construction time. `ForgejoScm` is the default/interim
//! adapter — switching to any other host is a one-adapter swap.
//!
//! ## Trait surface
//!
//! | Method                      | Purpose                                     |
//! |-----------------------------|---------------------------------------------|
//! | `list_open_pulls`           | Poll candidate PRs for the merge queue      |
//! | `get_pull`                  | Fetch single PR (incl. mergeability)        |
//! | `merge_pull`                | Land a PR via the host merge API            |
//! | `get_combined_commit_status`| Gate decision on a SHA                      |
//! | `post_commit_status`        | Post CI context result to a SHA             |
//! | `list_reviews`              | Collect reviewer approvals/rejections       |
//! | `fetch_ref`                 | Resolve a ref → commit SHA                  |
//! | `webhook_event_from_bytes`  | Parse inbound webhook bytes → `ScmEvent`    |
//! | `get_branch_protection`     | Query host branch-protection rules          |
//!
//! ## Future stubs (ADR-0517)
//!
//! `BuildGraphProvider` and `VfsProvider` are marked `// FUTURE` and compile
//! to empty trait bodies. They are present so downstream crates can reference
//! them by name once the implementation PRs land.
//!
//! ## Security invariants
//!
//! - No `unwrap`/`expect`/`panic` on the public API surface (ADR-0083 Tier-3).
//! - All secret-bearing fields (`token`) carry a `// data_class: INTERNAL_ONLY`
//!   annotation and have redacted `Debug` impls.
//! - `ScmToken` never derives `Debug` — callers see `ScmToken(<redacted>)`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// All errors that an `Scm` adapter may return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScmError {
    /// The operation was denied by the host (HTTP 401/403 or equivalent).
    Unauthorized,
    /// The addressed resource (PR, SHA, ref) was not found.
    NotFound(String),
    /// The host returned a transport-level error (network, timeout, etc.).
    Transport(String),
    /// The host returned a response that could not be decoded.
    UnexpectedResponse(String),
    /// The merge was rejected (e.g. merge conflict, branch-protection rule).
    MergeRejected(String),
    /// The webhook payload could not be parsed or its signature was invalid.
    WebhookParse(String),
}

impl std::fmt::Display for ScmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScmError::Unauthorized => f.write_str("scm: unauthorized"),
            ScmError::NotFound(r) => write!(f, "scm: not found: {r}"),
            ScmError::Transport(e) => write!(f, "scm: transport error: {e}"),
            ScmError::UnexpectedResponse(e) => write!(f, "scm: unexpected response: {e}"),
            ScmError::MergeRejected(r) => write!(f, "scm: merge rejected: {r}"),
            ScmError::WebhookParse(e) => write!(f, "scm: webhook parse error: {e}"),
        }
    }
}

impl std::error::Error for ScmError {}

pub type Result<T> = std::result::Result<T, ScmError>;

// ---------------------------------------------------------------------------
// Authentication credential (redacted Debug)
// ---------------------------------------------------------------------------

/// An opaque SCM authentication token.
///
/// `Debug` is deliberately redacted — the token MUST NOT appear in log output.
#[derive(Clone, PartialEq, Eq)]
pub struct ScmToken(String); // data_class: INTERNAL_ONLY

impl ScmToken {
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ScmToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ScmToken(<redacted>)")
    }
}

// ---------------------------------------------------------------------------
// Repository coordinates
// ---------------------------------------------------------------------------

/// Identifies a repository on an SCM host.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RepoCoords {
    /// Owner (org or user), e.g. `"oyatie"`.
    pub owner: String, // data_class: INTERNAL_ONLY
    /// Repository name, e.g. `"oyatie"`.
    pub name: String, // data_class: INTERNAL_ONLY
}

impl RepoCoords {
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }

    /// `"owner/name"` form used in URLs and log messages.
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}

// ---------------------------------------------------------------------------
// Neutral PR type
// ---------------------------------------------------------------------------

/// Merge strategy requested by the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}

/// Whether the PR can currently be merged according to the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Mergeability {
    /// Host confirms the PR can be merged without conflicts.
    Mergeable,
    /// Host reports a merge conflict.
    Conflicted,
    /// The host is still computing mergeability (e.g. Forgejo background check).
    Unknown,
}

/// A pull request as returned by the SCM host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PullRequest {
    /// PR number within the repository.
    pub number: u64, // data_class: INTERNAL_ONLY
    /// Short title.
    pub title: String, // data_class: INTERNAL_ONLY
    /// Current state (`"open"` | `"closed"`).
    pub state: String, // data_class: INTERNAL_ONLY
    /// Whether the PR is a draft (draft PRs are skipped by the merge queue).
    pub draft: bool, // data_class: INTERNAL_ONLY
    /// HEAD commit SHA of the PR branch.
    pub head_sha: String, // data_class: INTERNAL_ONLY
    /// Ref name of the PR's source branch, e.g. `"feat/my-feature"`.
    pub head_ref: String, // data_class: INTERNAL_ONLY
    /// Ref name of the target branch, e.g. `"dev"`.
    pub base_ref: String, // data_class: INTERNAL_ONLY
    /// Base commit SHA (tip of `base_ref` at time of last sync).
    pub base_sha: String, // data_class: INTERNAL_ONLY
    /// Current mergeability as reported by the host.
    pub mergeability: Mergeability, // data_class: INTERNAL_ONLY
    /// Repository this PR belongs to.
    pub repo: RepoCoords, // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// Commit status types
// ---------------------------------------------------------------------------

/// State of a commit status context.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CommitState {
    Pending,
    Success,
    Failure,
    Error,
}

impl CommitState {
    pub const fn as_str(self) -> &'static str {
        match self {
            CommitState::Pending => "pending",
            CommitState::Success => "success",
            CommitState::Failure => "failure",
            CommitState::Error => "error",
        }
    }

    /// Combined status aggregation: `Success` only when all are success; any
    /// failure → `Failure`; any pending → `Pending`; otherwise `Error`.
    pub fn aggregate(states: impl IntoIterator<Item = CommitState>) -> CommitState {
        let mut pending = false;
        let mut failure = false;
        for s in states {
            match s {
                CommitState::Failure => failure = true,
                CommitState::Pending => pending = true,
                CommitState::Error => return CommitState::Error,
                CommitState::Success => {}
            }
        }
        if failure {
            CommitState::Failure
        } else if pending {
            CommitState::Pending
        } else {
            CommitState::Success
        }
    }
}

impl std::fmt::Display for CommitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single commit status context posted to a SHA.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitStatus {
    /// The context string (e.g. `"cargo-nextest"`).
    pub context: String, // data_class: INTERNAL_ONLY
    /// The status state.
    pub state: CommitState, // data_class: INTERNAL_ONLY
    /// Human-readable description (max ~140 chars).
    pub description: String, // data_class: INTERNAL_ONLY
    /// Optional link to the CI build.
    pub target_url: Option<String>, // data_class: INTERNAL_ONLY
}

/// The aggregate (combined) commit status for a SHA — the logical AND of all
/// individual context statuses as computed by the host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombinedCommitStatus {
    /// The aggregate state.
    pub state: CommitState, // data_class: INTERNAL_ONLY
    /// Individual context statuses contributing to the aggregate.
    pub statuses: Vec<CommitStatus>, // data_class: INTERNAL_ONLY
}

/// Request to post a single commit status context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostStatusRequest {
    /// Commit SHA to annotate.
    pub sha: String, // data_class: INTERNAL_ONLY
    /// Context name.
    pub context: String, // data_class: INTERNAL_ONLY
    /// Status state.
    pub state: CommitState, // data_class: INTERNAL_ONLY
    /// Human-readable description.
    pub description: String, // data_class: INTERNAL_ONLY
    /// Optional target URL.
    pub target_url: Option<String>, // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// Review types
// ---------------------------------------------------------------------------

/// The outcome of a single review on a PR.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ReviewState {
    /// Reviewer explicitly approved the change.
    Approved,
    /// Reviewer requested changes (blocks merge).
    ChangesRequested,
    /// Reviewer left a comment without a formal verdict.
    Commented,
    /// The review was dismissed.
    Dismissed,
}

/// A single review on a pull request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Review {
    /// Reviewer's login name.
    pub reviewer: String, // data_class: INTERNAL_ONLY
    /// Review state.
    pub state: ReviewState, // data_class: INTERNAL_ONLY
    /// Opaque review ID from the host.
    pub id: u64, // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// ChangeSet — grouping of PRs that enter the merge queue together
// ---------------------------------------------------------------------------

/// A set of pull requests that are projected to land together (ADR-0111).
///
/// This is the neutral equivalent of the merge-queue slot. Each PR in the set
/// has already passed individual CI; the `ChangeSet` enters a combined
/// speculative CI run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSet {
    /// The PRs in this projected merge batch, in queue order.
    pub pulls: Vec<PullRequest>, // data_class: INTERNAL_ONLY
    /// The projected HEAD SHA after squashing all PRs onto `base_ref`.
    pub projected_sha: String, // data_class: INTERNAL_ONLY
    /// The base ref all PRs target.
    pub base_ref: String, // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// Branch-protection query result
// ---------------------------------------------------------------------------

/// Minimal view of a branch-protection rule as returned by the SCM host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchProtection {
    /// The branch name or pattern this rule applies to.
    pub branch: String, // data_class: INTERNAL_ONLY
    /// Whether at least one approval is required before merging.
    pub require_review: bool, // data_class: INTERNAL_ONLY
    /// Status-check contexts that must be green before merging.
    pub required_status_contexts: Vec<String>, // data_class: INTERNAL_ONLY
    /// Whether force-pushes to this branch are blocked.
    pub block_force_push: bool, // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// Webhook event model
// ---------------------------------------------------------------------------

/// The normalised SCM webhook event used by the CI kernels.
///
/// Host-specific webhook adapters parse raw bytes into this type via
/// `Scm::webhook_event_from_bytes`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScmEvent {
    /// A pull-request lifecycle change.
    PullRequest(PullRequestEvent),
    /// A commit-status update (posted by a CI system, not a human).
    CommitStatus(CommitStatusEvent),
    /// A ping / connection-handshake sent by the host on webhook registration.
    Ping { message: String },
    /// Any other event type the adapter does not specifically handle.
    Other { event_type: String },
}

/// A pull-request event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PullRequestEvent {
    /// The action that triggered the event.
    pub action: PullRequestAction, // data_class: INTERNAL_ONLY
    /// The pull request at the time of the event.
    pub pull: PullRequest, // data_class: INTERNAL_ONLY
    /// Opaque delivery ID from the host for idempotency.
    pub delivery_id: String, // data_class: INTERNAL_ONLY
}

/// Actions that can trigger a pull-request webhook event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PullRequestAction {
    Opened,
    Reopened,
    Synchronized,
    Closed,
    ReadyForReview,
}

/// A commit-status webhook event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitStatusEvent {
    /// The SHA the status was posted against.
    pub sha: String, // data_class: INTERNAL_ONLY
    /// The context that changed.
    pub context: String, // data_class: INTERNAL_ONLY
    /// The new state.
    pub state: CommitState, // data_class: INTERNAL_ONLY
    /// The repository the status belongs to.
    pub repo: RepoCoords, // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// Core Scm trait
// ---------------------------------------------------------------------------

/// SCM-agnostic operations required by the Oyatie CI stack (ADR-0517).
///
/// Implementors provide all VCS-host I/O; kernels depend only on this trait.
/// `ForgejoScm` in `oya-scm-forgejo` is the default/interim implementation.
///
/// ## Error contract
///
/// All methods return `Result<_>` and MUST NOT panic on the request path
/// (ADR-0083 Tier-3). Transport errors map to `ScmError::Transport`.
pub trait Scm {
    // ------------------------------------------------------------------
    // Pull-request queries
    // ------------------------------------------------------------------

    /// List all open pull requests whose base branch matches `base_ref`.
    ///
    /// Returns an empty `Vec` (not an error) when there are no open PRs.
    fn list_open_pulls(&self, repo: &RepoCoords, base_ref: &str) -> Result<Vec<PullRequest>>;

    /// Fetch a single pull request by number.
    fn get_pull(&self, repo: &RepoCoords, number: u64) -> Result<PullRequest>;

    // ------------------------------------------------------------------
    // Commit status
    // ------------------------------------------------------------------

    /// Fetch the combined (aggregated) commit status for a SHA.
    fn get_combined_commit_status(
        &self,
        repo: &RepoCoords,
        sha: &str,
    ) -> Result<CombinedCommitStatus>;

    /// Post a single commit status context for a SHA.
    fn post_commit_status(&self, repo: &RepoCoords, request: &PostStatusRequest) -> Result<()>;

    // ------------------------------------------------------------------
    // Reviews
    // ------------------------------------------------------------------

    /// List all reviews on the given PR.
    fn list_reviews(&self, repo: &RepoCoords, pr_number: u64) -> Result<Vec<Review>>;

    // ------------------------------------------------------------------
    // Merge
    // ------------------------------------------------------------------

    /// Merge a pull request using the specified method.
    ///
    /// Returns `ScmError::MergeRejected` if the host refuses the merge
    /// (conflict, branch-protection block, etc.).
    fn merge_pull(
        &self,
        repo: &RepoCoords,
        pr_number: u64,
        method: MergeMethod,
        commit_title: Option<&str>,
    ) -> Result<()>;

    // ------------------------------------------------------------------
    // Ref resolution
    // ------------------------------------------------------------------

    /// Resolve a ref (branch name, tag, or SHA) to a commit SHA.
    ///
    /// Returns `ScmError::NotFound` if the ref does not exist.
    fn fetch_ref(&self, repo: &RepoCoords, git_ref: &str) -> Result<String>;

    // ------------------------------------------------------------------
    // Webhook
    // ------------------------------------------------------------------

    /// Parse raw webhook bytes (already signature-verified) into a `ScmEvent`.
    ///
    /// `event_type` is the value of the host event-type header
    /// (e.g. `X-Forgejo-Event` or `X-GitHub-Event`).
    fn webhook_event_from_bytes(
        &self,
        event_type: &str,
        body: &[u8],
        delivery_id: &str,
    ) -> Result<ScmEvent>;

    // ------------------------------------------------------------------
    // Branch protection
    // ------------------------------------------------------------------

    /// Query the branch-protection rule for `branch`.
    ///
    /// Returns `ScmError::NotFound` when no rule exists for the branch.
    fn get_branch_protection(
        &self,
        repo: &RepoCoords,
        branch: &str,
    ) -> Result<BranchProtection>;
}

// ---------------------------------------------------------------------------
// FUTURE: BuildGraphProvider stub (ADR-0517)
// ---------------------------------------------------------------------------

/// FUTURE (ADR-0517 Phase 2): abstraction for querying the build graph.
///
/// Not yet implemented. Present so downstream crates can name the type.
/// Will be wired once the Buck2/BXL query surface is stable.
pub trait BuildGraphProvider {
    /// Returns the set of affected build targets for the given list of changed
    /// file paths.
    fn affected_targets(&self, changed_files: &[&str]) -> Result<Vec<String>>;
}

// ---------------------------------------------------------------------------
// FUTURE: VfsProvider stub (ADR-0517)
// ---------------------------------------------------------------------------

/// FUTURE (ADR-0517 Phase 2): abstraction for reading files from a commit tree
/// without a local clone.
///
/// Not yet implemented. Present so downstream crates can name the type.
pub trait VfsProvider {
    /// Read the content of `path` at `sha` from the remote SCM host.
    fn read_file(&self, repo: &RepoCoords, sha: &str, path: &str) -> Result<Vec<u8>>;
}

// ---------------------------------------------------------------------------
// MockScm — test double
// ---------------------------------------------------------------------------

/// A test double that pre-loads expected responses.
///
/// Suitable for unit-testing CI kernel logic without any network I/O.
#[derive(Default)]
pub struct MockScm {
    /// Canned open pulls (returned for any `list_open_pulls` call).
    pub open_pulls: Vec<PullRequest>,
    /// Canned combined status (returned for any `get_combined_commit_status`).
    pub combined_status: Option<CombinedCommitStatus>,
    /// Canned reviews (returned for any `list_reviews`).
    pub reviews: Vec<Review>,
    /// Canned `fetch_ref` SHA.
    pub ref_sha: Option<String>,
    /// If `Some`, all mutating calls (`post_commit_status`, `merge_pull`) fail
    /// with this error.
    pub force_error: Option<ScmError>,
    /// Webhook event to return from `webhook_event_from_bytes`.
    pub webhook_event: Option<ScmEvent>,
    /// Branch-protection rule to return.
    pub branch_protection: Option<BranchProtection>,
}

impl Scm for MockScm {
    fn list_open_pulls(&self, _repo: &RepoCoords, _base_ref: &str) -> Result<Vec<PullRequest>> {
        Ok(self.open_pulls.clone())
    }

    fn get_pull(&self, _repo: &RepoCoords, number: u64) -> Result<PullRequest> {
        self.open_pulls
            .iter()
            .find(|p| p.number == number)
            .cloned()
            .ok_or_else(|| ScmError::NotFound(format!("pull #{number}")))
    }

    fn get_combined_commit_status(
        &self,
        _repo: &RepoCoords,
        _sha: &str,
    ) -> Result<CombinedCommitStatus> {
        self.combined_status
            .clone()
            .ok_or_else(|| ScmError::NotFound("combined status not configured in mock".into()))
    }

    fn post_commit_status(&self, _repo: &RepoCoords, _request: &PostStatusRequest) -> Result<()> {
        if let Some(e) = &self.force_error {
            return Err(e.clone());
        }
        Ok(())
    }

    fn list_reviews(&self, _repo: &RepoCoords, _pr_number: u64) -> Result<Vec<Review>> {
        Ok(self.reviews.clone())
    }

    fn merge_pull(
        &self,
        _repo: &RepoCoords,
        _pr_number: u64,
        _method: MergeMethod,
        _commit_title: Option<&str>,
    ) -> Result<()> {
        if let Some(e) = &self.force_error {
            return Err(e.clone());
        }
        Ok(())
    }

    fn fetch_ref(&self, _repo: &RepoCoords, git_ref: &str) -> Result<String> {
        self.ref_sha
            .clone()
            .ok_or_else(|| ScmError::NotFound(format!("ref {git_ref}")))
    }

    fn webhook_event_from_bytes(
        &self,
        _event_type: &str,
        _body: &[u8],
        _delivery_id: &str,
    ) -> Result<ScmEvent> {
        self.webhook_event
            .clone()
            .ok_or_else(|| ScmError::WebhookParse("no webhook event configured in mock".into()))
    }

    fn get_branch_protection(
        &self,
        _repo: &RepoCoords,
        branch: &str,
    ) -> Result<BranchProtection> {
        self.branch_protection
            .clone()
            .ok_or_else(|| ScmError::NotFound(format!("branch-protection for {branch}")))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pull(number: u64) -> PullRequest {
        PullRequest {
            number,
            title: format!("PR #{number}"),
            state: "open".into(),
            draft: false,
            head_sha: format!("sha{number}"),
            head_ref: format!("feat/branch-{number}"),
            base_ref: "dev".into(),
            base_sha: "base000".into(),
            mergeability: Mergeability::Mergeable,
            repo: RepoCoords::new("oyatie", "oyatie"),
        }
    }

    #[test]
    fn scm_token_redacted_in_debug() {
        let tok = ScmToken::new("secret-token-value");
        assert_eq!(format!("{tok:?}"), "ScmToken(<redacted>)");
    }

    #[test]
    fn repo_coords_full_name() {
        let r = RepoCoords::new("oyatie", "oyatie");
        assert_eq!(r.full_name(), "oyatie/oyatie");
    }

    #[test]
    fn commit_state_aggregate_all_success() {
        let result = CommitState::aggregate([
            CommitState::Success,
            CommitState::Success,
            CommitState::Success,
        ]);
        assert_eq!(result, CommitState::Success);
    }

    #[test]
    fn commit_state_aggregate_any_failure() {
        let result = CommitState::aggregate([CommitState::Success, CommitState::Failure]);
        assert_eq!(result, CommitState::Failure);
    }

    #[test]
    fn commit_state_aggregate_any_pending() {
        let result = CommitState::aggregate([CommitState::Success, CommitState::Pending]);
        assert_eq!(result, CommitState::Pending);
    }

    #[test]
    fn commit_state_aggregate_error_short_circuits() {
        let result = CommitState::aggregate([CommitState::Pending, CommitState::Error]);
        assert_eq!(result, CommitState::Error);
    }

    #[test]
    fn mock_scm_list_open_pulls() {
        let mock = MockScm {
            open_pulls: vec![make_pull(1), make_pull(2)],
            ..MockScm::default()
        };
        let repo = RepoCoords::new("oyatie", "oyatie");
        let pulls = mock.list_open_pulls(&repo, "dev").unwrap();
        assert_eq!(pulls.len(), 2);
        assert_eq!(pulls[0].number, 1);
    }

    #[test]
    fn mock_scm_get_pull_not_found() {
        let mock = MockScm::default();
        let repo = RepoCoords::new("oyatie", "oyatie");
        let err = mock.get_pull(&repo, 99).unwrap_err();
        assert!(matches!(err, ScmError::NotFound(_)));
    }

    #[test]
    fn mock_scm_get_pull_found() {
        let mock = MockScm {
            open_pulls: vec![make_pull(5)],
            ..MockScm::default()
        };
        let repo = RepoCoords::new("oyatie", "oyatie");
        let pr = mock.get_pull(&repo, 5).unwrap();
        assert_eq!(pr.number, 5);
    }

    #[test]
    fn mock_scm_post_commit_status_force_error() {
        let mock = MockScm {
            force_error: Some(ScmError::Unauthorized),
            ..MockScm::default()
        };
        let repo = RepoCoords::new("oyatie", "oyatie");
        let req = PostStatusRequest {
            sha: "abc".into(),
            context: "cargo-fmt".into(),
            state: CommitState::Success,
            description: "passed".into(),
            target_url: None,
        };
        assert_eq!(
            mock.post_commit_status(&repo, &req).unwrap_err(),
            ScmError::Unauthorized
        );
    }

    #[test]
    fn mock_scm_merge_pull_ok() {
        let mock = MockScm::default();
        let repo = RepoCoords::new("oyatie", "oyatie");
        assert!(mock
            .merge_pull(&repo, 1, MergeMethod::Squash, None)
            .is_ok());
    }

    #[test]
    fn mock_scm_fetch_ref_not_configured() {
        let mock = MockScm::default();
        let repo = RepoCoords::new("oyatie", "oyatie");
        assert!(matches!(
            mock.fetch_ref(&repo, "dev").unwrap_err(),
            ScmError::NotFound(_)
        ));
    }

    #[test]
    fn mock_scm_fetch_ref_configured() {
        let mock = MockScm {
            ref_sha: Some("abc123".into()),
            ..MockScm::default()
        };
        let repo = RepoCoords::new("oyatie", "oyatie");
        assert_eq!(mock.fetch_ref(&repo, "dev").unwrap(), "abc123");
    }

    #[test]
    fn mock_scm_branch_protection_not_found() {
        let mock = MockScm::default();
        let repo = RepoCoords::new("oyatie", "oyatie");
        assert!(matches!(
            mock.get_branch_protection(&repo, "dev").unwrap_err(),
            ScmError::NotFound(_)
        ));
    }

    #[test]
    fn merge_method_variants_are_distinct() {
        assert_ne!(MergeMethod::Merge, MergeMethod::Squash);
        assert_ne!(MergeMethod::Squash, MergeMethod::Rebase);
    }

    #[test]
    fn review_state_variants_are_distinct() {
        assert_ne!(ReviewState::Approved, ReviewState::ChangesRequested);
    }

    #[test]
    fn mergeability_variants_are_distinct() {
        assert_ne!(Mergeability::Mergeable, Mergeability::Conflicted);
    }
}
