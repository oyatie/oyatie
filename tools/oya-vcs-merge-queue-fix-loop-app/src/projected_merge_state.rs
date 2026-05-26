//! Projected-merge-state runner-side module — ADR-0111 wave-A.
//!
//! The kernel
//! ([`oya_vcs_merge_queue_conflict_kernel::validate_projected_merge_state`])
//! is pure-domain (no I/O). This module is the runner-side I/O shell
//! that:
//!
//! 1. Invokes `git merge-tree dev_head pr.head dev_head` per queued PR
//!    via a side-effect trait ([`GitMergeTreeRunner`]) so tests can
//!    inject a fake.
//! 2. Parses the runner's stdout for conflict markers
//!    ([`parse_merge_tree_output`]) and yields a
//!    [`MergeTreeOutcome`].
//! 3. Loads the per-product `concurrent-safe-paths.yaml` whitelist
//!    ([`concurrent_safe_paths_for_product`]).
//! 4. Calls the kernel's `validate_projected_merge_state`.
//!
//! ADR-0083 Tier 1: all production code is `unwrap`/`expect`/`panic`
//! free; tests use these primitives under the `cfg(test)` exemption.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

pub use oya_vcs_merge_queue_conflict_kernel::{
    MergeTreeOutcome, ProjectedStateError, ProjectedStateReport, QueuedPr,
    validate_projected_merge_state,
};

/// Side-effect trait wrapping `git merge-tree`. The real
/// implementation shells out via `std::process::Command`; tests
/// inject a fake that returns canned stdout.
pub trait GitMergeTreeRunner {
    /// Run `git merge-tree <base> <ours> <theirs>` and return the
    /// raw stdout. The runner is responsible for cwd/env; this trait
    /// is the narrow contract.
    fn run_merge_tree(&self, base: &str, ours: &str, theirs: &str) -> Result<String, String>;
}

/// Parse `git merge-tree` output for conflict markers.
///
/// `git merge-tree` emits standard 3-way conflict-marker sections
/// delimited by `<<<<<<<` ... `=======` ... `>>>>>>>` when conflicts
/// exist; absence of the start marker is sufficient to declare the
/// merge clean.
///
/// Path extraction: each conflict hunk in stdout is preceded by an
/// "added in remote" / "added in local" / hunk-header line in the
/// form `<path>\n` followed by the marker block; we scan for any
/// line that appears immediately before the start-marker block.
/// When path extraction fails (unrecognised output shape), we report
/// `Conflict { paths: vec![] }` so the runner still refuses the
/// candidate — fail-closed.
pub fn parse_merge_tree_output(stdout: &str) -> MergeTreeOutcome {
    if !stdout.contains("<<<<<<<") {
        return MergeTreeOutcome::Clean;
    }

    let mut paths: BTreeSet<String> = BTreeSet::new();
    let mut prev_line: Option<&str> = None;
    for line in stdout.lines() {
        if line.starts_with("<<<<<<<")
            && let Some(prior) = prev_line
        {
            // Heuristic: the immediately-prior non-empty line
            // usually carries the offending path (git merge-tree
            // shells `<path>` headers above each conflict hunk).
            let trimmed = prior.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with('=')
                && !trimmed.starts_with('>')
                && !trimmed.starts_with('<')
            {
                paths.insert(trimmed.to_string());
            }
        }
        prev_line = Some(line);
    }

    MergeTreeOutcome::Conflict {
        paths: paths.into_iter().collect(),
    }
}

/// Load the runner-side concurrent-safe-paths whitelist for a given
/// product, given the raw YAML registry text and the product name.
///
/// The YAML schema (matching `registry/vcs/concurrent-safe-paths.yaml`):
///
/// ```yaml
/// products:
///   foundry:
///     safe_paths:
///       - docs/CHANGELOG.md
/// ```
///
/// This is a small std-only parser; we deliberately avoid a YAML
/// crate dep on the runner to keep the dependency surface narrow.
/// Unknown / missing product → empty set (the kernel will then treat
/// every overlap as unsafe, which is the strict v0 default).
pub fn concurrent_safe_paths_for_product(yaml_text: &str, product: &str) -> BTreeSet<String> {
    // Line-oriented parser. We scan for the product header
    // ("  <product>:") under `products:`, then collect any
    // "    - <path>" entries under its `safe_paths:` subkey, stopping
    // at the next sibling product or top-level key.
    let mut out: BTreeSet<String> = BTreeSet::new();
    let mut in_products = false;
    let mut in_target_product = false;
    let mut in_safe_paths = false;

    for line in yaml_text.lines() {
        // Strip trailing # comment.
        let trimmed_no_comment = match line.find('#') {
            Some(idx) => &line[..idx],
            None => line,
        };
        if trimmed_no_comment.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let content = line.trim();

        if indent == 0 {
            in_products = content.starts_with("products:");
            in_target_product = false;
            in_safe_paths = false;
            continue;
        }
        if !in_products {
            continue;
        }
        // 2-space indent → product key.
        if indent == 2 {
            in_target_product = content.starts_with(&format!("{product}:"));
            in_safe_paths = false;
            continue;
        }
        if !in_target_product {
            continue;
        }
        // 4-space indent → product subkey (safe_paths: etc).
        if indent == 4 {
            in_safe_paths = content.starts_with("safe_paths:");
            continue;
        }
        // 6-space indent → list entry under safe_paths.
        if indent == 6
            && in_safe_paths
            && let Some(rest) = content.strip_prefix("- ")
        {
            let path = rest.trim().trim_matches(|c| c == '"' || c == '\'');
            if !path.is_empty() {
                out.insert(path.to_string());
            }
        }
    }
    out
}

/// Compose the kernel call from the runner-side bits: invoke
/// `git merge-tree` for the candidate against `dev_head`, parse, and
/// hand to the kernel. This is the seam the
/// `conflict_avoidance_pre_admit` module wires into the admission
/// pipeline.
pub fn run_projected_merge_state_check(
    runner: &dyn GitMergeTreeRunner,
    dev_head: &str,
    queued_prs: &[QueuedPr],
    candidate: &QueuedPr,
    concurrent_safe_paths: &BTreeSet<String>,
) -> Result<ProjectedStateReport, ProjectedMergeStateRunError> {
    let stdout = runner
        .run_merge_tree(dev_head, &candidate.head_sha, dev_head)
        .map_err(ProjectedMergeStateRunError::GitInvocationFailed)?;
    let outcome = parse_merge_tree_output(&stdout);
    validate_projected_merge_state(
        dev_head,
        queued_prs,
        candidate,
        &outcome,
        concurrent_safe_paths,
    )
    .map_err(ProjectedMergeStateRunError::KernelRefused)
}

/// Runner-side error envelope wrapping the kernel's
/// [`ProjectedStateError`] plus the git-invocation failure mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectedMergeStateRunError {
    /// `git merge-tree` itself failed (binary missing, repository
    /// missing, etc.). The string carries the runner-supplied stderr.
    GitInvocationFailed(String),
    /// The kernel refused the candidate; carries the exact closed-enum
    /// reason so the runner can write the evidence file with the
    /// invariant name.
    KernelRefused(ProjectedStateError),
}

impl std::fmt::Display for ProjectedMergeStateRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitInvocationFailed(s) => write!(f, "git merge-tree invocation failed: {s}"),
            Self::KernelRefused(e) => write!(f, "kernel refused candidate: {e}"),
        }
    }
}

impl std::error::Error for ProjectedMergeStateRunError {}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubRunner {
        out: String,
        err: Option<String>,
    }
    impl GitMergeTreeRunner for StubRunner {
        fn run_merge_tree(&self, _b: &str, _o: &str, _t: &str) -> Result<String, String> {
            match &self.err {
                Some(e) => Err(e.clone()),
                None => Ok(self.out.clone()),
            }
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
    fn parse_merge_tree_clean_when_no_markers() {
        assert_eq!(parse_merge_tree_output(""), MergeTreeOutcome::Clean);
        assert_eq!(
            parse_merge_tree_output("some\nnormal\noutput\n"),
            MergeTreeOutcome::Clean
        );
    }

    #[test]
    fn parse_merge_tree_extracts_conflict_path() {
        let stdout = "src/lib.rs\n<<<<<<< ours\nfoo\n=======\nbar\n>>>>>>> theirs\n";
        let outcome = parse_merge_tree_output(stdout);
        match outcome {
            MergeTreeOutcome::Conflict { paths } => {
                assert_eq!(paths, vec!["src/lib.rs".to_string()]);
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    #[test]
    fn parse_merge_tree_conflict_without_path_header_still_refuses() {
        // If the runner emits markers without our heuristic path header,
        // we MUST still refuse (fail-closed): empty path list, Conflict
        // variant set.
        let stdout = "<<<<<<<\nfoo\n=======\nbar\n>>>>>>>\n";
        let outcome = parse_merge_tree_output(stdout);
        assert!(matches!(outcome, MergeTreeOutcome::Conflict { .. }));
    }

    #[test]
    fn run_projected_merge_state_check_propagates_kernel_refusal() {
        let runner = StubRunner {
            out: "x.rs\n<<<<<<<\n=======\n>>>>>>>\n".into(),
            err: None,
        };
        let candidate = pr(101, 'a', &["x.rs"]);
        let safe: BTreeSet<String> = BTreeSet::new();
        let err = run_projected_merge_state_check(&runner, &sha('0'), &[], &candidate, &safe)
            .unwrap_err();
        match err {
            ProjectedMergeStateRunError::KernelRefused(ProjectedStateError::ConflictDetected {
                ..
            }) => {}
            other => panic!("expected KernelRefused(ConflictDetected), got {other:?}"),
        }
    }

    #[test]
    fn run_projected_merge_state_check_surfaces_git_invocation_failure() {
        let runner = StubRunner {
            out: String::new(),
            err: Some("fatal: not a git repository".into()),
        };
        let candidate = pr(101, 'a', &["x.rs"]);
        let safe: BTreeSet<String> = BTreeSet::new();
        let err = run_projected_merge_state_check(&runner, &sha('0'), &[], &candidate, &safe)
            .unwrap_err();
        assert!(matches!(
            err,
            ProjectedMergeStateRunError::GitInvocationFailed(_)
        ));
    }

    #[test]
    fn run_projected_merge_state_check_happy_path() {
        let runner = StubRunner {
            out: "no conflicts here\n".into(),
            err: None,
        };
        let candidate = pr(101, 'a', &["x.rs"]);
        let safe: BTreeSet<String> = BTreeSet::new();
        let rep =
            run_projected_merge_state_check(&runner, &sha('0'), &[], &candidate, &safe).unwrap();
        assert_eq!(rep.candidate_pr_number, 101);
        assert!(rep.requires_test_rerun);
    }

    #[test]
    fn concurrent_safe_paths_for_product_extracts_per_product_safe_paths() {
        let yaml = "\
schema_version: 1
products:
  foundry:
    safe_paths:
      - docs/CHANGELOG.md
      - registry/quality/lanes.yaml
  workflow:
    safe_paths:
      - docs/STUDIO-CHANGELOG.md
";
        let foundry = concurrent_safe_paths_for_product(yaml, "foundry");
        assert!(foundry.contains("docs/CHANGELOG.md"));
        assert!(foundry.contains("registry/quality/lanes.yaml"));
        let workflow = concurrent_safe_paths_for_product(yaml, "workflow");
        assert!(workflow.contains("docs/STUDIO-CHANGELOG.md"));
        let absent = concurrent_safe_paths_for_product(yaml, "nope");
        assert!(absent.is_empty());
    }

    #[test]
    fn concurrent_safe_paths_for_product_empty_when_registry_empty() {
        let yaml = "schema_version: 1\nproducts: {}\n";
        assert!(concurrent_safe_paths_for_product(yaml, "foundry").is_empty());
    }
}
