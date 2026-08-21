//! # cloud-ci-slo-coverage
//!
//! Portable conformance gate for catalog SLO declarations. The producer owns all repository I/O:
//! it enumerates `registry/catalog/*.yaml`, maps each file stem to `crate_id`, and parses the
//! top-level `slo:` scalar into rows shaped as `{"crate_id", "source_path", "slo"}`. This crate
//! stays pure and reuses `check_slo_coverage::validate_slo_coverage` per row so the legacy
//! predicate and the cloud-ci gate cannot drift.
//!
//! `evaluate_keyed` returns one `Finding{code,key}` per invalid catalog row. Current oyatie corpus
//! is clean, so the disposition table marks all violation codes `frozen_empty`: a future missing
//! SLO cannot be laundered into the accepted baseline by regeneration.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use check_slo_coverage::{SloCatalogRecord, SloCoverageError, validate_slo_coverage};
use serde_json::Value;

/// The gate id, matching oya-ci config and the baseline ratchet.
pub const GATE_ID: &str = "cloud-ci-slo-coverage";

/// Stable blocking violation codes emitted by this gate.
pub const VIOLATION_CODES: [&str; 4] = [
    "slo_missing_or_blank_slo",
    "slo_empty_crate_id",
    "slo_no_catalog_records",
    // PR-C3 composition: an SLO row whose catalog record is silently stale (names no live
    // workspace crate AND carries no explicit non-live marker) is a coverage violation too — a
    // valid `slo:` on a dead-unmarked record is still a false-green at this surface.
    "slo_row_no_live_crate_unmarked",
];

/// The observed catalog census is BELOW its frozen pin. Blocking.
///
/// A drop has exactly two causes and the count cannot tell them apart: rows were legitimately
/// deleted (a retired crate, a consolidation), or the producer's enumeration collapsed and the
/// gate is about to report a clean pass over a corpus it never read. Re-freezing the pin in the
/// change that caused the drop is the only place a human can state which it was.
pub const CODE_CATALOG_CENSUS_DROP_UNATTRIBUTED: &str = "slo_catalog_census_drop_unattributed";

/// The observed catalog census is ABOVE its frozen pin. Blocking.
///
/// Growth is normally legitimate, but leaving it unattributed is what made the old FLOOR go stale:
/// the corpus drifted up, the guard's margin drifted with it, and nothing said so. Re-freezing on
/// this side costs one line in the change that added the rows and keeps the pin exact.
pub const CODE_CATALOG_CENSUS_GROWTH_UNATTRIBUTED: &str = "slo_catalog_census_growth_unattributed";

/// A census mismatch against the frozen pin: which side it fell on, and the remedy.
///
/// Deliberately NOT a [`Finding`] and deliberately NOT in [`VIOLATION_CODES`]: those codes are the
/// baseline-ratchet roster of things [`evaluate_keyed`] emits per catalog ROW. This is a property
/// of the corpus as a whole, checked by the live-corpus gate against a pin it owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CensusFinding {
    pub code: &'static str,
    pub detail: String,
}

/// Anti-vacuity census rule: the observed row count must EQUAL the frozen pin.
///
/// # Why this is an equality pin and not the floor it replaced
///
/// The gate used to carry `MIN_SLO_CATALOG_ROWS`, a FLOOR: at least N rows, or the producer is
/// presumed broken. That is a floor on a term whose legitimate direction is DOWN — catalog rows go
/// away when crates are retired — so every legitimate deletion had to hand-lower it, and reviewed
/// deletions repeatedly did not (#1451, #1413, then #1485, #1483). The floor was left
/// at 766 while the corpus fell to 755 and this gate sat RED on dev, discovered only later; it was
/// then re-anchored 776 -> 766 -> 738, each time chasing the corpus downward. A guard whose
/// maintenance is a separate act from the change that invalidates it will be stale, and a stale
/// floor is silently slack: at 738 against a 745-row corpus the producer could lose seven rows and
/// still read as a pass.
///
/// An equality pin cannot go stale, because staleness IS the failure. It fires in the change that
/// moves the number, where the author states what moved and a reviewer reads it — the same
/// discipline this repo already applies to its other census pins. It keeps every bit of the
/// anti-vacuity protection (a collapsed enumeration reads below the pin and blocks) and adds the
/// protection the floor never had (a PARTIAL collapse, above the floor but below the truth).
///
/// Comparison only, never subtraction: these are `usize`, and `frozen - observed` panics in debug
/// and WRAPS in release, where a wrapped difference reads as a passing margin.
#[must_use]
pub fn evaluate_catalog_census(observed: usize, frozen: usize) -> Option<CensusFinding> {
    if observed == frozen {
        return None;
    }
    let (code, cause) = if observed < frozen {
        (
            CODE_CATALOG_CENSUS_DROP_UNATTRIBUTED,
            "rows were legitimately removed, or the producer's enumeration collapsed and this \
             gate is reading a corpus it never fully saw",
        )
    } else {
        (
            CODE_CATALOG_CENSUS_GROWTH_UNATTRIBUTED,
            "rows were legitimately added, or the enumeration widened past the corpus it is \
             supposed to cover",
        )
    };
    Some(CensusFinding {
        code,
        detail: format!(
            "the slo-coverage face enumerated {observed} catalog rows but the frozen census pin is \
             {frozen} — {cause}, and the count cannot tell those apart. Re-freeze the pin to \
             {observed} in THIS change and state which it was."
        ),
    })
}

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
    let mut findings = BTreeSet::new();
    let rows = match input.get("rows").and_then(Value::as_array) {
        Some(rows) if !rows.is_empty() => rows,
        _ => {
            findings.insert(Finding::new(
                "slo_no_catalog_records",
                "<empty-slo-catalog>",
            ));
            return findings;
        }
    };

    for row in rows {
        let record = record_from_row(row);
        if let Err(error) = validate_slo_coverage(&[record]) {
            findings.insert(finding_for(error));
        }
        // PR-C3 composition: a row may carry a valid SLO yet name a silently-stale catalog
        // record. When the producer tags liveness (is_live + marker), enforce live-OR-marked
        // here too — a `slo:` on a dead-unmarked record cannot launder a stale row as covered.
        // Rows WITHOUT the liveness fields (legacy unit fixtures) are unaffected.
        if row_is_dead_and_unmarked(row) {
            let crate_id = row
                .get("crate_id")
                .and_then(Value::as_str)
                .unwrap_or("<empty-crate-id>");
            findings.insert(Finding::new("slo_row_no_live_crate_unmarked", crate_id));
        }
    }
    findings
}

/// True iff the producer tagged this row as NOT live AND carrying no explicit non-live marker.
/// Returns false when the liveness fields are absent (legacy fixtures) so the composition is
/// purely additive — it never fires unless the producer supplied `is_live`/`marker`.
fn row_is_dead_and_unmarked(row: &Value) -> bool {
    // `is_live` absent ⇒ the producer did not tag liveness ⇒ do not compose (false).
    let Some(is_live) = row.get("is_live").and_then(Value::as_bool) else {
        return false;
    };
    if is_live {
        return false;
    }
    let marked = row
        .get("marker")
        .and_then(Value::as_str)
        .is_some_and(|m| !m.trim().is_empty());
    !marked
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

    fn live_rows(records: &[(&str, Option<&str>, bool, Option<&str>)]) -> Value {
        json!({
            "rows": records
                .iter()
                .map(|(crate_id, slo, is_live, marker)| json!({
                    "crate_id": crate_id,
                    "slo": slo,
                    "is_live": is_live,
                    "marker": marker,
                }))
                .collect::<Vec<_>>()
        })
    }

    #[test]
    fn live_row_with_slo_is_green_under_composed_predicate() {
        let input = live_rows(&[("compute-domain", Some("preview-control-plane"), true, None)]);
        assert!(
            evaluate_keyed(&input).is_empty(),
            "got {:?}",
            evaluate_keyed(&input)
        );
        assert_eq!(evaluate(&input).verdict, Verdict::Green);
    }

    #[test]
    fn dead_unmarked_row_with_valid_slo_still_fails_the_composed_predicate() {
        // The PR-C3 false-green the tightening closes: a valid `slo:` on a stale record.
        let input = live_rows(&[(
            "oya-cloud-dcops-domain",
            Some("local-foundation-no-runtime-slo"),
            false,
            None,
        )]);
        let findings = evaluate_keyed(&input);
        assert!(
            findings.iter().any(|f| {
                f.code == "slo_row_no_live_crate_unmarked" && f.key == "oya-cloud-dcops-domain"
            }),
            "a dead-unmarked row must fail even WITH a valid SLO: {findings:?}"
        );
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn dead_but_marked_row_passes_the_composed_predicate() {
        let input = live_rows(&[(
            "oya-cloud-billing-adapter-fake",
            Some("no-runtime-slo"),
            false,
            Some("retired-compatibility-row-no-crate"),
        )]);
        assert!(
            evaluate_keyed(&input).is_empty(),
            "a dead BUT marked row must pass: {:?}",
            evaluate_keyed(&input)
        );
    }

    #[test]
    fn legacy_rows_without_liveness_fields_are_unaffected() {
        // Rows lacking is_live/marker (legacy fixtures) must not trigger the composed predicate.
        let input = rows(&[("oya-good-kernel", Some("preview-control-plane"))]);
        assert!(evaluate_keyed(&input).is_empty());
        assert_eq!(evaluate(&input).verdict, Verdict::Green);
    }

    // THE BEAD (oyatie-whk class, real instance). Replays the exact history the deleted floor
    // recorded in its own doc comment and shows the two-sided pin behaves correctly at every step
    // the floor got wrong.
    #[test]
    fn the_census_pin_fires_on_the_deletions_the_floor_slept_through() {
        // 2026-07-31, as recorded: floor 766, corpus fallen to 755 by #1451 (-20) and #1413 (-1).
        // The floor's failure mode was that it went RED late and unattributed. The pin goes red
        // IN the deleting change, naming the remedy.
        let finding = evaluate_catalog_census(755, 776).expect("a 21-row deletion must be caught");
        assert_eq!(finding.code, CODE_CATALOG_CENSUS_DROP_UNATTRIBUTED);
        assert!(
            finding.detail.contains("Re-freeze the pin to 755"),
            "the remedy must name the measured number: {}",
            finding.detail
        );
        // Re-freezing in that change clears it. This is the whole ratchet: the number moves with
        // an author attached, never on its own.
        assert_eq!(evaluate_catalog_census(755, 755), None);

        // THE CASE THE FLOOR COULD NOT SEE, and the reason this is a pin and not a smaller floor.
        // At floor 738 against the 745-row corpus that set it, the producer can lose seven rows —
        // a partial enumeration collapse — and the floor still reads as a PASS. Control first, so
        // this proves something about the new rule rather than about arithmetic.
        let old_floor: usize = 738;
        let after_partial_collapse: usize = 738;
        assert!(
            after_partial_collapse >= old_floor,
            "control: this shape genuinely PASSES the deleted floor, so the failure below can \
             only be the new rule"
        );
        let partial_collapse = evaluate_catalog_census(after_partial_collapse, 745)
            .expect("a partial collapse must be caught");
        assert_eq!(partial_collapse.code, CODE_CATALOG_CENSUS_DROP_UNATTRIBUTED);

        // The other side, with a distinct code: growth is not laundered as "still above the
        // floor", which is how the pin stops drifting slack in the first place.
        let growth = evaluate_catalog_census(746, 745).expect("growth must be attributed too");
        assert_eq!(growth.code, CODE_CATALOG_CENSUS_GROWTH_UNATTRIBUTED);
        assert_ne!(growth.code, partial_collapse.code);
    }

    // Total enumeration failure — the false green the original floor existed for — still blocks.
    #[test]
    fn a_collapsed_enumeration_still_fails_closed() {
        let finding = evaluate_catalog_census(0, 745).expect("zero rows must never be a pass");
        assert_eq!(finding.code, CODE_CATALOG_CENSUS_DROP_UNATTRIBUTED);
        // And it is not always-on: at the pin the rule is silent, including at a pin of zero, so
        // the rule has no number it can never satisfy.
        assert_eq!(evaluate_catalog_census(0, 0), None);
    }

    // The census codes are a corpus-level property, not per-row findings, so they must stay OUT of
    // the baseline-ratchet roster. If evaluate_keyed ever starts emitting them this fails, which is
    // the point: the roster and the evaluator must not drift.
    #[test]
    fn census_codes_are_not_row_violation_codes() {
        for code in [
            CODE_CATALOG_CENSUS_DROP_UNATTRIBUTED,
            CODE_CATALOG_CENSUS_GROWTH_UNATTRIBUTED,
        ] {
            assert!(
                !VIOLATION_CODES.contains(&code),
                "{code} leaked into the ratchet roster"
            );
        }
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
