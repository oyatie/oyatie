#![allow(dead_code)]

#[path = "../ci/assert-d1-read-your-writes-xfail.rs"]
mod gate;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|err| panic!("read {}: {}", path, err))
}

#[test]
fn checked_in_registry_buck_and_fixtures_pass() {
    let evaluation = gate::evaluate(
        Path::new(&repo_root()),
        "specs/d1-read-your-writes-xfail-registry.json",
    )
    .expect("D1 read-your-writes XFAIL evaluation should run");
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.file_results.len(), 2);
    assert_eq!(evaluation.fixture_results.len(), 4);
    assert!(evaluation.failures.is_empty());
}

#[test]
fn registry_rejects_phase2_green_claim() {
    let mutated = read_repo_file("specs/d1-read-your-writes-xfail-registry.json").replacen(
        "\"phase2_mechanism_landed\": false",
        "\"phase2_mechanism_landed\": true",
        1,
    );
    let failures = gate::registry_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "forbidden_true_or_missing_claim_phase2_mechanism_landed"),
        "{:?}",
        failures
    );
}

#[test]
fn good_fixture_is_xfail_without_policy_violations() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-good-xfail-classified-read-your-writes.json",
    );
    assert!(gate::fixture_policy_failures(&fixture).is_empty());
    assert!(gate::fixture_observed_violations(&fixture).is_empty());
}

#[test]
fn pass_classification_without_phase2_is_rejected() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-misclassified-green-without-phase2.json",
    );
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "xfail_misclassified_as_pass"),
        "{:?}",
        observed
    );
}

#[test]
fn missing_consistency_token_is_rejected() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-missing-consistency-token.json",
    );
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "missing_consistency_token"),
        "{:?}",
        observed
    );
}

#[test]
fn live_phase2_claim_without_authority_is_rejected() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-d1-read-your-writes-xfail/tc-0.10b-bad-phase2-green-claim-without-live-evidence.json",
    );
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "phase2_green_claim_without_live_evidence"),
        "{:?}",
        observed
    );
    assert!(
        observed
            .iter()
            .any(|item| item == "live_d1_conformance_claimed"),
        "{:?}",
        observed
    );
}

#[test]
fn registry_rejects_oya_cli_authority_route() {
    let mut mutated = read_repo_file("specs/d1-read-your-writes-xfail-registry.json");
    mutated.push_str("\n{\"bad_authority\": \"oya verify as live D1 conformance\"}\n");
    let failures = gate::registry_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "registry_maps_to_oya_cli_authority"),
        "{:?}",
        failures
    );
}
