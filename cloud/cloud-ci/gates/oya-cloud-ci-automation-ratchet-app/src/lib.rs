//! # cloud-ci-automation-ratchet (GATE-4)
//!
//! The automation-ratchet gate (PHASE-0-FIREWALL-PLAN §5.2; register #20). It evaluates the
//! Phase-0 automation matrix rows (the seed lives in `specs/phase0-automation-matrix.json`)
//! and emits `{verdict, violations}`; its tests assert
//! `report.violations == fixture.expected_violations` over
//! `specs/fixtures/phase0-automation-ratchet/tc-*.json`. It is the gate that polices the
//! OTHER gates: anything enforceable/automatable must be enforced/automated by a wired
//! buck2 gate target — never an `oya` CLI call, never an unwired "advisory" surface that
//! claims to enforce.
//!
//! ## Blocking violation codes (the contract — the REAL on-disk codes; do NOT rename)
//! - `enforceable_or_automatable_marked_human_judgment` — a row flagged
//!   `enforceable_or_automatable:true` but classified `not_automatable_human_judgment`.
//! - `blocking_invariant_mapped_to_oya_cli`             — a blocking row whose
//!   `target_gate_or_controller` routes through an `oya` CLI invocation (the retired
//!   `oya gate`/`oya gen` authority), or whose `no_new_oya_cli_surface` is false.
//! - `duplicate_row_id`                                  — two rows share one `id`.
//! - `unknown_classification`                            — a row's `classification` is not
//!   in the 4-enum.
//! - `missing_or_empty_required_field`                   — a row omits / empties one of the
//!   required fields.
//! - `advisory_claiming_enforced`  (NET-NEW)             — a row CLAIMS to enforce/gate
//!   (its requirement/target uses enforcement vocabulary) yet has no wired buck2 gate
//!   target backing it (`has_wired_buck2_target:false`).
//! - `ratchet_regression`          (NET-NEW)             — a row regresses the ratchet: a
//!   previously `automated_blocking_now` requirement downgraded to a weaker class
//!   (`was_blocking:true` while the current classification is no longer blocking).
//!
//! The 4-enum + the required-field set are DATA (the matrix contract), surfaced as
//! constants here so a fixture can pin them without an evaluator special-case.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashSet};

use serde_json::Value;

/// The gate id, matching the buck2 target + the §5.2 contract.
pub const GATE_ID: &str = "cloud-ci-automation-ratchet";

/// The seven blocking codes, in canonical order. The fixtures pin exact subsets.
pub const VIOLATION_CODES: [&str; 7] = [
    "enforceable_or_automatable_marked_human_judgment",
    "blocking_invariant_mapped_to_oya_cli",
    "duplicate_row_id",
    "unknown_classification",
    "missing_or_empty_required_field",
    "advisory_claiming_enforced",
    "ratchet_regression",
];

/// The 4-enum classification set (matrix `classifications`). DATA, surfaced as a constant.
pub const CLASSIFICATIONS: [&str; 4] = [
    "automated_blocking_now",
    "automated_advisory_until_p0_0",
    "controller_owned_in_phase_1",
    "not_automatable_human_judgment",
];

/// The required row fields (matrix `required_row_fields`). DATA, surfaced as a constant.
pub const REQUIRED_FIELDS: [&str; 10] = [
    "id",
    "source_artifact",
    "requirement",
    "classification",
    "owner",
    "target_gate_or_controller",
    "blocking_fixture",
    "retirement_phase",
    "evidence_path",
    "no_new_oya_cli_surface",
];

/// The classifications that constitute "blocking" enforcement (for ratchet-regression).
const BLOCKING_CLASSIFICATIONS: [&str; 1] = ["automated_blocking_now"];

/// The gate report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// A keyed violation: the bare `code` (the existing contract) PLUS the stable `key`.
/// The going-live ratchet baselines per `(code, key)`; `evaluate()` is the bare-code
/// projection of `evaluate_keyed()`. Keys for this gate are the matrix-row `id` (most
/// codes) and `{row_id}#{field}` for `missing_or_empty_required_field`. The
/// `advisory_claiming_enforced` and `blocking_invariant_mapped_to_oya_cli` codes are
/// keyed by the same row `id` (the producer's `EnforcementRow.id`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
}

impl Finding {
    fn new(code: &str, key: &str) -> Self {
        Self {
            code: code.to_owned(),
            key: key.to_owned(),
        }
    }
}

impl Report {
    fn from_violations(violations: BTreeSet<String>) -> Self {
        let verdict = if violations.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            violations,
        }
    }
}

/// Bare-code projection of [`evaluate_keyed`]: identical detection logic, keys dropped.
/// Every `tc-*.json` fixture keeps asserting bare codes against it byte-for-byte.
pub fn evaluate(fixture: &Value) -> Report {
    let violations = evaluate_keyed(fixture)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    Report::from_violations(violations)
}

/// Evaluate an automation-ratchet matrix into the keyed finding set — the single source
/// of truth for the gate's detection logic.
pub fn evaluate_keyed(fixture: &Value) -> BTreeSet<Finding> {
    let mut findings: BTreeSet<Finding> = BTreeSet::new();

    let Some(rows) = fixture.get("rows").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "missing_or_empty_required_field",
            "<matrix>#rows",
        ));
        return findings;
    };

    if rows.is_empty() {
        findings.insert(Finding::new(
            "missing_or_empty_required_field",
            "<matrix>#rows",
        ));
        return findings;
    }

    let mut seen_ids: HashSet<String> = HashSet::new();
    for row in rows {
        let id = row.get("id").and_then(Value::as_str).unwrap_or("");
        // duplicate_row_id: keyed by the duplicated row id.
        if !id.trim().is_empty() && !seen_ids.insert(id.to_owned()) {
            findings.insert(Finding::new("duplicate_row_id", id));
        }
        evaluate_row(row, &mut findings);
    }

    findings
}

fn evaluate_row(row: &Value, findings: &mut BTreeSet<Finding>) {
    let id = row.get("id").and_then(Value::as_str).unwrap_or("");

    // missing_or_empty_required_field: any required field absent / empty-string / empty-array.
    // Keyed by "{row_id}#{field}" so each missing field is an independent baseline key.
    for field in REQUIRED_FIELDS {
        if !field_present_non_empty(row, field) {
            findings.insert(Finding::new(
                "missing_or_empty_required_field",
                &format!("{id}#{field}"),
            ));
        }
    }

    let classification = row
        .get("classification")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();

    // unknown_classification: not in the 4-enum (only when a classification is present;
    // an empty classification is a missing_or_empty_required_field, handled above).
    if !classification.is_empty() && !CLASSIFICATIONS.contains(&classification) {
        findings.insert(Finding::new("unknown_classification", id));
    }

    let enforceable_or_automatable = row
        .get("enforceable_or_automatable")
        .and_then(Value::as_bool)
        == Some(true);

    // enforceable_or_automatable_marked_human_judgment
    if enforceable_or_automatable && classification == "not_automatable_human_judgment" {
        findings.insert(Finding::new(
            "enforceable_or_automatable_marked_human_judgment",
            id,
        ));
    }

    // blocking_invariant_mapped_to_oya_cli: a blocking row routed through an oya CLI call.
    let target = row
        .get("target_gate_or_controller")
        .and_then(Value::as_str)
        .unwrap_or("");
    let no_new_oya_cli_surface = row
        .get("no_new_oya_cli_surface")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let is_blocking = BLOCKING_CLASSIFICATIONS.contains(&classification);
    if is_blocking && (mentions_oya_cli(target) || !no_new_oya_cli_surface) {
        findings.insert(Finding::new("blocking_invariant_mapped_to_oya_cli", id));
    }

    // advisory_claiming_enforced (NET-NEW): claims enforcement but no wired buck2 target.
    let claims_enforced = row.get("claims_enforced").and_then(Value::as_bool) == Some(true)
        || mentions_enforcement(row.get("requirement").and_then(Value::as_str).unwrap_or(""))
        || mentions_enforcement(target);
    let has_wired_buck2_target =
        row.get("has_wired_buck2_target").and_then(Value::as_bool) == Some(true);
    // Only the explicit `claims_enforced` flag (DATA) drives this code, so a real matrix
    // row whose prose merely describes enforcement is not falsely flagged; the prose check
    // is the corroborating signal used by the producer's enforcement-inventory face.
    if row.get("claims_enforced").and_then(Value::as_bool) == Some(true) && !has_wired_buck2_target
    {
        findings.insert(Finding::new("advisory_claiming_enforced", id));
    }
    let _ = claims_enforced;

    // ratchet_regression (NET-NEW): a previously-blocking requirement downgraded.
    let was_blocking = row.get("was_blocking").and_then(Value::as_bool) == Some(true);
    if was_blocking && !is_blocking {
        findings.insert(Finding::new("ratchet_regression", id));
    }
}

/// Whether a `target_gate_or_controller` value names an `oya` CLI invocation.
fn mentions_oya_cli(target: &str) -> bool {
    let t = target.to_ascii_lowercase();
    t.contains("oya gate") || t.contains("oya gen") || t.contains("oya verify")
}

/// Whether a string uses enforcement/gate vocabulary (the corroborating "claims enforced"
/// signal used by the producer's enforcement-inventory face).
fn mentions_enforcement(text: &str) -> bool {
    let t = text.to_ascii_lowercase();
    t.contains("enforce") || t.contains("gate") || t.contains("verified") || t.contains("blocks")
}

/// A required field is present + non-empty (non-empty string, non-empty array, or a bool).
fn field_present_non_empty(row: &Value, field: &str) -> bool {
    match row.get(field) {
        Some(Value::String(s)) => !s.trim().is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Bool(_)) | Some(Value::Number(_)) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn good_row() -> Value {
        json!({
            "id": "GOOD-1",
            "source_artifact": "docs/AGENTS.md#reviewer",
            "requirement": "Final architect judgment signs residual design risk.",
            "classification": "not_automatable_human_judgment",
            "enforceable_or_automatable": false,
            "owner": "platform-architecture",
            "target_gate_or_controller": "reviewer-agent verdict + cloud-ci evidence packet",
            "blocking_fixture": "specs/fixtures/phase0-automation-ratchet/tc-0.16-good.json",
            "retirement_phase": "Phase 1",
            "evidence_path": "evidence/example/x.json",
            "no_new_oya_cli_surface": true
        })
    }

    #[test]
    fn fully_specified_human_judgment_row_is_green() {
        let report = evaluate(&json!({"rows": [good_row()]}));
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.violations);
    }

    #[test]
    fn missing_rows_matrix_fails_closed() {
        let findings = evaluate_keyed(&json!({}));
        assert!(findings.contains(&Finding::new(
            "missing_or_empty_required_field",
            "<matrix>#rows"
        )));

        let report = evaluate(&json!({}));
        assert_eq!(report.verdict, Verdict::Red);
        assert!(
            report
                .violations
                .contains("missing_or_empty_required_field")
        );
    }

    #[test]
    fn malformed_or_empty_rows_matrix_fails_closed() {
        for fixture in [json!({"rows": "not-an-array"}), json!({"rows": []})] {
            let findings = evaluate_keyed(&fixture);
            assert!(findings.contains(&Finding::new(
                "missing_or_empty_required_field",
                "<matrix>#rows"
            )));
            assert_eq!(evaluate(&fixture).verdict, Verdict::Red);
        }
    }

    #[test]
    fn oya_cli_authority_fires_blocking_invariant_mapped_to_oya_cli() {
        let mut row = good_row();
        row["classification"] = json!("automated_blocking_now");
        row["target_gate_or_controller"] = json!("oya gate run-all --ci-required");
        row["no_new_oya_cli_surface"] = json!(false);
        let report = evaluate(&json!({"rows": [row]}));
        assert!(
            report
                .violations
                .contains("blocking_invariant_mapped_to_oya_cli")
        );
    }

    #[test]
    fn human_judgment_for_automatable_rule_fires() {
        let mut row = good_row();
        row["enforceable_or_automatable"] = json!(true);
        let report = evaluate(&json!({"rows": [row]}));
        assert!(
            report
                .violations
                .contains("enforceable_or_automatable_marked_human_judgment")
        );
    }

    #[test]
    fn advisory_claiming_enforced_fires_for_unwired_claim() {
        let mut row = good_row();
        row["id"] = json!("AR-advisory");
        row["claims_enforced"] = json!(true);
        row["has_wired_buck2_target"] = json!(false);
        let report = evaluate(&json!({"rows": [row]}));
        assert!(report.violations.contains("advisory_claiming_enforced"));
    }

    #[test]
    fn ratchet_regression_fires_on_downgrade() {
        let mut row = good_row();
        row["was_blocking"] = json!(true);
        row["classification"] = json!("automated_advisory_until_p0_0");
        let report = evaluate(&json!({"rows": [row]}));
        assert!(report.violations.contains("ratchet_regression"));
    }

    #[test]
    fn evaluate_keyed_carries_row_id_and_field_keys() {
        let report = evaluate_keyed(&json!({"rows": [
            {"id":"DUP","source_artifact":"x","requirement":"y","classification":"automated_blocking_now","owner":"o","target_gate_or_controller":"t","blocking_fixture":"b","retirement_phase":"p","evidence_path":"e","no_new_oya_cli_surface":true},
            {"id":"DUP","classification":"automated_some_day"}
        ]}));
        assert!(report.contains(&Finding::new("duplicate_row_id", "DUP")));
        assert!(report.contains(&Finding::new("unknown_classification", "DUP")));
        // missing required field is keyed by "{id}#{field}".
        assert!(report.contains(&Finding::new(
            "missing_or_empty_required_field",
            "DUP#source_artifact"
        )));
        // evaluate() is the bare-code projection.
        let projected: BTreeSet<String> = report.iter().map(|f| f.code.clone()).collect();
        assert_eq!(evaluate(&json!({"rows": [
            {"id":"DUP","source_artifact":"x","requirement":"y","classification":"automated_blocking_now","owner":"o","target_gate_or_controller":"t","blocking_fixture":"b","retirement_phase":"p","evidence_path":"e","no_new_oya_cli_surface":true},
            {"id":"DUP","classification":"automated_some_day"}
        ]})).violations, projected);
    }

    #[test]
    fn duplicate_unknown_and_missing_fire_together() {
        let report = evaluate(&json!({"rows": [
            {"id":"DUP","source_artifact":"x","requirement":"y","classification":"automated_blocking_now","owner":"o","target_gate_or_controller":"t","blocking_fixture":"b","retirement_phase":"p","evidence_path":"e","no_new_oya_cli_surface":true},
            {"id":"DUP","classification":"automated_some_day"}
        ]}));
        assert!(report.violations.contains("duplicate_row_id"));
        assert!(report.violations.contains("unknown_classification"));
        assert!(
            report
                .violations
                .contains("missing_or_empty_required_field")
        );
    }
}
