//! # cloud-ci-total-accounting (GATE-2)
//!
//! The total-accounting gate that owns + produces `accounting-registry.generated.json`
//! (D-DOCTRINE total-accounting; PHASE-0-FIREWALL-PLAN §5.2). It evaluates accounting
//! rows and emits a `{verdict, violations}` report; its tests assert
//! `report.violations == fixture.expected_violations` over `specs/fixtures/total-accounting/tc-*.json`.
//!
//! ## Blocking violation codes (the contract — literal strings the gate emits)
//! - `unaccounted`     — a tracked path has no registry row
//! - `unowned`         — row has no OWNERS-resolvable owner (born-blocking: 0 OWNERS exist today)
//! - `unjustified`     — `justification_ref` empty OR points at a claim that does not resolve
//!   (the foundry-residue class: justified by ADR-0363's false "eradicated")
//! - `unreachable`     — `reachable_from` is empty / resolves to no live masterplan node
//! - `no_ttl_class`    — row has no `ttl.ttl_class` (feeds Gate-3)
//! - `registry_drift`  — committed registry != regenerated (hand-edit ⇒ RED)
//!
//! The evaluator is pure: fixtures (data-under-test) drive it; there are no scanner
//! special-cases. ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashSet};

use serde_json::Value;

/// The gate id, matching the buck2 target + the §5.2 contract.
pub const GATE_ID: &str = "cloud-ci-total-accounting";

/// The six blocking codes, in canonical order. The fixtures pin exact subsets.
pub const VIOLATION_CODES: [&str; 6] = [
    "unaccounted",
    "unowned",
    "unjustified",
    "unreachable",
    "no_ttl_class",
    "registry_drift",
];

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

/// Evaluate a total-accounting fixture/registry `Value` into a report.
///
/// The fixture shape mirrors the on-disk convention: a top-level object carrying
/// `rows: [..]` accounting records, with optional fixture-level signals
/// (`unaccounted_paths`, `registry_hand_edited`) that model the corpus-level RED
/// conditions a single in-memory fixture cannot otherwise express.
pub fn evaluate(fixture: &Value) -> Report {
    let mut violations: BTreeSet<String> = BTreeSet::new();

    // Corpus-level signals (modeled as DATA in the fixture, not scanner branches).
    if !optional_str_array(fixture, "unaccounted_paths").is_empty() {
        violations.insert("unaccounted".to_owned());
    }
    if fixture
        .get("registry_hand_edited")
        .and_then(Value::as_bool)
        == Some(true)
    {
        violations.insert("registry_drift".to_owned());
    }

    let rows = fixture
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut accounted_paths: HashSet<String> = HashSet::new();
    for row in &rows {
        evaluate_row(row, &mut violations);
        if let Some(path) = row.get("path").and_then(Value::as_str) {
            accounted_paths.insert(path.to_owned());
        }
    }

    // A tracked path declared (via `tracked_paths`) without a corresponding row is unaccounted.
    for tracked in optional_str_array(fixture, "tracked_paths") {
        if !accounted_paths.contains(&tracked) {
            violations.insert("unaccounted".to_owned());
        }
    }

    Report::from_violations(violations)
}

/// Per-row accounting checks. The justification check is the subtle one: an empty
/// `justification_ref` is unjustified, AND a non-empty ref that does not resolve
/// (`justification_resolves:false` — e.g. ADR-0363 claiming the file was "eradicated")
/// is ALSO unjustified. That is the foundry-residue exhibit.
fn evaluate_row(row: &Value, violations: &mut BTreeSet<String>) {
    // owner: null/empty ⇒ unowned
    if !field_non_empty_string(row, "owner") {
        violations.insert("unowned".to_owned());
    }

    // justification: empty ref OR an explicitly non-resolving ref ⇒ unjustified
    let has_justification_ref = field_non_empty_string(row, "justification_ref");
    let justification_resolves = row
        .get("justification_resolves")
        .and_then(Value::as_bool)
        .unwrap_or(has_justification_ref); // default: a present ref resolves unless told otherwise
    let claims_absent = row
        .get("justification_claims_absent")
        .and_then(Value::as_bool)
        == Some(true);
    if !has_justification_ref || !justification_resolves || claims_absent {
        violations.insert("unjustified".to_owned());
    }

    // reachability: empty array ⇒ unreachable
    if reachable_from(row).is_empty() {
        violations.insert("unreachable".to_owned());
    }

    // ttl_class: missing/empty ⇒ no_ttl_class
    let ttl_class_present = row
        .get("ttl")
        .and_then(|ttl| ttl.get("ttl_class"))
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    if !ttl_class_present {
        violations.insert("no_ttl_class".to_owned());
    }
}

fn reachable_from(row: &Value) -> Vec<String> {
    row.get("reachable_from")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn field_non_empty_string(row: &Value, field: &str) -> bool {
    row.get(field)
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
}

fn optional_str_array(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fully_accounted_row_is_green() {
        let fixture = json!({
            "rows": [{
                "path": "specs/masterplan.json",
                "owner": "council-architecture",
                "justification_ref": "ADR-0364",
                "justification_resolves": true,
                "reachable_from": ["root-hub", "masterplan"],
                "ttl": {"ttl_class": "spec"}
            }]
        });
        let report = evaluate(&fixture);
        assert_eq!(report.verdict, Verdict::Green);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn foundry_residue_orphan_is_unjustified() {
        let fixture = json!({
            "rows": [{
                "path": "oya/intelligence/catalog/oya-foundry-eval/src/lib.rs",
                "owner": "platform-intelligence",
                "justification_ref": "ADR-0363",
                "justification_resolves": false,
                "justification_claims_absent": true,
                "reachable_from": ["cargo-members"],
                "ttl": {"ttl_class": "code"}
            }]
        });
        let report = evaluate(&fixture);
        assert!(report.violations.contains("unjustified"));
    }

    #[test]
    fn each_code_fires_in_isolation() {
        // unowned
        assert!(evaluate(&json!({"rows":[{"path":"a","justification_ref":"ADR-1","reachable_from":["masterplan"],"ttl":{"ttl_class":"code"}}]}))
            .violations.contains("unowned"));
        // unreachable
        assert!(evaluate(&json!({"rows":[{"path":"a","owner":"o","justification_ref":"ADR-1","reachable_from":[],"ttl":{"ttl_class":"code"}}]}))
            .violations.contains("unreachable"));
        // no_ttl_class
        assert!(evaluate(&json!({"rows":[{"path":"a","owner":"o","justification_ref":"ADR-1","reachable_from":["masterplan"]}]}))
            .violations.contains("no_ttl_class"));
        // unaccounted
        assert!(evaluate(&json!({"unaccounted_paths":["new/file.rs"],"rows":[]}))
            .violations.contains("unaccounted"));
        // registry_drift
        assert!(evaluate(&json!({"registry_hand_edited":true,"rows":[]}))
            .violations.contains("registry_drift"));
    }
}
