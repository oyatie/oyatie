//! # cloud-ci-catalog-liveness
//!
//! Portable conformance gate enforcing the founder **live-OR-explicitly-marked** catalog policy:
//! every catalog record must either name a LIVE workspace crate or carry an EXPLICIT non-live
//! marker. This closes the catalog-liveness false-green permanently — a record whose crate has
//! been retired/moved (or never existed) cannot sit silently in the registry pretending to be a
//! live capability.
//!
//! The producer (`oya-cloud-ci-accounting-registry-app`) owns all repository I/O: it enumerates
//! `registry/catalog/*.yaml` via the config-declared `[catalog_liveness].catalog_record_globs`,
//! maps each file stem to `crate_id`, resolves the LIVE workspace crate-id universe IN-PROCESS via
//! `oya-workspace-members-kernel` (NEVER a `cargo metadata` / `buck2` shell-out, for
//! all-CLI-retirement and hermeticity), and parses the explicit non-live marker. Rows are shaped
//! as `{"crate_id", "source_path", "is_live", "marker"}` and this crate stays pure: `evaluate_keyed`
//! applies only the boolean policy over the face.
//!
//! `evaluate_keyed` returns one `Finding{code,key}` per silently-stale catalog row (a record that
//! is neither live nor marked). Post-PR-C1/PR-C2 the oyatie corpus carries ZERO such records, so
//! the disposition table marks the violation code `frozen_empty`: a future silently-stale record
//! cannot be laundered into the accepted baseline by regeneration (born-blocking, EMPTY baseline).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

/// The gate id, matching oya-ci config and the baseline ratchet.
pub const GATE_ID: &str = "cloud-ci-catalog-liveness";

/// Stable blocking violation codes emitted by this gate.
pub const VIOLATION_CODES: [&str; 3] = [
    "catalog_record_no_live_crate_unmarked",
    "catalog_record_empty_crate_id",
    "catalog_no_records",
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

/// True iff the row carries an EXPLICIT non-live marker the producer parsed from the record.
/// The producer stamps `marker` to the verbatim non-live `status:` value (e.g.
/// `retired-compatibility-row-no-crate` / `designed-ahead-row-no-crate` / `planned` /
/// `aspirational`) or a synthetic `non-claims-no-crate` token when the record's `non_claims`
/// block explicitly states no matching crate exists; an absent/blank marker is `null`/`""`.
/// Keeping this a single helper means the "what counts as explicitly marked" question has ONE
/// answer the producer fills and the gate reads — they cannot drift.
fn is_marked_non_live(row: &Value) -> bool {
    row.get("marker")
        .and_then(Value::as_str)
        .is_some_and(|m| !m.trim().is_empty())
}

fn is_live(row: &Value) -> bool {
    row.get("is_live").and_then(Value::as_bool) == Some(true)
}

fn crate_id_of(row: &Value) -> &str {
    row.get("crate_id").and_then(Value::as_str).unwrap_or_default()
}

/// Pure evaluator: takes `{"rows":[{"crate_id","source_path","is_live","marker"}, ...]}` and
/// emits one finding per silently-stale catalog row — a record that names NO live workspace crate
/// AND carries NO explicit non-live marker. An empty corpus is RED (born-blocking; an empty face
/// must not be laundered into a passing verdict). An empty `crate_id` is its own code (a malformed
/// record could otherwise pass by being neither live nor checkable).
pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let rows = match input.get("rows").and_then(Value::as_array) {
        Some(rows) if !rows.is_empty() => rows,
        _ => {
            findings.insert(Finding::new("catalog_no_records", "<empty-catalog>"));
            return findings;
        }
    };

    for row in rows {
        let crate_id = crate_id_of(row);
        if crate_id.trim().is_empty() {
            findings.insert(Finding::new(
                "catalog_record_empty_crate_id",
                "<empty-crate-id>",
            ));
            continue;
        }
        // live-OR-explicitly-marked: a record is OK iff its stem is a live workspace crate-id OR
        // it declares an explicit non-live marker. Anything else is silently stale.
        if !is_live(row) && !is_marked_non_live(row) {
            findings.insert(Finding::new(
                "catalog_record_no_live_crate_unmarked",
                crate_id,
            ));
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

    fn rows(records: &[(&str, bool, Option<&str>)]) -> Value {
        json!({
            "rows": records
                .iter()
                .map(|(crate_id, is_live, marker)| json!({
                    "crate_id": crate_id,
                    "source_path": format!("registry/catalog/{crate_id}.yaml"),
                    "is_live": is_live,
                    "marker": marker,
                }))
                .collect::<Vec<_>>()
        })
    }

    #[test]
    fn live_record_is_green() {
        let input = rows(&[
            ("cell-region", true, None),
            ("compute-domain", true, None),
        ]);
        assert_eq!(evaluate(&input).verdict, Verdict::Green);
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn dead_but_marked_record_is_green() {
        // The 53 marker records PR-C1/PR-C2 used must pass: not live, but explicitly marked.
        let input = rows(&[
            ("oya-cloud-billing-adapter-fake", false, Some("retired-compatibility-row-no-crate")),
            ("oya-cloud-dcops-kernel", false, Some("designed-ahead-row-no-crate")),
            ("some-planned-cap", false, Some("planned")),
            ("some-aspirational-cap", false, Some("aspirational")),
            ("some-no-claims-cap", false, Some("non-claims-no-crate")),
        ]);
        assert_eq!(evaluate(&input).verdict, Verdict::Green, "got {:?}", evaluate_keyed(&input));
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn dead_and_unmarked_record_is_red() {
        let input = rows(&[("oya-cloud-dcops-domain", false, None)]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "catalog_record_no_live_crate_unmarked");
        assert_eq!(finding.key, "oya-cloud-dcops-domain");
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn blank_marker_does_not_count_as_marked() {
        let input = rows(&[("dead-blank-marker", false, Some("   "))]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        assert_eq!(
            findings.iter().next().unwrap().code,
            "catalog_record_no_live_crate_unmarked"
        );
    }

    #[test]
    fn empty_crate_id_has_dedicated_code() {
        let input = rows(&[("", false, None)]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 1, "got {findings:?}");
        let finding = findings.iter().next().unwrap();
        assert_eq!(finding.code, "catalog_record_empty_crate_id");
        assert_eq!(finding.key, "<empty-crate-id>");
    }

    #[test]
    fn surface_all_one_finding_per_dead_unmarked_row() {
        let input = rows(&[
            ("live-one", true, None),
            ("dead-unmarked-a", false, None),
            ("dead-marked", false, Some("retired-compatibility-row-no-crate")),
            ("dead-unmarked-b", false, None),
        ]);
        let findings = evaluate_keyed(&input);
        assert_eq!(findings.len(), 2, "got {findings:?}");
        assert!(
            findings
                .iter()
                .all(|finding| finding.code == "catalog_record_no_live_crate_unmarked")
        );
        let keys: BTreeSet<&str> = findings.iter().map(|f| f.key.as_str()).collect();
        assert!(keys.contains("dead-unmarked-a") && keys.contains("dead-unmarked-b"));
    }

    #[test]
    fn evaluate_is_bare_projection_of_evaluate_keyed() {
        let input = rows(&[("dead-unmarked", false, None), ("", false, None)]);
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
        assert_eq!(finding.code, "catalog_no_records");
        assert_eq!(finding.key, "<empty-catalog>");
        assert_eq!(evaluate(&json!({ "rows": [] })).verdict, Verdict::Red);
    }
}
