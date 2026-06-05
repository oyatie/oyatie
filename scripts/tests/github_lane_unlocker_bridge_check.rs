#![allow(dead_code)]

#[path = "../ci/assert-github-lane-unlocker-bridge.rs"]
mod gate;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|error| {
        panic!("read {}: {}", path, error);
    })
}

#[test]
fn checked_in_github_lane_unlocker_contract_passes() {
    let evaluation = gate::evaluate(Path::new(&repo_root()));
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.failures.is_empty());
}

#[test]
fn workflow_rejects_retired_python_bridge_command() {
    let mut workflow = read_repo_file(".github/workflows/github-lane-unlocker-ci-cd.yml");
    workflow
        .push_str("\n          python3 scripts/ci/assert-github-lane-unlocker-bridge.py --json\n");
    let failures = gate::workflow_failures(&workflow);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("retired Python bridge checker")),
        "{:?}",
        failures
    );
}

#[test]
fn workflow_rejects_legacy_checkout_v4() {
    let workflow = read_repo_file(".github/workflows/github-lane-unlocker-ci-cd.yml")
        .replace("actions/checkout@v6", "actions/checkout@v4");
    let failures = gate::workflow_failures(&workflow);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("legacy checkout v4")),
        "{:?}",
        failures
    );
}

#[test]
fn workflow_rejects_checkout_credentials_persistence() {
    let workflow = read_repo_file(".github/workflows/github-lane-unlocker-ci-cd.yml")
        .replace("\n          persist-credentials: false", "");
    let failures = gate::workflow_failures(&workflow);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("must disable persisted checkout credentials")),
        "{:?}",
        failures
    );
}

#[test]
fn workflow_rejects_broad_or_write_permissions() {
    let workflow = read_repo_file(".github/workflows/github-lane-unlocker-ci-cd.yml")
        .replace("contents: read", "contents: write");
    let failures = gate::workflow_failures(&workflow);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("must not request broad or write token permissions")),
        "{:?}",
        failures
    );

    let mut workflow = read_repo_file(".github/workflows/github-lane-unlocker-ci-cd.yml");
    workflow.push_str("\npermissions: write-all\n");
    let failures = gate::workflow_failures(&workflow);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("must not request broad or write token permissions")),
        "{:?}",
        failures
    );
}

#[test]
fn workflow_rejects_long_lived_secret_consumption() {
    let mut workflow = read_repo_file(".github/workflows/github-lane-unlocker-ci-cd.yml");
    workflow.push_str("\n      - run: echo '${{ secrets.PERMANENT_TOKEN }}'\n");
    let failures = gate::workflow_failures(&workflow);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("long-lived GitHub secrets")),
        "{:?}",
        failures
    );
}

#[test]
fn spec_rejects_permanent_github_authority_claim() {
    let spec = read_repo_file("specs/github-lane-unlocker-bridge.json").replace(
        "\"github_is_permanent\": false",
        "\"github_is_permanent\": true",
    );
    let failures = gate::spec_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "claim_boundary.github_is_permanent must be false"),
        "{:?}",
        failures
    );
}

#[test]
fn spec_rejects_missing_sandboxed_pattern_source() {
    let spec = read_repo_file("specs/github-lane-unlocker-bridge.json")
        .replace("\"sapling\"", "\"sapling_removed\"");
    let failures = gate::spec_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "pattern_adoption_strategy.source_systems missing sapling"),
        "{:?}",
        failures
    );
}
