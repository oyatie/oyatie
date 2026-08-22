//! # cloud-ci-catalog-liveness
//!
//! Portable conformance gate enforcing the catalog-liveness policy:
//! every catalog record must either name a LIVE workspace crate or carry an EXPLICIT non-live
//! marker. Catalog YAML is optional census metadata — a live workspace crate without a row is
//! not a finding. Membership is the Cargo workspace plus the closed capability registry.
//! Unmarked extra YAML (a row whose crate is gone) remains RED.
//!
//! The producer (`oya-cloud-ci-accounting-registry-app`) owns all repository I/O: it enumerates
//! `registry/catalog/*.yaml` via the config-declared `[catalog_liveness].catalog_record_globs`,
//! maps each file stem to `crate_id`, resolves the LIVE workspace member universe IN-PROCESS via
//! `oya-workspace-members-kernel` (NEVER a `cargo metadata` / `buck2` shell-out, for
//! all-CLI-retirement and hermeticity), parses `traceability.source_crate`, and classifies explicit
//! non-live markers/exemptions. This crate stays pure: `evaluate_keyed` applies only boolean policy
//! over the face.
//!
//! `evaluate_keyed` returns one `Finding{code,key}` per catalog drift. The Oyatie corpus is
//! expected to carry ZERO findings, so the disposition table marks violation codes `frozen_empty`:
//! future stale rows and stale source paths cannot be laundered into the accepted baseline by
//! regeneration. Missing catalog YAML is not reverse-coverage debt.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use serde_json::Value;

/// The gate id, matching oya-ci config and the baseline ratchet.
pub const GATE_ID: &str = "cloud-ci-catalog-liveness";

/// Stable blocking violation codes emitted by this gate.
pub const VIOLATION_CODES: [&str; 7] = [
    "catalog_record_no_live_crate_unmarked",
    "catalog_record_source_crate_missing",
    "catalog_live_crate_without_row",
    "catalog_live_crate_empty_id",
    "catalog_live_crates_missing",
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
    row.get("crate_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

fn has_declared_source_crate(row: &Value) -> bool {
    row.get("source_crate")
        .and_then(Value::as_str)
        .is_some_and(|p| !p.trim().is_empty())
}

fn source_crate_exists(row: &Value) -> bool {
    row.get("source_crate_exists").and_then(Value::as_bool) == Some(true)
}

/// Pure evaluator. Input shape:
/// `{"rows":[{"crate_id","source_path","is_live","marker","source_crate","source_crate_exists"}],
///   "live_crates":[{"crate_id","member_path","has_catalog_row","exemption"}]}`.
///
/// It emits one finding per stale catalog row or stale non-historical source path. An empty
/// catalog row corpus is RED. A missing `live_crates` collection is also RED so a producer that
/// dropped the workspace member set cannot present as green. Live crates without a catalog row
/// are not findings.
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
        // If a live/unmarked row declares traceability.source_crate, that path must be live too.
        // Historical/non-live rows may keep old paths as provenance because the marker is explicit.
        if has_declared_source_crate(row) && !source_crate_exists(row) && !is_marked_non_live(row) {
            findings.insert(Finding::new(
                "catalog_record_source_crate_missing",
                crate_id,
            ));
        }
    }

    let Some(live_crates) = input.get("live_crates").and_then(Value::as_array) else {
        findings.insert(Finding::new(
            "catalog_live_crates_missing",
            "<missing-live-crates>",
        ));
        return findings;
    };

    for row in live_crates {
        let crate_id = crate_id_of(row);
        if crate_id.trim().is_empty() {
            findings.insert(Finding::new(
                "catalog_live_crate_empty_id",
                "<empty-crate-id>",
            ));
        }
        // Catalog YAML is optional. A live crate without a row is not a finding.
        // `catalog_live_crate_without_row` remains in VIOLATION_CODES for disposition
        // compatibility and is not emitted.
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
                    "source_crate": if *is_live {
                        json!(format!("{crate_id}/Cargo.toml"))
                    } else {
                        Value::Null
                    },
                    "source_crate_exists": *is_live,
                }))
                .collect::<Vec<_>>(),
            "live_crates": records
                .iter()
                .filter(|(_, is_live, _)| *is_live)
                .map(|(crate_id, _, _)| json!({
                    "crate_id": crate_id,
                    "member_path": crate_id,
                    "has_catalog_row": true,
                    "exemption": Value::Null,
                }))
                .collect::<Vec<_>>()
        })
    }

    #[test]
    fn live_record_is_green() {
        let input = rows(&[("cell-region", true, None), ("compute-domain", true, None)]);
        assert_eq!(evaluate(&input).verdict, Verdict::Green);
        assert!(evaluate_keyed(&input).is_empty());
    }

    #[test]
    fn dead_but_marked_record_is_green() {
        // The 53 marker records PR-C1/PR-C2 used must pass: not live, but explicitly marked.
        let input = rows(&[
            (
                "oya-cloud-billing-adapter-fake",
                false,
                Some("retired-compatibility-row-no-crate"),
            ),
            (
                "oya-cloud-dcops-kernel",
                false,
                Some("designed-ahead-row-no-crate"),
            ),
            ("some-planned-cap", false, Some("planned")),
            ("some-aspirational-cap", false, Some("aspirational")),
            ("some-no-claims-cap", false, Some("non-claims-no-crate")),
        ]);
        assert_eq!(
            evaluate(&input).verdict,
            Verdict::Green,
            "got {:?}",
            evaluate_keyed(&input)
        );
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
            (
                "dead-marked",
                false,
                Some("retired-compatibility-row-no-crate"),
            ),
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
    fn stale_source_crate_on_live_row_is_red() {
        let input = json!({
            "rows": [{
                "crate_id": "audit-emission-api",
                "source_path": "registry/catalog/audit-emission-api.yaml",
                "is_live": true,
                "marker": Value::Null,
                "source_crate": "crates/oya-audit-chain-emission-api/Cargo.toml",
                "source_crate_exists": false,
            }],
            "live_crates": [{
                "crate_id": "audit-emission-api",
                "member_path": "audit/ports/emission-api",
                "has_catalog_row": true,
                "exemption": Value::Null,
            }]
        });
        let findings = evaluate_keyed(&input);
        assert!(findings.iter().any(|finding| {
            finding.code == "catalog_record_source_crate_missing"
                && finding.key == "audit-emission-api"
        }));
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
    }

    #[test]
    fn marked_historical_row_may_keep_stale_source_crate() {
        let input = json!({
            "rows": [{
                "crate_id": "retired-cap",
                "source_path": "registry/catalog/retired-cap.yaml",
                "is_live": false,
                "marker": "retired-compatibility-row-no-crate",
                "source_crate": "crates/retired-cap/Cargo.toml",
                "source_crate_exists": false,
            }],
            "live_crates": []
        });
        assert!(evaluate_keyed(&input).is_empty());
        assert_eq!(evaluate(&input).verdict, Verdict::Green);
    }

    #[test]
    fn live_crate_without_row_is_green() {
        let input = json!({
            "rows": [{
                "crate_id": "cataloged-live",
                "source_path": "registry/catalog/cataloged-live.yaml",
                "is_live": true,
                "marker": Value::Null,
                "source_crate": "cataloged-live/Cargo.toml",
                "source_crate_exists": true,
            }],
            "live_crates": [
                {
                    "crate_id": "cataloged-live",
                    "member_path": "cataloged-live",
                    "has_catalog_row": true,
                    "exemption": Value::Null,
                },
                {
                    "crate_id": "missing-live",
                    "member_path": "missing-live",
                    "has_catalog_row": false,
                    "exemption": Value::Null,
                }
            ]
        });
        let findings = evaluate_keyed(&input);
        assert!(
            !findings.iter().any(|finding| {
                finding.code == "catalog_live_crate_without_row" && finding.key == "missing-live"
            }),
            "catalog YAML is optional: {findings:?}"
        );
        assert_eq!(evaluate(&input).verdict, Verdict::Green, "{findings:?}");
    }

    #[test]
    fn live_crate_without_row_needs_no_exemption() {
        let input = json!({
            "rows": [{
                "crate_id": "cataloged-live",
                "source_path": "registry/catalog/cataloged-live.yaml",
                "is_live": true,
                "marker": Value::Null,
                "source_crate": "cataloged-live/Cargo.toml",
                "source_crate_exists": true,
            }],
            "live_crates": [{
                "crate_id": "temporarily-exempt",
                "member_path": "temporarily-exempt",
                "has_catalog_row": false,
                "exemption": Value::Null,
            }]
        });
        assert!(
            evaluate_keyed(&input).is_empty(),
            "{:?}",
            evaluate_keyed(&input)
        );
    }

    #[test]
    fn missing_live_crates_collection_is_red() {
        let input = json!({
            "rows": [{
                "crate_id": "cataloged-live",
                "source_path": "registry/catalog/cataloged-live.yaml",
                "is_live": true,
                "marker": Value::Null,
                "source_crate": "cataloged-live/Cargo.toml",
                "source_crate_exists": true,
            }]
        });
        let findings = evaluate_keyed(&input);
        assert!(findings.iter().any(|finding| {
            finding.code == "catalog_live_crates_missing" && finding.key == "<missing-live-crates>"
        }));
        assert_eq!(evaluate(&input).verdict, Verdict::Red);
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
