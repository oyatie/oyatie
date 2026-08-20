//! # cloud-ci-total-accounting (GATE-2)
//!
//! The total-accounting gate that owns + produces `accounting-registry.generated.json`
//! (D-DOCTRINE total-accounting; PHASE-0-FIREWALL-PLAN §5.2). It evaluates accounting
//! rows and emits a `{verdict, violations}` report; its tests assert
//! `report.violations == fixture.expected_violations` over `specs/fixtures/total-accounting/tc-*.json`.
//!
//! ## Blocking violation codes (the contract — literal strings the gate emits)
//! - `unaccounted`     — a tracked path has no registry row, or producer data is
//!   too malformed to identify tracked/path-level accounting safely (stable sentinel key)
//! - `unowned`         — row has no OWNERS-resolvable owner (born-blocking: 0 OWNERS exist today)
//! - `unjustified`     — `justification_ref` empty OR points at a claim that does not resolve
//!   (the foundry-residue class: justified by ADR-0363's false "eradicated")
//! - `unreachable`     — `reachable_from` is empty / resolves to no live masterplan node
//! - `no_ttl_class`    — row has no `ttl.ttl_class` (feeds Gate-3)
//! - `ci_inventory_registry_drift`  — committed registry != regenerated (hand-edit ⇒ RED)
//! - `scratch_artifact`— row is unit_class `scratch` (build/test scratch by SHAPE, not
//!   registration; frozen_empty zero-tolerance — cannot be laundered by owner/justification)
//!
//! The evaluator is pure: fixtures (data-under-test) drive it; there are no scanner
//! special-cases. ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeSet, HashSet};

use serde_json::Value;

/// The gate id, matching the buck2 target + the §5.2 contract.
pub const GATE_ID: &str = "cloud-ci-total-accounting";

/// The blocking codes, in canonical order. The fixtures pin exact subsets.
pub const VIOLATION_CODES: [&str; 9] = [
    "unaccounted",
    "unowned",
    "unjustified",
    "unreachable",
    "no_ttl_class",
    "ci_inventory_registry_drift",
    "scratch_artifact",
    "move_derived_not_planned",
    "move_planned_not_derived",
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
/// that identifies the offending unit (a path / decision id / surface id). The
/// going-live ratchet baselines per `(code, key)`; `evaluate()` is the bare-code
/// projection of `evaluate_keyed()` so there is one source of truth and zero
/// duplicated logic. The key for total-accounting is the registry row `path`
/// (producer is git-ls-files-sorted, deterministic).
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

/// The synthetic key for codes that are corpus-level predicates rather than
/// per-unit findings (`ci_inventory_registry_drift`): they carry a single stable sentinel key
/// so the ratchet treats them uniformly (their baseline is permanently empty).
const GATE_KEY: &str = "<gate>";
/// Stable fail-closed key when the registry corpus has no valid `rows` array.
/// It intentionally uses `unaccounted`: no trustworthy rows means the gate cannot
/// prove tracked paths are accounted for.
const ROWS_KEY: &str = "<cloud-ci-total-accounting#rows>";
/// Stable fail-closed key when an individual row cannot be keyed by a valid path.
/// It intentionally uses `unaccounted`: a pathless row cannot account for any
/// deterministic tracked path in the keyed ratchet.
const ROW_KEY: &str = "<cloud-ci-total-accounting#row>";

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
///
/// This is the bare-code projection of [`evaluate_keyed`]: identical detection logic,
/// keys dropped. Every `tc-*.json` fixture + the born-blocking self-tests keep
/// asserting bare codes against it byte-for-byte.
pub fn evaluate(fixture: &Value) -> Report {
    let violations = evaluate_keyed(fixture)
        .into_iter()
        .map(|finding| finding.code)
        .collect();
    Report::from_violations(violations)
}

/// Evaluate a total-accounting fixture/registry `Value` into the keyed finding set.
///
/// The single source of truth for the gate's detection logic. Each finding pairs a
/// bare violation `code` with the stable `key` (the row `path`) the going-live
/// baseline ratchet freezes per code.
pub fn evaluate_keyed(fixture: &Value) -> BTreeSet<Finding> {
    let mut findings: BTreeSet<Finding> = BTreeSet::new();

    // Corpus-level signals (modeled as DATA in the fixture, not scanner branches).
    let unaccounted_paths = optional_str_array(fixture, "unaccounted_paths");
    for path in &unaccounted_paths {
        findings.insert(Finding::new("unaccounted", path));
    }
    let registry_hand_edited =
        fixture.get("registry_hand_edited").and_then(Value::as_bool) == Some(true);
    if registry_hand_edited {
        // ci_inventory_registry_drift is a binary committed!=regenerated predicate (frozen_empty):
        // not a per-path debt class, so it carries the single gate-level sentinel key.
        findings.insert(Finding::new("ci_inventory_registry_drift", GATE_KEY));
    }
    let tracked_paths = optional_str_array(fixture, "tracked_paths");

    let rows = fixture.get("rows").and_then(Value::as_array);
    let has_other_corpus_signal =
        registry_hand_edited || !unaccounted_paths.is_empty() || !tracked_paths.is_empty();
    if !matches!(rows, Some(rows) if !rows.is_empty()) && !has_other_corpus_signal {
        findings.insert(Finding::new("unaccounted", ROWS_KEY));
    }

    let mut accounted_paths: HashSet<String> = HashSet::new();
    if let Some(rows) = rows {
        for row in rows {
            evaluate_row(row, &mut findings);
            if let Some(path) = row_path(row) {
                accounted_paths.insert(path.to_owned());
            }
        }
    }

    // MOVE-PLAN CONFORMANCE (both directions).
    //
    // Two sources of truth exist for "this path relocates": the registry's derived
    // `disposition`/`destination`, and the committed `specs/reorg/<capability>-move-plan.json`
    // set. A capability cannot be called cut over while they disagree, and until now nothing
    // compared them — on the live tree 447 paths derive `move` with no plan covering them, and
    // the repo could not say whether the derivation over-reaches or the plan is incomplete.
    //
    // `planned_move_paths` arrives as DATA (the producer discovers it through the codemod's own
    // `select_move_plan`, which is what makes multi-plan fail closed). This evaluator never
    // touches the filesystem.
    //
    // Coverage is PREFIX-based because a plan names the moving crate/artifact root while the
    // registry rows are individual files beneath it. Exact-match would report every file under a
    // planned root as unplanned — thousands of false findings that would bury the real gap.
    let planned_move_paths = optional_str_array(fixture, "planned_move_paths");
    let path_is_planned = |path: &str| {
        planned_move_paths
            .iter()
            .any(|planned| path == planned || path.starts_with(&format!("{planned}/")))
    };
    if let Some(rows) = rows {
        for row in rows {
            let Some(path) = row_path(row) else { continue };
            let disposition = row
                .get("disposition")
                .and_then(Value::as_str)
                .unwrap_or_default();
            match disposition {
                // Derived to move, but no committed plan carries it.
                "move" if !path_is_planned(path) => {
                    findings.insert(Finding::new("move_derived_not_planned", path));
                }
                // A plan moves a path the registry says is already home. One of the two is wrong.
                "retain" if path_is_planned(path) => {
                    findings.insert(Finding::new("move_planned_not_derived", path));
                }
                _ => {}
            }
        }
    }

    // A tracked path declared (via `tracked_paths`) without a corresponding row is unaccounted.
    for tracked in tracked_paths {
        if !accounted_paths.contains(&tracked) {
            findings.insert(Finding::new("unaccounted", &tracked));
        }
    }

    findings
}

/// Per-row accounting checks. The justification check is the subtle one: an empty
/// `justification_ref` is unjustified, AND a non-empty ref that does not resolve
/// (`justification_resolves:false` — e.g. ADR-0363 claiming the file was "eradicated")
/// is ALSO unjustified. That is the foundry-residue exhibit. Every per-row code is
/// keyed by the row `path` when present, or a stable row-level sentinel when absent.
fn evaluate_row(row: &Value, findings: &mut BTreeSet<Finding>) {
    let key = row_path(row).unwrap_or(ROW_KEY);
    if key == ROW_KEY {
        findings.insert(Finding::new("unaccounted", key));
    }

    // scratch_artifact: a SHAPE-based class (build/test scratch identified by name/location
    // via the producer's unit-class-policy DATA table, NOT by registration). This fires on the
    // class alone, BEFORE any owner/justification/reachability check, so registering those
    // fields CANNOT launder a scratch-shaped artifact past the gate — the registration-bypass
    // hole the generic husk-block-on-new leaves open (ADR-0555; founder "impossible to commit
    // scratch"). Disposition pins it frozen_empty ⇒ zero-tolerance, never grandfathered.
    if row.get("unit_class").and_then(Value::as_str) == Some("scratch") {
        findings.insert(Finding::new("scratch_artifact", key));
    }

    // owner: null/empty ⇒ unowned
    if !field_non_empty_string(row, "owner") {
        findings.insert(Finding::new("unowned", key));
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
        findings.insert(Finding::new("unjustified", key));
    }

    // reachability: empty array ⇒ unreachable
    if reachable_from(row).is_empty() {
        findings.insert(Finding::new("unreachable", key));
    }

    // ttl_class: missing/empty ⇒ no_ttl_class
    let ttl_class_present = row
        .get("ttl")
        .and_then(|ttl| ttl.get("ttl_class"))
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    if !ttl_class_present {
        findings.insert(Finding::new("no_ttl_class", key));
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

fn row_path(row: &Value) -> Option<&str> {
    row.get("path")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A fully-accounted row (green on every pre-existing code) carrying a disposition, so the
    /// conformance tests below assert ONLY the move-plan codes and cannot pass by accident on
    /// some unrelated finding.
    fn row_ok(path: &str, disposition: &str) -> Value {
        json!({
            "path": path,
            "owner": "council-architecture",
            "justification_ref": "ADR-0364",
            "justification_resolves": true,
            "reachable_from": ["root-hub"],
            "ttl": {"ttl_class": "spec"},
            "disposition": disposition
        })
    }

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
                "path": "oya/intelligence/catalog/oya-intelligence-eval/src/lib.rs",
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
    fn evaluate_keyed_carries_the_row_path_as_key() {
        let fixture = json!({
            "rows": [{
                "path": "oya/intelligence/catalog/oya-intelligence-eval/src/lib.rs",
                "owner": "platform-intelligence",
                "justification_ref": "ADR-0363",
                "justification_resolves": false,
                "reachable_from": ["cargo-members"],
                "ttl": {"ttl_class": "code"}
            }]
        });
        let findings = evaluate_keyed(&fixture);
        assert!(findings.contains(&Finding::new(
            "unjustified",
            "oya/intelligence/catalog/oya-intelligence-eval/src/lib.rs"
        )));
        // evaluate() is exactly the bare-code projection of evaluate_keyed().
        let projected: BTreeSet<String> = findings.iter().map(|f| f.code.clone()).collect();
        assert_eq!(evaluate(&fixture).violations, projected);
    }

    #[test]
    fn registry_drift_keyed_to_gate_sentinel() {
        let findings = evaluate_keyed(&json!({"registry_hand_edited": true, "rows": []}));
        assert!(findings.contains(&Finding::new("ci_inventory_registry_drift", GATE_KEY)));
    }

    #[test]
    fn malformed_or_empty_rows_fail_closed() {
        for fixture in [
            json!({}),
            json!({"rows": "not-an-array"}),
            json!({"rows": []}),
        ] {
            let findings = evaluate_keyed(&fixture);
            assert!(
                findings.contains(&Finding::new("unaccounted", ROWS_KEY)),
                "fixture must fail closed: {fixture:?}"
            );
            assert_eq!(evaluate(&fixture).verdict, Verdict::Red);
        }
    }

    #[test]
    fn row_without_path_fails_closed_with_stable_key() {
        let findings = evaluate_keyed(&json!({
            "rows": [{
                "owner": "platform-ci",
                "justification_ref": "ADR-0555",
                "reachable_from": ["masterplan"],
                "ttl": {"ttl_class": "code"}
            }]
        }));
        assert!(findings.contains(&Finding::new("unaccounted", ROW_KEY)));
        assert!(!findings.contains(&Finding::new("unaccounted", "")));
        assert_eq!(evaluate(&json!({"rows": [{}]})).verdict, Verdict::Red);
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
        assert!(
            evaluate(&json!({"unaccounted_paths":["new/file.rs"],"rows":[]}))
                .violations
                .contains("unaccounted")
        );
        // ci_inventory_registry_drift
        assert!(
            evaluate(&json!({"registry_hand_edited":true,"rows":[]}))
                .violations
                .contains("ci_inventory_registry_drift")
        );
        // scratch_artifact (unit_class scratch fires regardless of other fields)
        assert!(evaluate(&json!({"rows":[{"path":"x.log","unit_class":"scratch","owner":"o","justification_ref":"ADR-1","reachable_from":["masterplan"],"ttl":{"ttl_class":"scratch"}}]}))
            .violations.contains("scratch_artifact"));
    }

    #[test]
    fn scratch_artifact_cannot_be_laundered_by_registration() {
        // The registration-bypass hole the generic husk-block-on-new leaves open: a scratch
        // file that registers owner + resolving justification + reachability + ttl_class would
        // emit ZERO of the per-row debt codes. The SHAPE-based scratch_artifact code closes it —
        // it fires on unit_class alone, so a fully-registered scratch row is STILL RED and the
        // ONLY violation is scratch_artifact (no unowned/unjustified/unreachable/no_ttl_class).
        let fully_registered_scratch = json!({
            "rows": [{
                "path": "run-slice.sh",
                "unit_class": "scratch",
                "owner": "platform-ci",
                "justification_ref": "ADR-0555",
                "justification_resolves": true,
                "reachable_from": ["specs/masterplan.json"],
                "ttl": {"ttl_class": "scratch", "budget_days": 0, "action": "delete", "protected": false}
            }]
        });
        let findings = evaluate_keyed(&fully_registered_scratch);
        assert!(findings.contains(&Finding::new("scratch_artifact", "run-slice.sh")));
        let codes: BTreeSet<String> = findings.iter().map(|f| f.code.clone()).collect();
        assert_eq!(
            codes,
            BTreeSet::from(["scratch_artifact".to_owned()]),
            "a fully-registered scratch row must fire ONLY scratch_artifact (registration cannot suppress the shape-based code, and cannot leave any other debt code)"
        );
        assert_eq!(evaluate(&fully_registered_scratch).verdict, Verdict::Red);
    }
    /// Derived `move` with no committed plan covering it — the 447-path class on the live tree.
    #[test]
    fn a_derived_move_with_no_committed_plan_is_reported() {
        let fixture = serde_json::json!({
            "planned_move_paths": ["oya/intelligence/crates/oya-codeview-cli"],
            "rows": [row_ok("oya/intelligence/crates/oya-orphan/src/lib.rs", "move")],
        });
        let findings = evaluate_keyed(&fixture);
        assert!(
            findings.contains(&Finding::new(
                "move_derived_not_planned",
                "oya/intelligence/crates/oya-orphan/src/lib.rs"
            )),
            "got {findings:?}"
        );
    }

    /// The other direction: a plan moves a path the registry says is already home.
    #[test]
    fn a_planned_move_the_registry_says_should_stay_is_reported() {
        let fixture = serde_json::json!({
            "planned_move_paths": ["iam/settled"],
            "rows": [row_ok("iam/settled/src/lib.rs", "retain")],
        });
        let findings = evaluate_keyed(&fixture);
        assert!(
            findings.contains(&Finding::new(
                "move_planned_not_derived",
                "iam/settled/src/lib.rs"
            )),
            "got {findings:?}"
        );
    }

    /// Coverage is PREFIX-based: a plan names the moving root, the registry rows are the files
    /// beneath it. Exact-match here would report every file under a planned root as unplanned.
    #[test]
    fn a_file_under_a_planned_root_counts_as_planned() {
        let fixture = serde_json::json!({
            "planned_move_paths": ["oya/intelligence/crates/oya-codeview-cli"],
            "rows": [row_ok("oya/intelligence/crates/oya-codeview-cli/src/main.rs", "move")],
        });
        let findings = evaluate_keyed(&fixture);
        assert!(
            !findings
                .iter()
                .any(|f| f.code == "move_derived_not_planned"),
            "a file beneath a planned root must not be reported unplanned; got {findings:?}"
        );
    }

    /// A sibling whose name merely PREFIXES a planned path is not covered by it.
    #[test]
    fn prefix_matching_respects_path_boundaries() {
        let fixture = serde_json::json!({
            "planned_move_paths": ["iam/svc"],
            "rows": [row_ok("iam/svc-other/src/lib.rs", "move")],
        });
        let findings = evaluate_keyed(&fixture);
        assert!(
            findings.contains(&Finding::new(
                "move_derived_not_planned",
                "iam/svc-other/src/lib.rs"
            )),
            "iam/svc-other must NOT be covered by a plan for iam/svc; got {findings:?}"
        );
    }

    /// No committed plan at all ⇒ every derived move is unplanned. Fails toward reporting
    /// disagreement rather than certifying agreement the tree cannot support.
    #[test]
    fn an_empty_plan_set_reports_every_derived_move() {
        let fixture = serde_json::json!({
            "rows": [row_ok("iam/x/src/lib.rs", "move")],
        });
        let findings = evaluate_keyed(&fixture);
        assert!(
            findings.contains(&Finding::new(
                "move_derived_not_planned",
                "iam/x/src/lib.rs"
            )),
            "got {findings:?}"
        );
    }
}
