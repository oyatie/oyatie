#![allow(dead_code)]

#[path = "../ci/assert-d1-seam-contracts.rs"]
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
fn checked_in_registry_contracts_and_fixtures_pass() {
    let evaluation = gate::evaluate(
        Path::new(&repo_root()),
        "specs/d1-seam-contracts-registry.json",
    )
    .expect("D1 seam contract evaluation should run");
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.contract_results.len(), 2);
    assert_eq!(evaluation.fixture_results.len(), 3);
    assert!(evaluation.failures.is_empty());
}

#[test]
fn a2a_contract_rejects_missing_optional_consistency_token() {
    let mutated = read_repo_file("contracts/proto/d1/a2a/mutation/v1/entity_mutation.proto")
        .replace(
            "optional string consistency_token = 4;",
            "string stale_consistency_token = 4;",
        );
    let failures = gate::a2a_proto_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("consistency_token")),
        "{:?}",
        failures
    );
}

#[test]
fn a2a_contract_rejects_proto_required_consistency_token() {
    let mutated = read_repo_file("contracts/proto/d1/a2a/mutation/v1/entity_mutation.proto")
        .replace(
            "optional string consistency_token = 4;",
            "required string consistency_token = 4;",
        );
    let failures = gate::a2a_proto_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "a2a:proto_required_consistency_token"),
        "{:?}",
        failures
    );
}

#[test]
fn a2b_contract_rejects_missing_idempotency_key() {
    let mutated =
        read_repo_file("contracts/proto/d1/a2b/workflow/v1/workflow_ai_step_invocation.proto")
            .replace(
                "string idempotency_key = 6;",
                "string missing_key_probe = 6;",
            );
    let failures = gate::a2b_proto_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("missing_field:idempotency_key")),
        "{:?}",
        failures
    );
}

#[test]
fn bad_fixture_reports_missing_consistency_token() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-d1-seam-contracts/tc-0.5-bad-missing-consistency-token.json",
    );
    let failures = gate::fixture_policy_failures(&fixture);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "missing_consistency_token"),
        "{:?}",
        failures
    );
}

#[test]
fn registry_rejects_phase0_completion_claim() {
    let mutated = read_repo_file("specs/d1-seam-contracts-registry.json").replacen(
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
fn automation_matrix_maps_ac010_to_d1_seam_gate() {
    let matrix = read_repo_file("specs/phase0-automation-matrix.json");
    let coverage = read_repo_file("specs/phase0-automation-coverage-registry.json");
    let compact_matrix = matrix
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    let compact_coverage = coverage
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    assert!(compact_matrix.contains("\"id\":\"AC-0.10-d1-consistency-token\""));
    assert!(
        compact_matrix.contains("\"target_gate_or_controller\":\"//:d1-seam-contracts-check\"")
    );
    assert!(
        matrix.contains("\"verification_command\": \"buck2 build //:d1-seam-contracts-check\"")
    );
    assert!(compact_matrix.contains("consistency_token"));
    assert!(compact_matrix.contains("\"live_required_context_execution_proven\":false"));
    assert!(compact_matrix.contains("\"p0_0_green\":false"));
    assert!(compact_matrix.contains("\"phase0_complete\":false"));
    assert!(compact_coverage.contains("\"id\":\"AC-0.10\""));
    assert!(compact_coverage.contains("//:d1-seam-contracts-check"));
    assert!(compact_coverage.contains("consistency_token"));
}
