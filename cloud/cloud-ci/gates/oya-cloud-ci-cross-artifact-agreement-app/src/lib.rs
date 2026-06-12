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

/// A keyed violation: the bare `code` (the existing contract) PLUS the stable `key`
/// that identifies the offending unit. The going-live ratchet baselines per
/// `(code, key)`; `evaluate()` is the bare-code projection of `evaluate_keyed()`.
/// Keys for this gate are: the decision `id` (orphan/unpropagated/dual),
/// `{decision_id}@{face}` (status_disagreement), `{source_id}->{target_id}`
/// (supersession_half_edge), and `{shared-value}@{sorted face names}`
/// (generated_face_drift).
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
///   "id_mismatches": [                 // filename id != front-matter id (the collision mask)
///     "ADR-0552-x.md:ADR-0552!=ADR-0553"
///   ],
///   "next_free_id": "ADR-0554",        // allocator output (producer --next-adr)
///   "generated_face_axes": {           // shared values two generated faces must agree on
///     "catalog.json": 6,
///     "contracts.json": 7
///   }
/// }
/// ```
/// Bare-code projection of [`evaluate_keyed`]: identical detection logic, keys dropped.
/// Every `tc-*.json` fixture + the born-blocking self-test keep asserting bare codes
/// against it byte-for-byte.
pub fn evaluate(fixture: &Value) -> Report {
    let violations = evaluate_keyed(fixture)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    Report::from_violations(violations)
}

/// Evaluate a cross-artifact-agreement corpus into the keyed finding set — the single
/// source of truth for the gate's detection logic.
pub fn evaluate_keyed(fixture: &Value) -> BTreeSet<Finding> {
    let mut findings: BTreeSet<Finding> = BTreeSet::new();

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
        evaluate_decision(decision, &mut findings);
    }

    // supersession_half_edge: every edge must be reciprocal. Keyed by the directed pair.
    for (id, targets) in &supersedes {
        for target in targets {
            // Only assert reciprocity when the counterpart decision is in-corpus; an
            // edge to an out-of-corpus id is not evidence of a half-edge here.
            if known_ids.contains(target)
                && !superseded_by.get(target).is_some_and(|set| set.contains(id))
            {
                findings.insert(Finding::new(
                    "supersession_half_edge",
                    &format!("{id}->{target}"),
                ));
            }
        }
    }
    for (id, sources) in &superseded_by {
        for source in sources {
            if known_ids.contains(source)
                && !supersedes.get(source).is_some_and(|set| set.contains(id))
            {
                findings.insert(Finding::new(
                    "supersession_half_edge",
                    &format!("{source}->{id}"),
                ));
            }
        }
    }

    // dual_decision_collision: an id carried by more than one decision file. Keyed by id.
    for id in str_array(fixture, "duplicate_ids") {
        findings.insert(Finding::new("dual_decision_collision", &id));
    }

    // decision_id_mismatch: a decision file whose front-matter id disagrees with its
    // filename number. The producer keys its dup map by the front-matter id, so a
    // mismatch silently re-keys the file and can MASK a dual_decision_collision
    // (FRIC-1781320000); it is therefore a violation in its own right. Keyed by the
    // producer's `<file>:<filename-id>!=<front-matter-id>` entry.
    for entry in str_array(fixture, "id_mismatches") {
        findings.insert(Finding::new("decision_id_mismatch", &entry));
    }

    // generated_face_drift: two generated faces disagree on a shared value. Keyed by
    // "<shared-value-name>@{sorted face names}".
    if let Some(axes) = fixture.get("generated_face_axes").and_then(Value::as_object) {
        let distinct: BTreeSet<String> = axes
            .values()
            .map(|value| value.to_string())
            .collect();
        if distinct.len() > 1 {
            let faces: Vec<&str> = axes.keys().map(String::as_str).collect();
            // BTreeMap keys are already sorted; join the disagreeing face names.
            let key = format!("axes_count@{{{}}}", faces.join(","));
            findings.insert(Finding::new("generated_face_drift", &key));
        }
    }

    findings
}

/// Per-decision propagation + status-agreement checks. Keyed by the decision `id`
/// (plus `@face` for status_disagreement).
fn evaluate_decision(decision: &Value, findings: &mut BTreeSet<Finding>) {
    let id = decision.get("id").and_then(Value::as_str).unwrap_or("");
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
            findings.insert(Finding::new("orphan_decision", id));
        } else if !(in_spec && in_masterplan && in_roadmap) {
            // Reaches some faces but not all required ones.
            findings.insert(Finding::new("unpropagated_decision", id));
        }
    }

    // status_disagreement: any face records a status that differs from the ADR's status.
    if let Some(face_statuses) = decision.get("face_statuses").and_then(Value::as_object) {
        for (face, face_status) in face_statuses {
            if let Some(other) = face_status.as_str()
                && !other.trim().eq_ignore_ascii_case(status)
            {
                findings.insert(Finding::new(
                    "status_disagreement",
                    &format!("{id}@{face}"),
                ));
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

    /// RED fixture (FRIC-1781320000): a filename/front-matter id disagreement — the
    /// re-keying vector that can mask a duplicate-numbered ADR pair — must go RED.
    #[test]
    fn id_mismatch_fires_decision_id_mismatch() {
        let fixture = json!({
            "decisions": [],
            "id_mismatches": ["ADR-0552-x.md:ADR-0552!=ADR-0553"]
        });
        let report = evaluate(&fixture);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(report.violations.contains("decision_id_mismatch"));
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
    fn evaluate_keyed_carries_stable_keys() {
        // half-edge keyed by directed pair, dual keyed by id, drift keyed by faces.
        let fixture = json!({
            "decisions": [{
                "id": "ADR-0511",
                "status": "Superseded",
                "in_spec": true, "in_masterplan": false, "in_roadmap": false,
                "supersedes": ["ADR-0359"], "superseded_by": []
            }, {
                "id": "ADR-0359",
                "status": "Superseded",
                "in_spec": true, "in_masterplan": false, "in_roadmap": false,
                "supersedes": [], "superseded_by": []
            }],
            "duplicate_ids": ["ADR-0377"],
            "id_mismatches": ["ADR-0552-x.md:ADR-0552!=ADR-0553"],
            "generated_face_axes": {"catalog.json": 6, "contracts.json": 7}
        });
        let findings = evaluate_keyed(&fixture);
        assert!(findings.contains(&Finding::new("supersession_half_edge", "ADR-0511->ADR-0359")));
        assert!(findings.contains(&Finding::new("dual_decision_collision", "ADR-0377")));
        assert!(findings.contains(&Finding::new(
            "decision_id_mismatch",
            "ADR-0552-x.md:ADR-0552!=ADR-0553"
        )));
        assert!(findings.contains(&Finding::new(
            "generated_face_drift",
            "axes_count@{catalog.json,contracts.json}"
        )));
        // evaluate() is the bare-code projection.
        let projected: BTreeSet<String> = findings.iter().map(|f| f.code.clone()).collect();
        assert_eq!(evaluate(&fixture).violations, projected);
    }

    #[test]
    fn status_disagreement_keyed_by_decision_and_face() {
        let findings = evaluate_keyed(&json!({"decisions":[{
            "id":"ADR-0500","status":"Accepted",
            "in_spec":true,"in_masterplan":true,"in_roadmap":true,
            "face_statuses":{"roadmap":"Superseded"}
        }]}));
        assert!(findings.contains(&Finding::new("status_disagreement", "ADR-0500@roadmap")));
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
