//! Read-surface resurrection lane of the read-contract/entry-surface gate
//! (masterplan v2 consolidation, Sub-AC 4.4).
//!
//! `evaluate_masterplan_v2_history_only_retirement` (lib.rs) proves the
//! DECLARED read policy: a `retired-git-history-only` path may not appear in
//! read contracts, projection rows, or read-path references.
//! `evaluate_masterplan_v2_entry_surfaces` (lib.rs)
//! proves the BOUNDED entry surface: entry-surface read contracts equal
//! exactly the root-hub allowlist and never revive a superseded entrypoint.
//! This module closes the remaining hole — the ON-DISK surface itself. A
//! superseded plan authority (docs/MASTERPLAN.md, docs/ROADMAP.md, the
//! retired planning specs) can be "resurrected" by stripping its archive
//! front-matter/markers and refilling it with live-looking plan content while
//! every declared contract still reads clean. The sweep fails closed on
//!
//! - **resurrected live authority** — a governed on-disk surface that no
//!   longer declares itself non-live (`live_plan_authority: false` for
//!   front-matter docs; an absorbed/archived/provenance/superseded status
//!   marker or explicit `live_plan_authority: false` for JSON specs);
//! - **missing canonical pointer** — a governed on-disk surface that no
//!   longer points its readers at `/specs/masterplan.json`
//!   (`canonical_authority:`/`absorbed_by:` line or field);
//! - **history-only surface still present** — a surface dispositioned
//!   `retired-git-history-only` still has candidate-HEAD bytes;
//! - **unaudited opaque data** — a governed data file (e.g. a `.jsonl`
//!   provenance ledger) whose disposition row's `surface_class` does not
//!   classify it as provenance/archive;
//! - **sweep-coverage drift** — a governed surface the corpus never scanned
//!   (`read_surface_corpus_uncovered`), a corpus row for an ungoverned path
//!   (`read_surface_corpus_unexpected`), a duplicate row, a row without a
//!   usable `exists` fact, or a row with no readable facts at all. A surface
//!   the sweep cannot prove archived is never admitted as archived.
//!
//! The evaluator is pure: the caller assembles the `read_surface_corpus`
//! from the tree (the complete tracked-path membership from the committed
//! scm-facts face plus on-disk front-matter/JSON facts) — the evaluator itself
//! does no I/O. The tracked-path list is also used to expand retired repo
//! globs, so `.omc/**`-class surfaces cannot evade candidate-HEAD absence.
//! The governed universe is derived MECHANICALLY from
//! `masterplan_v2.surface_dispositions` (every repo-file row dispositioned
//! `absorbed`, `retired-git-history-only`, or `generated-projection`), never
//! from a hand list. History-only surfaces pass only with `exists: false`.
//! Carve-outs live as DATA, never as evaluator branches.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde_json::Value;

use crate::Finding;
use crate::non_empty_field;
use crate::normalize_read_path_for_match;

/// Validator id for the read-surface resurrection sweep lane.
pub const READ_SURFACE_RESURRECTION_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/masterplan-v2-read-surface-resurrection";

/// The blocking violation code this lane emits (shared with the declared
/// read-contract archive lane: both police the same read-path contract).
pub const RESURRECTION_CODE: &str = "masterplan_read_contract_invalid";

const DISPOSITION_ABSORBED: &str = "absorbed";
const DISPOSITION_RETIRED_GIT_HISTORY_ONLY: &str = "retired-git-history-only";
const DISPOSITION_GENERATED_PROJECTION: &str = "generated-projection";

/// Status markers that mechanically declare a surface non-live. Matches the
/// stale-entrypoint marker vocabulary used by the entry-surface lane plus the
/// explicit archive tokens the consolidation writes into retired specs.
const NON_LIVE_STATUS_MARKERS: [&str; 6] = [
    "not-live-plan-authority",
    "superseded",
    "retired",
    "provenance",
    "historical",
    "absorbed",
];

fn resurrection(key: &str) -> Finding {
    Finding::new(RESURRECTION_CODE, key)
}

/// A governed on-disk read surface derived from `surface_dispositions`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernedSurface {
    /// The disposition row's `path` as written (violation-key display form).
    display_path: String,
    /// `absorbed` | `retired-git-history-only` | `generated-projection`.
    disposition: String,
    /// The disposition row's `surface_class` (empty when absent).
    surface_class: String,
}

/// Evaluate the masterplan v2 read-surface resurrection sweep.
///
/// `corpus` shape (assembled by the caller from the tree):
/// ```jsonc
/// {
///   "tracked_paths": ["specs/masterplan.json", "docs/MASTERPLAN.md"],
///   "surfaces": [
///     { "path": "docs/ROADMAP.md", "exists": true,
///       "front_matter": "doc_class: RoadmapArchive\n…" },          // *.md
///     { "path": "/specs/master-plan-sequencing.json", "exists": true,
///       "document": { "_metadata": { "status": "absorbed-…" } } }, // *.json
///     { "path": ".omc/ultragoal/friction-ledger.jsonl", "exists": true,
///       "opaque_data": true },                                     // data
///     { "path": ".omc/ultragoal/goals.json", "exists": false }     // deleted
///   ]
/// }
/// ```
pub fn evaluate_masterplan_read_surface_resurrections(
    masterplan: &Value,
    corpus: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2").and_then(Value::as_object) else {
        findings.insert(resurrection("<missing-masterplan_v2>"));
        return findings;
    };

    let governed = governed_read_surfaces(v2.get("surface_dispositions"), &mut findings);
    if governed.is_empty() {
        return findings;
    }
    let retired_repo_globs =
        history_only_retired_repo_glob_prefixes(v2.get("surface_dispositions"));
    evaluate_retired_repo_glob_absence(
        corpus.get("tracked_paths"),
        &retired_repo_globs,
        &mut findings,
    );

    let Some(rows) = corpus.get("surfaces").and_then(Value::as_array) else {
        findings.insert(resurrection("<missing-read-surface-corpus>"));
        return findings;
    };

    let mut rows_by_path: BTreeMap<String, &Value> = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        let Some(path) = non_empty_field(row, "path") else {
            findings.insert(resurrection(&format!("read_surface_corpus[{index}].path")));
            continue;
        };
        let normalized = normalize_read_path_for_match(path);
        if normalized.is_empty() {
            findings.insert(resurrection(&format!("read_surface_corpus[{index}].path")));
            continue;
        }
        if rows_by_path.insert(normalized.clone(), row).is_some() {
            findings.insert(resurrection(&format!(
                "{path}.duplicate_read_surface_corpus_row"
            )));
        }
        if !governed.contains_key(&normalized) {
            findings.insert(resurrection(&format!(
                "{path}.read_surface_corpus_unexpected"
            )));
        }
    }

    for (normalized, surface) in &governed {
        let Some(row) = rows_by_path.get(normalized) else {
            findings.insert(resurrection(&format!(
                "{}.read_surface_corpus_uncovered",
                surface.display_path
            )));
            continue;
        };
        evaluate_surface_row(row, surface, &mut findings);
    }

    findings
}

/// Derive the governed on-disk read-surface universe from
/// `surface_dispositions`: every row dispositioned `absorbed`,
/// `retired-git-history-only`, or `generated-projection` whose path is a
/// repo file path (no `#fragment`, no glob, not a home-directory store).
fn governed_read_surfaces(
    surfaces: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, GovernedSurface> {
    let mut governed = BTreeMap::new();
    let Some(surfaces) = surfaces.and_then(Value::as_array) else {
        findings.insert(resurrection("<missing-surface_dispositions>"));
        return governed;
    };

    for surface in surfaces {
        let Some(disposition) = non_empty_field(surface, "disposition") else {
            continue;
        };
        if disposition != DISPOSITION_ABSORBED
            && disposition != DISPOSITION_RETIRED_GIT_HISTORY_ONLY
            && disposition != DISPOSITION_GENERATED_PROJECTION
        {
            continue;
        }
        let Some(path) = non_empty_field(surface, "path") else {
            continue;
        };
        if !is_repo_file_path(path) {
            continue;
        }
        let normalized = normalize_read_path_for_match(path);
        if normalized.is_empty() {
            continue;
        }
        governed.insert(
            normalized,
            GovernedSurface {
                display_path: path.to_owned(),
                disposition: disposition.to_owned(),
                surface_class: non_empty_field(surface, "surface_class")
                    .unwrap_or_default()
                    .to_owned(),
            },
        );
    }

    governed
}

fn is_repo_file_path(path: &str) -> bool {
    !path.contains('#') && !path.contains('*') && !path.trim_start().starts_with('~')
}

fn history_only_retired_repo_glob_prefixes(surfaces: Option<&Value>) -> BTreeMap<String, String> {
    let mut prefixes = BTreeMap::new();
    let Some(surfaces) = surfaces.and_then(Value::as_array) else {
        return prefixes;
    };

    for surface in surfaces {
        if non_empty_field(surface, "disposition") != Some(DISPOSITION_RETIRED_GIT_HISTORY_ONLY) {
            continue;
        }
        let Some(path) = non_empty_field(surface, "path") else {
            continue;
        };
        if path.contains('#') || path.trim_start().starts_with('~') {
            continue;
        }
        let Some(prefix) = path.strip_suffix("/**") else {
            continue;
        };
        let normalized = normalize_read_path_for_match(prefix);
        if !normalized.is_empty() {
            prefixes.insert(format!("{normalized}/"), path.to_owned());
        }
    }

    prefixes
}

fn evaluate_retired_repo_glob_absence(
    tracked_paths: Option<&Value>,
    retired_repo_globs: &BTreeMap<String, String>,
    findings: &mut BTreeSet<Finding>,
) {
    if retired_repo_globs.is_empty() {
        return;
    }
    let Some(tracked_paths) = tracked_paths.and_then(Value::as_array) else {
        findings.insert(resurrection("<missing-read-surface-tracked-paths>"));
        return;
    };

    for (index, path) in tracked_paths.iter().enumerate() {
        let Some(path) = path.as_str().filter(|path| !path.trim().is_empty()) else {
            findings.insert(resurrection(&format!(
                "read_surface_tracked_paths[{index}]"
            )));
            continue;
        };
        let normalized = normalize_read_path_for_match(path);
        if retired_repo_globs
            .keys()
            .any(|prefix| normalized.starts_with(prefix))
        {
            findings.insert(resurrection(&format!(
                "{path}.retired_git_history_only_surface_present"
            )));
        }
    }
}

fn evaluate_surface_row(row: &Value, surface: &GovernedSurface, findings: &mut BTreeSet<Finding>) {
    let path = surface.display_path.as_str();
    let exists = match row.get("exists").and_then(Value::as_bool) {
        Some(exists) => exists,
        None => {
            findings.insert(resurrection(&format!("{path}.read_surface_corpus_exists")));
            return;
        }
    };

    if surface.disposition == DISPOSITION_RETIRED_GIT_HISTORY_ONLY {
        if exists {
            findings.insert(resurrection(&format!(
                "{path}.retired_git_history_only_surface_present"
            )));
        }
        return;
    }

    if !exists {
        return;
    }

    if let Some(front_matter) = non_empty_field(row, "front_matter") {
        evaluate_front_matter_facts(front_matter, surface, findings);
    } else if let Some(document) = row.get("document").filter(|value| value.is_object()) {
        evaluate_document_facts(document, surface, findings);
    } else if row.get("opaque_data").and_then(Value::as_bool) == Some(true) {
        let surface_class = surface.surface_class.to_ascii_lowercase();
        if !surface_class.contains("provenance") && !surface_class.contains("archive") {
            findings.insert(resurrection(&format!(
                "{path}.resurrected.opaque_surface_class"
            )));
        }
    } else {
        findings.insert(resurrection(&format!("{path}.read_surface_facts")));
    }
}

/// Front-matter facts for a Markdown surface: exact trimmed-line markers.
fn evaluate_front_matter_facts(
    front_matter: &str,
    surface: &GovernedSurface,
    findings: &mut BTreeSet<Finding>,
) {
    let path = surface.display_path.as_str();
    let lines: Vec<&str> = front_matter.lines().map(str::trim).collect();

    let declared_not_live = lines.contains(&"live_plan_authority: false");
    if !declared_not_live {
        findings.insert(resurrection(&format!(
            "{path}.resurrected.live_plan_authority"
        )));
    }

    let canonical_pointer = lines.iter().any(|line| {
        line.strip_prefix("canonical_authority:")
            .or_else(|| line.strip_prefix("absorbed_by:"))
            .is_some_and(|target| target.contains("specs/masterplan.json"))
    });
    if !canonical_pointer {
        findings.insert(resurrection(&format!(
            "{path}.resurrected.canonical_authority"
        )));
    }
}

/// Document facts for a JSON surface: `_meta`/`_metadata`/top-level markers.
fn evaluate_document_facts(
    document: &Value,
    surface: &GovernedSurface,
    findings: &mut BTreeSet<Finding>,
) {
    let path = surface.display_path.as_str();

    let declared_not_live = document_status_has_non_live_marker(document)
        || document_declares_live_plan_authority_false(document);
    if !declared_not_live {
        findings.insert(resurrection(&format!(
            "{path}.resurrected.live_plan_authority"
        )));
    }

    let canonical_pointer = ["absorbed_by", "superseded_by", "canonical_authority"]
        .iter()
        .any(|field| {
            document_scopes(document).any(|scope| {
                non_empty_field(scope, field)
                    .is_some_and(|target| target.contains("specs/masterplan.json"))
            })
        });
    if !canonical_pointer {
        findings.insert(resurrection(&format!(
            "{path}.resurrected.canonical_authority"
        )));
    }
}

/// The scopes a JSON spec may declare archive markers in: the document root
/// plus its `_meta`/`_metadata` header objects.
fn document_scopes(document: &Value) -> impl Iterator<Item = &Value> {
    std::iter::once(document).chain(
        ["_meta", "_metadata"]
            .into_iter()
            .filter_map(|field| document.get(field)),
    )
}

fn document_status_has_non_live_marker(document: &Value) -> bool {
    document_scopes(document).any(|scope| {
        non_empty_field(scope, "status").is_some_and(|status| {
            let status = status.to_ascii_lowercase();
            NON_LIVE_STATUS_MARKERS
                .iter()
                .any(|marker| status.contains(marker))
        })
    })
}

fn document_declares_live_plan_authority_false(document: &Value) -> bool {
    document_scopes(document)
        .any(|scope| scope.get("live_plan_authority").and_then(Value::as_bool) == Some(false))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn minimal_masterplan() -> Value {
        json!({
            "masterplan_v2": {
                "surface_dispositions": [
                    {
                        "path": "/specs/masterplan.json",
                        "surface_class": "canonical-authority",
                        "disposition": "canonical-authority"
                    },
                    {
                        "path": "/specs/masterplan.json#v1-legacy-fragments",
                        "surface_class": "absorbed-legacy-plan-authority",
                        "disposition": "absorbed"
                    },
                    {
                        "path": "/specs/master-plan-sequencing.json",
                        "surface_class": "absorbed-provenance",
                        "disposition": "absorbed"
                    },
                    {
                        "path": "docs/MASTERPLAN.md",
                        "surface_class": "generated-projection",
                        "disposition": "generated-projection"
                    },
                    {
                        "path": "docs/ROADMAP.md",
                        "surface_class": "retired-authority",
                        "disposition": "retired-git-history-only"
                    },
                    {
                        "path": ".omc/ultragoal/friction-ledger.jsonl",
                        "surface_class": "retired-harness-store",
                        "disposition": "retired-git-history-only"
                    },
                    {
                        "path": ".omc/ultragoal/goals.json",
                        "surface_class": "missing-legacy-surface",
                        "disposition": "retired-git-history-only"
                    },
                    {
                        "path": ".omc/**",
                        "surface_class": "retired-harness-store",
                        "disposition": "retired-git-history-only"
                    },
                    {
                        "path": "~/.omx/**",
                        "surface_class": "retired-external-harness-store",
                        "disposition": "retired-git-history-only"
                    }
                ]
            }
        })
    }

    fn archived_roadmap_front_matter() -> &'static str {
        "doc_class: RoadmapArchive\n\
         canonical_authority: /specs/masterplan.json\n\
         live_plan_authority: false\n\
         read_contract:\n  \
           read_timing_class: provenance-archive\n"
    }

    fn clean_corpus() -> Value {
        json!({
            "tracked_paths": [],
            "surfaces": [
                {
                    "path": "/specs/master-plan-sequencing.json",
                    "exists": true,
                    "document": {
                        "_metadata": {
                            "status": "absorbed-provenance-not-live-plan-authority",
                            "absorbed_by": "/specs/masterplan.json#masterplan_v2"
                        }
                    }
                },
                {
                    "path": "docs/MASTERPLAN.md",
                    "exists": true,
                    "front_matter": "doc_class: MasterPlan\n\
                        canonical_authority: /specs/masterplan.json\n\
                        live_plan_authority: false\n\
                        read_contract:\n  read_timing_class: on-demand\n"
                },
                {
                    "path": "docs/ROADMAP.md",
                    "exists": false
                },
                {
                    "path": ".omc/ultragoal/friction-ledger.jsonl",
                    "exists": false
                },
                {
                    "path": ".omc/ultragoal/goals.json",
                    "exists": false
                }
            ]
        })
    }

    #[test]
    fn clean_history_only_retirement_sweep_is_green() {
        let findings =
            evaluate_masterplan_read_surface_resurrections(&minimal_masterplan(), &clean_corpus());
        assert!(
            findings.is_empty(),
            "history-only retired surfaces absent from candidate HEAD must pass: {findings:?}"
        );
    }

    #[test]
    fn resurrected_generated_markdown_authority_fails_closed_per_missing_marker() {
        let mut corpus = clean_corpus();
        for row in corpus["surfaces"].as_array_mut().expect("surfaces") {
            if row["path"] == "docs/MASTERPLAN.md" {
                row["front_matter"] = json!("doc_class: MasterPlan\nstatus: Accepted\n");
            }
        }
        let findings =
            evaluate_masterplan_read_surface_resurrections(&minimal_masterplan(), &corpus);
        for key in [
            "docs/MASTERPLAN.md.resurrected.live_plan_authority",
            "docs/MASTERPLAN.md.resurrected.canonical_authority",
        ] {
            assert!(
                findings.contains(&Finding::new(RESURRECTION_CODE, key)),
                "missing {key} in {findings:?}"
            );
        }
    }

    #[test]
    fn history_only_retirement_requires_candidate_head_absence() {
        let masterplan = json!({
            "masterplan_v2": {
                "surface_dispositions": [{
                    "path": "docs/ROADMAP.md",
                    "surface_class": "retired-authority",
                    "disposition": "retired-git-history-only"
                }]
            }
        });
        let corpus = json!({
            "surfaces": [{
                "path": "docs/ROADMAP.md",
                "exists": true,
                "front_matter": archived_roadmap_front_matter()
            }]
        });

        let findings = evaluate_masterplan_read_surface_resurrections(&masterplan, &corpus);
        assert!(findings.contains(&Finding::new(
            RESURRECTION_CODE,
            "docs/ROADMAP.md.retired_git_history_only_surface_present"
        )));
    }

    #[test]
    fn candidate_file_under_history_only_glob_fails_closed() {
        let mut corpus = clean_corpus();
        corpus["tracked_paths"] = json!([".omc/state/team/mailbox.json"]);

        let findings =
            evaluate_masterplan_read_surface_resurrections(&minimal_masterplan(), &corpus);
        assert!(findings.contains(&Finding::new(
            RESURRECTION_CODE,
            ".omc/state/team/mailbox.json.retired_git_history_only_surface_present"
        )));
    }

    #[test]
    fn history_only_absence_requires_no_content_facts() {
        let findings =
            evaluate_masterplan_read_surface_resurrections(&minimal_masterplan(), &clean_corpus());
        assert!(
            findings.is_empty(),
            "absent history-only rows need no readable content facts: {findings:?}"
        );
    }

    #[test]
    fn resurrected_json_spec_fails_closed() {
        let mut corpus = clean_corpus();
        for row in corpus["surfaces"].as_array_mut().expect("surfaces") {
            if row["path"] == "/specs/master-plan-sequencing.json" {
                // Live-looking spec: absorbed markers stripped.
                row["document"] = json!({ "_metadata": { "status": "Accepted" } });
            }
        }
        let findings =
            evaluate_masterplan_read_surface_resurrections(&minimal_masterplan(), &corpus);
        for key in [
            "/specs/master-plan-sequencing.json.resurrected.live_plan_authority",
            "/specs/master-plan-sequencing.json.resurrected.canonical_authority",
        ] {
            assert!(
                findings.contains(&Finding::new(RESURRECTION_CODE, key)),
                "missing {key} in {findings:?}"
            );
        }
    }

    #[test]
    fn uncovered_governed_surface_fails_closed() {
        let mut corpus = clean_corpus();
        corpus["surfaces"]
            .as_array_mut()
            .expect("surfaces")
            .retain(|row| row["path"] != "docs/ROADMAP.md");
        let findings =
            evaluate_masterplan_read_surface_resurrections(&minimal_masterplan(), &corpus);
        assert!(findings.contains(&Finding::new(
            RESURRECTION_CODE,
            "docs/ROADMAP.md.read_surface_corpus_uncovered"
        )));
    }

    #[test]
    fn ungoverned_corpus_row_fails_closed() {
        let mut corpus = clean_corpus();
        corpus["surfaces"]
            .as_array_mut()
            .expect("surfaces")
            .push(json!({ "path": "docs/standards/anti-patterns.md", "exists": true }));
        let findings =
            evaluate_masterplan_read_surface_resurrections(&minimal_masterplan(), &corpus);
        assert!(findings.contains(&Finding::new(
            RESURRECTION_CODE,
            "docs/standards/anti-patterns.md.read_surface_corpus_unexpected"
        )));
    }

    #[test]
    fn missing_or_malformed_corpus_fails_closed() {
        let findings = evaluate_masterplan_read_surface_resurrections(
            &minimal_masterplan(),
            &json!({ "surfaces": "not-an-array" }),
        );
        assert!(findings.contains(&Finding::new(
            RESURRECTION_CODE,
            "<missing-read-surface-corpus>"
        )));

        let findings = evaluate_masterplan_read_surface_resurrections(
            &json!({ "masterplan_v2": { } }),
            &clean_corpus(),
        );
        assert!(findings.contains(&Finding::new(
            RESURRECTION_CODE,
            "<missing-surface_dispositions>"
        )));

        let findings = evaluate_masterplan_read_surface_resurrections(
            &json!({ "not_masterplan": true }),
            &clean_corpus(),
        );
        assert!(findings.contains(&Finding::new(RESURRECTION_CODE, "<missing-masterplan_v2>")));
    }

    #[test]
    fn row_without_exists_or_facts_fails_closed() {
        let mut corpus = clean_corpus();
        for row in corpus["surfaces"].as_array_mut().expect("surfaces") {
            if row["path"] == "docs/ROADMAP.md" {
                row.as_object_mut().expect("row").remove("exists");
            }
            if row["path"] == "docs/MASTERPLAN.md" {
                row.as_object_mut().expect("row").remove("front_matter");
            }
        }
        let findings =
            evaluate_masterplan_read_surface_resurrections(&minimal_masterplan(), &corpus);
        assert!(findings.contains(&Finding::new(
            RESURRECTION_CODE,
            "docs/ROADMAP.md.read_surface_corpus_exists"
        )));
        assert!(findings.contains(&Finding::new(
            RESURRECTION_CODE,
            "docs/MASTERPLAN.md.read_surface_facts"
        )));
    }

    #[test]
    fn absorbed_opaque_data_requires_provenance_surface_class() {
        let mut masterplan = minimal_masterplan();
        for surface in masterplan["masterplan_v2"]["surface_dispositions"]
            .as_array_mut()
            .expect("surface_dispositions")
        {
            if surface["path"] == ".omc/ultragoal/friction-ledger.jsonl" {
                surface["surface_class"] = json!("operational-data");
                surface["disposition"] = json!("absorbed");
            }
        }
        let mut corpus = clean_corpus();
        for row in corpus["surfaces"].as_array_mut().expect("surfaces") {
            if row["path"] == ".omc/ultragoal/friction-ledger.jsonl" {
                row["exists"] = json!(true);
                row["opaque_data"] = json!(true);
            }
        }
        let findings = evaluate_masterplan_read_surface_resurrections(&masterplan, &corpus);
        assert!(findings.contains(&Finding::new(
            RESURRECTION_CODE,
            ".omc/ultragoal/friction-ledger.jsonl.resurrected.opaque_surface_class"
        )));
    }

    #[test]
    fn glob_fragment_and_home_paths_are_not_governed() {
        // `.omc/**`, `~/.omx/**`, and `#v1-legacy-fragments` rows must not
        // demand corpus coverage: only repo file paths are sweepable.
        let findings =
            evaluate_masterplan_read_surface_resurrections(&minimal_masterplan(), &clean_corpus());
        assert!(
            !findings
                .iter()
                .any(|finding| finding.key.contains("read_surface_corpus_uncovered")),
            "non-file disposition rows must not demand sweep coverage: {findings:?}"
        );
    }
}
