//! Fix-at-any-stage re-validation handler — ADR-0111 wave-A.
//!
//! When IP-005 (or any agent) pushes a new commit to a PR branch
//! while that PR is already queued, the merge-queue receives a
//! `pull_request.synchronize` webhook (per ADR-0112). This module
//! implements the re-validation protocol:
//!
//! 1. **Detect** the queued PR by `pr_number`.
//! 2. **Invalidate** every queued PR at position ≥ i.
//! 3. **Re-validate** in order — for each position j ∈ [i..n],
//!    re-run the projected-merge-state check; tests re-fire against
//!    the new projected base (the runner enqueues a CI workflow run,
//!    not modelled by this module).
//! 4. **Re-position**: if the candidate's new diff now overlaps an
//!    earlier queued PR that previously didn't overlap, push the PR
//!    back behind `PR_{i+1}`. Per-PR `MAX_REPOSITION = 3` cap; on the
//!    4th re-position the candidate transitions to `parked` (human
//!    review required).
//!
//! `MAX_REPOSITION` is `pub const`; ADR-0111 §"Fairness invariants"
//! locks it at 3.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::projected_merge_state::{
    GitMergeTreeRunner, ProjectedMergeStateRunError, ProjectedStateError, QueuedPr,
    run_projected_merge_state_check,
};

/// ADR-0111 §"Fairness invariants" — per-PR cap on how many times a
/// PR can be pushed back during fix-at-any-stage re-validation before
/// the queue parks it for human review (no starvation guarantee).
pub const MAX_REPOSITION: u32 = 3;

/// Outcome of the fix-at-any-stage re-validation pass for a single
/// `pull_request.synchronize` event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevalidateOutcome {
    /// Every position ≥ i re-validated cleanly; the queue order is
    /// unchanged. `revalidated_positions` lists which positions ran
    /// the check (for audit / convergence-proof).
    AllAdmitted { revalidated_positions: Vec<u64> },
    /// The candidate now overlaps an earlier-queued PR; reposition
    /// has been applied (and tracked against `MAX_REPOSITION`). The
    /// runner re-orders the queue and emits the per-tick evidence
    /// file.
    Repositioned {
        candidate_pr_number: u64,
        new_reposition_count: u32,
    },
    /// The candidate already hit `MAX_REPOSITION` — the runner must
    /// transition the PR to `parked` state (per
    /// `oya-vcs-review-mergequeue-kernel::ParkedReason::
    /// ChangesRequested`, or a new RepositionLimitExceeded variant if
    /// the kernel chooses to extend its closed enum). For now this
    /// variant carries the PR number and reposition count so the
    /// runner can match its parking-reason emission accordingly.
    ParkedForRepositionExhaustion {
        candidate_pr_number: u64,
        reposition_count: u32,
    },
    /// The kernel refused the candidate outright (conflict detected
    /// or malformed sha) — the runner cancels the candidate's queue
    /// entry and writes evidence with the refusal reason.
    RefusedByKernel {
        candidate_pr_number: u64,
        reason: ProjectedMergeStateRunError,
    },
}

/// Input for one `pull_request.synchronize` event.
#[derive(Clone, Debug)]
pub struct SynchronizeEvent {
    pub pr_number: u64,
    pub new_head_sha: String,
    /// Updated `touched_paths` after the new commit. The runner
    /// extracts via `git diff --name-only <base>..<new_head>`.
    pub new_touched_paths: Vec<String>,
}

/// State the runner threads through repeated re-validations: maps
/// pr_number → cumulative reposition count.
pub type RepositionCounts = BTreeMap<u64, u32>;

/// Handle one `pull_request.synchronize` event against the current
/// queue. The runner is responsible for persisting `reposition_counts`
/// across calls (so a PR that's been pushed back 3 times stays
/// pushed-back-3 across process restarts).
///
/// `queue` is the ordered slice (position 0 = head of queue). The
/// candidate's position is looked up by `pr_number`; if not found we
/// return a no-op `AllAdmitted` (the synchronize fired on a PR that
/// isn't queued, which is fine).
pub fn handle_synchronize_event(
    runner: &dyn GitMergeTreeRunner,
    dev_head: &str,
    queue: &mut [QueuedPr],
    event: &SynchronizeEvent,
    concurrent_safe_paths: &BTreeSet<String>,
    reposition_counts: &mut RepositionCounts,
) -> RevalidateOutcome {
    // Step 1: detect the candidate's position.
    let Some(idx) = queue.iter().position(|p| p.pr_number == event.pr_number) else {
        // PR not queued — synchronize webhook may fire for any PR;
        // treat as a clean no-op.
        return RevalidateOutcome::AllAdmitted {
            revalidated_positions: Vec::new(),
        };
    };

    // Step 2: apply the synchronize update locally (new head sha +
    // new touched paths). We mutate the queue slot in place.
    queue[idx].head_sha = event.new_head_sha.clone();
    queue[idx].new_touched_paths_replace(&event.new_touched_paths);

    // Step 3: re-validate from idx outward. For each position j, the
    // prior PRs (indices < j) form the queued context; the PR at j
    // is the candidate.
    let mut revalidated = Vec::new();
    for j in idx..queue.len() {
        let (head, tail) = queue.split_at(j);
        let candidate = tail
            .first()
            .cloned()
            .unwrap_or_else(unreachable_queue_split);
        match run_projected_merge_state_check(
            runner,
            dev_head,
            head,
            &candidate,
            concurrent_safe_paths,
        ) {
            Ok(_report) => {
                revalidated.push(candidate.pr_number);
            }
            Err(ProjectedMergeStateRunError::KernelRefused(
                ProjectedStateError::PathOverlapRefused { .. },
            )) => {
                // Reposition: bump the counter; if exceeded, park.
                let count = reposition_counts.entry(candidate.pr_number).or_insert(0);
                *count = count.saturating_add(1);
                if *count > MAX_REPOSITION {
                    return RevalidateOutcome::ParkedForRepositionExhaustion {
                        candidate_pr_number: candidate.pr_number,
                        reposition_count: *count,
                    };
                }
                return RevalidateOutcome::Repositioned {
                    candidate_pr_number: candidate.pr_number,
                    new_reposition_count: *count,
                };
            }
            Err(other) => {
                return RevalidateOutcome::RefusedByKernel {
                    candidate_pr_number: candidate.pr_number,
                    reason: other,
                };
            }
        }
    }

    RevalidateOutcome::AllAdmitted {
        revalidated_positions: revalidated,
    }
}

/// Helper: replace touched_paths from `new_touched_paths` slice. Lives
/// on `QueuedPr` as a free fn via this trait so we don't reach into
/// the kernel-owned struct for mutation (the kernel struct stays a
/// data carrier; mutation logic lives runner-side).
trait QueuedPrMutate {
    fn new_touched_paths_replace(&mut self, paths: &[String]);
}

impl QueuedPrMutate for QueuedPr {
    fn new_touched_paths_replace(&mut self, paths: &[String]) {
        self.touched_paths = paths.to_vec();
    }
}

/// Tier-1-safe "should be unreachable" marker. We return a synthetic
/// QueuedPr with an obviously invalid pr_number (0) and empty paths
/// so the calling loop completes without panicking; the wrapping
/// match arm treats this as a no-op since pr_number 0 will fail the
/// sha check downstream.
///
/// In practice this branch is unreachable because we constrain
/// `j ∈ [idx..queue.len())` so `tail.first()` is always `Some`. The
/// helper exists only because the compiler doesn't see that the
/// split-at index is in-bounds.
fn unreachable_queue_split() -> QueuedPr {
    QueuedPr {
        pr_number: 0,
        head_sha: "0".repeat(40),
        base_sha: "0".repeat(40),
        touched_paths: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubRunner {
        out: String,
    }
    impl GitMergeTreeRunner for StubRunner {
        fn run_merge_tree(&self, _b: &str, _o: &str, _t: &str) -> Result<String, String> {
            Ok(self.out.clone())
        }
    }

    fn sha(c: char) -> String {
        std::iter::repeat_n(c, 40).collect()
    }

    fn pr(number: u64, head_c: char, paths: &[&str]) -> QueuedPr {
        QueuedPr {
            pr_number: number,
            head_sha: sha(head_c),
            base_sha: sha('0'),
            touched_paths: paths.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    #[test]
    fn synchronize_for_unqueued_pr_is_noop() {
        let runner = StubRunner { out: "".into() };
        let mut queue: Vec<QueuedPr> = vec![pr(1, 'a', &["a.rs"])];
        let mut counts: RepositionCounts = BTreeMap::new();
        let ev = SynchronizeEvent {
            pr_number: 999,
            new_head_sha: sha('z'),
            new_touched_paths: vec!["z.rs".into()],
        };
        let outcome = handle_synchronize_event(
            &runner,
            &sha('0'),
            &mut queue,
            &ev,
            &BTreeSet::new(),
            &mut counts,
        );
        match outcome {
            RevalidateOutcome::AllAdmitted {
                revalidated_positions,
            } => assert!(revalidated_positions.is_empty()),
            other => panic!("expected no-op AllAdmitted, got {other:?}"),
        }
    }

    #[test]
    fn synchronize_re_validates_clean_position() {
        let runner = StubRunner { out: "".into() };
        let mut queue: Vec<QueuedPr> = vec![pr(1, 'a', &["a.rs"]), pr(2, 'b', &["b.rs"])];
        let mut counts: RepositionCounts = BTreeMap::new();
        let ev = SynchronizeEvent {
            pr_number: 2,
            new_head_sha: sha('c'),
            new_touched_paths: vec!["b.rs".into()],
        };
        let outcome = handle_synchronize_event(
            &runner,
            &sha('0'),
            &mut queue,
            &ev,
            &BTreeSet::new(),
            &mut counts,
        );
        match outcome {
            RevalidateOutcome::AllAdmitted {
                revalidated_positions,
            } => assert_eq!(revalidated_positions, vec![2]),
            other => panic!("expected AllAdmitted, got {other:?}"),
        }
        // The runner-side mutation took effect.
        assert_eq!(queue[1].head_sha, sha('c'));
    }

    #[test]
    fn synchronize_overlap_triggers_reposition() {
        // PR#1 originally touched a.rs, PR#2 touched b.rs.
        // Synchronize pushes PR#2's diff onto a.rs (overlapping #1).
        // Expected: Repositioned, new_reposition_count=1.
        let runner = StubRunner { out: "".into() };
        let mut queue: Vec<QueuedPr> = vec![pr(1, 'a', &["a.rs"]), pr(2, 'b', &["b.rs"])];
        let mut counts: RepositionCounts = BTreeMap::new();
        let ev = SynchronizeEvent {
            pr_number: 2,
            new_head_sha: sha('c'),
            new_touched_paths: vec!["a.rs".into()],
        };
        let outcome = handle_synchronize_event(
            &runner,
            &sha('0'),
            &mut queue,
            &ev,
            &BTreeSet::new(),
            &mut counts,
        );
        match outcome {
            RevalidateOutcome::Repositioned {
                candidate_pr_number,
                new_reposition_count,
            } => {
                assert_eq!(candidate_pr_number, 2);
                assert_eq!(new_reposition_count, 1);
            }
            other => panic!("expected Repositioned, got {other:?}"),
        }
        assert_eq!(counts.get(&2).copied(), Some(1));
    }

    #[test]
    fn fourth_reposition_parks_the_pr() {
        // ADR-0111 §"Fairness invariants": MAX_REPOSITION = 3. The
        // FOURTH overlap (count would become 4 > 3) parks the PR.
        let runner = StubRunner { out: "".into() };
        let mut queue: Vec<QueuedPr> = vec![pr(1, 'a', &["a.rs"]), pr(2, 'b', &["b.rs"])];
        let mut counts: RepositionCounts = BTreeMap::new();
        counts.insert(2, MAX_REPOSITION); // already at the cap
        let ev = SynchronizeEvent {
            pr_number: 2,
            new_head_sha: sha('c'),
            new_touched_paths: vec!["a.rs".into()],
        };
        let outcome = handle_synchronize_event(
            &runner,
            &sha('0'),
            &mut queue,
            &ev,
            &BTreeSet::new(),
            &mut counts,
        );
        match outcome {
            RevalidateOutcome::ParkedForRepositionExhaustion {
                candidate_pr_number,
                reposition_count,
            } => {
                assert_eq!(candidate_pr_number, 2);
                assert!(reposition_count > MAX_REPOSITION);
            }
            other => panic!("expected ParkedForRepositionExhaustion, got {other:?}"),
        }
    }

    #[test]
    fn synchronize_with_kernel_conflict_refusal_propagates() {
        let runner = StubRunner {
            out: "a.rs\n<<<<<<<\n=======\n>>>>>>>\n".into(),
        };
        let mut queue: Vec<QueuedPr> = vec![pr(1, 'a', &["a.rs"])];
        let mut counts: RepositionCounts = BTreeMap::new();
        let ev = SynchronizeEvent {
            pr_number: 1,
            new_head_sha: sha('b'),
            new_touched_paths: vec!["a.rs".into()],
        };
        let outcome = handle_synchronize_event(
            &runner,
            &sha('0'),
            &mut queue,
            &ev,
            &BTreeSet::new(),
            &mut counts,
        );
        assert!(matches!(outcome, RevalidateOutcome::RefusedByKernel { .. }));
    }

    #[test]
    fn max_reposition_constant_locked_to_three() {
        // ADR-0111 lock-in.
        assert_eq!(MAX_REPOSITION, 3);
    }
}
