#![allow(dead_code)]

#[path = "../ci/assert-cloud-cell-elasticity-policy.rs"]
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
fn checked_in_cloud_cell_elasticity_policy_passes() {
    let evaluation = gate::evaluate(Path::new(&repo_root()));
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.failures.is_empty());
    assert_eq!(evaluation.official_sources, 19);
    assert_eq!(evaluation.policy_ids, 10);
    assert_eq!(evaluation.forbidden_anti_patterns, 7);
}

#[test]
fn contract_rejects_first_party_helm_authority() {
    let spec = read_repo_file("specs/cloud-cell-elasticity-policy.json").replace(
        "\"first_party_helm_template_authority\": false",
        "\"first_party_helm_template_authority\": true",
    );
    let failures = gate::contract_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("first_party_helm_template_authority=false")),
        "{:?}",
        failures
    );
}

#[test]
fn contract_rejects_missing_cue_authority() {
    let spec = read_repo_file("specs/cloud-cell-elasticity-policy.json").replace(
        "\"first_party_desired_state\": \"cue\"",
        "\"first_party_desired_state\": \"helm\"",
    );
    let failures = gate::contract_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("first-party desired state must be CUE")),
        "{:?}",
        failures
    );
}

#[test]
fn contract_rejects_missing_scale_to_zero_gate() {
    let spec = read_repo_file("specs/cloud-cell-elasticity-policy.json").replace(
        "\"id\": \"scale_to_zero_eligibility_gate\"",
        "\"id\": \"scale_to_zero_eligibility_gate_removed\"",
    );
    let failures = gate::contract_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "policy/backlog id missing scale_to_zero_eligibility_gate"),
        "{:?}",
        failures
    );
}

#[test]
fn contract_rejects_missing_gke_and_cue_sources() {
    let spec = read_repo_file("specs/cloud-cell-elasticity-policy.json")
        .replace(
            "https://cloud.google.com/kubernetes-engine/docs/concepts/horizontalpodautoscaler",
            "https://example.invalid/missing-hpa",
        )
        .replace(
            "https://cue.dev/docs/getting-started-with-kubernetes-cue/",
            "https://example.invalid/missing-cue",
        );
    let failures = gate::contract_failures(&spec);
    assert!(
        failures.iter().any(|failure| failure.contains(
            "https://cloud.google.com/kubernetes-engine/docs/concepts/horizontalpodautoscaler"
        )),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure
                .contains("https://cue.dev/docs/getting-started-with-kubernetes-cue/")),
        "{:?}",
        failures
    );
}
