// cloud-ci-hyperscaler-parity-taxonomy live-corpus gate. ADR-0083 Tier-3: integration
// tests assert with unwrap/expect.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use ci_parity_claim_evidence::{Verdict, evaluate, evaluate_keyed};
use serde_json::Value;

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

fn live_spec() -> Value {
    let path = repo_root().join("specs/cloud-hyperscaler-parity-taxonomy.json");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

#[test]
fn live_hyperscaler_taxonomy_is_green() {
    let spec = live_spec();
    let findings = evaluate_keyed(&spec);
    assert!(
        findings.is_empty(),
        "live hyperscaler parity taxonomy must satisfy the Rust cloud-ci gate: {findings:?}"
    );
    assert_eq!(evaluate(&spec).verdict, Verdict::Green);
}

#[test]
fn live_gate_rejects_mutated_overclaims_and_external_ci_lanes() {
    let mut spec = live_spec();
    spec["claim_matrix"]["can_claim_now"][0]["claim"] = Value::String(
        "Oyatie Cloud is production-ready with feature-parity and public SLA.".to_owned(),
    );
    assert!(
        evaluate(&spec)
            .violations
            .contains("claim_matrix_can_claim_now_forbidden_overclaim")
    );

    let mut spec = live_spec();
    spec["strict_separation_constraints"]["allowed_evidence_lanes"]
        .as_array_mut()
        .expect("allowed_evidence_lanes array")
        .push(Value::String("GitHub Actions".to_owned()));
    assert!(
        evaluate(&spec)
            .violations
            .contains("forbidden_allowed_evidence_lane")
    );
}

#[test]
fn live_gate_rejects_unofficial_sources_and_vague_evidence() {
    let mut spec = live_spec();
    spec["official_source_evidence"][0]["url"] =
        Value::String("https://example.com/not-official".to_owned());
    assert!(
        evaluate(&spec)
            .violations
            .contains("source_url_not_official")
    );

    let mut spec = live_spec();
    spec["category_taxonomy"][0]["required_evidence"] =
        Value::Array(vec![Value::String("TODO evidence later".to_owned())]);
    let violations = evaluate(&spec).violations;
    assert!(violations.contains("category_missing_required_evidence_class"));
    assert!(violations.contains("category_vague_evidence_marker"));
}
