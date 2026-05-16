//! Projected-merge-state conflict kernel — ADR-0111 wave-A.
//!
//! This crate is the canonical port-in-kernel for the merge-queue
//! projected-merge-state algorithm. It models:
//!
//! - [`QueuedPr`] — the minimum PR shape the validator needs: PR
//!   number, head sha, base sha, and the set of touched paths.
//! - [`MergeTreeOutcome`] — the parsed verdict produced by the runner
//!   from `git merge-tree dev_head pr.head dev_head` invocation (the
//!   runner side-effects live outside this kernel).
//! - [`validate_projected_merge_state`] — the pure-function validator
//!   that asserts the THREE invariants from ADR-0111 §"Projected-merge-
//!   state simulation":
//!
//!     1. **Diff cleanliness** — no conflict markers in the projected
//!        merge of `dev_head + accumulated_PRs + candidate`.
//!     2. **Path-overlap check** — candidate's `touched_paths` must
//!        not intersect any prior queued PR's `touched_paths` unless
//!        the runner has whitelisted the overlap via the
//!        `concurrent-safe-paths.yaml` registry (which is registered
//!        per-product; this kernel accepts a pre-resolved set of safe
//!        paths so the registry stays out of the kernel).
//!     3. **SHA well-formedness** — every input sha must be 40 lowercase
//!        hex characters (a kernel-side defensive contract; the runner
//!        normalises before calling).
//!
//! Invariant 3 from the ADR text ("Test re-run against projected
//! base") is a runner-side concern (it triggers a CI workflow run);
//! the kernel does not model the CI state machine — it only emits the
//! `requires_test_rerun: true` flag in [`ProjectedStateReport`] so the
//! runner knows to enqueue a re-run for the candidate.
//!
//! ADR-0056 port-in-kernel discipline: this crate is pure-domain. No
//! I/O, no clock, no randomness, no shelling out to git, no external
//! deps beyond `std`. The companion
//! `oya-foundry-vcs-merge-queue-fix-loop-app` binary owns the
//! filesystem, `git merge-tree` invocation, and webhook surfaces.
//!
//! ADR-0083 Tier 1: production code is `unwrap`/`expect`/`panic` free.
//! Tests legitimately use these primitives under the `cfg(test)`
//! exemption.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

/// A queued PR's minimum projection for projected-merge-state
/// validation.
///
/// `touched_paths` is the union of repository-relative paths the PR
/// edits (the runner extracts via `git diff --name-only`). The kernel
/// uses `BTreeSet`-flavoured semantics under the hood (the public
/// field stays `Vec<String>` so callers can build the type in stable
/// order); duplicates inside one PR are tolerated and de-duplicated
/// before the path-overlap check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedPr {
    pub pr_number: u64,
    pub head_sha: String,
    pub base_sha: String,
    pub touched_paths: Vec<String>,
}

/// The parsed verdict from the runner's `git merge-tree` invocation
/// for a single (projected-base, candidate-head) pair. Two variants:
///
/// - `Clean` — no conflict markers found in `merge-tree` stdout; the
///   candidate's projected diff applies cleanly.
/// - `Conflict { paths }` — at least one conflicted path; the runner
///   should have surfaced the offending file list (it scans for
///   `<<<<<<<` style markers and groups by hunk).
///
/// The kernel never executes git; this enum is the contract between
/// runner-side parsing and kernel-side validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MergeTreeOutcome {
    Clean,
    Conflict { paths: Vec<String> },
}

/// Successful projected-state report returned by
/// [`validate_projected_merge_state`].
///
/// `accumulated_path_count` is the union-cardinality of every prior
/// queued PR's `touched_paths` after de-duplication — a stat the
/// runner emits into the per-tick evidence file for convergence
/// auditing.
///
/// `requires_test_rerun` is always `true` for the candidate: ADR-0111
/// §"Projected-merge-state simulation" invariant 3 says tests MUST
/// re-run if `projected_base_i` differs from any previously-tested
/// base. Since the kernel cannot know the candidate's tested-base
/// history, it conservatively reports `true` — the runner may suppress
/// the rerun if it can prove identical base.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedStateReport {
    pub candidate_pr_number: u64,
    pub accumulated_path_count: usize,
    pub queue_depth_at_admission: usize,
    pub requires_test_rerun: bool,
}

/// Closed enum of every reason `validate_projected_merge_state` can
/// refuse a candidate. ADR-0111 wave-A locks the variant set; new
/// failure modes require an ADR amendment and a kernel version bump.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectedStateError {
    /// Invariant 1 failure: `git merge-tree` reported conflict
    /// markers. `paths` is the (possibly empty if the runner could
    /// not extract paths) set of files in conflict. The candidate
    /// must be refused admission per ADR-0111 §"Conflict-avoidance
    /// pre-admit gate".
    ConflictDetected {
        candidate_pr_number: u64,
        paths: Vec<String>,
    },
    /// Invariant 2 failure: candidate's `touched_paths` intersects a
    /// prior queued PR's `touched_paths` and the intersection is not
    /// in the runner-supplied `concurrent_safe_paths` whitelist.
    /// ADR-0111 §"Open questions" decision-2: v0 strict, whitelist
    /// relaxation is wave-D.
    PathOverlapRefused {
        candidate_pr_number: u64,
        conflicting_pr_number: u64,
        overlapping_paths: Vec<String>,
    },
    /// The runner passed an empty queue AND the candidate itself is
    /// missing; this is a programmer error (the validator requires
    /// the candidate as a separate argument so this can only be hit
    /// by a malformed kernel call). Surfaced as a closed-enum variant
    /// to keep the error contract total per ADR-0056 §"Total error
    /// types".
    EmptyQueuedPrSet,
    /// Invariant 3 (kernel-side defensive): a sha did not match the
    /// 40-lowercase-hex contract. The string carries the offending
    /// value so the runner can include it in evidence.
    MalformedSha(String),
}

impl fmt::Display for ProjectedStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictDetected {
                candidate_pr_number,
                paths,
            } => write!(
                f,
                "conflict detected for candidate PR #{}; conflicted_paths={:?}",
                candidate_pr_number, paths
            ),
            Self::PathOverlapRefused {
                candidate_pr_number,
                conflicting_pr_number,
                overlapping_paths,
            } => write!(
                f,
                "path-overlap refused: candidate PR #{} overlaps queued PR #{} on paths {:?}",
                candidate_pr_number, conflicting_pr_number, overlapping_paths
            ),
            Self::EmptyQueuedPrSet => write!(
                f,
                "empty queued-pr set; the kernel was called without any candidate (programmer error)"
            ),
            Self::MalformedSha(s) => {
                write!(f, "malformed sha; expected 40-lowercase-hex, got {:?}", s)
            }
        }
    }
}

impl std::error::Error for ProjectedStateError {}

/// Returns `true` iff `s` is exactly 40 lowercase hex characters.
fn is_well_formed_sha(s: &str) -> bool {
    if s.len() != 40 {
        return false;
    }
    s.bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Validate the projected merge state for `candidate` against the
/// already-queued PRs (`queued_prs`, position 0..N in admission order)
/// rooted at `dev_head`.
///
/// `merge_tree_outcome` is the runner-supplied result of executing
/// `git merge-tree projected_base candidate.head dev_head` after the
/// runner has applied each prior queued PR onto `dev_head` (the
/// runner stages this on transient `merge-queue-staging-i` refs per
/// ADR-0111 §"Consequences").
///
/// `concurrent_safe_paths` is the pre-resolved (per-product) set of
/// paths the runner has loaded from
/// `registry/vcs/concurrent-safe-paths.yaml`. v0 ships empty so any
/// overlap fails; future wave-D relaxation populates docs/CHANGELOG.md
/// etc.
///
/// Returns `Ok(ProjectedStateReport)` on admission acceptance. The
/// runner advances `dev` to `projected_head_i` only after this returns
/// Ok AND the subsequent test-rerun (signalled by
/// `requires_test_rerun`) lands green.
///
/// Errors are total per [`ProjectedStateError`]; the runner pattern-
/// matches and writes the corresponding evidence-file path.
pub fn validate_projected_merge_state(
    dev_head: &str,
    queued_prs: &[QueuedPr],
    candidate: &QueuedPr,
    merge_tree_outcome: &MergeTreeOutcome,
    concurrent_safe_paths: &BTreeSet<String>,
) -> Result<ProjectedStateReport, ProjectedStateError> {
    // Invariant 3 (defensive sha check) — run first so malformed
    // inputs short-circuit before we touch the cross-product.
    if !is_well_formed_sha(dev_head) {
        return Err(ProjectedStateError::MalformedSha(dev_head.to_string()));
    }
    if !is_well_formed_sha(&candidate.head_sha) {
        return Err(ProjectedStateError::MalformedSha(candidate.head_sha.clone()));
    }
    if !is_well_formed_sha(&candidate.base_sha) {
        return Err(ProjectedStateError::MalformedSha(candidate.base_sha.clone()));
    }
    for queued in queued_prs {
        if !is_well_formed_sha(&queued.head_sha) {
            return Err(ProjectedStateError::MalformedSha(queued.head_sha.clone()));
        }
        if !is_well_formed_sha(&queued.base_sha) {
            return Err(ProjectedStateError::MalformedSha(queued.base_sha.clone()));
        }
    }

    // Invariant 1: diff cleanliness.
    if let MergeTreeOutcome::Conflict { paths } = merge_tree_outcome {
        return Err(ProjectedStateError::ConflictDetected {
            candidate_pr_number: candidate.pr_number,
            paths: paths.clone(),
        });
    }

    // Invariant 2: path-overlap check.
    let candidate_paths: BTreeSet<&str> =
        candidate.touched_paths.iter().map(String::as_str).collect();

    for queued in queued_prs {
        let queued_paths: BTreeSet<&str> =
            queued.touched_paths.iter().map(String::as_str).collect();
        let raw_overlap: BTreeSet<&str> = candidate_paths
            .intersection(&queued_paths)
            .copied()
            .collect();
        if raw_overlap.is_empty() {
            continue;
        }
        // Filter the overlap by the concurrent-safe-paths whitelist.
        // Any path in the whitelist is considered concurrent-safe and
        // does NOT count against the candidate.
        let unsafe_overlap: Vec<String> = raw_overlap
            .into_iter()
            .filter(|p| !concurrent_safe_paths.contains(*p))
            .map(|p| p.to_string())
            .collect();
        if !unsafe_overlap.is_empty() {
            return Err(ProjectedStateError::PathOverlapRefused {
                candidate_pr_number: candidate.pr_number,
                conflicting_pr_number: queued.pr_number,
                overlapping_paths: unsafe_overlap,
            });
        }
    }

    // Compute accumulated_path_count: union over every queued PR's
    // touched paths (candidate excluded; the report describes the
    // *prior* state). De-duplicated via BTreeSet.
    let mut accumulated: BTreeSet<&str> = BTreeSet::new();
    for queued in queued_prs {
        for p in &queued.touched_paths {
            accumulated.insert(p.as_str());
        }
    }

    Ok(ProjectedStateReport {
        candidate_pr_number: candidate.pr_number,
        accumulated_path_count: accumulated.len(),
        queue_depth_at_admission: queued_prs.len(),
        requires_test_rerun: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sha(c: char) -> String {
        std::iter::repeat(c).take(40).collect()
    }

    fn pr(number: u64, head_c: char, base_c: char, paths: &[&str]) -> QueuedPr {
        QueuedPr {
            pr_number: number,
            head_sha: sha(head_c),
            base_sha: sha(base_c),
            touched_paths: paths.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    #[test]
    fn clean_admission_into_empty_queue_accepted() {
        // ADR-0111 §"Projected-merge-state simulation": first PR into
        // an empty queue against dev_head — merge-tree clean — must
        // accept. accumulated_path_count = 0 (no prior PRs).
        let dev = sha('0');
        let candidate = pr(101, 'a', '0', &["src/foo.rs"]);
        let safe: BTreeSet<String> = BTreeSet::new();
        let rep = validate_projected_merge_state(
            &dev,
            &[],
            &candidate,
            &MergeTreeOutcome::Clean,
            &safe,
        )
        .expect("clean admission must succeed");
        assert_eq!(rep.candidate_pr_number, 101);
        assert_eq!(rep.accumulated_path_count, 0);
        assert_eq!(rep.queue_depth_at_admission, 0);
        assert!(rep.requires_test_rerun);
    }

    #[test]
    fn conflict_detected_refuses_admission() {
        // Invariant 1: the runner-supplied MergeTreeOutcome::Conflict
        // must translate to a ConflictDetected error carrying the
        // offending paths.
        let dev = sha('0');
        let candidate = pr(202, 'b', '0', &["src/bar.rs"]);
        let safe: BTreeSet<String> = BTreeSet::new();
        let err = validate_projected_merge_state(
            &dev,
            &[],
            &candidate,
            &MergeTreeOutcome::Conflict {
                paths: vec!["src/bar.rs".into()],
            },
            &safe,
        )
        .unwrap_err();
        match err {
            ProjectedStateError::ConflictDetected {
                candidate_pr_number,
                paths,
            } => {
                assert_eq!(candidate_pr_number, 202);
                assert_eq!(paths, vec!["src/bar.rs".to_string()]);
            }
            other => panic!("expected ConflictDetected, got {other:?}"),
        }
    }

    #[test]
    fn path_overlap_with_queued_pr_refused_strict() {
        // Invariant 2 (v0 strict): candidate touches `src/lib.rs`
        // which queued PR #100 also touched. With empty whitelist the
        // candidate MUST be refused with PathOverlapRefused.
        let dev = sha('0');
        let queued = pr(100, 'a', '0', &["src/lib.rs", "src/util.rs"]);
        let candidate = pr(101, 'b', '0', &["src/lib.rs"]);
        let safe: BTreeSet<String> = BTreeSet::new();
        let err = validate_projected_merge_state(
            &dev,
            &[queued],
            &candidate,
            &MergeTreeOutcome::Clean,
            &safe,
        )
        .unwrap_err();
        match err {
            ProjectedStateError::PathOverlapRefused {
                candidate_pr_number,
                conflicting_pr_number,
                overlapping_paths,
            } => {
                assert_eq!(candidate_pr_number, 101);
                assert_eq!(conflicting_pr_number, 100);
                assert_eq!(overlapping_paths, vec!["src/lib.rs".to_string()]);
            }
            other => panic!("expected PathOverlapRefused, got {other:?}"),
        }
    }

    #[test]
    fn path_overlap_in_whitelist_admitted() {
        // Wave-D relaxation pre-wired: if `docs/CHANGELOG.md` is in
        // the runner-supplied concurrent_safe_paths set, an overlap on
        // ONLY that path no longer refuses admission.
        let dev = sha('0');
        let queued = pr(100, 'a', '0', &["docs/CHANGELOG.md"]);
        let candidate = pr(101, 'b', '0', &["docs/CHANGELOG.md"]);
        let mut safe: BTreeSet<String> = BTreeSet::new();
        safe.insert("docs/CHANGELOG.md".to_string());
        let rep = validate_projected_merge_state(
            &dev,
            &[queued],
            &candidate,
            &MergeTreeOutcome::Clean,
            &safe,
        )
        .expect("whitelist must absorb the overlap");
        assert_eq!(rep.accumulated_path_count, 1);
    }

    #[test]
    fn empty_queue_is_accepted_not_an_error() {
        // ADR-0111 §"Projected-merge-state simulation" allows empty
        // queue (the very first PR after a quiet period). The kernel
        // must NOT return EmptyQueuedPrSet for that case; that variant
        // is reserved for the programmer-error path where the runner
        // calls the kernel without a candidate. Here we exercise the
        // happy path with empty queue.
        let dev = sha('0');
        let candidate = pr(1, 'a', '0', &["src/x.rs"]);
        let safe: BTreeSet<String> = BTreeSet::new();
        let rep = validate_projected_merge_state(
            &dev,
            &[],
            &candidate,
            &MergeTreeOutcome::Clean,
            &safe,
        )
        .expect("empty queue happy path");
        assert_eq!(rep.queue_depth_at_admission, 0);
    }

    #[test]
    fn malformed_sha_rejected() {
        // Invariant 3 (defensive): a 7-char short sha must trip the
        // MalformedSha guard before any cross-product work.
        let dev = "deadbee".to_string(); // 7 chars
        let candidate = pr(1, 'a', '0', &["src/x.rs"]);
        let safe: BTreeSet<String> = BTreeSet::new();
        let err = validate_projected_merge_state(
            &dev,
            &[],
            &candidate,
            &MergeTreeOutcome::Clean,
            &safe,
        )
        .unwrap_err();
        assert!(matches!(err, ProjectedStateError::MalformedSha(_)));
    }

    #[test]
    fn malformed_sha_rejects_uppercase_hex() {
        // Sha contract is *lowercase* hex; uppercase must trip the
        // guard so the runner's normalisation path stays mandatory.
        let dev = sha('0');
        let mut candidate = pr(1, 'a', '0', &["src/x.rs"]);
        candidate.head_sha = "A".repeat(40);
        let safe: BTreeSet<String> = BTreeSet::new();
        let err = validate_projected_merge_state(
            &dev,
            &[],
            &candidate,
            &MergeTreeOutcome::Clean,
            &safe,
        )
        .unwrap_err();
        assert!(matches!(err, ProjectedStateError::MalformedSha(_)));
    }

    #[test]
    fn three_pr_deep_projected_state_no_overlap_accepted() {
        // ADR-0111 §"Projected-merge-state simulation": admit PR_4 at
        // position 3 against a queue [PR_1, PR_2, PR_3] where each
        // touches a disjoint file. Accumulated path count = 3 (one per
        // queued PR), depth = 3, candidate cleared.
        let dev = sha('0');
        let q1 = pr(1, 'a', '0', &["a.rs"]);
        let q2 = pr(2, 'b', '0', &["b.rs"]);
        let q3 = pr(3, 'c', '0', &["c.rs"]);
        let candidate = pr(4, 'd', '0', &["d.rs"]);
        let safe: BTreeSet<String> = BTreeSet::new();
        let rep = validate_projected_merge_state(
            &dev,
            &[q1, q2, q3],
            &candidate,
            &MergeTreeOutcome::Clean,
            &safe,
        )
        .expect("3-deep clean admission");
        assert_eq!(rep.queue_depth_at_admission, 3);
        assert_eq!(rep.accumulated_path_count, 3);
    }

    #[test]
    fn three_pr_deep_with_late_overlap_refused() {
        // Candidate at position 3 overlaps with the FIRST queued PR
        // (not the most recent) — ensure we scan every queued PR, not
        // only the immediate predecessor.
        let dev = sha('0');
        let q1 = pr(1, 'a', '0', &["shared.rs"]);
        let q2 = pr(2, 'b', '0', &["b.rs"]);
        let q3 = pr(3, 'c', '0', &["c.rs"]);
        let candidate = pr(4, 'd', '0', &["shared.rs"]);
        let safe: BTreeSet<String> = BTreeSet::new();
        let err = validate_projected_merge_state(
            &dev,
            &[q1, q2, q3],
            &candidate,
            &MergeTreeOutcome::Clean,
            &safe,
        )
        .unwrap_err();
        match err {
            ProjectedStateError::PathOverlapRefused {
                conflicting_pr_number,
                ..
            } => assert_eq!(conflicting_pr_number, 1),
            other => panic!("expected PathOverlapRefused on PR#1, got {other:?}"),
        }
    }

    #[test]
    fn error_display_includes_pr_number() {
        // Display impls are part of the runner's evidence-file
        // serialisation contract; lock the format roughly.
        let err = ProjectedStateError::ConflictDetected {
            candidate_pr_number: 7,
            paths: vec!["x.rs".into()],
        };
        let s = format!("{err}");
        assert!(s.contains("PR #7"));
        assert!(s.contains("x.rs"));
    }
}
