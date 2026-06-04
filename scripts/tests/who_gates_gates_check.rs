#![allow(dead_code)]

#[path = "../ci/assert-who-gates-gates.rs"]
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
        "specs/who-gates-gates-registry.json",
    )
    .expect("who-gates-the-gates evaluation should run");
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.file_results.len(), 2);
    assert_eq!(evaluation.fixture_results.len(), 5);
    assert!(evaluation.failures.is_empty());
}

#[test]
fn registry_rejects_phase0_green_claim() {
    let mutated = read_repo_file("specs/who-gates-gates-registry.json").replacen(
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
fn actual_gate_surface_requires_self_mutations() {
    let red_green = read_repo_file("specs/red-green-fixture-contract.json");
    let harness = read_repo_file("scripts/tests/red_green_fixture_contract_check.test.sh");
    let buck = read_repo_file("BUCK");
    assert!(gate::actual_gate_surface_failures(&red_green, &harness, &buck).is_empty());
    let failures = gate::actual_gate_surface_failures(
        &red_green,
        &harness.replace("remove-red-marker", "removed-marker"),
        &buck,
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure == "red_green_harness_missing_self_mutation_remove-red-marker"),
        "{:?}",
        failures
    );
}

#[test]
fn good_fixture_has_no_observed_violations() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-who-gates-gates/tc-0.11-good-known-bad-meta-gate.json",
    );
    assert!(gate::fixture_policy_failures(&fixture).is_empty());
    assert!(gate::fixture_observed_violations(&fixture).is_empty());
}

#[test]
fn missing_known_bad_fixture_is_rejected() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-missing-known-bad-fixture.json",
    );
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "missing_known_bad_fixture"),
        "{:?}",
        observed
    );
}

#[test]
fn vacuous_pass_condition_is_rejected() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-vacuous-pass-condition.json",
    );
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed.iter().any(|item| item == "vacuous_pass_condition"),
        "{:?}",
        observed
    );
}

#[test]
fn missing_self_mutation_test_is_rejected() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-missing-self-mutation-test.json",
    );
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "missing_self_mutation_test"),
        "{:?}",
        observed
    );
}

#[test]
fn oya_cli_authority_route_is_rejected_case_insensitively() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-who-gates-gates/tc-0.11-bad-oya-cli-authority-route.json",
    );
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "oya_cli_authority_route"),
        "{:?}",
        observed
    );
    let mutated = fixture.replace("oya verify", "OYA VERIFY");
    let observed_upper = gate::fixture_observed_violations(&mutated);
    assert!(
        observed_upper
            .iter()
            .any(|item| item == "oya_cli_authority_route"),
        "{:?}",
        observed_upper
    );
}
