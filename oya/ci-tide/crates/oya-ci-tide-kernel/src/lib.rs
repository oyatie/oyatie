//! # oya-ci-tide-kernel
//!
//! Pure-domain kernel for the oya-ci tide component (Phase 2, ADR-0513).
//! No I/O, no async, no network. #![forbid(unsafe_code)].
//!
//! Owns:
//! - [`TideConfig`] — resolved runtime configuration
//! - [`PullRequest`] / [`CommitStatusState`] / [`Review`] / [`MergeMethod`] /
//!   [`MergeState`] — forge API projection types
//! - [`is_mergeable`] — the eligibility predicate (THE core logic)
//! - [`ForgeClient`] trait seam — I/O boundary for the adapter layer
//!
//! ## Eligibility invariant (ADR-0513 tide / ADR-0111)
//!
//! A PR is merge-eligible iff ALL of:
//! 1. The configured `required_status_context` has state `success` on the HEAD SHA.
//! 2. The number of approving reviews >= `approval_policy.min_approvals`.
//! 3. No blocking label (`hold` / `do-not-merge`) is present.
//! 4. The PR is not stale/behind the protected base. Stale approved PRs are a
//!    controller-owned branch-refresh action, never a merge.
//! 5. The forge reports the PR as clean and mergeable (no conflicts).
//!
//! ## Forge of record (D-FORGE)
//!
//! GitHub interim until the Sapling-inspired bespoke SCM.
//!
//! ## Security
//!
//! - `dry_run` defaults to `true` — tide merges NOTHING until explicitly configured live.
//! - Token never hardcoded; always read from `OYA_GITHUB_TOKEN`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// All tide kernel errors. HTTP / reqwest details live in the adapter layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TideError {
    /// A downstream forge API call failed (transport or non-2xx).
    Downstream(String),
    /// A required field was missing or malformed.
    InvalidInput(String),
}

impl std::fmt::Display for TideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TideError::Downstream(why) => write!(f, "forge downstream error: {why}"),
            TideError::InvalidInput(why) => write!(f, "invalid input: {why}"),
        }
    }
}

impl std::error::Error for TideError {}

pub type Result<T> = std::result::Result<T, TideError>;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Env var carrying the forge API token (injected from the deploy substrate).
/// Never hardcoded.
pub const ENV_GITHUB_TOKEN: &str = "OYA_GITHUB_TOKEN";

/// Default forge API base URL (GitHub, forge of record).
pub const DEFAULT_FORGE_BASE_URL: &str = "https://api.github.com";

/// Default repository owner.
pub const DEFAULT_REPO_OWNER: &str = "jason931225";

/// Default repository name.
pub const DEFAULT_REPO_NAME: &str = "oyatie";

/// Default base branch that tide manages.
pub const DEFAULT_BASE_BRANCH: &str = "dev";

/// Default required commit-status context (must match branch-protection rule).
pub const DEFAULT_REQUIRED_STATUS_CONTEXT: &str = "oya-ci-required";

/// Default minimum approving reviews required.
pub const DEFAULT_MIN_APPROVALS: u32 = 1;

/// Default poll interval in seconds.
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 60;

/// Labels that block merge, case-insensitive prefix match.
pub const BLOCKING_LABEL_PREFIXES: &[&str] = &["hold", "do-not-merge"];

/// Merge methods supported by the forge merge API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MergeMethod {
    /// `merge` — create a merge commit.
    Merge,
    /// `rebase` — rebase then fast-forward (default; preserves linear history).
    Rebase,
    /// `squash` — squash all commits into one.
    Squash,
}

impl MergeMethod {
    /// The string value the forge merge API expects (GitHub `merge_method`).
    pub const fn as_str(self) -> &'static str {
        match self {
            MergeMethod::Merge => "merge",
            MergeMethod::Rebase => "rebase",
            MergeMethod::Squash => "squash",
        }
    }

    /// Parse from env-var string. Unrecognised / blank → `Rebase`.
    pub fn from_env_value(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "merge" => MergeMethod::Merge,
            "squash" => MergeMethod::Squash,
            _ => MergeMethod::Rebase,
        }
    }
}

impl std::str::FromStr for MergeMethod {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from_env_value(s))
    }
}

/// Forge-projected merge-state vocabulary.
///
/// GitHub exposes this as the REST `mergeable_state` string. The kernel keeps a
/// closed subset for the queue decisions it owns and treats unknown/absent
/// values conservatively.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MergeState {
    /// Branch is up to date and mergeable once all other gates pass.
    Clean,
    /// Branch is behind the protected base. Tide must refresh the branch and
    /// let `oya-ci-required` rerun on the new head before any merge action.
    Behind,
    /// Textual conflict or otherwise dirty merge state.
    Dirty,
    /// Forge reports pending/unstable checks outside the required context.
    Unstable,
    /// Forge reports a policy blocker.
    Blocked,
    /// Forge has not computed or did not provide a recognized state.
    Unknown,
}

impl MergeState {
    /// Parse the forge's merge-state string. Unrecognised / blank → `Unknown`.
    pub fn from_forge_state(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "clean" | "has_hooks" => MergeState::Clean,
            "behind" => MergeState::Behind,
            "dirty" | "conflicting" => MergeState::Dirty,
            "unstable" => MergeState::Unstable,
            "blocked" | "draft" => MergeState::Blocked,
            _ => MergeState::Unknown,
        }
    }

    /// True when the queue should refresh the PR branch instead of merging it.
    pub const fn is_stale_base(self) -> bool {
        matches!(self, MergeState::Behind)
    }
}

impl std::str::FromStr for MergeState {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from_forge_state(s))
    }
}

/// Approval policy — how many approving reviews are required.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalPolicy {
    /// Minimum number of distinct approving reviews.
    pub min_approvals: u32,
}

impl Default for ApprovalPolicy {
    fn default() -> Self {
        ApprovalPolicy {
            min_approvals: DEFAULT_MIN_APPROVALS,
        }
    }
}

/// Resolved, validated runtime configuration for tide.
#[derive(Clone, Debug)]
pub struct TideConfig {
    /// Forge API base URL (e.g. `https://api.github.com`).
    pub forge_base_url: String,
    /// Repository owner (e.g. `oya-admin`).
    pub repo_owner: String,
    /// Repository name (e.g. `oyatie`).
    pub repo_name: String,
    /// Base branch to poll PRs against (default: `dev`).
    pub base_branch: String,
    /// Required commit-status context that must be `success` (default: `oya-ci-required`).
    pub required_status_context: String,
    /// Approval policy (default: 1 approving review).
    pub approval_policy: ApprovalPolicy,
    /// Poll interval in seconds (default: 60).
    pub poll_interval_secs: u64,
    /// Merge method (default: `rebase`).
    pub merge_method: MergeMethod,
    /// Safety guard: when `true` tide logs "WOULD MERGE" but never calls the
    /// merge API. Defaults to `true` — must be explicitly set to `false` to
    /// enable live merging.
    pub dry_run: bool,
}

impl TideConfig {
    /// Build config from a key→value lookup (injectable for tests).
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Self {
        let forge_base_url = get("OYA_TIDE_FORGE_BASE_URL")
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_FORGE_BASE_URL.to_owned());
        let repo_owner = get("OYA_TIDE_REPO_OWNER")
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_REPO_OWNER.to_owned());
        let repo_name = get("OYA_TIDE_REPO_NAME")
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_REPO_NAME.to_owned());
        let base_branch = get("OYA_TIDE_BASE_BRANCH")
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE_BRANCH.to_owned());
        let required_status_context = get("OYA_TIDE_REQUIRED_STATUS_CONTEXT")
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_REQUIRED_STATUS_CONTEXT.to_owned());
        let min_approvals: u32 = get("OYA_TIDE_MIN_APPROVALS")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(DEFAULT_MIN_APPROVALS);
        let poll_interval_secs: u64 = get("OYA_TIDE_POLL_INTERVAL_SECS")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(DEFAULT_POLL_INTERVAL_SECS);
        let merge_method = get("OYA_TIDE_MERGE_METHOD")
            .as_deref()
            .map(MergeMethod::from_env_value)
            .unwrap_or(MergeMethod::Rebase);
        // dry_run defaults to true; must be explicitly "false" to enable live merging.
        let dry_run = get("OYA_TIDE_DRY_RUN")
            .map(|v| !v.trim().eq_ignore_ascii_case("false"))
            .unwrap_or(true);

        TideConfig {
            forge_base_url,
            repo_owner,
            repo_name,
            base_branch,
            required_status_context,
            approval_policy: ApprovalPolicy { min_approvals },
            poll_interval_secs,
            merge_method,
            dry_run,
        }
    }

    /// Build config from the process environment.
    pub fn from_env() -> Self {
        Self::from_lookup(|key| std::env::var(key).ok())
    }
}

// ---------------------------------------------------------------------------
// Forge API projection types
// ---------------------------------------------------------------------------

/// Projected pull-request data from the forge API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PullRequest {
    /// PR number.
    pub number: u64,
    /// PR title.
    pub title: String,
    /// HEAD commit SHA.
    pub head_sha: String,
    /// Base branch (target of the PR).
    pub base_ref: String,
    /// Whether the forge considers the PR mergeable (no conflicts).
    /// `None` means the forge has not computed it yet (still processing).
    pub mergeable: Option<bool>,
    /// Forge-specific merge-state projection (`mergeable_state` on GitHub).
    pub merge_state: MergeState,
    /// Labels on the PR.
    pub labels: Vec<String>,
}

/// Combined commit-status state from the forge
/// (`GET /api/v1/repos/<owner>/<repo>/commits/<sha>/statuses`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitStatusState {
    /// At least one status is pending, none are failure/error.
    Pending,
    /// All non-pending statuses are success (or no statuses exist for the context).
    Success,
    /// At least one status is failure.
    Failure,
    /// At least one status is error.
    Error,
    /// No status for the required context exists at all.
    Missing,
}

impl CommitStatusState {
    /// Parse from the forge's state string.
    pub fn from_forge_state(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "success" => CommitStatusState::Success,
            "pending" => CommitStatusState::Pending,
            "failure" => CommitStatusState::Failure,
            "error" => CommitStatusState::Error,
            _ => CommitStatusState::Missing,
        }
    }
}

impl std::str::FromStr for CommitStatusState {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from_forge_state(s))
    }
}

/// A single review from the forge's pull-request reviews API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Review {
    /// Reviewer login.
    pub reviewer: String,
    /// Review state (`APPROVED`, `REQUEST_CHANGES`, `COMMENT`, etc.).
    pub state: ReviewState,
    /// Forge review submission timestamp, when present. RFC3339 strings sort
    /// lexicographically for the normalized UTC values returned by the forge.
    pub submitted_at: Option<String>,
    /// Forge review id, used to break timestamp ties when present.
    pub id: Option<u64>,
    /// Cross-page API order assigned by the adapter as reviews are returned.
    /// Larger values are later in the API result stream and are authoritative
    /// only when timestamp/id metadata is absent or tied.
    pub api_order: u64,
}

/// Review state vocabulary (GitHub).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewState {
    Approved,
    RequestChanges,
    Comment,
    Dismissed,
    /// Any unrecognised state.
    Unknown,
}

impl ReviewState {
    pub fn from_forge_state(s: &str) -> Self {
        match s.trim().to_ascii_uppercase().as_str() {
            "APPROVED" => ReviewState::Approved,
            "REQUEST_CHANGES" | "CHANGES_REQUESTED" => ReviewState::RequestChanges,
            "COMMENT" => ReviewState::Comment,
            "DISMISSED" => ReviewState::Dismissed,
            _ => ReviewState::Unknown,
        }
    }
}

impl std::str::FromStr for ReviewState {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from_forge_state(s))
    }
}

// ---------------------------------------------------------------------------
// Eligibility predicate — the core tide logic
// ---------------------------------------------------------------------------

/// Input to the eligibility predicate.
#[derive(Clone, Debug)]
pub struct EligibilityInput<'a> {
    pub pr: &'a PullRequest,
    /// The state of `required_status_context` on the PR's HEAD SHA.
    pub status_state: CommitStatusState,
    /// All reviews on the PR.
    pub reviews: &'a [Review],
    pub config: &'a TideConfig,
}

/// Reason a PR is NOT eligible for merge (returned for structured logging).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IneligibleReason {
    /// Required CI status is not `success`.
    StatusNotSuccess { actual: CommitStatusState },
    /// Insufficient approving reviews.
    InsufficientApprovals { actual: u32, required: u32 },
    /// The forge reports the PR is not mergeable (conflicts).
    NotMergeable,
    /// PR has valid approval/CI but is behind the protected base. The queue must
    /// refresh the branch and wait for `oya-ci-required` on the new head.
    StaleBase,
    /// A blocking label is present.
    BlockingLabel { label: String },
}

impl std::fmt::Display for IneligibleReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IneligibleReason::StatusNotSuccess { actual } => {
                write!(f, "required status not success (got {actual:?})")
            }
            IneligibleReason::InsufficientApprovals { actual, required } => {
                write!(f, "insufficient approvals ({actual}/{required})")
            }
            IneligibleReason::NotMergeable => write!(f, "PR is not mergeable (conflicts)"),
            IneligibleReason::StaleBase => {
                write!(
                    f,
                    "PR is behind the protected base; branch refresh required"
                )
            }
            IneligibleReason::BlockingLabel { label } => {
                write!(f, "blocking label present: {label}")
            }
        }
    }
}

/// Check whether a pull request is eligible for merge.
///
/// Returns `Ok(())` when the PR passes all four gates, or the first
/// [`IneligibleReason`] that makes it ineligible.
///
/// # Rules (all must hold)
///
/// 1. `status_state == CommitStatusState::Success` for the required context.
/// 2. Count of `ReviewState::Approved` reviews >= `config.approval_policy.min_approvals`.
/// 3. No label matches a blocking prefix (`hold`, `do-not-merge`), case-insensitive.
/// 4. `pr.merge_state == MergeState::Clean`; stale PRs are branch-refresh
///    work, while other non-clean states are not merge work.
/// 5. `pr.mergeable == Some(true)` (not `None` or `Some(false)`).
pub fn is_mergeable(input: &EligibilityInput<'_>) -> std::result::Result<(), IneligibleReason> {
    // Rule 1 — CI status must be success.
    if input.status_state != CommitStatusState::Success {
        return Err(IneligibleReason::StatusNotSuccess {
            actual: input.status_state,
        });
    }

    // Rule 2 — sufficient distinct latest approving reviews. Later reviews from
    // the same reviewer supersede older approvals, so REQUEST_CHANGES /
    // DISMISSED states cannot be bypassed by a stale approval on another page.
    let approvals = latest_approving_review_count(input.reviews);
    if approvals < input.config.approval_policy.min_approvals {
        return Err(IneligibleReason::InsufficientApprovals {
            actual: approvals,
            required: input.config.approval_policy.min_approvals,
        });
    }

    // Rule 3 — no blocking labels. A held PR must not be auto-refreshed either.
    for label in &input.pr.labels {
        let lower = label.to_ascii_lowercase();
        for prefix in BLOCKING_LABEL_PREFIXES {
            if lower.starts_with(prefix) {
                return Err(IneligibleReason::BlockingLabel {
                    label: label.clone(),
                });
            }
        }
    }

    // Rule 4 — only clean merge states can proceed to merge. Stale approved PRs
    // are updated/requeued; every other non-clean/unknown state fails closed.
    match input.pr.merge_state {
        MergeState::Clean => {}
        MergeState::Behind => return Err(IneligibleReason::StaleBase),
        MergeState::Dirty | MergeState::Unstable | MergeState::Blocked | MergeState::Unknown => {
            return Err(IneligibleReason::NotMergeable);
        }
    }

    // Rule 5 — PR must be mergeable (no conflicts).
    if input.pr.mergeable != Some(true) {
        return Err(IneligibleReason::NotMergeable);
    }

    Ok(())
}

fn latest_approving_review_count(reviews: &[Review]) -> u32 {
    let mut latest: Vec<&Review> = Vec::new();

    for review in reviews {
        match latest
            .iter()
            .position(|existing| existing.reviewer == review.reviewer)
        {
            Some(index) => {
                if review_is_later(review, latest[index]) {
                    latest[index] = review;
                }
            }
            None => latest.push(review),
        }
    }

    latest
        .into_iter()
        .filter(|review| review.state == ReviewState::Approved)
        .count() as u32
}

fn review_is_later(candidate: &Review, current: &Review) -> bool {
    match (&candidate.submitted_at, &current.submitted_at) {
        (Some(candidate_ts), Some(current_ts)) if candidate_ts != current_ts => {
            return candidate_ts > current_ts;
        }
        _ => {}
    }

    match (candidate.id, current.id) {
        (Some(candidate_id), Some(current_id)) if candidate_id != current_id => {
            return candidate_id > current_id;
        }
        _ => {}
    }

    candidate.api_order > current.api_order
}

// ---------------------------------------------------------------------------
// ForgeClient trait seam — I/O boundary
// ---------------------------------------------------------------------------

/// Forge API client seam. Implemented by `oya-ci-tide-github-adapter`.
/// All methods are synchronous (adapter wraps reqwest blocking or spawns
/// via `tokio::task::spawn_blocking`).
pub trait ForgeClient: Send + Sync {
    /// List open pull requests against `base_branch`.
    fn list_open_pulls(&self, base_branch: &str) -> Result<Vec<PullRequest>>;

    /// Get the combined commit status for the given SHA, filtered to the
    /// `required_context`. Returns `CommitStatusState::Missing` when no
    /// status exists for that context.
    fn get_commit_status(&self, sha: &str, required_context: &str) -> Result<CommitStatusState>;

    /// List all reviews on a pull request.
    fn list_reviews(&self, pr_number: u64) -> Result<Vec<Review>>;

    /// Get a single pull request (to refresh `mergeable` field).
    fn get_pull(&self, pr_number: u64) -> Result<PullRequest>;

    /// Refresh a stale pull-request branch against the protected base.
    ///
    /// Implementations MUST pass the observed head SHA as a compare-and-swap
    /// guard so a concurrent author push cannot be overwritten by the queue.
    fn update_branch(&self, pr_number: u64, expected_head_sha: &str) -> Result<()>;

    /// Merge a pull request using the given method.
    ///
    /// Returns `Ok(())` on 200/204, `Err(TideError::Downstream)` otherwise.
    fn merge_pull(&self, pr_number: u64, method: MergeMethod) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Tests — eligibility predicate, all positive and negative cases
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- helpers ---

    fn default_config() -> TideConfig {
        TideConfig {
            forge_base_url: DEFAULT_FORGE_BASE_URL.to_owned(),
            repo_owner: DEFAULT_REPO_OWNER.to_owned(),
            repo_name: DEFAULT_REPO_NAME.to_owned(),
            base_branch: DEFAULT_BASE_BRANCH.to_owned(),
            required_status_context: DEFAULT_REQUIRED_STATUS_CONTEXT.to_owned(),
            approval_policy: ApprovalPolicy { min_approvals: 1 },
            poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
            merge_method: MergeMethod::Rebase,
            dry_run: true,
        }
    }

    fn approved_pr() -> PullRequest {
        PullRequest {
            number: 42,
            title: "feat: add tide".to_owned(),
            head_sha: "deadbeef01234567".to_owned(),
            base_ref: "dev".to_owned(),
            mergeable: Some(true),
            merge_state: MergeState::Clean,
            labels: vec![],
        }
    }

    fn approvals(n: usize) -> Vec<Review> {
        (0..n)
            .map(|i| Review {
                reviewer: format!("reviewer-{i}"),
                state: ReviewState::Approved,
                submitted_at: Some(format!("2026-06-01T00:00:{i:02}Z")),
                id: Some(i as u64),
                api_order: i as u64,
            })
            .collect()
    }

    fn input_all_green<'a>(
        pr: &'a PullRequest,
        reviews: &'a [Review],
        config: &'a TideConfig,
    ) -> EligibilityInput<'a> {
        EligibilityInput {
            pr,
            status_state: CommitStatusState::Success,
            reviews,
            config,
        }
    }

    // --- positive case ---

    #[test]
    fn green_approved_mergeable_no_hold_is_eligible() {
        let cfg = default_config();
        let pr = approved_pr();
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(is_mergeable(&input), Ok(()));
    }

    #[test]
    fn two_approvals_satisfies_min_two_policy() {
        let mut cfg = default_config();
        cfg.approval_policy.min_approvals = 2;
        let pr = approved_pr();
        let reviews = approvals(2);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(is_mergeable(&input), Ok(()));
    }

    // --- Rule 1: status not success ---

    #[test]
    fn pending_status_is_ineligible() {
        let cfg = default_config();
        let pr = approved_pr();
        let reviews = approvals(1);
        let input = EligibilityInput {
            pr: &pr,
            status_state: CommitStatusState::Pending,
            reviews: &reviews,
            config: &cfg,
        };
        assert_eq!(
            is_mergeable(&input),
            Err(IneligibleReason::StatusNotSuccess {
                actual: CommitStatusState::Pending
            })
        );
    }

    #[test]
    fn failure_status_is_ineligible() {
        let cfg = default_config();
        let pr = approved_pr();
        let reviews = approvals(1);
        let input = EligibilityInput {
            pr: &pr,
            status_state: CommitStatusState::Failure,
            reviews: &reviews,
            config: &cfg,
        };
        assert!(matches!(
            is_mergeable(&input),
            Err(IneligibleReason::StatusNotSuccess { .. })
        ));
    }

    #[test]
    fn missing_status_is_ineligible() {
        let cfg = default_config();
        let pr = approved_pr();
        let reviews = approvals(1);
        let input = EligibilityInput {
            pr: &pr,
            status_state: CommitStatusState::Missing,
            reviews: &reviews,
            config: &cfg,
        };
        assert!(matches!(
            is_mergeable(&input),
            Err(IneligibleReason::StatusNotSuccess { .. })
        ));
    }

    #[test]
    fn error_status_is_ineligible() {
        let cfg = default_config();
        let pr = approved_pr();
        let reviews = approvals(1);
        let input = EligibilityInput {
            pr: &pr,
            status_state: CommitStatusState::Error,
            reviews: &reviews,
            config: &cfg,
        };
        assert!(matches!(
            is_mergeable(&input),
            Err(IneligibleReason::StatusNotSuccess { .. })
        ));
    }

    // --- Rule 2: insufficient approvals ---

    #[test]
    fn zero_approvals_with_min_one_is_ineligible() {
        let cfg = default_config();
        let pr = approved_pr();
        let reviews: Vec<Review> = vec![];
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(
            is_mergeable(&input),
            Err(IneligibleReason::InsufficientApprovals {
                actual: 0,
                required: 1
            })
        );
    }

    #[test]
    fn one_approval_with_min_two_is_ineligible() {
        let mut cfg = default_config();
        cfg.approval_policy.min_approvals = 2;
        let pr = approved_pr();
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(
            is_mergeable(&input),
            Err(IneligibleReason::InsufficientApprovals {
                actual: 1,
                required: 2
            })
        );
    }

    #[test]
    fn comment_reviews_do_not_count_as_approvals() {
        let cfg = default_config();
        let pr = approved_pr();
        let reviews = vec![
            Review {
                reviewer: "alice".to_owned(),
                state: ReviewState::Comment,
                submitted_at: None,
                id: None,
                api_order: 0,
            },
            Review {
                reviewer: "bob".to_owned(),
                state: ReviewState::RequestChanges,
                submitted_at: None,
                id: None,
                api_order: 1,
            },
        ];
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(
            is_mergeable(&input),
            Err(IneligibleReason::InsufficientApprovals {
                actual: 0,
                required: 1
            })
        );
    }

    #[test]
    fn later_request_changes_from_same_reviewer_invalidates_stale_approval() {
        let cfg = default_config();
        let pr = approved_pr();
        let reviews = vec![
            Review {
                reviewer: "alice".to_owned(),
                state: ReviewState::Approved,
                submitted_at: Some("2026-06-01T00:00:00Z".to_owned()),
                id: Some(10),
                api_order: 0,
            },
            Review {
                reviewer: "alice".to_owned(),
                state: ReviewState::RequestChanges,
                submitted_at: Some("2026-06-01T00:01:00Z".to_owned()),
                id: Some(11),
                api_order: 1,
            },
        ];
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(
            is_mergeable(&input),
            Err(IneligibleReason::InsufficientApprovals {
                actual: 0,
                required: 1
            })
        );
    }

    #[test]
    fn review_api_order_breaks_cross_page_metadata_ties() {
        let cfg = default_config();
        let pr = approved_pr();
        let reviews = vec![
            Review {
                reviewer: "alice".to_owned(),
                state: ReviewState::RequestChanges,
                submitted_at: None,
                id: None,
                api_order: 0,
            },
            Review {
                reviewer: "alice".to_owned(),
                state: ReviewState::Approved,
                submitted_at: None,
                id: None,
                api_order: 51,
            },
        ];
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(is_mergeable(&input), Ok(()));
    }

    // --- Rule 3: not mergeable ---

    #[test]
    fn pr_with_conflicts_is_ineligible() {
        let cfg = default_config();
        let mut pr = approved_pr();
        pr.mergeable = Some(false);
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(is_mergeable(&input), Err(IneligibleReason::NotMergeable));
    }

    #[test]
    fn pr_with_mergeable_none_is_ineligible() {
        let cfg = default_config();
        let mut pr = approved_pr();
        pr.mergeable = None;
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(is_mergeable(&input), Err(IneligibleReason::NotMergeable));
    }

    #[test]
    fn stale_approved_pr_refreshes_before_merge() {
        let cfg = default_config();
        let mut pr = approved_pr();
        pr.merge_state = MergeState::Behind;
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(is_mergeable(&input), Err(IneligibleReason::StaleBase));
    }

    #[test]
    fn non_clean_non_behind_merge_states_are_ineligible() {
        for merge_state in [
            MergeState::Dirty,
            MergeState::Unstable,
            MergeState::Blocked,
            MergeState::Unknown,
        ] {
            let cfg = default_config();
            let mut pr = approved_pr();
            pr.merge_state = merge_state;
            let reviews = approvals(1);
            let input = input_all_green(&pr, &reviews, &cfg);
            assert_eq!(
                is_mergeable(&input),
                Err(IneligibleReason::NotMergeable),
                "expected {merge_state:?} to fail closed even when mergeable=true"
            );
        }
    }

    #[test]
    fn blocking_label_prevents_stale_branch_refresh() {
        let cfg = default_config();
        let mut pr = approved_pr();
        pr.merge_state = MergeState::Behind;
        pr.labels = vec!["hold: security review".to_owned()];
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert!(matches!(
            is_mergeable(&input),
            Err(IneligibleReason::BlockingLabel { .. })
        ));
    }

    // --- Rule 4: blocking labels ---

    #[test]
    fn hold_label_blocks_merge() {
        let cfg = default_config();
        let mut pr = approved_pr();
        pr.labels = vec!["hold".to_owned()];
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert!(matches!(
            is_mergeable(&input),
            Err(IneligibleReason::BlockingLabel { .. })
        ));
    }

    #[test]
    fn hold_prefix_label_blocks_merge() {
        let cfg = default_config();
        let mut pr = approved_pr();
        pr.labels = vec!["hold: waiting for design review".to_owned()];
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert!(matches!(
            is_mergeable(&input),
            Err(IneligibleReason::BlockingLabel { .. })
        ));
    }

    #[test]
    fn do_not_merge_label_blocks_merge() {
        let cfg = default_config();
        let mut pr = approved_pr();
        pr.labels = vec!["do-not-merge".to_owned()];
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert!(matches!(
            is_mergeable(&input),
            Err(IneligibleReason::BlockingLabel { .. })
        ));
    }

    #[test]
    fn do_not_merge_slash_variant_blocks_merge() {
        let cfg = default_config();
        let mut pr = approved_pr();
        pr.labels = vec!["do-not-merge/work-in-progress".to_owned()];
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert!(matches!(
            is_mergeable(&input),
            Err(IneligibleReason::BlockingLabel { .. })
        ));
    }

    #[test]
    fn blocking_label_check_is_case_insensitive() {
        let cfg = default_config();
        let mut pr = approved_pr();
        pr.labels = vec!["HOLD".to_owned()];
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert!(matches!(
            is_mergeable(&input),
            Err(IneligibleReason::BlockingLabel { .. })
        ));
    }

    #[test]
    fn unrelated_label_does_not_block() {
        let cfg = default_config();
        let mut pr = approved_pr();
        pr.labels = vec!["enhancement".to_owned(), "area/ci".to_owned()];
        let reviews = approvals(1);
        let input = input_all_green(&pr, &reviews, &cfg);
        assert_eq!(is_mergeable(&input), Ok(()));
    }

    // --- config defaults ---

    #[test]
    fn dry_run_defaults_to_true_when_env_unset() {
        let cfg = TideConfig::from_lookup(|_| None);
        assert!(cfg.dry_run, "dry_run must default to true for safety");
    }

    #[test]
    fn dry_run_false_only_when_explicitly_false() {
        let cfg = TideConfig::from_lookup(|k| {
            if k == "OYA_TIDE_DRY_RUN" {
                Some("false".to_owned())
            } else {
                None
            }
        });
        assert!(!cfg.dry_run);
    }

    #[test]
    fn dry_run_any_other_value_stays_true() {
        for val in &["true", "yes", "1", "FALSE_NOT", ""] {
            let v = val.to_string();
            let cfg = TideConfig::from_lookup(move |k| {
                if k == "OYA_TIDE_DRY_RUN" {
                    Some(v.clone())
                } else {
                    None
                }
            });
            assert!(
                cfg.dry_run,
                "expected dry_run=true for OYA_TIDE_DRY_RUN={val}"
            );
        }
    }

    #[test]
    fn merge_method_defaults_to_rebase() {
        let cfg = TideConfig::from_lookup(|_| None);
        assert_eq!(cfg.merge_method, MergeMethod::Rebase);
    }

    #[test]
    fn config_defaults_match_constants() {
        let cfg = TideConfig::from_lookup(|_| None);
        assert_eq!(cfg.base_branch, DEFAULT_BASE_BRANCH);
        assert_eq!(cfg.required_status_context, DEFAULT_REQUIRED_STATUS_CONTEXT);
        assert_eq!(cfg.approval_policy.min_approvals, DEFAULT_MIN_APPROVALS);
        assert_eq!(cfg.poll_interval_secs, DEFAULT_POLL_INTERVAL_SECS);
    }

    // --- MergeMethod / ReviewState parsing ---

    #[test]
    fn merge_method_parses_correctly() {
        assert_eq!(MergeMethod::from_env_value("merge"), MergeMethod::Merge);
        assert_eq!(MergeMethod::from_env_value("squash"), MergeMethod::Squash);
        assert_eq!(MergeMethod::from_env_value("rebase"), MergeMethod::Rebase);
        assert_eq!(MergeMethod::from_env_value("bogus"), MergeMethod::Rebase);
        assert_eq!(MergeMethod::from_env_value(""), MergeMethod::Rebase);
    }

    #[test]
    fn merge_state_parses_github_vocabulary() {
        assert_eq!(MergeState::from_forge_state("clean"), MergeState::Clean);
        assert_eq!(MergeState::from_forge_state("behind"), MergeState::Behind);
        assert_eq!(MergeState::from_forge_state("dirty"), MergeState::Dirty);
        assert_eq!(
            MergeState::from_forge_state("unstable"),
            MergeState::Unstable
        );
        assert_eq!(MergeState::from_forge_state("blocked"), MergeState::Blocked);
        assert_eq!(
            MergeState::from_forge_state("surprise"),
            MergeState::Unknown
        );
    }

    #[test]
    fn review_state_parses_correctly() {
        assert_eq!(
            ReviewState::from_forge_state("APPROVED"),
            ReviewState::Approved
        );
        assert_eq!(
            ReviewState::from_forge_state("REQUEST_CHANGES"),
            ReviewState::RequestChanges
        );
        assert_eq!(
            ReviewState::from_forge_state("CHANGES_REQUESTED"),
            ReviewState::RequestChanges
        );
        assert_eq!(
            ReviewState::from_forge_state("COMMENT"),
            ReviewState::Comment
        );
        assert_eq!(
            ReviewState::from_forge_state("DISMISSED"),
            ReviewState::Dismissed
        );
        assert_eq!(
            ReviewState::from_forge_state("unknown_thing"),
            ReviewState::Unknown
        );
    }

    #[test]
    fn commit_status_state_parses_correctly() {
        assert_eq!(
            CommitStatusState::from_forge_state("success"),
            CommitStatusState::Success
        );
        assert_eq!(
            CommitStatusState::from_forge_state("pending"),
            CommitStatusState::Pending
        );
        assert_eq!(
            CommitStatusState::from_forge_state("failure"),
            CommitStatusState::Failure
        );
        assert_eq!(
            CommitStatusState::from_forge_state("error"),
            CommitStatusState::Error
        );
        assert_eq!(
            CommitStatusState::from_forge_state("unknown"),
            CommitStatusState::Missing
        );
    }

    // --- status is evaluated before approvals (ordering) ---

    #[test]
    fn status_check_precedes_approval_check() {
        // Even with no approvals, the first error should be StatusNotSuccess.
        let cfg = default_config();
        let pr = approved_pr();
        let reviews: Vec<Review> = vec![];
        let input = EligibilityInput {
            pr: &pr,
            status_state: CommitStatusState::Failure,
            reviews: &reviews,
            config: &cfg,
        };
        assert!(matches!(
            is_mergeable(&input),
            Err(IneligibleReason::StatusNotSuccess { .. })
        ));
    }
}
