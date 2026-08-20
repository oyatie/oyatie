#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::{fs, path::PathBuf};

fn repo_root() -> PathBuf {
    let mut path = std::env::current_dir().expect("current_dir");
    loop {
        if path.join(".github/workflows/oya-ci-required.yml").is_file() {
            return path;
        }
        assert!(path.pop(), "could not locate repository root");
    }
}

fn workflow() -> String {
    fs::read_to_string(repo_root().join(".github/workflows/oya-ci-required.yml"))
        .expect("read required workflow")
}

#[test]
fn affected_set_job_is_retired_and_hub_exclusivity_runs_in_the_test_job() {
    let workflow = workflow();
    assert!(
        !workflow.contains("  gate-affected-target-set:"),
        "the affected-set lane is retired; cargo test --workspace is the affected set (ADR-0716)"
    );
    assert!(
        !workflow.contains("--trusted-baseline"),
        "the trusted-baseline artifact machinery is retired with the lane"
    );
    assert!(
        workflow.contains("oya-cloud-ci-hub-exclusivity"),
        "the hub-exclusivity producer must still be invoked by the merge path"
    );
    assert!(
        workflow.contains("--live-open-prs"),
        "hub-exclusivity must opt into live open-PR file facts (not fixture-only)"
    );
    assert!(
        workflow.contains("pull-requests: read"),
        "the invoking job needs pull-requests:read for open PR + files list REST"
    );
    let header = workflow.split("\njobs:\n").next().expect("workflow header");
    assert!(
        !header.contains("  actions: read"),
        "actions:read must not be workflow-scoped"
    );
}
