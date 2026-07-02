//! Read-contract/entry-surface gate component (masterplan v2 consolidation,
//! Sub-AC 4.4.1).
//!
//! This module is the DECLARED-POLICY half of the read-contract/entry-surface
//! gate; the sibling `read_surface_resurrection` module is the ON-DISK sweep
//! half. Together the three lanes prove the bounded read path:
//!
//! - **read-path contract validation**
//!   ([`evaluate_masterplan_v2_read_contract_archives`]) — every surviving
//!   reference to a stale read path (a surface `surface_dispositions` marks
//!   `archived-with-provenance`) must be archive-only: a `read_contract`,
//!   `projection_freshness` row, or explicit `read_path_references` entry
//!   pointing at an archived path must carry
//!   `read_timing_class: provenance-archive`. Anything else makes a retired
//!   plan authority look live again and fails closed with
//!   [`READ_CONTRACT_CODE`].
//! - **bounded entry-surface enforcement**
//!   ([`evaluate_masterplan_v2_entry_surfaces`]) — the mandatory agent entry
//!   surface equals EXACTLY the `agent_entry_surface_allowlist` in
//!   `/specs/root-hub-pointers.json` (set equality, no numeric slack): every
//!   allowlisted path carries an agent-audience `entry-surface` read
//!   contract, no extra read contract claims `entry-surface`, and every
//!   allowlisted path resolves to a live root-hub entrypoint. Drift fails
//!   closed with [`ENTRY_SURFACE_CODE`].
//! - **stale/superseded surface flagging** — a root-hub entrypoint marked
//!   absorbed, retired, historical, provenance-only, or superseded (or whose
//!   `current_path` is null) may never re-enter the mandatory-read surface;
//!   the lane flags the resurrection on the allowlist row
//!   (`allowlisted_superseded_entrypoint`), the read contract
//!   (`superseded_entry_surface_read_contract`), and the root-hub entrypoint
//!   itself (`root_hub_entry_superseded`).
//!
//! The evaluators are pure `Value` → findings functions: the caller supplies
//! the masterplan and root-hub documents; there is no I/O and no scanner
//! special-case. Carve-outs live as DATA, never as evaluator branches.
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::DISPOSITION_ARCHIVED_WITH_PROVENANCE;
use crate::Finding;
use crate::non_empty_field;
use crate::normalize_read_path_for_match;

/// Validator id for the bounded entry-surface lane.
pub const ENTRY_SURFACE_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/masterplan-v2-entry-surface";
/// The root-hub fragment that owns the bounded entry-surface allowlist.
pub const ENTRY_SURFACE_ALLOWLIST_REF: &str =
    "/specs/root-hub-pointers.json#agent_entry_surface_allowlist";
/// The blocking violation code the entry-surface lane emits.
pub const ENTRY_SURFACE_CODE: &str = "masterplan_entry_surface_invalid";
/// The blocking violation code the read-path contract lane emits (shared with
/// the on-disk `read_surface_resurrection` sweep: both police the same
/// read-path contract).
pub const READ_CONTRACT_CODE: &str = "masterplan_read_contract_invalid";

const READ_CONTRACT_ARCHIVED_TIMING_CLASS: &str = "provenance-archive";
const READ_CONTRACT_ENTRY_TIMING_CLASS: &str = "entry-surface";

/// Whether the masterplan declares any read-path contract surface at all —
/// the trigger for running the read-contract archive lane in
/// `evaluate_keyed`.
pub(crate) fn masterplan_read_contract_gate_present(masterplan: &Value) -> bool {
    masterplan.get("masterplan_v2").is_some_and(|v2| {
        v2.get("read_contracts").is_some()
            || v2.get("projection_freshness").is_some()
            || v2.get("read_path_references").is_some()
    })
}

/// Evaluate the masterplan v2 read-contract archive guard.
///
/// Stale read paths are the surfaces explicitly archived with provenance by the
/// consolidation. Any surviving reference to such a path must be archive-only:
/// a `read_contract`, projection freshness row, or explicit read-path reference
/// that points at an archived path must carry `read_timing_class:
/// provenance-archive`.
pub fn evaluate_masterplan_v2_read_contract_archives(masterplan: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2").and_then(Value::as_object) else {
        findings.insert(Finding::new(READ_CONTRACT_CODE, "<missing-masterplan_v2>"));
        return findings;
    };

    let archived_paths = archived_read_paths(v2.get("surface_dispositions"));
    if archived_paths.is_empty() {
        return findings;
    }

    evaluate_archived_read_contract_rows(v2.get("read_contracts"), &archived_paths, &mut findings);
    evaluate_archived_projection_freshness_rows(
        v2.get("projection_freshness"),
        &archived_paths,
        &mut findings,
    );
    evaluate_archived_explicit_read_path_references(
        v2.get("read_path_references"),
        &archived_paths,
        &mut findings,
    );

    findings
}

/// Evaluate the masterplan v2 bounded entry-surface contract.
///
/// The root hub owns the small allowlist of artifacts agents may treat as
/// mandatory entry surfaces. The masterplan read contracts must mark exactly
/// that same set as `entry-surface`, and no root-hub entrypoint that is marked
/// absorbed, retired, historical, provenance-only, or superseded may re-enter
/// the mandatory-read surface.
pub fn evaluate_masterplan_v2_entry_surfaces(
    masterplan: &Value,
    root_hub: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    let Some(v2) = masterplan.get("masterplan_v2").and_then(Value::as_object) else {
        findings.insert(Finding::new(ENTRY_SURFACE_CODE, "<missing-masterplan_v2>"));
        return findings;
    };

    let Some(allowlist) = root_hub.get("agent_entry_surface_allowlist") else {
        findings.insert(Finding::new(
            ENTRY_SURFACE_CODE,
            "root_hub.agent_entry_surface_allowlist",
        ));
        return findings;
    };
    if !allowlist.is_object() {
        findings.insert(Finding::new(
            ENTRY_SURFACE_CODE,
            "root_hub.agent_entry_surface_allowlist",
        ));
        return findings;
    }

    if non_empty_field(allowlist, "read_timing_class") != Some(READ_CONTRACT_ENTRY_TIMING_CLASS) {
        findings.insert(Finding::new(
            ENTRY_SURFACE_CODE,
            "root_hub.agent_entry_surface_allowlist.read_timing_class",
        ));
    }
    if non_empty_field(allowlist, "validator") != Some(ENTRY_SURFACE_VALIDATOR) {
        findings.insert(Finding::new(
            ENTRY_SURFACE_CODE,
            "root_hub.agent_entry_surface_allowlist.validator",
        ));
    }
    if non_empty_field(allowlist, "source_of_truth") != Some(ENTRY_SURFACE_ALLOWLIST_REF) {
        findings.insert(Finding::new(
            ENTRY_SURFACE_CODE,
            "root_hub.agent_entry_surface_allowlist.source_of_truth",
        ));
    }

    let allowed_paths = collect_entry_surface_path_array(
        allowlist.get("paths"),
        "root_hub.agent_entry_surface_allowlist.paths",
        &mut findings,
    );
    let superseded_paths = collect_entry_surface_path_array(
        allowlist.get("superseded_entrypoints"),
        "root_hub.agent_entry_surface_allowlist.superseded_entrypoints",
        &mut findings,
    );
    let actual_paths =
        collect_masterplan_entry_surface_paths(v2.get("read_contracts"), &mut findings);

    match root_hub.get("entry_points").and_then(Value::as_object) {
        Some(entry_points) => {
            for (normalized_path, display_path) in &allowed_paths {
                let mut found = false;
                let mut stale = false;
                for entry in entry_points.values() {
                    if root_hub_entry_current_path_normalized(entry).as_deref()
                        == Some(normalized_path.as_str())
                    {
                        found = true;
                        stale |= root_hub_entrypoint_is_superseded(entry);
                    }
                }
                if !found {
                    findings.insert(Finding::new(
                        ENTRY_SURFACE_CODE,
                        &format!("{display_path}.root_hub_entry_points"),
                    ));
                }
                if stale {
                    findings.insert(Finding::new(
                        ENTRY_SURFACE_CODE,
                        &format!("{display_path}.root_hub_entry_superseded"),
                    ));
                }
            }
        }
        None => {
            findings.insert(Finding::new(ENTRY_SURFACE_CODE, "root_hub.entry_points"));
        }
    }

    for (normalized_path, display_path) in &allowed_paths {
        if !actual_paths.contains_key(normalized_path) {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("{display_path}.missing_entry_surface_read_contract"),
            ));
        }
        if superseded_paths.contains_key(normalized_path) {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("{display_path}.allowlisted_superseded_entrypoint"),
            ));
        }
    }

    for (normalized_path, display_path) in &actual_paths {
        if !allowed_paths.contains_key(normalized_path) {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("{display_path}.unexpected_entry_surface_read_contract"),
            ));
        }
        if superseded_paths.contains_key(normalized_path) {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("{display_path}.superseded_entry_surface_read_contract"),
            ));
        }
    }

    findings
}

fn collect_masterplan_entry_surface_paths(
    read_contracts: Option<&Value>,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, String> {
    let mut paths = BTreeMap::new();
    let Some(read_contracts) = read_contracts.and_then(Value::as_array) else {
        findings.insert(Finding::new(
            ENTRY_SURFACE_CODE,
            "masterplan_v2.read_contracts",
        ));
        return paths;
    };

    for (index, contract) in read_contracts.iter().enumerate() {
        if non_empty_field(contract, "read_timing_class") != Some(READ_CONTRACT_ENTRY_TIMING_CLASS)
        {
            continue;
        }
        let Some(path) = non_empty_field(contract, "path") else {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("masterplan_v2.read_contracts[{index}].path"),
            ));
            continue;
        };
        if !read_contract_audience_contains(contract, "agents") {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("{path}.read_contract.audience.agents"),
            ));
        }
        let normalized = normalize_read_path_for_match(path);
        if normalized.is_empty() {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("masterplan_v2.read_contracts[{index}].path"),
            ));
            continue;
        }
        if paths.insert(normalized, path.to_owned()).is_some() {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("{path}.duplicate_entry_surface_read_contract"),
            ));
        }
    }

    if paths.is_empty() {
        findings.insert(Finding::new(
            ENTRY_SURFACE_CODE,
            "<empty-masterplan-entry-surface-read-contracts>",
        ));
    }

    paths
}

fn collect_entry_surface_path_array(
    value: Option<&Value>,
    field_key: &str,
    findings: &mut BTreeSet<Finding>,
) -> BTreeMap<String, String> {
    let mut paths = BTreeMap::new();
    let Some(values) = value.and_then(Value::as_array) else {
        findings.insert(Finding::new(ENTRY_SURFACE_CODE, field_key));
        return paths;
    };

    if values.is_empty() {
        findings.insert(Finding::new(
            ENTRY_SURFACE_CODE,
            &format!("{field_key}.empty"),
        ));
        return paths;
    }

    for (index, value) in values.iter().enumerate() {
        let Some(path) = value
            .as_str()
            .map(str::trim)
            .filter(|path| !path.is_empty())
        else {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("{field_key}[{index}]"),
            ));
            continue;
        };
        let normalized = normalize_read_path_for_match(path);
        if normalized.is_empty() {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("{field_key}[{index}]"),
            ));
            continue;
        }
        if paths.insert(normalized, path.to_owned()).is_some() {
            findings.insert(Finding::new(
                ENTRY_SURFACE_CODE,
                &format!("{path}.duplicate_entry_surface_allowlist_path"),
            ));
        }
    }

    paths
}

fn root_hub_entry_current_path_normalized(entry: &Value) -> Option<String> {
    non_empty_field(entry, "current_path").map(normalize_read_path_for_match)
}

pub(crate) fn root_hub_entrypoint_is_superseded(entry: &Value) -> bool {
    entry.get("current_path").is_some_and(Value::is_null)
        || root_hub_status_field_has_stale_marker(entry, "authority_status")
        || root_hub_status_field_has_stale_marker(entry, "current_path_status")
        || root_hub_status_field_has_stale_marker(entry, "migration_phase")
        || root_hub_status_field_has_stale_marker(entry, "status")
}

fn root_hub_status_field_has_stale_marker(entry: &Value, field: &str) -> bool {
    const STALE_ENTRYPOINT_MARKERS: [&str; 5] = [
        "superseded",
        "retired",
        "provenance",
        "historical",
        "absorbed",
    ];
    non_empty_field(entry, field).is_some_and(|value| {
        let value = value.to_ascii_lowercase();
        STALE_ENTRYPOINT_MARKERS
            .iter()
            .any(|marker| value.contains(marker))
    })
}

fn read_contract_audience_contains(contract: &Value, expected: &str) -> bool {
    contract
        .get("audience")
        .and_then(Value::as_array)
        .is_some_and(|audiences| {
            audiences
                .iter()
                .any(|audience| audience.as_str() == Some(expected))
        })
}

fn archived_read_paths(surfaces: Option<&Value>) -> Vec<String> {
    let Some(surfaces) = surfaces.and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut paths = BTreeSet::new();
    for surface in surfaces {
        if non_empty_field(surface, "disposition") != Some(DISPOSITION_ARCHIVED_WITH_PROVENANCE) {
            continue;
        }
        if let Some(path) = non_empty_field(surface, "path") {
            paths.insert(path.to_owned());
        }
    }

    paths.into_iter().collect()
}

fn evaluate_archived_read_contract_rows(
    read_contracts: Option<&Value>,
    archived_paths: &[String],
    findings: &mut BTreeSet<Finding>,
) {
    let Some(read_contracts) = read_contracts.and_then(Value::as_array) else {
        return;
    };

    for contract in read_contracts {
        let Some(path) = non_empty_field(contract, "path") else {
            continue;
        };
        if read_path_is_archived(path, archived_paths)
            && non_empty_field(contract, "read_timing_class")
                != Some(READ_CONTRACT_ARCHIVED_TIMING_CLASS)
        {
            findings.insert(Finding::new(
                READ_CONTRACT_CODE,
                &format!("{path}.read_contract.read_timing_class"),
            ));
        }
    }
}

fn evaluate_archived_projection_freshness_rows(
    freshness: Option<&Value>,
    archived_paths: &[String],
    findings: &mut BTreeSet<Finding>,
) {
    let Some(rows) = freshness
        .and_then(|value| value.get("projections"))
        .and_then(Value::as_array)
    else {
        return;
    };

    for row in rows {
        let Some(path) = non_empty_field(row, "path") else {
            continue;
        };
        if read_path_is_archived(path, archived_paths)
            && non_empty_field(row, "read_timing_class")
                != Some(READ_CONTRACT_ARCHIVED_TIMING_CLASS)
        {
            findings.insert(Finding::new(
                READ_CONTRACT_CODE,
                &format!("{path}.projection_freshness.read_timing_class"),
            ));
        }
    }
}

fn evaluate_archived_explicit_read_path_references(
    references: Option<&Value>,
    archived_paths: &[String],
    findings: &mut BTreeSet<Finding>,
) {
    let Some(references) = references.and_then(Value::as_array) else {
        return;
    };

    for (index, reference) in references.iter().enumerate() {
        let Some(path) = non_empty_field(reference, "path")
            .or_else(|| non_empty_field(reference, "target_path"))
        else {
            continue;
        };
        let read_timing_class = non_empty_field(reference, "read_timing_class")
            .or_else(|| non_empty_field(reference, "reference_timing_class"));

        if read_path_is_archived(path, archived_paths)
            && read_timing_class != Some(READ_CONTRACT_ARCHIVED_TIMING_CLASS)
        {
            findings.insert(Finding::new(
                READ_CONTRACT_CODE,
                &format!("{path}.read_path_references[{index}].read_timing_class"),
            ));
        }
    }
}

fn read_path_is_archived(path: &str, archived_paths: &[String]) -> bool {
    archived_paths
        .iter()
        .any(|archived_path| archived_read_path_matches(path, archived_path))
}

fn archived_read_path_matches(path: &str, archived_path: &str) -> bool {
    let path = normalize_read_path_for_match(path);
    let archived_path = normalize_read_path_for_match(archived_path);
    if path == archived_path {
        return true;
    }

    archived_path.strip_suffix("/**").is_some_and(|prefix| {
        path == prefix
            || path
                .strip_prefix(prefix)
                .is_some_and(|suffix| suffix.starts_with('/'))
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn finding(code: &str, key: &str) -> Finding {
        Finding::new(code, key)
    }

    /// A masterplan whose archived surfaces are referenced ONLY through
    /// provenance-archive read contracts, projection rows, and read-path
    /// references — the green baseline for the read-contract lane.
    fn archive_clean_masterplan() -> Value {
        json!({
            "masterplan_v2": {
                "surface_dispositions": [
                    {"path": "docs/ROADMAP.md", "disposition": "archived-with-provenance"},
                    {"path": ".omc/ultragoal/**", "disposition": "archived-with-provenance"},
                    {"path": "docs/MASTERPLAN.md", "disposition": "generated-projection"}
                ],
                "read_contracts": [
                    {
                        "path": "/specs/masterplan.json",
                        "audience": ["agents", "humans"],
                        "read_timing_class": "entry-surface"
                    },
                    {
                        "path": "docs/MASTERPLAN.md",
                        "audience": ["humans"],
                        "read_timing_class": "on-demand"
                    },
                    {
                        "path": "docs/ROADMAP.md",
                        "audience": ["humans"],
                        "read_timing_class": "provenance-archive"
                    }
                ],
                "projection_freshness": {
                    "projections": [
                        {"path": "docs/ROADMAP.md", "read_timing_class": "provenance-archive"}
                    ]
                },
                "read_path_references": [
                    {
                        "path": ".omc/ultragoal/goals.json",
                        "read_timing_class": "provenance-archive",
                        "source": "consolidation provenance ledger"
                    }
                ]
            }
        })
    }

    /// A masterplan whose entry-surface read contracts exactly match the
    /// bounded root-hub allowlist — the green baseline for the entry lane.
    fn entry_surface_masterplan() -> Value {
        json!({
            "masterplan_v2": {
                "read_contracts": [
                    {
                        "path": "/specs/masterplan.json",
                        "audience": ["agents", "humans"],
                        "read_timing_class": "entry-surface"
                    },
                    {
                        "path": "/specs/root-hub-pointers.json",
                        "audience": ["agents"],
                        "read_timing_class": "entry-surface"
                    },
                    {
                        "path": "docs/MASTERPLAN.md",
                        "audience": ["humans"],
                        "read_timing_class": "on-demand"
                    }
                ]
            }
        })
    }

    fn bounded_root_hub() -> Value {
        json!({
            "agent_entry_surface_allowlist": {
                "read_timing_class": "entry-surface",
                "validator": ENTRY_SURFACE_VALIDATOR,
                "source_of_truth": ENTRY_SURFACE_ALLOWLIST_REF,
                "paths": [
                    "/specs/masterplan.json",
                    "/specs/root-hub-pointers.json"
                ],
                "superseded_entrypoints": [
                    "/specs/master-plan-sequencing.json",
                    "docs/ROADMAP.md"
                ]
            },
            "entry_points": {
                "masterplan": {
                    "current_path": "/specs/masterplan.json",
                    "authority_status": "live-canonical"
                },
                "root_hub": {
                    "current_path": "/specs/root-hub-pointers.json",
                    "authority_status": "live-canonical"
                },
                "sequencing": {
                    "current_path": "/specs/master-plan-sequencing.json",
                    "authority_status": "superseded-by-masterplan-v2"
                }
            }
        })
    }

    // ------------------------------------------------------------------
    // read-path contract validation lane — pass behavior
    // ------------------------------------------------------------------

    #[test]
    fn read_contract_lane_passes_archive_only_references() {
        let findings = evaluate_masterplan_v2_read_contract_archives(&archive_clean_masterplan());
        assert!(
            findings.is_empty(),
            "archive-only references to archived paths must be green: {findings:?}"
        );
    }

    #[test]
    fn read_contract_lane_passes_when_nothing_is_archived() {
        let findings = evaluate_masterplan_v2_read_contract_archives(&json!({
            "masterplan_v2": {
                "surface_dispositions": [
                    {"path": "docs/MASTERPLAN.md", "disposition": "generated-projection"}
                ],
                "read_contracts": [
                    {"path": "docs/MASTERPLAN.md", "read_timing_class": "on-demand"}
                ]
            }
        }));
        assert!(
            findings.is_empty(),
            "no archived surfaces means no archive obligations: {findings:?}"
        );
    }

    // ------------------------------------------------------------------
    // read-path contract validation lane — fail behavior
    // ------------------------------------------------------------------

    #[test]
    fn read_contract_lane_fails_closed_without_masterplan_v2() {
        let findings = evaluate_masterplan_v2_read_contract_archives(&json!({}));
        assert_eq!(
            findings,
            BTreeSet::from([finding(READ_CONTRACT_CODE, "<missing-masterplan_v2>")]),
        );
    }

    #[test]
    fn read_contract_lane_flags_non_archive_read_contract_on_archived_path() {
        let mut masterplan = archive_clean_masterplan();
        masterplan["masterplan_v2"]["read_contracts"][2]["read_timing_class"] = json!("on-demand");

        let findings = evaluate_masterplan_v2_read_contract_archives(&masterplan);
        assert!(findings.contains(&finding(
            READ_CONTRACT_CODE,
            "docs/ROADMAP.md.read_contract.read_timing_class"
        )));
    }

    #[test]
    fn read_contract_lane_flags_non_archive_projection_row_on_archived_path() {
        let mut masterplan = archive_clean_masterplan();
        masterplan["masterplan_v2"]["projection_freshness"]["projections"][0]["read_timing_class"] =
            json!("entry-surface");

        let findings = evaluate_masterplan_v2_read_contract_archives(&masterplan);
        assert!(findings.contains(&finding(
            READ_CONTRACT_CODE,
            "docs/ROADMAP.md.projection_freshness.read_timing_class"
        )));
    }

    #[test]
    fn read_contract_lane_flags_non_archive_read_path_reference_variants() {
        let mut masterplan = archive_clean_masterplan();
        masterplan["masterplan_v2"]["read_path_references"] = json!([
            {
                // `target_path` + `reference_timing_class` field aliases must
                // be honored — a rename cannot dodge the archive contract.
                "target_path": ".omc/ultragoal/goals.json",
                "reference_timing_class": "on-demand",
                "source": "stale runtime pointer"
            }
        ]);

        let findings = evaluate_masterplan_v2_read_contract_archives(&masterplan);
        assert!(findings.contains(&finding(
            READ_CONTRACT_CODE,
            ".omc/ultragoal/goals.json.read_path_references[0].read_timing_class"
        )));
    }

    #[test]
    fn read_contract_lane_flags_missing_timing_class_as_non_archive() {
        let mut masterplan = archive_clean_masterplan();
        let contract = &mut masterplan["masterplan_v2"]["read_contracts"][2];
        contract
            .as_object_mut()
            .unwrap()
            .remove("read_timing_class");

        let findings = evaluate_masterplan_v2_read_contract_archives(&masterplan);
        assert!(findings.contains(&finding(
            READ_CONTRACT_CODE,
            "docs/ROADMAP.md.read_contract.read_timing_class"
        )));
    }

    // ------------------------------------------------------------------
    // read-path contract validation lane — path matching semantics
    // ------------------------------------------------------------------

    #[test]
    fn archived_glob_disposition_covers_nested_paths_and_the_root() {
        assert!(archived_read_path_matches(
            ".omc/ultragoal/goals.json",
            ".omc/ultragoal/**"
        ));
        assert!(archived_read_path_matches(
            ".omc/ultragoal",
            ".omc/ultragoal/**"
        ));
        assert!(!archived_read_path_matches(
            ".omc/ultragoal-live/goals.json",
            ".omc/ultragoal/**"
        ));
    }

    #[test]
    fn archived_path_matching_normalizes_fragments_slashes_and_dot_prefixes() {
        assert!(archived_read_path_matches(
            "/docs/ROADMAP.md#wave-3",
            "docs/ROADMAP.md"
        ));
        assert!(archived_read_path_matches(
            "./docs/ROADMAP.md",
            "/docs/ROADMAP.md/"
        ));
        assert!(!archived_read_path_matches(
            "docs/ROADMAP.md.bak",
            "docs/ROADMAP.md"
        ));
    }

    // ------------------------------------------------------------------
    // bounded entry-surface lane — pass behavior
    // ------------------------------------------------------------------

    #[test]
    fn entry_surface_lane_passes_exact_allowlist_equality() {
        let findings =
            evaluate_masterplan_v2_entry_surfaces(&entry_surface_masterplan(), &bounded_root_hub());
        assert!(
            findings.is_empty(),
            "exact entry-surface set equality must be green: {findings:?}"
        );
    }

    // ------------------------------------------------------------------
    // bounded entry-surface lane — fail behavior
    // ------------------------------------------------------------------

    #[test]
    fn entry_surface_lane_fails_closed_without_masterplan_v2() {
        let findings = evaluate_masterplan_v2_entry_surfaces(&json!({}), &bounded_root_hub());
        assert_eq!(
            findings,
            BTreeSet::from([finding(ENTRY_SURFACE_CODE, "<missing-masterplan_v2>")]),
        );
    }

    #[test]
    fn entry_surface_lane_fails_closed_without_allowlist() {
        let findings =
            evaluate_masterplan_v2_entry_surfaces(&entry_surface_masterplan(), &json!({}));
        assert_eq!(
            findings,
            BTreeSet::from([finding(
                ENTRY_SURFACE_CODE,
                "root_hub.agent_entry_surface_allowlist"
            )]),
        );
    }

    #[test]
    fn entry_surface_lane_rejects_drifted_allowlist_metadata() {
        let mut root_hub = bounded_root_hub();
        root_hub["agent_entry_surface_allowlist"]["read_timing_class"] = json!("on-demand");
        root_hub["agent_entry_surface_allowlist"]["validator"] = json!("some-other-gate");
        root_hub["agent_entry_surface_allowlist"]["source_of_truth"] = json!("docs/ROADMAP.md");

        let findings =
            evaluate_masterplan_v2_entry_surfaces(&entry_surface_masterplan(), &root_hub);
        for key in [
            "root_hub.agent_entry_surface_allowlist.read_timing_class",
            "root_hub.agent_entry_surface_allowlist.validator",
            "root_hub.agent_entry_surface_allowlist.source_of_truth",
        ] {
            assert!(
                findings.contains(&finding(ENTRY_SURFACE_CODE, key)),
                "missing {key} in {findings:?}"
            );
        }
    }

    #[test]
    fn entry_surface_lane_rejects_unbounded_extra_entry_contract() {
        let mut masterplan = entry_surface_masterplan();
        masterplan["masterplan_v2"]["read_contracts"][2]["read_timing_class"] =
            json!("entry-surface");
        masterplan["masterplan_v2"]["read_contracts"][2]["audience"] = json!(["agents"]);

        let findings = evaluate_masterplan_v2_entry_surfaces(&masterplan, &bounded_root_hub());
        assert_eq!(
            findings,
            BTreeSet::from([finding(
                ENTRY_SURFACE_CODE,
                "docs/MASTERPLAN.md.unexpected_entry_surface_read_contract"
            )]),
        );
    }

    #[test]
    fn entry_surface_lane_rejects_allowlisted_path_without_entry_contract() {
        let mut masterplan = entry_surface_masterplan();
        masterplan["masterplan_v2"]["read_contracts"][1]["read_timing_class"] = json!("on-demand");

        let findings = evaluate_masterplan_v2_entry_surfaces(&masterplan, &bounded_root_hub());
        assert_eq!(
            findings,
            BTreeSet::from([finding(
                ENTRY_SURFACE_CODE,
                "/specs/root-hub-pointers.json.missing_entry_surface_read_contract"
            )]),
        );
    }

    #[test]
    fn entry_surface_lane_rejects_empty_entry_surface() {
        let masterplan = json!({
            "masterplan_v2": {
                "read_contracts": [
                    {
                        "path": "docs/MASTERPLAN.md",
                        "audience": ["humans"],
                        "read_timing_class": "on-demand"
                    }
                ]
            }
        });

        let findings = evaluate_masterplan_v2_entry_surfaces(&masterplan, &bounded_root_hub());
        assert!(findings.contains(&finding(
            ENTRY_SURFACE_CODE,
            "<empty-masterplan-entry-surface-read-contracts>"
        )));
    }

    #[test]
    fn entry_surface_lane_rejects_missing_agent_audience() {
        let mut masterplan = entry_surface_masterplan();
        masterplan["masterplan_v2"]["read_contracts"][1]["audience"] = json!(["humans"]);

        let findings = evaluate_masterplan_v2_entry_surfaces(&masterplan, &bounded_root_hub());
        assert!(findings.contains(&finding(
            ENTRY_SURFACE_CODE,
            "/specs/root-hub-pointers.json.read_contract.audience.agents"
        )));
    }

    #[test]
    fn entry_surface_lane_rejects_duplicate_entry_contracts_and_allowlist_rows() {
        let mut masterplan = entry_surface_masterplan();
        masterplan["masterplan_v2"]["read_contracts"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                // Same surface spelled differently: normalization must
                // collapse it into a duplicate, not a second entry.
                "path": "specs/masterplan.json",
                "audience": ["agents"],
                "read_timing_class": "entry-surface"
            }));

        let mut root_hub = bounded_root_hub();
        root_hub["agent_entry_surface_allowlist"]["paths"]
            .as_array_mut()
            .unwrap()
            .push(json!("./specs/masterplan.json"));

        let findings = evaluate_masterplan_v2_entry_surfaces(&masterplan, &root_hub);
        assert!(findings.contains(&finding(
            ENTRY_SURFACE_CODE,
            "specs/masterplan.json.duplicate_entry_surface_read_contract"
        )));
        assert!(findings.contains(&finding(
            ENTRY_SURFACE_CODE,
            "./specs/masterplan.json.duplicate_entry_surface_allowlist_path"
        )));
    }

    #[test]
    fn entry_surface_lane_rejects_missing_or_incomplete_root_hub_entry_points() {
        let mut root_hub = bounded_root_hub();
        root_hub.as_object_mut().unwrap().remove("entry_points");
        let findings =
            evaluate_masterplan_v2_entry_surfaces(&entry_surface_masterplan(), &root_hub);
        assert!(findings.contains(&finding(ENTRY_SURFACE_CODE, "root_hub.entry_points")));

        let mut root_hub = bounded_root_hub();
        root_hub["entry_points"]
            .as_object_mut()
            .unwrap()
            .remove("root_hub");
        let findings =
            evaluate_masterplan_v2_entry_surfaces(&entry_surface_masterplan(), &root_hub);
        assert!(findings.contains(&finding(
            ENTRY_SURFACE_CODE,
            "/specs/root-hub-pointers.json.root_hub_entry_points"
        )));
    }

    // ------------------------------------------------------------------
    // stale/superseded surface flagging behavior
    // ------------------------------------------------------------------

    #[test]
    fn superseded_entrypoint_resurrection_is_flagged_on_every_surface() {
        // Resurrect the retired sequencing spec into the mandatory entry
        // surface: allowlist it AND give it an entry-surface read contract
        // while the root hub still marks it superseded.
        let mut masterplan = entry_surface_masterplan();
        masterplan["masterplan_v2"]["read_contracts"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "path": "/specs/master-plan-sequencing.json",
                "audience": ["agents"],
                "read_timing_class": "entry-surface"
            }));

        let mut root_hub = bounded_root_hub();
        root_hub["agent_entry_surface_allowlist"]["paths"]
            .as_array_mut()
            .unwrap()
            .push(json!("/specs/master-plan-sequencing.json"));

        let findings = evaluate_masterplan_v2_entry_surfaces(&masterplan, &root_hub);
        for key in [
            "/specs/master-plan-sequencing.json.root_hub_entry_superseded",
            "/specs/master-plan-sequencing.json.allowlisted_superseded_entrypoint",
            "/specs/master-plan-sequencing.json.superseded_entry_surface_read_contract",
        ] {
            assert!(
                findings.contains(&finding(ENTRY_SURFACE_CODE, key)),
                "missing {key} in {findings:?}"
            );
        }
    }

    #[test]
    fn stale_entrypoint_markers_cover_the_full_marker_vocabulary() {
        for marker in [
            "superseded",
            "RETIRED",
            "provenance-archive",
            "historical-record",
            "absorbed-into-masterplan-v2",
        ] {
            let entry = json!({"current_path": "x.json", "authority_status": marker});
            assert!(
                root_hub_entrypoint_is_superseded(&entry),
                "marker {marker} must flag the entrypoint as superseded"
            );
        }

        let null_path = json!({"current_path": null});
        assert!(root_hub_entrypoint_is_superseded(&null_path));

        let live = json!({"current_path": "x.json", "authority_status": "live-canonical"});
        assert!(!root_hub_entrypoint_is_superseded(&live));
    }

    #[test]
    fn stale_markers_are_honored_on_every_status_field() {
        for field in [
            "authority_status",
            "current_path_status",
            "migration_phase",
            "status",
        ] {
            let entry = json!({"current_path": "x.json", field: "retired"});
            assert!(
                root_hub_entrypoint_is_superseded(&entry),
                "stale marker in {field} must flag the entrypoint"
            );
        }
    }

    // ------------------------------------------------------------------
    // gate-presence trigger
    // ------------------------------------------------------------------

    #[test]
    fn read_contract_gate_presence_requires_a_declared_read_surface() {
        assert!(!masterplan_read_contract_gate_present(&json!({})));
        assert!(!masterplan_read_contract_gate_present(
            &json!({"masterplan_v2": {}})
        ));
        for field in [
            "read_contracts",
            "projection_freshness",
            "read_path_references",
        ] {
            assert!(
                masterplan_read_contract_gate_present(&json!({"masterplan_v2": {field: []}})),
                "{field} alone must arm the read-contract lane"
            );
        }
    }
}
