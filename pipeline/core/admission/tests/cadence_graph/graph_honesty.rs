//! The advisory buck2 graph lane: its cadence, and the absence that makes
//! it advisory.

/// The buck2 graph runs on every push to `dev` and is advisory.
///
/// Advisory is the property that keeps the lane inside the dual-proof
/// prohibition, and it is invisible in the lane's own file: a workflow cannot
/// say "nothing depends on me". So the cadence is read here, and the absence
/// is read from the two files that could create the dependency -- a `needs:`
/// on this job, or its name in either required occupant set.
#[test]
fn the_buck2_graph_is_advisory_on_every_push() {
    let y = super::read(".github/workflows/buck2-graph-honesty.yml");
    assert!(
        y.contains("\n  push:\n    branches: [dev]\n"),
        "must run per push"
    );
    assert!(!y.contains("schedule:"), "the weekly cadence is retired");
    assert!(
        y.contains("buck2 build //..."),
        "the graph is what it builds"
    );

    let job = "buck2-graph-honesty";
    assert!(
        !pipeline_admission::PRESUBMIT_JOBS.contains(&job)
            && !pipeline_admission::POSTSUBMIT_JOBS.contains(&job)
    );
    for cadence in [
        ".github/workflows/presubmit.yml",
        ".github/workflows/postsubmit.yml",
    ] {
        assert!(
            !super::read(cadence).contains(job),
            "{cadence} must not wait on the graph"
        );
    }
}
