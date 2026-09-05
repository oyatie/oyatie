//! The called commit-signing workflow judges the lane from protected source.
//!
//! It lives in its own file because the required workflow is AT its 300-line
//! budget: a gate written inline there cannot land at all, so the extension
//! path is a reusable workflow, and the law that pins the protected step graph
//! has to follow the job to where the job went.

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
    assert!(
        presubmit.contains("uses: ./.github/workflows/commit-signing.yml"),
        "{presubmit}"
    );
    assert!(
        presubmit.contains("occ \"${{ needs.commit-signing.result }}\""),
        "the fan-in must demand success on a pull request and tolerate the skip \
         elsewhere; plain `req` would wedge the queue on a skipped job: {presubmit}"
    );
}
