//! # oya-ci-tide-kernel
//!
//! Pure-domain kernel for the oya-ci tide component (Phase 2, ADR-0513).
//! No I/O, no async, no network. #![forbid(unsafe_code)].
//!
//! Owns:
//! - [`TideConfig`] — resolved runtime configuration
//! - [`PullRequest`] / [`CommitStatusState`] / [`Review`] / [`MergeMethod`] —
//!   Forgejo API projection types
//! - [`is_mergeable`] — the eligibility predicate (THE core logic)
//! - [`ForgejoClient`] trait seam — I/O boundary for the adapter layer
//!
//! ## Eligibility invariant (ADR-0513 tide / ADR-0111)
//!
//! A PR is merge-eligible iff ALL of:
//! 1. The configured `required_status_context` has state `success` on the HEAD SHA.
//! 2. The number of approving reviews >= `approval_policy.min_approvals`.
//! 3. Forgejo reports the PR as mergeable (no conflicts).
//! 4. No blocking label (`hold` / `do-not-merge`) is present.
//!
//! ## Security
//!
//! - `dry_run` defaults to `true` — tide merges NOTHING until explicitly configured live.
//! - Token never hardcoded; always read from `OYA_FORGEJO_TOKEN`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// All tide kernel errors. HTTP / reqwest details live in the adapter layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TideError {
    /// A downstream Forgejo API call failed (transport or non-2xx).
    Downstream(String),
    /// A required field was missing or malformed.
    InvalidInput(String),
}

impl std::fmt::Display for TideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TideError::Downstream(why) => write!(f, "forgejo downstream error: {why}"),
            TideError::InvalidInput(why) => write!(f, "invalid input: {why}"),
        }
    }
}

impl std::error::Error for TideError {}

pub type Result<T> = std::result::Result<T, TideError>;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Env var carrying the Forgejo API token (injected from the deploy substrate).
/// Never hardcoded.
pub const ENV_FORGEJO_TOKEN: &str = "OYA_FORGEJO_TOKEN";

/// Default Forgejo base URL (in-cluster).
pub const DEFAULT_FORGEJO_BASE_URL: &str = "http://forgejo.oya-forge.svc.cluster.local:3000";

/// Default repository owner.
pub const DEFAULT_REPO_OWNER: &str = "oya-admin";

/// Default repository name.
pub const DEFAULT_REPO_NAME: &str = "oyatie";

/// Default base branch that tide manages.
pub const DEFAULT_BASE_BRANCH: &str = "dev";

/// Default required commit-status context (must match branch-protection rule).
pub const DEFAULT_REQUIRED_STATUS_CONTEXT: &str = "oya-ci-gate";

/// Default minimum approving reviews required.
pub const DEFAULT_MIN_APPROVALS: u32 = 1;

/// Default poll interval in seconds.
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 60;

/// Labels that block merge, case-insensitive prefix match.
pub const BLOCKING_LABEL_PREFIXES: &[&str] = &["hold", "do-not-merge"];

/// Merge methods supported by the Forgejo merge API.
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
    /// The string value Forgejo's API expects in `Do` field.
    pub const fn as_str(self) -> &'static str {
        match self {
            MergeMethod::Merge => "merge",
            MergeMethod::Rebase => "rebase",
            MergeMethod::Squash => "squash",
        }
    }

    /// Parse from env-var string. Unrecognised / blank → `Rebase`.
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "merge" => MergeMethod::Merge,
            "squash" => MergeMethod::Squash,
            _ => MergeMethod::Rebase,
        }
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
    /// Forgejo API base URL (e.g. `http://forgejo.oya-forge.svc.cluster.local:3000`).
    pub forgejo_base_url: String,
    /// Repository owner (e.g. `oya-admin`).
    pub repo_owner: String,
    /// Repository name (e.g. `oyatie`).
    pub repo_name: String,
    /// Base branch to poll PRs against (default: `dev`).
    pub base_branch: String,
    /// Required commit-status context that must be `success` (default: `oya-ci-gate`).
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
        let forgejo_base_url = get("OYA_TIDE_FORGEJO_BASE_URL")
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_FORGEJO_BASE_URL.to_owned());
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
            .map(MergeMethod::from_str)
            .unwrap_or(MergeMethod::Rebase);
        // dry_run defaults to true; must be explicitly "false" to enable live merging.
        let dry_run = get("OYA_TIDE_DRY_RUN")
            .map(|v| v.trim().to_ascii_lowercase() != "false")
            .unwrap_or(true);

        TideConfig {
            forgejo_base_url,
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
// Forgejo API projection types
// ---------------------------------------------------------------------------

/// Projected pull-request data from Forgejo API.
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
    /// Whether Forgejo considers the PR mergeable (no conflicts).
    /// `None` means Forgejo hasn't computed it yet (still processing).
    pub mergeable: Option<bool>,
    /// Labels on the PR.
    pub labels: Vec<String>,
}

/// Combined commit-status state from Forgejo
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
    /// Parse from Forgejo's state string.
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "success" => CommitStatusState::Success,
            "pending" => CommitStatusState::Pending,
            "failure" => CommitStatusState::Failure,
            "error" => CommitStatusState::Error,
            _ => CommitStatusState::Missing,
        }
    }
}

/// A single review from Forgejo's pull-request reviews API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Review {
    /// Reviewer login.
    pub reviewer: String,
    /// Review state (`APPROVED`, `REQUEST_CHANGES`, `COMMENT`, etc.).
    pub state: ReviewState,
}

/// Review state vocabulary (Forgejo mirrors GitHub's).
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
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_uppercase().as_str() {
            "APPROVED" => ReviewState::Approved,
            "REQUEST_CHANGES" | "CHANGES_REQUESTED" => ReviewState::RequestChanges,
            "COMMENT" => ReviewState::Comment,
            "DISMISSED" => ReviewState::Dismissed,
            _ => ReviewState::Unknown,
        }
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
    /// Forgejo reports the PR is not mergeable (conflicts).
    NotMergeable,
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
/// 3. `pr.mergeable == Some(true)` (not `None` or `Some(false)`).
/// 4. No label matches a blocking prefix (`hold`, `do-not-merge`), case-insensitive.
pub fn is_mergeable(input: &EligibilityInput<'_>) -> std::result::Result<(), IneligibleReason> {
    // Rule 1 — CI status must be success.
    if input.status_state != CommitStatusState::Success {
        return Err(IneligibleReason::StatusNotSuccess {
            actual: input.status_state,
        });
    }

    // Rule 2 — sufficient approving reviews.
    let approvals = input
        .reviews
        .iter()
        .filter(|r| r.state == ReviewState::Approved)
        .count() as u32;
    if approvals < input.config.approval_policy.min_approvals {
        return Err(IneligibleReason::InsufficientApprovals {
            actual: approvals,
            required: input.config.approval_policy.min_approvals,
        });
    }

    // Rule 3 — PR must be mergeable (no conflicts).
    if input.pr.mergeable != Some(true) {
        return Err(IneligibleReason::NotMergeable);
    }

    // Rule 4 — no blocking labels.
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

    Ok(())
}

// ---------------------------------------------------------------------------
// ForgejoClient trait seam — I/O boundary
// ---------------------------------------------------------------------------

/// Forgejo API client seam. Implemented by `oya-ci-tide-forgejo-adapter`.
/// All methods are synchronous (adapter wraps reqwest blocking or spawns
/// via `tokio::task::spawn_blocking`).
pub trait ForgejoClient: Send + Sync {
    /// List open pull requests against `base_branch`.
    fn list_open_pulls(&self, base_branch: &str) -> Result<Vec<PullRequest>>;

    /// Get the combined commit status for the given SHA, filtered to the
    /// `required_context`. Returns `CommitStatusState::Missing` when no
    /// status exists for that context.
    fn get_commit_status(
        &self,
        sha: &str,
        required_context: &str,
    ) -> Result<CommitStatusState>;

    /// List all reviews on a pull request.
    fn list_reviews(&self, pr_number: u64) -> Result<Vec<Review>>;

    /// Get a single pull request (to refresh `mergeable` field).
    fn get_pull(&self, pr_number: u64) -> Result<PullRequest>;

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
            forgejo_base_url: DEFAULT_FORGEJO_BASE_URL.to_owned(),
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
            labels: vec![],
        }
    }

    fn approvals(n: usize) -> Vec<Review> {
        (0..n)
            .map(|i| Review {
                reviewer: format!("reviewer-{i}"),
                state: ReviewState::Approved,
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
            },
            Review {
                reviewer: "bob".to_owned(),
                state: ReviewState::RequestChanges,
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
            assert!(cfg.dry_run, "expected dry_run=true for OYA_TIDE_DRY_RUN={val}");
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
        assert_eq!(MergeMethod::from_str("merge"), MergeMethod::Merge);
        assert_eq!(MergeMethod::from_str("squash"), MergeMethod::Squash);
        assert_eq!(MergeMethod::from_str("rebase"), MergeMethod::Rebase);
        assert_eq!(MergeMethod::from_str("bogus"), MergeMethod::Rebase);
        assert_eq!(MergeMethod::from_str(""), MergeMethod::Rebase);
    }

    #[test]
    fn review_state_parses_correctly() {
        assert_eq!(ReviewState::from_str("APPROVED"), ReviewState::Approved);
        assert_eq!(
            ReviewState::from_str("REQUEST_CHANGES"),
            ReviewState::RequestChanges
        );
        assert_eq!(
            ReviewState::from_str("CHANGES_REQUESTED"),
            ReviewState::RequestChanges
        );
        assert_eq!(ReviewState::from_str("COMMENT"), ReviewState::Comment);
        assert_eq!(ReviewState::from_str("DISMISSED"), ReviewState::Dismissed);
        assert_eq!(ReviewState::from_str("unknown_thing"), ReviewState::Unknown);
    }

    #[test]
    fn commit_status_state_parses_correctly() {
        assert_eq!(
            CommitStatusState::from_str("success"),
            CommitStatusState::Success
        );
        assert_eq!(
            CommitStatusState::from_str("pending"),
            CommitStatusState::Pending
        );
        assert_eq!(
            CommitStatusState::from_str("failure"),
            CommitStatusState::Failure
        );
        assert_eq!(
            CommitStatusState::from_str("error"),
            CommitStatusState::Error
        );
        assert_eq!(
            CommitStatusState::from_str("unknown"),
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
