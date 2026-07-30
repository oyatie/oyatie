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
    // Looked up by slice_id, not array position: the sharded-policy migration
    // (contract-slice-policy.json is now GENERATED from slices/*.json, merged in
    // deterministic slice_id order) makes the "slices" array's element order an
    // aggregation implementation detail, not something a caller may rely on.
    let spec_path = policy["slices"]
        .as_array()
        .expect("slices array")
        .iter()
        .find(|slice| slice["slice_id"] == "contract-slice-conformance-exemplar")
        .and_then(|slice| slice["spec_path"].as_str())
        .expect("contract-slice-conformance-exemplar slice must be declared")
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

// ---- Hardening round: fail-open holes found by cross-model review ------------
//
// Each of these FAILS against the pre-hardening evaluator (the hole it guards)
// and passes after. Self-contained: one synthetic slice + spec through the
// public evaluator.

fn eval_slice(mut slice: Value, spec: Value) -> ci_contract_slice_conformance::Report {
    let path = "specs/_synthetic-hardening.json";
    slice["spec_path"] = json!(path);
    let corpus: BTreeMap<String, Value> = BTreeMap::from([(path.to_owned(), spec)]);
    evaluate_configured(&synthetic_policy(slice), &corpus)
}

fn base_spec() -> Value {
    json!({ "slice_id": "x", "spec_kind": "contract-slice" })
}

#[test]
fn hardening_fail_safe_forbidden_matching_catches_every_obfuscation() {
    // A: canonical [a-z0-9]-only matching catches separator + zero-width evasions
    // uniformly, and single-token markers stay substring (python311 trips python3).
    for (label, text, marker) in [
        (
            "zero-width",
            "produc\u{200B}tion\u{200B}ready",
            "production ready",
        ),
        (
            "hyphen",
            "this is production-ready today",
            "production ready",
        ),
        ("zw+hyphen", "produc\u{200B}tion-ready", "production ready"),
    ] {
        let slice = json!({ "required_fields": [], "forbidden_markers": [marker] });
        let mut spec = base_spec();
        spec["claim"] = json!(text);
        assert!(
            eval_slice(slice, spec)
                .violations
                .contains("contract_slice_forbidden_marker"),
            "{label}: an obfuscated forbidden phrase must still be caught"
        );
    }
    // The round-2 substring regression: python311 must trip universal `python3`.
    let mut spec = base_spec();
    spec["verification"] = json!("python311 scripts/x.py");
    assert!(
        eval_slice(json!({ "required_fields": [] }), spec)
            .violations
            .contains("contract_slice_forbidden_marker"),
        "python311 must still trip the universal python3 marker"
    );
}

#[test]
fn hardening_bidi_reorder_control_in_content_is_red_but_i18n_is_green() {
    // P0 round-4: a bidi RLO override renders "production ready" but canonicalizes
    // reversed — its PRESENCE must fail closed.
    let mut spec = base_spec();
    spec["claim"] = json!("production \u{202E}ydaer\u{202C}");
    assert!(
        eval_slice(json!({ "required_fields": [] }), spec)
            .violations
            .contains("contract_slice_bidi_control_in_content"),
        "a bidi-reorder control in content must fail closed"
    );
    // A plain spec and a legitimate Korean-text spec (non-ASCII, no reorder
    // controls) must NOT be flagged.
    assert!(
        !eval_slice(json!({ "required_fields": [] }), base_spec())
            .violations
            .contains("contract_slice_bidi_control_in_content"),
        "a plain spec must not be flagged"
    );
    let mut spec = base_spec();
    spec["title"] = json!("접근 제어 정책 — 시스템 준비 완료");
    assert!(
        !eval_slice(json!({ "required_fields": [] }), spec)
            .violations
            .contains("contract_slice_bidi_control_in_content"),
        "legitimate non-ASCII i18n text must not be flagged"
    );
}

#[test]
fn hardening_conditional_must_subset_of_absent_subject_is_red() {
    // P0.2a: an absent/non-array subject must be a violation, not a vacuous pass.
    let slice = json!({
        "required_fields": [],
        "required_object_array_members": [{
            "field": "rows", "member_key": "id", "members": ["A"],
            "conditional_assertions": [{ "when_member": "A", "field": "tiers", "must_subset_of": ["x"] }]
        }]
    });
    let mut spec = base_spec();
    spec["rows"] = json!([{ "id": "A" }]); // no "tiers" field at all
    assert!(
        eval_slice(slice, spec)
            .violations
            .contains("contract_slice_conditional_field_not_subset"),
        "must_subset_of over an absent subject must RED"
    );
}

#[test]
fn hardening_conditional_malformed_mode_on_nonmatching_row_is_red() {
    // P0.2c: a mode-less assertion whose selector matches no row must still RED.
    let slice = json!({
        "required_fields": [],
        "required_object_array_members": [{
            "field": "rows", "member_key": "id", "members": ["A"],
            "conditional_assertions": [{ "when_member": "B", "field": "x" }]
        }]
    });
    let mut spec = base_spec();
    spec["rows"] = json!([{ "id": "A" }]);
    assert!(
        eval_slice(slice, spec)
            .violations
            .contains("contract_slice_conditional_assertion_no_mode"),
        "a malformed assertion must RED even when its selector matches no row"
    );
}

#[test]
fn hardening_conditional_when_member_in_nonstring_element_is_red() {
    // P0.2c: a non-string element in when_member_in must fail closed.
    let slice = json!({
        "required_fields": [],
        "required_object_array_members": [{
            "field": "rows", "member_key": "id", "members": ["A"],
            "conditional_assertions": [{ "when_member_in": ["A", 5], "field": "x", "must_be_true": true }]
        }]
    });
    let mut spec = base_spec();
    spec["rows"] = json!([{ "id": "A", "x": true }]);
    assert!(
        eval_slice(slice, spec)
            .violations
            .contains("contract_slice_conditional_assertion_bad_selector"),
        "a non-string when_member_in element must fail closed"
    );
}

#[test]
fn hardening_field_implies_required_string_antecedent_triggers() {
    // P0.3: a string "true" antecedent must not silently disable the implication.
    let slice = json!({
        "required_fields": [],
        "required_object_array_members": [{
            "field": "regimes", "member_key": "id", "members": ["A"],
            "field_implies_required": [{ "if_field": "guard", "then_required_fields": ["companion"] }]
        }]
    });
    let mut spec = base_spec();
    spec["regimes"] = json!([{ "id": "A", "guard": "true" }]); // string, companion absent
    assert!(
        eval_slice(slice, spec)
            .violations
            .contains("contract_slice_conditional_required_field_absent"),
        "a wrong-typed antecedent must trigger the implication (fail closed)"
    );
}

#[test]
fn hardening_enum_is_string_strict_by_default_and_scalar_is_opt_in() {
    // P1.4: default enum is type-preserving; a number does not satisfy "90".
    let strict =
        json!({ "required_fields": [], "enum_constraints": [{ "field": "n", "allowed": ["90"] }] });
    let mut spec = base_spec();
    spec["n"] = json!(90);
    assert!(
        eval_slice(strict, spec.clone())
            .violations
            .contains("contract_slice_enum_violation"),
        "a numeric leaf must NOT satisfy a string-authored enum by default"
    );
    // opt-in restores the numeric-pin behavior.
    let opt_in = json!({
        "required_fields": [],
        "enum_constraints": [{ "field": "n", "allowed": ["90"], "match_scalar": true }]
    });
    assert_eq!(
        eval_slice(opt_in, spec).verdict,
        Verdict::Green,
        "match_scalar: true must accept the numeric leaf"
    );
}

#[test]
fn hardening_required_array_members_is_string_strict_by_default() {
    // P1.4: default array membership is type-preserving.
    let slice = json!({
        "required_fields": [],
        "required_array_members": [{ "field": "arr", "members": ["90"] }]
    });
    let mut spec = base_spec();
    spec["arr"] = json!([90]);
    assert!(
        eval_slice(slice, spec)
            .violations
            .contains("contract_slice_missing_array_member"),
        "a numeric array element must NOT satisfy a string-authored member by default"
    );
}

#[test]
fn hardening_malformed_string_list_configs_fail_closed() {
    // P2: a non-string element in a string-list config must fail closed, not drop.
    let cases = [
        json!({ "required_fields": [], "enum_constraints": [{ "field": "f", "allowed": [1] }] }),
        json!({ "required_fields": [], "required_array_members": [{ "field": "f", "members": [1] }] }),
        json!({ "required_fields": [], "exact_array_fields": [{ "field": "f", "values": [1] }] }),
        json!({ "required_fields": [], "required_markers": [{ "field": "f", "markers": [1] }] }),
        json!({ "required_fields": [], "required_false_fields": ["ok", 5] }),
        json!({ "required_fields": [], "exact_projected_sequence": [{ "field": "f", "member_field": "m", "values": [1] }] }),
        json!({ "required_fields": [], "projected_value_sets": [{ "field": "f", "member_field": "m", "exact_values": [1] }] }),
        // round-3 finding B: the same silent-drop lived in these sibling paths.
        json!({ "required_fields": [], "forbidden_markers": ["blocked", 7] }),
        json!({ "required_fields": [], "forbidden_field_markers": [{ "field": "f", "markers": ["blocked", 7] }] }),
        json!({ "required_fields": [], "required_object_array_members": [{ "field": "f", "members": ["A", 5] }] }),
        json!({ "required_fields": [], "required_object_array_members": [{ "field": "f", "members": ["A"], "member_enum_constraints": [{ "field": "g", "allowed": ["ok", false] }] }] }),
    ];
    for slice in cases {
        let report = eval_slice(slice.clone(), base_spec());
        assert!(
            report
                .violations
                .iter()
                .any(|code| code.contains("malformed")),
            "a mistyped string-list must fail closed with a *_malformed finding: {slice} -> {:?}",
            report.findings
        );
    }
}

#[test]
fn hardening_wrong_typed_cardinality_and_empty_pattern_fail_closed() {
    // P2: a string cardinality bound and an empty regex must fail closed.
    let card = json!({
        "required_fields": [],
        "array_cardinality": [{ "field": "arr", "min": "1" }]
    });
    let mut spec = base_spec();
    spec["arr"] = json!([{ "id": "a" }]);
    assert!(
        eval_slice(card, spec.clone())
            .violations
            .contains("contract_slice_array_cardinality_malformed"),
        "a string min bound must fail closed"
    );
    let pattern = json!({
        "required_fields": [],
        "field_patterns": [{ "field": "f", "pattern": "" }]
    });
    let mut spec = base_spec();
    spec["f"] = json!("anything");
    assert!(
        eval_slice(pattern, spec)
            .violations
            .contains("contract_slice_bad_pattern"),
        "an empty regex (matches everything) must fail closed"
    );
}

#[test]
fn hardening_non_bool_match_scalar_fails_closed() {
    // C: match_scalar as a string must not be silently treated as false.
    for slice in [
        json!({ "required_fields": [], "enum_constraints": [{ "field": "f", "allowed": ["x"], "match_scalar": "true" }] }),
        json!({ "required_fields": [], "required_array_members": [{ "field": "f", "members": ["x"], "match_scalar": "true" }] }),
    ] {
        let report = eval_slice(slice.clone(), base_spec());
        assert!(
            report
                .violations
                .contains("contract_slice_malformed_policy_value"),
            "a non-bool match_scalar must fail closed: {slice} -> {:?}",
            report.findings
        );
    }
}

#[test]
fn hardening_non_array_primitive_container_fails_closed() {
    // D: a primitive whose top-level config is an OBJECT (not a list-of-rules)
    // must fail closed instead of silently no-opping to Green.
    for key in [
        "enum_constraints",
        "required_array_members",
        "exact_array_fields",
        "required_object_array_members",
        "required_markers",
        "forbidden_field_markers",
        "field_patterns",
        "exact_projected_sequence",
        "array_cardinality",
        "projected_value_sets",
    ] {
        let mut slice = json!({ "required_fields": [] });
        slice[key] = json!({ "oops": "not a list of rules" });
        let report = eval_slice(slice, base_spec());
        assert!(
            report
                .violations
                .contains("contract_slice_malformed_policy_value"),
            "a non-array {key} container must fail closed: {:?}",
            report.findings
        );
    }
}

/// Proves the converted TALOS-001 slice enforces (retires
/// scripts/tests/talos_001_substrate_slice_check.py): demoting an Accepted
/// authority out of its enum boundary and dropping a required-surface source
/// ADR both RED.
#[test]
fn talos_001_substrate_slice_is_enforced() {
    let root = repo_root();
    let policy = load_policy(&root);
    let spec = "specs/talos-001-substrate-slice.json";
    assert!(
        policy["slices"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["spec_path"] == spec),
        "TALOS-001 slice must be declared in the policy"
    );

    // Demoting ADR-0378 out of Accepted must violate the enum boundary. (This
    // assertion previously targeted ADR-0382's Proposed pin; ADR-0382 is Rejected
    // and no longer an authority of this slice, so the enum-boundary property is
    // re-anchored on a live Accepted authority rather than dropped.)
    let mut corpus = live_corpus(&root, &policy);
    corpus.get_mut(spec).unwrap()["authority"]["ADR-0378"]["status"] = json!("Proposed");
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "contract_slice_enum_violation"
                && finding.key == "talos-001-substrate-slice:authority.ADR-0378.status"),
        "demoting ADR-0378 out of Accepted must be rejected: {:?}",
        report.findings
    );

    // Dropping ADR-0378 from the local vfkit/Talos smoke surface's source_adrs
    // must violate the per-member conditional must_contain assertion.
    let mut corpus = live_corpus(&root, &policy);
    let matrix = corpus.get_mut(spec).unwrap()["matrix"].as_array_mut().unwrap();
    let row = matrix
        .iter_mut()
        .find(|row| row["id"] == "local_vfkit_talos_smoke")
        .unwrap();
    row["source_adrs"] = json!(["ADR-0370"]);
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report.findings.iter().any(|finding| {
            finding.code == "contract_slice_conditional_field_missing_contains"
                && finding.key
                    == "talos-001-substrate-slice:matrix:local_vfkit_talos_smoke:source_adrs:ADR-0378"
        }),
        "dropping ADR-0378 from local_vfkit_talos_smoke.source_adrs must be rejected: {:?}",
        report.findings
    );

    // Dropping a declared source_paths member (not just source_adrs) must also violate
    // the per-member must_contain assertion — every path TALOS-001 claims to validate stays
    // pinned, so silently narrowing the matrix's source-path boundary is caught.
    let mut corpus = live_corpus(&root, &policy);
    let matrix = corpus.get_mut(spec).unwrap()["matrix"].as_array_mut().unwrap();
    let row = matrix
        .iter_mut()
        .find(|row| row["id"] == "local_vfkit_talos_smoke")
        .unwrap();
    row["source_paths"] = json!(["infra/talos/local/talos-local.sh"]);
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report.findings.iter().any(|finding| {
            finding.code == "contract_slice_conditional_field_missing_contains"
                && finding.key
                    == "talos-001-substrate-slice:matrix:local_vfkit_talos_smoke:source_paths:infra/talos/smoke-kata.sh"
        }),
        "dropping infra/talos/smoke-kata.sh from local_vfkit_talos_smoke.source_paths must be \
         rejected: {:?}",
        report.findings
    );

    // Removing a whole matrix member must violate the exact-set membership check.
    // (Previously targeted sidero_zero_day_matrix, removed with ADR-0382's
    // rejection; re-anchored on a live member so exact-set enforcement stays
    // covered rather than silently losing a case.)
    let mut corpus = live_corpus(&root, &policy);
    corpus.get_mut(spec).unwrap()["matrix"]
        .as_array_mut()
        .unwrap()
        .retain(|row| row["id"] != "managed_k8s_surface");
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report.findings.iter().any(|finding| {
            finding.code == "contract_slice_missing_object_array_member"
                && finding.key == "talos-001-substrate-slice:matrix:managed_k8s_surface"
        }),
        "dropping the managed_k8s_surface member must be rejected: {:?}",
        report.findings
    );

    // A duplicate matrix id (same id twice) must violate cardinality uniqueness.
    let mut corpus = live_corpus(&root, &policy);
    let matrix = corpus.get_mut(spec).unwrap()["matrix"].as_array_mut().unwrap();
    let dup = matrix[0].clone();
    matrix.push(dup);
    let report = evaluate_configured(&policy, &corpus);
    assert!(
        report.findings.iter().any(|finding| {
            finding.code == "contract_slice_array_not_unique"
                && finding.key == "talos-001-substrate-slice:matrix:id:local_vfkit_talos_smoke"
        }),
        "a duplicate matrix id must be rejected: {:?}",
        report.findings
    );
}
