// cloud-ci-contract-slice-conformance gate. Reads the committed policy + slice
// specs directly and proves the pure evaluator is Green on the live exemplar and
// RED on each contract-slice doctrine violation. It deliberately does not extend
// any retired local gate CLI authority; merge authority stays cloud-ci via
// oya-ci-required.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use ci_contract_slice_conformance::{GATE_ID, Verdict, evaluate_configured};
use serde_json::{Value, json};

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current dir");
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
    root.join("ci/facade/contract-slice-conformance")
}

fn load_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn load_policy(root: &Path) -> Value {
    load_json(&gate_dir(root).join("contract-slice-policy.json"))
}

fn live_corpus(root: &Path, policy: &Value) -> BTreeMap<String, Value> {
    let mut corpus = BTreeMap::new();
    for slice in policy["slices"].as_array().expect("slices array") {
        // spec_path is repo-root-relative so real specs (specs/*.json) resolve.
        let rel = slice["spec_path"].as_str().expect("spec_path string");
        corpus.insert(rel.to_owned(), load_json(&root.join(rel)));
    }
    corpus
}

#[test]
fn committed_policy_declares_rust_primary_path_and_gate_id() {
    let policy = load_policy(&repo_root());
    assert_eq!(policy["gate_id"], GATE_ID);
    assert_eq!(policy["primary_execution_path"], "rust_buck2_cloud_ci_gate");
    assert!(
        !policy["slices"].as_array().expect("slices").is_empty(),
        "policy must declare at least one slice"
    );
}

#[test]
fn live_exemplar_slice_is_green_under_the_gate() {
    let root = repo_root();
    let policy = load_policy(&root);
    let corpus = live_corpus(&root, &policy);
    let report = evaluate_configured(&policy, &corpus);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "live contract-slice corpus must be green: {:#?}",
        report.findings
    );
}

#[test]
fn red_mutations_match_the_retired_python_validator_contracts() {
    let root = repo_root();
    let policy = load_policy(&root);
    let spec_path = policy["slices"][0]["spec_path"]
        .as_str()
        .expect("spec_path")
        .to_owned();

    // (1) a dropped required field must surface missing_required_field.
    let mut corpus = live_corpus(&root, &policy);
    corpus
        .get_mut(&spec_path)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("non_claims");
    assert!(
        evaluate_configured(&policy, &corpus)
            .violations
            .contains("contract_slice_missing_required_field"),
        "missing required field must be rejected"
    );

    // (2) a baked-in interpreter command must surface forbidden_marker.
    let mut corpus = live_corpus(&root, &policy);
    corpus
        .get_mut(&spec_path)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert(
            "verification".to_owned(),
            json!("python3 scripts/tests/x_check.py"),
        );
    assert!(
        evaluate_configured(&policy, &corpus)
            .violations
            .contains("contract_slice_forbidden_marker"),
        "a python3 command baked into the contract must be rejected"
    );

    // (3) an out-of-enum spec_kind must surface enum_violation.
    let mut corpus = live_corpus(&root, &policy);
    corpus
        .get_mut(&spec_path)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("spec_kind".to_owned(), json!("not-a-contract-slice"));
    assert!(
        evaluate_configured(&policy, &corpus)
            .violations
            .contains("contract_slice_enum_violation"),
        "an out-of-enum spec_kind must be rejected"
    );

    // (4) a non-Rust primary execution path must surface primary_path_not_rust.
    let mut mutated = policy.clone();
    mutated
        .as_object_mut()
        .unwrap()
        .insert("primary_execution_path".to_owned(), json!("python_script"));
    let corpus = live_corpus(&root, &policy);
    assert!(
        evaluate_configured(&mutated, &corpus)
            .violations
            .contains("contract_slice_primary_path_not_rust"),
        "a non-Rust primary execution path must be rejected"
    );
}

/// Proves the converted CELL-002 slice genuinely enforces (not tautologically
/// green): a status downgrade and a dropped source ADR must both be caught.
#[test]
fn cell_002_slice_rejects_status_downgrade_and_missing_source_adr() {
    let root = repo_root();
    let policy = load_policy(&root);
    let cell_spec = "specs/cell-002-promotion-automation-contract.json";
    // The slice must actually be wired into the live policy.
    assert!(
        policy["slices"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["spec_path"] == cell_spec),
        "CELL-002 slice must be declared in the policy"
    );

    // status downgrade Proposed-target -> Accepted violates the enum.
    let mut corpus = live_corpus(&root, &policy);
    corpus
        .get_mut(cell_spec)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("status".to_owned(), json!("Accepted"));
    assert!(
        evaluate_configured(&policy, &corpus)
            .violations
            .contains("contract_slice_enum_violation"),
        "a CELL-002 status downgrade must be rejected"
    );

    // dropping ADR-0341 from source_adrs violates required_array_members.
    let mut corpus = live_corpus(&root, &policy);
    corpus
        .get_mut(cell_spec)
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("source_adrs".to_owned(), json!(["ADR-0348", "ADR-0351"]));
    assert!(
        evaluate_configured(&policy, &corpus)
            .violations
            .contains("contract_slice_missing_array_member"),
        "a missing required source ADR must be rejected"
    );
}

/// Proves the six-input promotion gate is ENFORCED, not just present: dropping an
/// input and flipping a refusal_behavior to best-effort must both RED.
#[test]
fn cell_002_six_input_promotion_gate_is_enforced() {
    let root = repo_root();
    let policy = load_policy(&root);
    let cell_spec = "specs/cell-002-promotion-automation-contract.json";

    // Drop G6 and weaken G1's refusal to best-effort.
    let mut corpus = live_corpus(&root, &policy);
    corpus.get_mut(cell_spec).unwrap()["promotion_gate"]["six_inputs"] = json!([
        { "id": "G1_error_budget", "name": "Error budget intact", "source_adr": "ADR-0341",
          "evidence_authority": "observability", "required_evidence_fields": ["cell_id"],
          "refusal_behavior": "best_effort" }
    ]);
    let violations = evaluate_configured(&policy, &corpus).violations;
    assert!(
        violations.contains("contract_slice_missing_object_array_member"),
        "dropping a promotion-gate input must be rejected: {violations:?}"
    );
    assert!(
        violations.contains("contract_slice_object_member_enum_violation"),
        "a non-fail-closed promotion-gate input must be rejected: {violations:?}"
    );
}
