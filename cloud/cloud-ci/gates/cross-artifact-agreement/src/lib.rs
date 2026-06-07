//! # cloud-ci-cross-artifact-agreement (GATE-1)
//!
//! The cross-artifact-agreement gate that asserts every decision agrees across the
//! artifacts that must propagate it — the ADR front-matter, the spec corpus, the
//! masterplan graph, the roadmap/sequencing graph, the reciprocal supersession edges,
//! and the GENERATED faces (PHASE-0-FIREWALL-PLAN §5.2; amends ADR-0365). It evaluates a
//! cross-artifact corpus `Value` and emits a `{verdict, violations}` report; its tests
//! assert `report.violations == fixture.expected_violations` over
//! `specs/fixtures/cross-artifact-agreement/tc-*.json`.
//!
//! ## Blocking violation codes (the contract — literal strings the gate emits)
//! - `orphan_decision`        — an Accepted decision reaches NO propagation face
//!   (absent from spec AND masterplan AND roadmap): a decision nothing points at.
//! - `unpropagated_decision`  — an Accepted decision reaches SOME but not ALL of its
//!   required propagation faces (e.g. has an ADR + spec but no masterplan/roadmap node).
//! - `status_disagreement`    — the decision's status disagrees across the faces that
//!   record it (e.g. ADR `Accepted` while the roadmap node marks it `superseded`).
//! - `generated_face_drift`   — two GENERATED faces that must agree on a shared value
//!   disagree (the frozen live exhibit: `catalog.json axes_count:6` vs
//!   `contracts.json axes_count:7`).
//! - `dual_decision_collision`— two distinct decision FILES share one decision id (the
//!   two ADR-0377 files).
//! - `supersession_half_edge` — a supersession edge that is not reciprocal (A `supersedes`
//!   B while B's `superseded_by` omits A, or the reverse): a half-built edge.
//!
//! The evaluator is pure: the fixture (data-under-test) drives it; there are no scanner
//! special-cases. Carve-outs/exceptions live as DATA, never as evaluator branches.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// The gate id, matching the buck2 target + the §5.2 contract.
pub const GATE_ID: &str = "cloud-ci-cross-artifact-agreement";

/// The six blocking codes, in canonical order. The fixtures pin exact subsets.
pub const VIOLATION_CODES: [&str; 6] = [
    "orphan_decision",
    "unpropagated_decision",
    "status_disagreement",
    "generated_face_drift",
    "dual_decision_collision",
    "supersession_half_edge",
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

/// Evaluate a cross-artifact-agreement fixture/corpus `Value` into a report.
///
/// The fixture shape mirrors the on-disk crosswalk face:
/// ```jsonc
/// {
///   "decisions": [
///     {
///       "id": "ADR-0515",
///       "status": "Accepted",          // the ADR front-matter status
///       "in_spec": true,               // appears in the spec corpus
///       "in_masterplan": true,         // appears as a masterplan node
///       "in_roadmap": true,            // appears as a roadmap/sequencing node
///       "supersedes": ["ADR-0511"],
///       "superseded_by": [],
///       "face_statuses": {             // status as each face records it (optional)
///         "roadmap": "Accepted"
///       }
///     }
///   ],
///   "duplicate_ids": ["ADR-0377"],     // ids carried by >1 decision file (DATA signal)
///   "generated_face_axes": {           // shared values two generated faces must agree on
///     "catalog.json": 6,
///     "contracts.json": 7
///   }
/// }
/// ```
pub fn evaluate(fixture: &Value) -> Report {
    let mut violations: BTreeSet<String> = BTreeSet::new();

    let decisions = fixture
        .get("decisions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    // Index every decision's supersession edges so half-edges can be detected
    // symmetrically (A.supersedes vs B.superseded_by, and the reverse).
    let mut supersedes: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut superseded_by: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut known_ids: BTreeSet<String> = BTreeSet::new();
    for decision in &decisions {
        let Some(id) = decision.get("id").and_then(Value::as_str) else {
            continue;
        };
        known_ids.insert(id.to_owned());
        supersedes.insert(id.to_owned(), str_set(decision, "supersedes"));
        superseded_by.insert(id.to_owned(), str_set(decision, "superseded_by"));
    }

    for decision in &decisions {
        evaluate_decision(decision, &mut violations);
    }

    // supersession_half_edge: every edge must be reciprocal.
    for (id, targets) in &supersedes {
        for target in targets {
            // Only assert reciprocity when the counterpart decision is in-corpus; an
            // edge to an out-of-corpus id is not evidence of a half-edge here.
            if known_ids.contains(target)
                && !superseded_by.get(target).is_some_and(|set| set.contains(id))
            {
                violations.insert("supersession_half_edge".to_owned());
            }
        }
    }
    for (id, sources) in &superseded_by {
        for source in sources {
            if known_ids.contains(source)
                && !supersedes.get(source).is_some_and(|set| set.contains(id))
            {
                violations.insert("supersession_half_edge".to_owned());
            }
        }
    }

    // dual_decision_collision: an id carried by more than one decision file.
    if !str_array(fixture, "duplicate_ids").is_empty() {
        violations.insert("dual_decision_collision".to_owned());
    }

    // generated_face_drift: two generated faces disagree on a shared value.
    if let Some(axes) = fixture.get("generated_face_axes").and_then(Value::as_object) {
        let distinct: BTreeSet<String> = axes
            .values()
            .map(|value| value.to_string())
            .collect();
        if distinct.len() > 1 {
            violations.insert("generated_face_drift".to_owned());
        }
    }

    Report::from_violations(violations)
}

/// Per-decision propagation + status-agreement checks.
fn evaluate_decision(decision: &Value, violations: &mut BTreeSet<String>) {
    let status = decision
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();

    // Propagation is only required for live (Accepted) decisions. A Superseded/Proposed
    // decision is not expected to carry masterplan/roadmap nodes.
    let is_accepted = status.eq_ignore_ascii_case("accepted");

    let in_spec = bool_field(decision, "in_spec");
    let in_masterplan = bool_field(decision, "in_masterplan");
    let in_roadmap = bool_field(decision, "in_roadmap");

    if is_accepted {
        let reaches_any = in_spec || in_masterplan || in_roadmap;
        if !reaches_any {
            // Reaches no propagation face at all: a decision nothing points at.
            violations.insert("orphan_decision".to_owned());
        } else if !(in_spec && in_masterplan && in_roadmap) {
            // Reaches some faces but not all required ones.
            violations.insert("unpropagated_decision".to_owned());
        }
    }

    // status_disagreement: any face records a status that differs from the ADR's status.
    if let Some(face_statuses) = decision.get("face_statuses").and_then(Value::as_object) {
        for face_status in face_statuses.values() {
            if let Some(other) = face_status.as_str()
                && !other.trim().eq_ignore_ascii_case(status)
            {
                violations.insert("status_disagreement".to_owned());
            }
        }
    }
}

fn bool_field(value: &Value, field: &str) -> bool {
    value.get(field).and_then(Value::as_bool).unwrap_or(false)
}

fn str_set(value: &Value, field: &str) -> BTreeSet<String> {
    str_array(value, field).into_iter().collect()
}

fn str_array(value: &Value, field: &str) -> Vec<String> {
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
    fn all_four_agree_is_green() {
        let fixture = json!({
            "decisions": [{
                "id": "ADR-0515",
                "status": "Accepted",
                "in_spec": true,
                "in_masterplan": true,
                "in_roadmap": true,
                "supersedes": ["ADR-0511"],
                "superseded_by": [],
                "face_statuses": {"roadmap": "Accepted"}
            }, {
                "id": "ADR-0511",
                "status": "Superseded",
                "in_spec": true,
                "in_masterplan": false,
                "in_roadmap": false,
                "supersedes": [],
                "superseded_by": ["ADR-0515"]
            }]
        });
        let report = evaluate(&fixture);
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.violations);
    }

    #[test]
    fn axes_drift_fires_generated_face_drift() {
        let fixture = json!({
            "decisions": [],
            "generated_face_axes": {"catalog.json": 6, "contracts.json": 7}
        });
        assert!(evaluate(&fixture)
            .violations
            .contains("generated_face_drift"));
    }

    #[test]
    fn duplicate_id_fires_dual_decision_collision() {
        let fixture = json!({"decisions": [], "duplicate_ids": ["ADR-0377"]});
        assert!(evaluate(&fixture)
            .violations
            .contains("dual_decision_collision"));
    }

    #[test]
    fn half_supersession_fires_half_edge() {
        // ADR-0511 supersedes ADR-0359, but ADR-0359.superseded_by omits ADR-0511.
        let fixture = json!({
            "decisions": [{
                "id": "ADR-0511",
                "status": "Superseded",
                "in_spec": true, "in_masterplan": false, "in_roadmap": false,
                "supersedes": ["ADR-0359"], "superseded_by": ["ADR-0515"]
            }, {
                "id": "ADR-0359",
                "status": "Superseded",
                "in_spec": true, "in_masterplan": false, "in_roadmap": false,
                "supersedes": [], "superseded_by": ["ADR-0515"]
            }]
        });
        assert!(evaluate(&fixture)
            .violations
            .contains("supersession_half_edge"));
    }

    #[test]
    fn each_propagation_code_fires_in_isolation() {
        // orphan_decision: accepted but reaches nothing.
        assert!(evaluate(&json!({"decisions":[{"id":"ADR-1","status":"Accepted","in_spec":false,"in_masterplan":false,"in_roadmap":false}]}))
            .violations.contains("orphan_decision"));
        // unpropagated_decision: accepted, in spec, missing masterplan/roadmap.
        assert!(evaluate(&json!({"decisions":[{"id":"ADR-1","status":"Accepted","in_spec":true,"in_masterplan":false,"in_roadmap":false}]}))
            .violations.contains("unpropagated_decision"));
        // status_disagreement: ADR Accepted, roadmap face says Superseded.
        assert!(evaluate(&json!({"decisions":[{"id":"ADR-1","status":"Accepted","in_spec":true,"in_masterplan":true,"in_roadmap":true,"face_statuses":{"roadmap":"Superseded"}}]}))
            .violations.contains("status_disagreement"));
    }
}
