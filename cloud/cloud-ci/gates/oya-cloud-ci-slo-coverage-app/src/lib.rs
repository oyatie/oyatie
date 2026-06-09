//! # cloud-ci-slo-coverage
//!
//! Portable conformance gate for catalog SLO declarations. The producer owns all repository I/O:
//! it enumerates `registry/catalog/*.yaml`, maps each file stem to `crate_id`, and parses the
//! top-level `slo:` scalar into rows shaped as `{"crate_id", "slo"}`. This crate stays pure and
//! reuses `oya_check_slo_coverage::validate_slo_coverage` per row so the legacy predicate and the
//! cloud-ci gate cannot drift.
//!
//! `evaluate_keyed` returns one `Finding{code,key}` per invalid catalog row. Current oyatie corpus
//! is clean, so the disposition table marks both codes `frozen_empty`: a future missing SLO cannot
//! be laundered into the accepted baseline by regeneration.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use oya_check_slo_coverage::{SloCatalogRecord, SloCoverageError, validate_slo_coverage};
use serde_json::Value;

/// The gate id, matching oya-ci config and the baseline ratchet.
pub const GATE_ID: &str = "cloud-ci-slo-coverage";

/// Stable blocking violation codes emitted by this gate.
pub const VIOLATION_CODES: [&str; 3] = [
    "slo_missing_or_blank_slo",
    "slo_empty_crate_id",
    "slo_no_catalog_records",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// A keyed violation: the stable `code` plus the offending catalog `key`.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
}

impl Report {
    fn from_codes(violations: BTreeSet<String>) -> Self {
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

fn record_from_row(row: &Value) -> SloCatalogRecord {
    SloCatalogRecord {
        crate_id: row
            .get("crate_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        slo: row.get("slo").and_then(Value::as_str).map(str::to_owned),
    }
}

fn finding_for(error: SloCoverageError) -> Finding {
    match error {
        SloCoverageError::EmptyCrateId => Finding::new("slo_empty_crate_id", "<empty-crate-id>"),
        SloCoverageError::MissingSlo { crate_id } => {
            Finding::new("slo_missing_or_blank_slo", &crate_id)
        }
    }
}

/// Pure evaluator: takes `{"rows":[{"crate_id":"...","slo":"..."}, ...]}` and emits one
/// finding per invalid SLO catalog row. Running the legacy validator per row converts its
/// fail-fast whole-corpus contract into surface-all cloud-ci findings without re-deriving policy.
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let rows = input
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut findings = BTreeSet::new();
    if rows.is_empty() {
        findings.insert(Finding::new(
            "slo_no_catalog_records",
            "<empty-slo-catalog>",
        ));
        return findings;
    }
    for row in &rows {
        let record = record_from_row(row);
        if let Err(error) = validate_slo_coverage(&[record]) {
            findings.insert(finding_for(error));
        }
    }
    findings
}

/// Bare-code projection of [`evaluate_keyed`] — the single source of truth for the verdict.
pub fn evaluate(input: &Value) -> Report {
    let codes: BTreeSet<String> = evaluate_keyed(input).into_iter().map(|f| f.code).collect();
    Report::from_codes(codes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn rows(records: &[(&str, Option<&str>)]) -> Value {
        json!({
            "rows": records
                .iter()
                .map(|(crate_id, slo)| json!({ "crate_id": crate_id, "slo": slo }))
                .collect::<Vec<_>>()
        })
    }

    #[test]
    fn conforming_catalog_rows_are_green() {
        let input = rows(&[
            (
                "oya-intelligence-capability-kernel",
                Some("preview-control-plane"),
            ),
            ("oya-workspace-docs-kernel", Some("preview-data-plane")),
        ]);
        assert_eq!(evaluate(&input).verdict, Verdict::Green);
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn missing_slo_maps_to_stable_finding() {
        let input = rows(&[("oya-intelligence-capability-kernel", None)]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "slo_missing_or_blank_slo");
        assert_eq!(finding.key, "oya-intelligence-capability-kernel");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn blank_slo_maps_to_same_policy_code() {
        let input = rows(&[("oya-intelligence-capability-kernel", Some("  "))]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "slo_missing_or_blank_slo");
        assert_eq!(finding.key, "oya-intelligence-capability-kernel");
    }

    #[test]
    fn empty_crate_id_has_dedicated_code() {
        let input = rows(&[("", Some("preview-control-plane"))]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "slo_empty_crate_id");
        assert_eq!(finding.key, "<empty-crate-id>");
    }

    #[test]
    fn surface_all_one_finding_per_bad_catalog_row() {
        let input = rows(&[
            ("oya-good-kernel", Some("preview-control-plane")),
            ("oya-missing-domain", None),
            ("oya-blank-app", Some("")),
        ]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 2, "got {findings:?}");
        assert!(
            findings
                .iter()
                .all(|finding| finding.code == "slo_missing_or_blank_slo")
        );
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let input = rows(&[("oya-missing-domain", None), ("", Some("preview"))]);
        let projected: BTreeSet<String> = evaluate_keyed(&input)
            .into_iter()
            .map(|finding| finding.code)
            .collect();
        assert_eq!(evaluate(&input).violations, projected);
    }

    #[test]
    fn empty_corpus_is_red_to_prevent_false_green() {
        let findings = evaluate_keyed(&json!({ "rows": [] }));
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "slo_no_catalog_records");
        assert_eq!(finding.key, "<empty-slo-catalog>");
        assert_eq!(evaluate(&json!({ "rows": [] })).verdict, Verdict::Red);
    }
}
