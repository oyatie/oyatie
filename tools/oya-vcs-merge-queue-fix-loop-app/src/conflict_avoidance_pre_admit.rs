//! Conflict-avoidance pre-admit gate — ADR-0111 wave-A.
//!
//! Wraps `projected_merge_state::run_projected_merge_state_check` as
//! the FIRST admission gate (cheaper than pr-tests). Per ADR-0111
//! §"Conflict-avoidance pre-admit gate (FIRST gate, not last)":
//!
//! ```
//! Order today (human-driven):    Order in agentic mode (this ADR):
//! 1. PR opened                    1. PR opened
//! 2. pr-tests runs                2. conflict-avoidance pre-admit (cheap)
//! 3. pr-review fires              3. pr-tests runs (expensive)
//! 4. conflict check at merge      4. pr-review fires
//! 5. merge                        5. merge
//! ```
//!
//! The pre-admit check is ~1 second (`git merge-tree` against projected
//! base); pr-tests is minutes. Failing fast on conflict saves the CI
//! budget.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

use crate::projected_merge_state::{
    GitMergeTreeRunner, ProjectedMergeStateRunError, ProjectedStateReport, QueuedPr,
    run_projected_merge_state_check,
};

/// The verdict surfaced by the pre-admit gate. The
/// `oya-vcs-merge-queue-fix-loop-app::run` admission loop
/// pattern-matches on this and either:
///
/// - calls `Scheduler::admit` (Admit), or
/// - writes an evidence file + posts a PR comment with the refusal
///   surface (Refuse).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreAdmitVerdict {
    Admit(ProjectedStateReport),
    Refuse(ProjectedMergeStateRunError),
}

/// Run the pre-admit gate for `candidate` given the current queued
/// PRs and the per-product concurrent-safe-paths whitelist.
///
/// This is intentionally a thin wrapper: it exists so the admission
/// loop has a single named seam to call (and so future wave-D
/// relaxations / metrics emission can add behaviour without changing
/// the admission-loop call site).
pub fn pre_admit_check(
    runner: &dyn GitMergeTreeRunner,
    dev_head: &str,
    queued_prs: &[QueuedPr],
    candidate: &QueuedPr,
    concurrent_safe_paths: &BTreeSet<String>,
) -> PreAdmitVerdict {
    match run_projected_merge_state_check(
        runner,
        dev_head,
        queued_prs,
        candidate,
        concurrent_safe_paths,
    ) {
        Ok(report) => PreAdmitVerdict::Admit(report),
        Err(err) => PreAdmitVerdict::Refuse(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projected_merge_state::QueuedPr;

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
    fn pre_admit_admits_clean_candidate() {
        let runner = StubRunner { out: "".into() };
        let candidate = pr(101, 'a', &["src/foo.rs"]);
        let verdict = pre_admit_check(&runner, &sha('0'), &[], &candidate, &BTreeSet::new());
        assert!(matches!(verdict, PreAdmitVerdict::Admit(_)));
    }

    #[test]
    fn pre_admit_refuses_path_overlap() {
        let runner = StubRunner { out: "".into() };
        let q = pr(100, 'a', &["src/lib.rs"]);
        let candidate = pr(101, 'b', &["src/lib.rs"]);
        let verdict = pre_admit_check(&runner, &sha('0'), &[q], &candidate, &BTreeSet::new());
        assert!(matches!(verdict, PreAdmitVerdict::Refuse(_)));
    }

    #[test]
    fn pre_admit_refuses_on_conflict_marker() {
        let runner = StubRunner {
            out: "src/foo.rs\n<<<<<<<\n=======\n>>>>>>>\n".into(),
        };
        let candidate = pr(101, 'a', &["src/foo.rs"]);
        let verdict = pre_admit_check(&runner, &sha('0'), &[], &candidate, &BTreeSet::new());
        match verdict {
            PreAdmitVerdict::Refuse(ProjectedMergeStateRunError::KernelRefused(_)) => {}
            other => panic!("expected Refuse(KernelRefused), got {other:?}"),
        }
    }
}
