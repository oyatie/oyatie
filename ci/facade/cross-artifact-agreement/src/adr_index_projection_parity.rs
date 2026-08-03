//! Gate-coverage check 3/3 (born-advisory): generated-projection parity for the
//! ADR index.
//!
//! The #1327 defect class (d): the generated projections `docs/ADR-INDEX.md` and
//! `docs/machine-readable/decisions.json` were not regenerated through their
//! producer, so they drifted from the `docs/decisions/*.md` corpus. The
//! `docs/automation/adr-index-pipeline.md` spec promised "PR fails if generated
//! output differs" but that gate was never implemented
//! (`registry/fixuptasks.jsonl` F-CLOUDCI-CANON-DRIFT-AUTOGEN-GATE). This check
//! implements that promise WITHOUT shelling out: it invokes the SAME producer
//! library code the `oya doc adr-index` step uses — the pure kernel
//! [`oya_check_adr_index`] — to re-render both projections from the source records
//! and compares them to the committed bytes.
//!
//! Two failure surfaces:
//! - **byte-parity** — [`oya_check_adr_index::validate_adr_index`] re-renders both
//!   files from `records` (the source-of-truth ADR set) and reports
//!   `MarkdownDrift` / `JsonDrift` when a committed file is stale or hand-edited.
//! - **id-set staleness** — the record id set must exactly cover the ADR files on
//!   disk (`source_adr_ids`, derived from `docs/decisions/*.md` filenames): an ADR
//!   added or removed without regenerating the projections shows up as a
//!   `missing_adr` / `phantom_adr` finding even when the rest re-renders cleanly.
//!
//! Each drift Finding names the offending file and the first differing row so the
//! FAIL output is actionable. Pure evaluator over caller-supplied records + bytes.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::BTreeSet;

use oya_check_adr_index::{
    AdrDecisionRecord, AdrIndexArtifacts, AdrIndexError, generate_adr_index, validate_adr_index,
};
use serde_json::Value;

use crate::Finding;

/// Validator id recorded by the ADR-index projection-parity contract.
pub const ADR_INDEX_PARITY_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/adr-index-projection-parity";

/// The advisory violation code this check emits.
pub const ADR_INDEX_PROJECTION_STALE_CODE: &str = "adr_index_projection_stale";

/// Repo-relative path of the generated human ADR index projection.
pub const ADR_INDEX_MD_PATH: &str = "docs/ADR-INDEX.md";

/// Repo-relative path of the generated machine-readable ADR mirror.
pub const DECISIONS_JSON_PATH: &str = "docs/machine-readable/decisions.json";

fn stale(key: &str) -> Finding {
    Finding::new(ADR_INDEX_PROJECTION_STALE_CODE, key)
}

/// Why a regeneration of the ADR-index projection could not be produced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdrProjectionEmitError {
    /// `decisions.json` did not carry a decodable `decisions[]` record array.
    MalformedDecisionsJson { reason: String },
    /// The records decoded but the producer kernel refused to render them.
    Unrenderable(AdrIndexError),
}

/// Decode the authoritative ADR record array out of `docs/machine-readable/decisions.json`.
///
/// `decisions[]` is the one hand-authored surface: every other byte of BOTH
/// projections is derived from it. This is the only complete `AdrDecisionRecord`
/// source that exists today — the strict `corpus-doc-parser` IR path
/// ([`oya_check_adr_index::generate_adr_index_from_ir`]) parses only a subset of
/// the live `docs/decisions/*.md` corpus, and that crate forbids rendering a
/// partial parse as the authoritative index.
pub fn adr_records_from_decisions_json(
    decisions: &Value,
) -> Result<Vec<AdrDecisionRecord>, AdrProjectionEmitError> {
    let malformed = |reason: String| AdrProjectionEmitError::MalformedDecisionsJson { reason };
    let entries = decisions["decisions"]
        .as_array()
        .ok_or_else(|| malformed("decisions[] is missing or not an array".to_owned()))?;
    let mut records = Vec::with_capacity(entries.len());
    for entry in entries {
        let string_field = |field: &str| -> Result<String, AdrProjectionEmitError> {
            entry[field]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| malformed(format!("decisions[] entry missing string field {field}")))
        };
        // Absent relation arrays are legitimately empty, but a present non-array
        // is malformed input and must not silently decode as "no relations".
        let string_list = |field: &str| -> Result<Vec<String>, AdrProjectionEmitError> {
            match &entry[field] {
                Value::Null => Ok(Vec::new()),
                Value::Array(items) => items
                    .iter()
                    .map(|item| {
                        item.as_str().map(str::to_owned).ok_or_else(|| {
                            malformed(format!("decisions[] {field} holds a non-string"))
                        })
                    })
                    .collect(),
                _ => Err(malformed(format!("decisions[] {field} is not an array"))),
            }
        };
        let number = entry["number"]
            .as_u64()
            .and_then(|number| u16::try_from(number).ok())
            .ok_or_else(|| malformed("decisions[] entry has no u16 number".to_owned()))?;
        records.push(AdrDecisionRecord {
            number,
            id: string_field("adr")?,
            title: string_field("title")?,
            status: string_field("status")?,
            owner: string_field("owner")?,
            date: string_field("date")?,
            path: string_field("path")?,
            supersedes: string_list("supersedes")?,
            superseded_by: string_list("superseded_by")?,
            related: string_list("related")?,
        });
    }
    Ok(records)
}

/// Regenerate BOTH ADR-index projection faces from the committed record array.
///
/// This is the emitter the parity gate has always had but discarded: the same
/// producer kernel the gate byte-compares against, exposed so the projections can
/// be *produced* rather than hand-maintained. Everything derived — `_metadata`
/// totals, numbering, gaps, next-ADR, status counts, the full ADR-INDEX.md table,
/// at-a-glance and sources-scanned rows — comes back computed from `decisions[]`.
///
/// Pure: no filesystem access. Writing the returned bytes to the committed
/// projections is deliberately NOT done here — those two paths are committed merge
/// surfaces, not materialized faces, so a writer belongs with the governed
/// de-commit change, not in a gate evaluated on every CI run.
pub fn regenerate_adr_index_projection(
    decisions: &Value,
) -> Result<AdrIndexArtifacts, AdrProjectionEmitError> {
    let records = adr_records_from_decisions_json(decisions)?;
    generate_adr_index(records).map_err(AdrProjectionEmitError::Unrenderable)
}

/// The first line of `expected` that differs from `actual`, truncated for a
/// compact, actionable finding key. Returns `<len-only-diff>` when the content
/// matches line-for-line but the length differs (e.g. a trailing edit).
fn first_differing_row(expected: &str, actual: &str) -> String {
    let mut expected_lines = expected.lines();
    let mut actual_lines = actual.lines();
    let mut row = 1usize;
    loop {
        match (expected_lines.next(), actual_lines.next()) {
            (Some(e), Some(a)) if e == a => row += 1,
            (Some(e), _) => return format!("row {row}: {}", truncate(e)),
            (None, Some(a)) => return format!("row {row} (extra on disk): {}", truncate(a)),
            (None, None) => return "<len-only-diff>".to_owned(),
        }
    }
}

fn truncate(line: &str) -> String {
    const MAX: usize = 80;
    if line.chars().count() <= MAX {
        line.to_owned()
    } else {
        let head: String = line.chars().take(MAX).collect();
        format!("{head}…")
    }
}

/// Evaluate ADR-index projection parity. `records` is the source-of-truth ADR
/// record set (the authoritative decision inventory); `on_disk_markdown` /
/// `on_disk_json` are the committed projection bytes; `source_adr_ids` is the ADR
/// id set enumerated from `docs/decisions/*.md` on disk.
pub fn evaluate_adr_index_projection_parity(
    records: &[AdrDecisionRecord],
    on_disk_markdown: &str,
    on_disk_json: &str,
    source_adr_ids: &BTreeSet<String>,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    // 1) id-set staleness: the record set must exactly cover the ADR files.
    let record_ids: BTreeSet<String> = records.iter().map(|record| record.id.clone()).collect();
    for missing in source_adr_ids.difference(&record_ids) {
        findings.insert(stale(&format!(
            "{DECISIONS_JSON_PATH}#missing_adr:{missing}"
        )));
    }
    for phantom in record_ids.difference(source_adr_ids) {
        findings.insert(stale(&format!(
            "{DECISIONS_JSON_PATH}#phantom_adr:{phantom}"
        )));
    }

    // 2) byte-parity via the producer kernel.
    match validate_adr_index(records.iter().cloned(), on_disk_markdown, on_disk_json) {
        Ok(_) => {}
        Err(AdrIndexError::MarkdownDrift) => {
            findings.insert(stale(&format!(
                "{ADR_INDEX_MD_PATH}#{}",
                drift_row(records, on_disk_markdown, Projection::Markdown)
            )));
        }
        Err(AdrIndexError::JsonDrift) => {
            findings.insert(stale(&format!(
                "{DECISIONS_JSON_PATH}#{}",
                drift_row(records, on_disk_json, Projection::Json)
            )));
        }
        Err(other) => {
            findings.insert(stale(&format!("<adr-index-unrenderable>@{other:?}")));
        }
    }

    findings
}

enum Projection {
    Markdown,
    Json,
}

/// Re-render the projection and report the first differing row vs the committed
/// bytes. On the drift path the kernel already re-rendered successfully, so a
/// re-render error here degrades to a generic marker rather than panicking.
fn drift_row(records: &[AdrDecisionRecord], on_disk: &str, which: Projection) -> String {
    match generate_adr_index(records.iter().cloned()) {
        Ok(artifacts) => {
            let expected = match which {
                Projection::Markdown => &artifacts.markdown,
                Projection::Json => &artifacts.json,
            };
            first_differing_row(expected, on_disk)
        }
        Err(_) => "<drift>".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    fn record(number: u16, status: &str) -> AdrDecisionRecord {
        AdrDecisionRecord {
            number,
            id: format!("ADR-{number:04}"),
            title: format!("Fixture decision {number}"),
            status: status.to_owned(),
            owner: "council-architecture".to_owned(),
            date: "2026-01-01".to_owned(),
            path: format!("decisions/ADR-{number:04}-fixture.md"),
            supersedes: Vec::new(),
            superseded_by: Vec::new(),
            related: Vec::new(),
        }
    }

    fn records() -> Vec<AdrDecisionRecord> {
        vec![record(1, "Accepted"), record(2, "Proposed")]
    }

    fn source_ids(records: &[AdrDecisionRecord]) -> BTreeSet<String> {
        records.iter().map(|record| record.id.clone()).collect()
    }

    fn keys(findings: &BTreeSet<Finding>) -> Vec<String> {
        findings.iter().map(|f| f.key.clone()).collect()
    }

    #[test]
    fn freshly_generated_projections_are_green() {
        let records = records();
        let artifacts = generate_adr_index(records.iter().cloned()).unwrap();
        let findings = evaluate_adr_index_projection_parity(
            &records,
            &artifacts.markdown,
            &artifacts.json,
            &source_ids(&records),
        );
        assert!(findings.is_empty(), "{:?}", keys(&findings));
    }

    #[test]
    fn a_hand_edited_markdown_projection_fails_closed() {
        let records = records();
        let artifacts = generate_adr_index(records.iter().cloned()).unwrap();
        let edited = format!("{}\n\n## Hand edit\n", artifacts.markdown);
        let findings = evaluate_adr_index_projection_parity(
            &records,
            &edited,
            &artifacts.json,
            &source_ids(&records),
        );
        assert_eq!(findings.len(), 1, "{:?}", keys(&findings));
        assert!(
            keys(&findings)[0].starts_with(ADR_INDEX_MD_PATH),
            "{:?}",
            keys(&findings)
        );
    }

    #[test]
    fn a_hand_edited_json_projection_fails_closed() {
        let records = records();
        let artifacts = generate_adr_index(records.iter().cloned()).unwrap();
        let edited = artifacts
            .json
            .replace("Fixture decision 1", "Tampered title");
        let findings = evaluate_adr_index_projection_parity(
            &records,
            &artifacts.markdown,
            &edited,
            &source_ids(&records),
        );
        assert_eq!(findings.len(), 1, "{:?}", keys(&findings));
        assert!(
            keys(&findings)[0].starts_with(DECISIONS_JSON_PATH),
            "{:?}",
            keys(&findings)
        );
    }

    #[test]
    fn an_adr_file_added_without_regenerating_the_projections_is_flagged() {
        let records = records();
        let artifacts = generate_adr_index(records.iter().cloned()).unwrap();
        // A new ADR file ADR-0003 exists on disk but the projections were not
        // regenerated, so the record set does not carry it.
        let mut source = source_ids(&records);
        source.insert("ADR-0003".to_owned());
        let findings = evaluate_adr_index_projection_parity(
            &records,
            &artifacts.markdown,
            &artifacts.json,
            &source,
        );
        assert_eq!(
            keys(&findings),
            vec![format!("{DECISIONS_JSON_PATH}#missing_adr:ADR-0003")]
        );
    }

    #[test]
    fn a_projection_record_with_no_adr_file_is_flagged_as_phantom() {
        let records = records();
        let artifacts = generate_adr_index(records.iter().cloned()).unwrap();
        let mut source = source_ids(&records);
        source.remove("ADR-0002");
        let findings = evaluate_adr_index_projection_parity(
            &records,
            &artifacts.markdown,
            &artifacts.json,
            &source,
        );
        assert_eq!(
            keys(&findings),
            vec![format!("{DECISIONS_JSON_PATH}#phantom_adr:ADR-0002")]
        );
    }

    /// The emitter round-trips: rendering the record array back out of a
    /// generated `decisions.json` reproduces BOTH faces byte-for-byte.
    #[test]
    fn the_emitter_round_trips_its_own_generated_projection() {
        let artifacts = generate_adr_index(records()).unwrap();
        let decisions: Value = serde_json::from_str(&artifacts.json).unwrap();

        let regenerated = regenerate_adr_index_projection(&decisions).unwrap();

        assert_eq!(regenerated.json, artifacts.json);
        assert_eq!(regenerated.markdown, artifacts.markdown);
    }

    /// RED fixture: the nine derived surfaces are recomputed, not copied. A
    /// record appended to `decisions[]` by hand (the ONE authored edit) makes the
    /// emitter re-derive totals, next-ADR, status counts and the markdown table,
    /// so the hand-edited file is no longer byte-parity with its own regeneration.
    #[test]
    fn appending_one_record_regenerates_every_derived_surface() {
        let artifacts = generate_adr_index(records()).unwrap();
        let mut decisions: Value = serde_json::from_str(&artifacts.json).unwrap();
        decisions["decisions"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "adr": "ADR-0003",
                "number": 3,
                "title": "Fixture decision 3",
                "status": "Proposed",
                "owner": "council-architecture",
                "date": "2026-01-01",
                "path": "decisions/ADR-0003-fixture.md",
                "supersedes": [],
                "superseded_by": [],
                "related": []
            }));

        let regenerated = regenerate_adr_index_projection(&decisions).unwrap();

        // Derived _metadata: recomputed, not inherited from the hand edit.
        assert!(regenerated.json.contains("\"total_adrs\": 3"));
        assert!(regenerated.json.contains("\"next_adr\": \"ADR-0004\""));
        assert_eq!(regenerated.report.records, 3);
        assert_eq!(regenerated.report.status_counts.get("Proposed"), Some(&2));
        // Derived markdown surfaces: at-a-glance, table row, sources scanned.
        assert!(regenerated.markdown.contains("**Total ADRs:** 3"));
        assert!(regenerated.markdown.contains("**Next ADR number:** 0004"));
        assert!(regenerated.markdown.contains("| ADR-0003 |"));
        assert!(
            regenerated
                .markdown
                .contains("directory listing — 3 ADR files")
        );
        // The stale hand-edited file fails parity against its own regeneration.
        assert!(
            !evaluate_adr_index_projection_parity(
                &adr_records_from_decisions_json(&decisions).unwrap(),
                &artifacts.markdown,
                &artifacts.json,
                &BTreeSet::new(),
            )
            .is_empty()
        );
    }

    #[test]
    fn malformed_decisions_json_is_rejected_rather_than_silently_emptied() {
        assert!(matches!(
            adr_records_from_decisions_json(&serde_json::json!({})),
            Err(AdrProjectionEmitError::MalformedDecisionsJson { .. })
        ));
        assert!(matches!(
            adr_records_from_decisions_json(&serde_json::json!({
                "decisions": [{ "adr": "ADR-0001", "number": 1 }]
            })),
            Err(AdrProjectionEmitError::MalformedDecisionsJson { .. })
        ));
        // A present-but-wrong-typed relation field must not decode as "empty".
        let artifacts = generate_adr_index(records()).unwrap();
        let mut decisions: Value = serde_json::from_str(&artifacts.json).unwrap();
        decisions["decisions"][0]["related"] = Value::String("ADR-0002".to_owned());
        assert!(matches!(
            adr_records_from_decisions_json(&decisions),
            Err(AdrProjectionEmitError::MalformedDecisionsJson { .. })
        ));
    }

    #[test]
    fn every_finding_uses_the_advisory_code() {
        let records = records();
        let artifacts = generate_adr_index(records.iter().cloned()).unwrap();
        let findings = evaluate_adr_index_projection_parity(
            &records,
            "totally wrong",
            &artifacts.json,
            &source_ids(&records),
        );
        assert!(!findings.is_empty());
        for finding in &findings {
            assert_eq!(finding.code, ADR_INDEX_PROJECTION_STALE_CODE);
        }
    }
}
