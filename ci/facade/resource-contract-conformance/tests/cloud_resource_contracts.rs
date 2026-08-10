// cloud-ci-cloud-resource-contracts live-corpus gate. ADR-0083 Tier-3: integration tests
// assert with unwrap/expect while production evaluation remains panic-free.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use ci_resource_contract_conformance::{Verdict, evaluate_configured};
use serde_json::{Value, json};

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn gate_dir(root: &Path) -> PathBuf {
    root.join("ci/facade/resource-contract-conformance")
}

fn load_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn load_policy(root: &Path) -> Value {
    load_json(&gate_dir(root).join("cloud-resource-contracts-policy.json"))
}

fn live_corpus(root: &Path, policy: &Value) -> Value {
    let spec_inputs = policy["spec_inputs"]
        .as_object()
        .expect("policy spec_inputs");
    let mut corpus = serde_json::Map::new();
    for (key, rel) in spec_inputs {
        let rel = rel
            .as_str()
            .unwrap_or_else(|| panic!("spec_inputs.{key} must be a string"));
        corpus.insert(key.clone(), load_json(&root.join(rel)));
    }
    Value::Object(corpus)
}

#[test]
fn committed_policy_declares_retired_python_sources_and_rust_primary_path() {
    let policy = load_policy(&repo_root());
    assert_eq!(policy["gate_id"], "cloud-ci-cloud-resource-contracts");
    assert_eq!(policy["primary_execution_path"], "rust_buck2_cloud_ci_gate");

    let retired: Vec<&str> = policy["source_migration_slice"]
        .as_array()
        .expect("source_migration_slice")
        .iter()
        .map(|row| row["legacy_path"].as_str().expect("legacy_path"))
        .collect();
    assert_eq!(
        retired,
        vec![
            "scripts/tests/cloud_resource_contract_parity_catalog_check.py",
            "scripts/tests/cloud_control_plane_operation_contract_check.py",
            "scripts/tests/cloud_enforceability_facets_check.py",
        ],
        "the first migration slice must explicitly cover the three P0 cloud-resource Python validators"
    );
    assert!(policy["source_migration_slice"].as_array().unwrap().iter().all(|row| {
        row["replacement_target"]
            == "//ci/facade/resource-contract-conformance:ci-resource-contract-conformance-gate"
            && row["disposition"] == "retired_primary_path"
    }));
}

#[test]
fn live_cloud_resource_contracts_are_green_under_rust_gate() {
    let root = repo_root();
    let policy = load_policy(&root);
    let corpus = live_corpus(&root, &policy);
    let report = evaluate_configured(&policy, &corpus);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "live cloud resource contracts must satisfy the Rust cloud-ci gate: {:#?}",
        report.findings
    );
}

#[test]
fn red_mutations_match_the_retired_python_validator_contracts() {
    let root = repo_root();
    let policy = load_policy(&root);

    let mut corpus = live_corpus(&root, &policy);
    corpus["cloud_resource_contract_parity_catalog"]["claim_controls"]["can_claim_now"] =
        json!(["feature parity with AWS is production ready"]);
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report
            .violations
            .contains("cloud_resource_contract_forbidden_positive_claim"),
        "resource catalog feature/parity overclaim must be rejected: {:#?}",
        report.findings
    );

    let mut corpus = live_corpus(&root, &policy);
    corpus["cloud_control_plane_operation_contract"]["operation_ledger_entry"]["required_fields"] =
        json!(["operation_id"]);
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report
            .violations
            .contains("cloud_operation_ledger_missing_field"),
        "operation ledger missing required fields must be rejected: {:#?}",
        report.findings
    );

    let mut corpus = live_corpus(&root, &policy);
    corpus["cloud_enforceability_facets"]["resource_enforceability"][0]["cedar_policy"]["default"] =
        json!("allow");
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report
            .violations
            .contains("cloud_enforceability_cedar_not_default_deny"),
        "Cedar allow-by-default mutation must be rejected: {:#?}",
        report.findings
    );
}
