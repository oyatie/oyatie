#![allow(dead_code)]

#[path = "../ci/assert-cross-artifact-agreement.rs"]
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
fn checked_in_registry_packets_buck_and_fixtures_pass() {
    let evaluation = gate::evaluate(
        Path::new(&repo_root()),
        "specs/cross-artifact-agreement-registry.json",
    )
    .expect("cross-artifact agreement evaluation should run");
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.file_results.len(), 6);
    assert_eq!(evaluation.fixture_results.len(), 5);
    assert!(evaluation.failures.is_empty());
}

#[test]
fn packet_registry_rejects_missing_register_18_packet() {
    let mutated = read_repo_file("specs/decision-propagation-packets.json").replacen(
        "P0.7-D18-merge-conflict-elimination",
        "P0.7-D18-missing-merge-conflict-elimination",
        1,
    );
    let failures = gate::packet_registry_failures(&mutated);
    assert!(
        failures.iter().any(|failure| failure
            == "missing_decision_propagation_packet_P0.7-D18-merge-conflict-elimination"),
        "{:?}",
        failures
    );
}

#[test]
fn packet_registry_rejects_missing_roadmap_artifact() {
    let packet = read_repo_file("specs/decision-propagation-packets.json");
    let token = "\"roadmap\": {";
    let mutated = packet.replacen(token, "\"roadmap_missing\": {", 1);
    let failures = gate::packet_registry_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("missing_agreement_artifact_roadmap")),
        "{:?}",
        failures
    );
}

#[test]
fn missing_masterplan_roadmap_fixture_reports_both_violations() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-missing-masterplan-roadmap.json",
    );
    let failures = gate::fixture_policy_failures(&fixture);
    assert!(failures.is_empty(), "{:?}", failures);
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "missing_masterplan_artifact")
    );
    assert!(
        observed
            .iter()
            .any(|item| item == "missing_roadmap_artifact")
    );
}

#[test]
fn idea_refine_fixture_reports_unreconciled_output() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-unreconciled-idea-refine-output.json",
    );
    let failures = gate::fixture_policy_failures(&fixture);
    assert!(failures.is_empty(), "{:?}", failures);
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "idea_refine_output_unreconciled"),
        "{:?}",
        observed
    );
}

#[test]
fn generated_divergence_fixture_reports_decisions_json_drift() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-cross-artifact-agreement/tc-0.8-bad-generated-decisions-divergence.json",
    );
    let failures = gate::fixture_policy_failures(&fixture);
    assert!(failures.is_empty(), "{:?}", failures);
    let observed = gate::fixture_observed_violations(&fixture);
    assert!(
        observed
            .iter()
            .any(|item| item == "generated_decisions_json_diverged"),
        "{:?}",
        observed
    );
}

#[test]
fn registry_rejects_phase0_completion_claim() {
    let mutated = read_repo_file("specs/cross-artifact-agreement-registry.json").replacen(
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
