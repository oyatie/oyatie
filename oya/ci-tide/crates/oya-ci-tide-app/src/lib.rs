//! # oya-ci-tide-app
//!
//! Tide merge-queue loop for the oya-ci platform (Phase 2, ADR-0513).
//!
//! ## Responsibilities
//!
//! - Poll GitHub every `config.poll_interval_secs` for open PRs against the
//!   configured `base_branch`.
//! - For each open PR: fetch commit status + reviews + fresh mergeable state.
//! - Evaluate eligibility via `kernel::is_mergeable`.
//! - When stale-but-approved: refresh the PR branch and wait for
//!   `oya-ci-required` to rerun on the new head before merging.
//! - When eligible and `dry_run == true`: log `"WOULD MERGE #<n>"`.
//! - When eligible and `dry_run == false`: call `client.merge_pull(n, method)`.
//!
//! ## Safety default
//!
//! `dry_run` defaults to `true` in [`TideConfig`]. Tide merges NOTHING until
//! `OYA_TIDE_DRY_RUN=false` is explicitly set in the deploy substrate.
//!
//! ## Deferred (Phase 2+)
//!
//! - Batching (merge multiple eligible PRs atomically)
//! - Speculative retest (re-run gate on the projected merge commit before merging)
//! - Deploy manifests (Helm chart / ExternalSecret / NetworkPolicy)
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the poll loop path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use ci_tide_kernel::{
    EligibilityInput, ForgeClient, IneligibleReason, TideConfig, TideError, is_mergeable,
};
use std::sync::Arc;
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// TideRunner
// ---------------------------------------------------------------------------

/// The tide merge-queue runner.
///
/// Holds an [`Arc`]-wrapped [`ForgeClient`] so it can be shared across the
/// blocking tasks spawned inside the async poll loop.
pub struct TideRunner {
    config: TideConfig,
    client: Arc<dyn ForgeClient>,
}

impl TideRunner {
    /// Create a new runner with the given config and client.
    pub fn new(config: TideConfig, client: Arc<dyn ForgeClient>) -> Self {
        TideRunner { config, client }
    }

    /// Run one poll cycle: list open PRs, evaluate each, merge eligible ones.
    ///
    /// Returns the number of PRs that were merged (or would-be-merged in
    /// dry-run mode). Errors fetching individual PRs are logged and skipped
    /// rather than aborting the whole cycle.
    pub fn run_once(&self) -> u32 {
        let pulls = match self.client.list_open_pulls(&self.config.base_branch) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    base_branch = %self.config.base_branch,
                    error = %e,
                    "tide: failed to list open PRs — skipping cycle"
                );
                return 0;
            }
        };

        info!(
            base_branch = %self.config.base_branch,
            count = pulls.len(),
            dry_run = self.config.dry_run,
            "tide: poll cycle"
        );

        let mut acted = 0u32;

        for pr in &pulls {
            // Fetch the fresh PR (mergeable can be null initially; re-fetch to
            // get a resolved value after GitHub finishes computing it).
            let fresh_pr = match self.client.get_pull(pr.number) {
                Ok(p) => p,
                Err(e) => {
                    warn!(pr = pr.number, error = %e, "tide: failed to refresh PR — skipping");
                    continue;
                }
            };

            let status_state = match self
                .client
                .get_commit_status(&fresh_pr.head_sha, &self.config.required_status_context)
            {
                Ok(s) => s,
                Err(e) => {
                    warn!(
                        pr = pr.number,
                        sha = %fresh_pr.head_sha,
                        error = %e,
                        "tide: failed to fetch commit status — skipping"
                    );
                    continue;
                }
            };

            let reviews = match self.client.list_reviews(pr.number) {
                Ok(r) => r,
                Err(e) => {
                    warn!(
                        pr = pr.number,
                        error = %e,
                        "tide: failed to fetch reviews — skipping"
                    );
                    continue;
                }
            };

            let input = EligibilityInput {
                pr: &fresh_pr,
                status_state,
                reviews: &reviews,
                config: &self.config,
            };

            match is_mergeable(&input) {
                Ok(()) => {
                    acted += 1;
                    if self.config.dry_run {
                        info!(
                            pr = pr.number,
                            title = %fresh_pr.title,
                            sha = %fresh_pr.head_sha,
                            method = %self.config.merge_method.as_str(),
                            "tide: WOULD MERGE #{} (dry_run=true)",
                            pr.number
                        );
                    } else {
                        info!(
                            pr = pr.number,
                            title = %fresh_pr.title,
                            sha = %fresh_pr.head_sha,
                            method = %self.config.merge_method.as_str(),
                            "tide: merging #{}",
                            pr.number
                        );
                        match self.client.merge_pull(pr.number, self.config.merge_method) {
                            Ok(()) => {
                                info!(pr = pr.number, "tide: merged #{}", pr.number);
                            }
                            Err(TideError::Downstream(why)) => {
                                error!(
                                    pr = pr.number,
                                    error = %why,
                                    "tide: merge failed — will retry on next cycle"
                                );
                                acted -= 1; // Don't count failed merges.
                            }
                            Err(e) => {
                                error!(
                                    pr = pr.number,
                                    error = %e,
                                    "tide: merge error"
                                );
                                acted -= 1;
                            }
                        }
                    }
                }
                Err(IneligibleReason::StaleBase) => {
                    acted += 1;
                    if self.config.dry_run {
                        info!(
                            pr = pr.number,
                            title = %fresh_pr.title,
                            sha = %fresh_pr.head_sha,
                            "tide: WOULD REFRESH stale base for #{} (dry_run=true)",
                            pr.number
                        );
                    } else {
                        info!(
                            pr = pr.number,
                            title = %fresh_pr.title,
                            sha = %fresh_pr.head_sha,
                            "tide: refreshing stale base for #{} before merge eligibility",
                            pr.number
                        );
                        match self.client.update_branch(pr.number, &fresh_pr.head_sha) {
                            Ok(()) => {
                                info!(
                                    pr = pr.number,
                                    "tide: stale branch refresh requested for #{}; waiting for oya-ci-required on the new head",
                                    pr.number
                                );
                            }
                            Err(TideError::Downstream(why)) => {
                                error!(
                                    pr = pr.number,
                                    error = %why,
                                    "tide: stale branch refresh failed — will retry on next cycle"
                                );
                                acted -= 1;
                            }
                            Err(e) => {
                                error!(
                                    pr = pr.number,
                                    error = %e,
                                    "tide: stale branch refresh error"
                                );
                                acted -= 1;
                            }
                        }
                    }
                }
                Err(reason) => {
                    info!(
                        pr = pr.number,
                        reason = %reason,
                        "tide: PR #{} not eligible",
                        pr.number
                    );
                }
            }
        }

        acted
    }

    /// Run the tide loop indefinitely, sleeping `poll_interval_secs` between cycles.
    ///
    /// Blocks the calling thread. Call from a `tokio::task::spawn_blocking` or
    /// a dedicated OS thread.
    pub fn run_loop(&self) {
        let interval = std::time::Duration::from_secs(self.config.poll_interval_secs);
        info!(
            poll_interval_secs = self.config.poll_interval_secs,
            dry_run = self.config.dry_run,
            base_branch = %self.config.base_branch,
            "tide: starting merge-queue loop"
        );
        loop {
            self.run_once();
            std::thread::sleep(interval);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ci_tide_kernel::{
        ApprovalPolicy, CommitStatusState, MergeMethod, MergeState, PullRequest, Result, Review,
        ReviewState,
    };
    use std::sync::Mutex;

    struct MockForgeClient {
        pulls: Vec<PullRequest>,
        fresh_pull: PullRequest,
        status: CommitStatusState,
        reviews: Vec<Review>,
        merged: Mutex<Vec<u64>>,
        updated: Mutex<Vec<(u64, String)>>,
    }

    impl ForgeClient for MockForgeClient {
        fn list_open_pulls(&self, _base_branch: &str) -> Result<Vec<PullRequest>> {
            Ok(self.pulls.clone())
        }

        fn get_commit_status(
            &self,
            _sha: &str,
            _required_context: &str,
        ) -> Result<CommitStatusState> {
            Ok(self.status)
        }

        fn list_reviews(&self, _pr_number: u64) -> Result<Vec<Review>> {
            Ok(self.reviews.clone())
        }

        fn get_pull(&self, _pr_number: u64) -> Result<PullRequest> {
            Ok(self.fresh_pull.clone())
        }

        fn update_branch(&self, pr_number: u64, expected_head_sha: &str) -> Result<()> {
            self.updated
                .lock()
                .expect("updated mutex poisoned")
                .push((pr_number, expected_head_sha.to_owned()));
            Ok(())
        }

        fn merge_pull(&self, pr_number: u64, _method: MergeMethod) -> Result<()> {
            self.merged
                .lock()
                .expect("merged mutex poisoned")
                .push(pr_number);
            Ok(())
        }
    }

    fn config(dry_run: bool) -> TideConfig {
        TideConfig {
            forge_base_url: "https://api.github.com".to_owned(),
            repo_owner: "jason931225".to_owned(),
            repo_name: "oyatie".to_owned(),
            base_branch: "dev".to_owned(),
            required_status_context: "oya-ci-required".to_owned(),
            approval_policy: ApprovalPolicy { min_approvals: 1 },
            poll_interval_secs: 60,
            merge_method: MergeMethod::Rebase,
            dry_run,
        }
    }

    fn pr(merge_state: MergeState) -> PullRequest {
        PullRequest {
            number: 7,
            title: "ci: tide stale refresh".to_owned(),
            head_sha: "abc123".to_owned(),
            base_ref: "dev".to_owned(),
            mergeable: Some(true),
            merge_state,
            labels: vec![],
        }
    }

    fn approval() -> Review {
        Review {
            reviewer: "reviewer".to_owned(),
            state: ReviewState::Approved,
            submitted_at: Some("2026-06-30T00:00:00Z".to_owned()),
            id: Some(1),
            api_order: 0,
        }
    }

    #[test]
    fn stale_approved_green_pr_refreshes_branch_instead_of_merging() {
        let stale = pr(MergeState::Behind);
        let client = Arc::new(MockForgeClient {
            pulls: vec![stale.clone()],
            fresh_pull: stale,
            status: CommitStatusState::Success,
            reviews: vec![approval()],
            merged: Mutex::new(Vec::new()),
            updated: Mutex::new(Vec::new()),
        });
        let runner = TideRunner::new(config(false), client.clone());

        assert_eq!(runner.run_once(), 1);
        assert_eq!(
            *client.updated.lock().expect("updated mutex poisoned"),
            vec![(7, "abc123".to_owned())]
        );
        assert!(
            client
                .merged
                .lock()
                .expect("merged mutex poisoned")
                .is_empty()
        );
    }
}
