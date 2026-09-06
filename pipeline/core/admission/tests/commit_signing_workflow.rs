//! The called commit-signing workflow judges the lane from protected source.
//!
//! It lives in its own file because the required workflow is AT its 300-line
//! budget: a gate written inline there cannot land at all, so the extension
//! path is a reusable workflow, and the law that pins the protected step graph
//! has to follow the job to where the job went.

use pipeline_admission::WORKFLOW_FILES;

fn workflow() -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repository root")
        .to_path_buf();
    std::fs::read_to_string(root.join(".github/workflows/commit-signing.yml"))
        .expect("the commit-signing workflow")
}

/// The binary that judges a candidate is built from the PROTECTED revision.
/// Building it from the candidate would let a lane rewrite the gate in the
/// same commit the gate is judging, and pass itself.
#[test]
fn the_gate_is_compiled_from_protected_source_not_the_candidate() {
    let y = workflow();

    assert!(y.contains("ref: ${{ github.workflow_sha }}"), "{y}");
    assert!(y.contains("path: trusted"), "{y}");
    assert!(
        y.contains("--manifest-path \"$GITHUB_WORKSPACE/trusted/Cargo.toml\""),
        "the build must resolve the trusted manifest, not the candidate's: {y}"
    );
    assert!(!y.contains("$GITHUB_WORKSPACE/candidate/Cargo.toml"), "{y}");
}

/// The range is the LANE's own commits: `HEAD^1..HEAD^2` on the merge commit.
///
/// Both halves are load-bearing and neither is inferable from the other.
/// Judging `HEAD` would judge the host-forged merge commit, which carries no
/// author signature and would refuse every lane. Judging `HEAD^1..HEAD` would
/// sweep in commits already on `dev`, which no author of this lane can repair.
#[test]
fn the_range_is_the_lane_between_the_merge_parents() {
    let y = workflow();

    assert!(
        y.contains("base_sha=\"$(git rev-parse --verify 'HEAD^1^{commit}')\""),
        "{y}"
    );
    assert!(
        y.contains("head_sha=\"$(git rev-parse --verify 'HEAD^2^{commit}')\""),
        "the lane is the SECOND parent; judging HEAD judges the merge commit: {y}"
    );
    assert!(
        y.contains("/pipeline-commit-signing-app\" \"$base_sha\" \"$head_sha\""),
        "{y}"
    );
}

/// A gate that cannot be reached by anything is not a gate. The required
/// graph must call it, or it runs nowhere and refuses nothing.
#[test]
fn the_required_graph_calls_this_workflow_and_waits_on_it() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repository root")
        .to_path_buf();
    let presubmit = std::fs::read_to_string(root.join(".github/workflows/presubmit.yml"))
        .expect("the presubmit workflow");

    assert!(workflow().contains("workflow_call:"), "it must be callable");
    assert!(
        workflow().contains("if: github.event_name == 'pull_request'"),
        "a merge-queue commit is the host's squash of the lane, single-parent and \
         PGP-signed by web-flow; judging it would refuse every queue entry"
    );
    // FULLY QUALIFIED, never `./`. The required workflow is pinned at
    // `refs/heads/dev`, so a local reference resolves against the candidate
    // and the run does not start — which is not a theory: a `./` caller
    // landed on dev and every presubmit run in the repository failed to
    // start until it was replaced, including the runs of the fix.
    assert!(
        presubmit.contains("uses: oyatie/oyatie/.github/workflows/commit-signing.yml@dev"),
        "{presubmit}"
    );
    assert!(
        presubmit.contains("occ \"${{ needs.commit-signing.result }}\""),
        "the fan-in must demand success on a pull request and tolerate the skip \
         elsewhere; plain `req` would wedge the queue on a skipped job: {presubmit}"
    );
}

/// THE STEP GRAPH IS CLOSED. Exactly these five steps, in this order.
///
/// `protected_policy_workflow.rs` holds this law for the three gates that
/// stayed inline in the required workflow; a gate that moved to a called file
/// escaped it, and `contains()` assertions do not replace it. They cannot see
/// an ADDED step — a second `cargo build` pointed at `candidate/`, say, which
/// would compile the judge from the tree it is judging while every existing
/// assertion stayed true.
#[test]
fn the_step_graph_admits_nothing_beyond_these_five_steps() {
    let y = workflow();
    let named: Vec<&str> = y
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- name: "))
        .collect();

    assert_eq!(
        named,
        [
            "Check out candidate tree",
            "Check out protected admission source",
            "Build protected commit-signing application",
            "Admit the lane's commits",
        ],
        "the named steps are closed; the only unnamed one is the toolchain"
    );
    // Counted by the step's SHAPE, because an added step need carry neither
    // key this file reads. A bare `- run: cp -r candidate/… trusted/` slipped
    // past a count of `- name:` plus `- uses:` while overwriting the trusted
    // checkout — compiling the judge from the tree it judges, with every
    // other assertion here still true and no second `cargo build` in sight.
    assert_eq!(
        y.split("\n      - ").skip(1).count(),
        5,
        "five steps exactly, whatever keys they carry"
    );
    assert_eq!(
        y.matches("cargo build").count(),
        1,
        "one build, from the trusted manifest and no other"
    );
}

/// EVERY workflow in the directory carries semantic display names.
///
/// `semantic_names.rs` scans `presubmit.yml` and the one called workflow that
/// existed when it was written; a job moved out of the required file into a
/// called one leaves that scan silently. `deny (licenses bans sources)` did
/// exactly that, and could have been renamed to a decision number while every
/// existing assertion stayed green.
///
/// The check lives here rather than there because that file sits at the
/// 300-line budget with no room to grow — the same wall that forced these
/// workflows out of `presubmit.yml` in the first place.
#[test]
fn every_workflow_carries_semantic_display_names() {
    // EVERY workflow, from the pinned occupant set — not a literal pair. A
    // list of filenames is the same shape as the ban that let `./` through
    // one PR ago: the next file anyone adds would be silently unscanned.
    for name in WORKFLOW_FILES {
        let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("repository root")
            .join(".github/workflows")
            .join(name);
        let file = std::fs::read_to_string(&at).unwrap_or_else(|_| panic!("{name} must exist"));
        for line in file.lines() {
            let Some(display) = line.trim().strip_prefix("name: ") else {
                continue;
            };
            // The same rule `semantic_names.rs` applies: a decision
            // identifier is provenance, never an operator-facing name.
            for marker in ["ADR-", "D-"] {
                let numbered = display.split(marker).skip(1).any(|tail| {
                    tail.bytes()
                        .next()
                        .is_some_and(|byte| byte.is_ascii_digit())
                });
                assert!(
                    !numbered,
                    "{name}.yml exposes a decision identifier as an operational name: {display}"
                );
            }
        }
    }
}
