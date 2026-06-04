#![allow(dead_code)]

#[path = "../ci/assert-structural-lock-revert.rs"]
mod gate;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|err| {
        panic!("read {}: {}", path, err);
    })
}

#[test]
fn checked_in_registry_buck_and_fixtures_pass() {
    let evaluation = gate::evaluate(
        Path::new(&repo_root()),
        "specs/structural-lock-revert-registry.json",
    )
    .expect("structural-lock/revert evaluation should run");
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.file_results.len(), 2);
    assert_eq!(evaluation.fixture_results.len(), 5);
    assert!(evaluation.failures.is_empty());
}

#[test]
fn registry_rejects_phase0_completion_claim() {
    let mutated = read_repo_file("specs/structural-lock-revert-registry.json").replacen(
        "\"phase0_complete\": false",
        "\"phase0_complete\": true",
        1,
    );
    let failures = gate::registry_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "forbidden_true_or_missing_claim_phase0_complete"),
        "{:?}",
        failures
    );
}

#[test]
fn missing_revert_fixture_reports_missing_protected_flow_revert() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-missing-protected-revert-evidence.json",
    );
    let failures = gate::fixture_policy_failures(&fixture);
    assert!(failures.is_empty(), "{:?}", failures);
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "missing_protected_flow_revert_evidence"),
        "{:?}",
        observed
    );
}

#[test]
fn overlapping_structural_lanes_report_serialization_violations() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-overlapping-structural-lanes.json",
    );
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "structural_path_overlap_detected"),
        "{:?}",
        observed
    );
    assert!(
        observed
            .iter()
            .any(|item| item == "parallel_structural_lane_not_serialized"),
        "{:?}",
        observed
    );
}

#[test]
fn mechanical_claim_fixture_reports_forbidden_authority_and_status_mutation() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-mechanical-lock-claim.json",
    );
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "forbidden_mechanical_structural_lock_claim"),
        "{:?}",
        observed
    );
    assert!(
        observed
            .iter()
            .any(|item| item == "status_mutation_performed"),
        "{:?}",
        observed
    );
}

#[test]
fn stale_ttl_fixture_reports_ttl_violation() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-structural-lock-revert/tc-0.9-bad-stale-lock-ttl.json",
    );
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "lock_ttl_not_future_or_expired"),
        "{:?}",
        observed
    );
}

#[test]
fn registry_rejects_oya_cli_authority_route() {
    let mut mutated = read_repo_file("specs/structural-lock-revert-registry.json");
    mutated.push_str("\n{\"bad_authority\": \"oya gate run-all as protected branch authority\"}\n");
    let failures = gate::registry_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "registry_maps_to_oya_cli_authority"),
        "{:?}",
        failures
    );
}
