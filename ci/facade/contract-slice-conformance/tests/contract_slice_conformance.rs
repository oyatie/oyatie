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

/// Proves the rollback-audit fixture SHAPE is now mechanically enforced (the gap
/// CodeRabbit flagged): dropping an audit-row field and flipping post_state both RED.
#[test]
fn cell_002_rollback_audit_fixture_shape_is_enforced() {
    let root = repo_root();
    let policy = load_policy(&root);
    let fixture = "specs/fixtures/cell-002-promotion-automation/rollback-audit-row.json";

    let mut corpus = live_corpus(&root, &policy);
    corpus.get_mut(fixture).unwrap()["audit_row"]
        .as_object_mut()
        .unwrap()
        .remove("rollback_pointer");
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report.findings.iter().any(|finding| finding.code
            == "contract_slice_missing_required_field"
            && finding.key == "cell-002-rollback-audit-fixture:audit_row.rollback_pointer"),
        "the dropped audit-row field must be rejected with its exact key: {:?}",
        report.findings
    );

    let mut corpus = live_corpus(&root, &policy);
    corpus.get_mut(fixture).unwrap()["audit_row"]["post_state"] = json!("Committed");
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "contract_slice_enum_violation"
                && finding.key == "cell-002-rollback-audit-fixture:audit_row.post_state"),
        "a fixture whose post_state is not RolledBack must be rejected with its exact key: {:?}",
        report.findings
    );
}

/// Proves the converted FINOPS-001 slice enforces (retires
/// scripts/tests/finops_cost_attribution_contract_check.py): a status downgrade
/// and a dropped accepted-authority ADR both RED.
#[test]
fn finops_001_cost_attribution_slice_is_enforced() {
    let root = repo_root();
    let policy = load_policy(&root);
    let spec = "specs/finops-cost-attribution.json";

    let mut corpus = live_corpus(&root, &policy);
    corpus.get_mut(spec).unwrap()["_meta"]["status"] = json!("Draft");
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "contract_slice_enum_violation"
                && finding.key == "finops-001-cost-attribution:_meta.status"),
        "a finops status downgrade must be rejected: {:?}",
        report.findings
    );

    let mut corpus = live_corpus(&root, &policy);
    corpus.get_mut(spec).unwrap()["_meta"]["authority_boundary"]["accepted_authority"] =
        json!(["ADR-0198", "ADR-0199"]);
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report.findings.iter().any(|finding| finding.code == "contract_slice_missing_array_member"
            && finding.key
                == "finops-001-cost-attribution:_meta.authority_boundary.accepted_authority:ADR-0174"),
        "dropping ADR-0174 from accepted_authority must be rejected: {:?}",
        report.findings
    );
}

// ---- New-primitive live-corpus RED cases (Phase 5) --------------------------
//
// These exercise the DSL primitives added by the enrichment against the REAL
// committed exemplar fixture (loaded from disk, not a `json!` literal), each
// asserting the exact `Finding.key` a converted slice will depend on. The live
// policy itself is left unchanged — the four conversion PRs add the data entries;
// this PR proves the primitives fire end-to-end with the keys they will use.

const EXEMPLAR_SPEC_PATH: &str =
    "ci/facade/contract-slice-conformance/fixtures/exemplar-slice.json";

fn exemplar_corpus(root: &Path) -> BTreeMap<String, Value> {
    BTreeMap::from([(
        EXEMPLAR_SPEC_PATH.to_owned(),
        load_json(&root.join(EXEMPLAR_SPEC_PATH)),
    )])
}

fn synthetic_policy(slice: Value) -> Value {
    json!({
        "gate_id": GATE_ID,
        "primary_execution_path": "rust_buck2_cloud_ci_gate",
        "slices": [slice],
    })
}

fn has_finding(report: &ci_contract_slice_conformance::Report, code: &str, key: &str) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.code == code && finding.key == key)
}

#[test]
fn exact_array_fields_fires_on_the_live_exemplar_with_exact_key() {
    let root = repo_root();
    let slice = json!({
        "slice_id": "live-exact-array",
        "spec_path": EXEMPLAR_SPEC_PATH,
        "required_fields": [],
        "exact_array_fields": [
            { "field": "required_contract_fields", "values": ["field_a", "field_b"] }
        ]
    });
    // green against the untouched committed fixture.
    assert_eq!(
        evaluate_configured(&synthetic_policy(slice.clone()), &exemplar_corpus(&root)).verdict,
        Verdict::Green
    );
    // reordering the real array trips array_not_exact with its exact key.
    let mut corpus = exemplar_corpus(&root);
    corpus.get_mut(EXEMPLAR_SPEC_PATH).unwrap()["required_contract_fields"] =
        json!(["field_b", "field_a"]);
    let report = evaluate_configured(&synthetic_policy(slice), &corpus);
    assert!(
        has_finding(
            &report,
            "contract_slice_array_not_exact",
            "live-exact-array:required_contract_fields"
        ),
        "reordering the exemplar's required_contract_fields must RED with its exact key: {:?}",
        report.findings
    );
}

#[test]
fn field_patterns_fires_on_the_live_exemplar_with_exact_key() {
    let root = repo_root();
    let slice = json!({
        "slice_id": "live-pattern",
        "spec_path": EXEMPLAR_SPEC_PATH,
        "required_fields": [],
        "field_patterns": [
            { "field": "cloud_ci_gate", "pattern": "^cloud-ci-[a-z0-9-]+$" }
        ]
    });
    assert_eq!(
        evaluate_configured(&synthetic_policy(slice.clone()), &exemplar_corpus(&root)).verdict,
        Verdict::Green
    );
    let mut corpus = exemplar_corpus(&root);
    corpus.get_mut(EXEMPLAR_SPEC_PATH).unwrap()["cloud_ci_gate"] = json!("NOT A GATE ID");
    let report = evaluate_configured(&synthetic_policy(slice), &corpus);
    assert!(
        has_finding(
            &report,
            "contract_slice_pattern_mismatch",
            "live-pattern:cloud_ci_gate"
        ),
        "a cloud_ci_gate that violates the id grammar must RED with its exact key: {:?}",
        report.findings
    );
}

#[test]
fn separator_normalized_forbidden_marker_fires_on_a_live_spec() {
    let root = repo_root();
    let slice = json!({
        "slice_id": "live-forbidden-sep",
        "spec_path": EXEMPLAR_SPEC_PATH,
        "required_fields": [],
        "forbidden_markers": ["production ready"]
    });
    // A hyphenated evasion baked into the real fixture still trips the marker.
    let mut corpus = exemplar_corpus(&root);
    corpus.get_mut(EXEMPLAR_SPEC_PATH).unwrap()["headline"] =
        json!("this substrate is production-ready");
    let report = evaluate_configured(&synthetic_policy(slice), &corpus);
    assert!(
        has_finding(
            &report,
            "contract_slice_forbidden_marker",
            "live-forbidden-sep:production ready"
        ),
        "a separator-substituted forbidden phrase must RED with its exact key: {:?}",
        report.findings
    );
}

#[test]
fn required_markers_any_of_and_array_cardinality_fire_on_the_live_exemplar() {
    let root = repo_root();
    // any_of over the exemplar's non_claims (which already contains "fixture only …").
    let any_of = json!({
        "slice_id": "live-anyof",
        "spec_path": EXEMPLAR_SPEC_PATH,
        "required_fields": [],
        "required_markers": [{
            "field": "non_claims",
            "quantifier": "any_of",
            "markers": ["absent phrase one", "absent phrase two"]
        }]
    });
    let report = evaluate_configured(&synthetic_policy(any_of), &exemplar_corpus(&root));
    assert!(
        has_finding(
            &report,
            "contract_slice_required_marker_none_present",
            "live-anyof:non_claims"
        ),
        "any_of must RED when none of the accepted wordings are present: {:?}",
        report.findings
    );

    // array_cardinality: the exemplar's required_contract_fields has exactly 2 members.
    let cardinality = json!({
        "slice_id": "live-card",
        "spec_path": EXEMPLAR_SPEC_PATH,
        "required_fields": [],
        "array_cardinality": [{ "field": "required_contract_fields", "min": 3 }]
    });
    let report = evaluate_configured(&synthetic_policy(cardinality), &exemplar_corpus(&root));
    assert!(
        has_finding(
            &report,
            "contract_slice_array_below_min",
            "live-card:required_contract_fields"
        ),
        "a 2-member array must RED against min=3 with its exact key: {:?}",
        report.findings
    );
}
