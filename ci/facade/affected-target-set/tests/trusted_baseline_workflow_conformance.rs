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

fn affected_job(workflow: &str) -> &str {
    let start = workflow
        .find("  gate-affected-target-set:\n")
        .expect("affected-set job exists");
    let tail = &workflow[start..];
    let end = tail[1..]
        .find("\n  gate-")
        .map_or(tail.len(), |offset| offset + 1);
    &tail[..end]
}

#[test]
fn actions_read_is_job_scoped_to_the_only_consumer() {
    let workflow = workflow();
    let header = workflow.split("\njobs:\n").next().expect("workflow header");
    assert!(!header.contains("  actions: read"));

    let job = affected_job(&workflow);
    assert!(job.contains("    permissions:\n      contents: read\n      actions: read\n"));
}

#[test]
fn hub_exclusivity_live_producer_is_invoked_on_binding_affected_set_step() {
    let workflow = workflow();
    let job = affected_job(&workflow);
    assert!(
        job.contains("oya-cloud-ci-hub-exclusivity-bin"),
        "binding step must build the hub-exclusivity producer binary"
    );
    assert!(
        job.contains("--live-open-prs"),
        "binding step must opt into live open-PR file facts (not fixture-only)"
    );
    assert!(
        job.contains("pull-requests: read"),
        "job needs pull-requests:read for open PR + files list REST"
    );
}

#[test]
fn exact_producer_is_the_sole_attempt_bound_publisher_and_cold_fallback_remains() {
    let workflow = workflow();
    let job = affected_job(&workflow);

    assert_eq!(workflow.matches("name: build-health-baseline-").count(), 1);
    assert_eq!(workflow.matches("name: test-health-baseline-").count(), 1);
    for kind in ["build", "test"] {
        let binding = format!(
            "name: {kind}-health-baseline-${{{{ github.sha }}}}-${{{{ github.run_id }}}}-${{{{ github.run_attempt }}}}-gate-affected-target-set"
        );
        assert!(
            job.contains(&binding),
            "missing exact artifact binding {binding}"
        );
    }
    assert_eq!(
        job.matches("github.event_name == 'push' && github.ref == 'refs/heads/dev'")
            .count(),
        2
    );
    assert!(job.contains("--trusted-baseline"));
    assert!(job.contains("git worktree add --quiet --detach"));
    assert!(job.contains("buck2 build //... --keep-going"));
    assert!(job.contains("buck2 test //... --keep-going"));
}
