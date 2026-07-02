//! Read-path contract-coverage lane of the read-contract/entry-surface gate
//! (masterplan v2 consolidation, Sub-AC 5.1).
//!
//! The sibling lanes prove the DECLARED read policy
//! (`evaluate_masterplan_v2_read_contract_archives`), the BOUNDED entry
//! surface (`evaluate_masterplan_v2_entry_surfaces`), and the ON-DISK archive
//! markers of superseded surfaces
//! (`evaluate_masterplan_read_surface_resurrections`). This lane closes the
//! coverage hole those three leave open: a SURVIVING doc/JSON on the repo
//! read paths that simply never declares any read contract at all. Every
//! read-path member must carry a machine-checkable contract header/field —
//! audience, read-timing class (entry-surface / on-demand /
//! provenance-archive), and freshness rule — or the gate fails closed with
//! [`READ_PATH_CONTRACT_CODE`].
//!
//! The read-path UNIVERSE is derived MECHANICALLY, never from a hand list:
//! `/specs/root-hub-pointers.json#read_path_read_contracts.root_markdown`
//! plus every live (non-superseded) `entry_points` row whose `current_path`
//! resolves to a repo file. Contract facts come from the artifact itself —
//! a front-matter `read_contract:` block for Markdown, a `read_contract`
//! object at the document root or inside `_meta`/`_metadata` for JSON — or
//! from a central `/specs/masterplan.json#masterplan_v2.read_contracts` row
//! (the single-writer declaration used for generated projections that must
//! never be hand-edited). Non-doc/JSON members (data ledgers, YAML catalog
//! rows, directories) are opaque data surfaces: enumerated by the corpus,
//! contract-exempt by the policy's `opaque_data_rule`.
//!
//! The lane also refuses ENTRY-SURFACE INFLATION: no embedded or central
//! contract may claim `read_timing_class: entry-surface` unless its path is
//! in `agent_entry_surface_allowlist.paths` — the bounded entry surface can
//! never grow through a stray artifact header.
//!
//! The evaluator is pure `Value` → findings: the caller assembles the
//! `read_path_contract_corpus` from the tree; there is no I/O and no scanner
//! special-case. Carve-outs live as DATA (the root-hub policy block), never
//! as evaluator branches. ADR-0083 Tier-3: production code carries no
//! unwrap/expect/panic.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::Finding;
use crate::non_empty_field;
use crate::normalize_read_path_for_match;
use crate::read_contract_entry_surface::root_hub_entrypoint_is_superseded;

/// Validator id for the read-path contract-coverage lane.
pub const READ_PATH_CONTRACT_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/masterplan-v2-read-path-contract-coverage";

/// The root-hub fragment that owns the read-path contract policy (scope,
/// vocabularies, root markdown surfaces).
pub const READ_PATH_CONTRACT_POLICY_REF: &str =
    "/specs/root-hub-pointers.json#read_path_read_contracts";

/// The blocking violation code this lane emits.
pub const READ_PATH_CONTRACT_CODE: &str = "read_path_read_contract_missing";

/// The three read-timing classes of the Seed read-contract ontology. The
/// class vocabulary is CONTRACT, not carve-out, so it is pinned here and in
/// the policy block; the two must agree.
const READ_TIMING_CLASSES: [&str; 3] = ["entry-surface", "on-demand", "provenance-archive"];

/// The audience vocabulary of the read-contract ontology.
const AUDIENCE_VOCABULARY: [&str; 3] = ["agents", "cloud-ci-gates", "humans"];

const ENTRY_SURFACE_CLASS: &str = "entry-surface";

fn missing_contract(key: &str) -> Finding {
    Finding::new(READ_PATH_CONTRACT_CODE, key)
}

/// Derive the mechanical read-path universe from the root hub: the policy
/// block's `root_markdown` surfaces plus every live (non-superseded)
/// `entry_points` row whose `current_path` normalizes to a repo file path
/// (no glob, no home-directory store). Best-effort: a missing policy block
/// or entry-points map yields the smaller derivable set; the evaluator
/// itself fails closed on the missing policy.
pub fn read_path_contract_universe(root_hub: &Value) -> BTreeSet<String> {
    let mut universe = BTreeSet::new();

    if let Some(paths) = root_hub
        .get("read_path_read_contracts")
        .and_then(|policy| policy.get("root_markdown"))
        .and_then(Value::as_array)
    {
        for path in paths.iter().filter_map(Value::as_str) {
            let normalized = normalize_read_path_for_match(path);
            if is_repo_file_read_path(&normalized) {
                universe.insert(normalized);
            }
        }
    }

    if let Some(entry_points) = root_hub.get("entry_points").and_then(Value::as_object) {
        for entry in entry_points.values() {
            if root_hub_entrypoint_is_superseded(entry) {
                continue;
            }
            let Some(current_path) = non_empty_field(entry, "current_path") else {
                continue;
            };
            let normalized = normalize_read_path_for_match(current_path);
            if is_repo_file_read_path(&normalized) {
                universe.insert(normalized);
            }
        }
    }

    universe
}

fn is_repo_file_read_path(path: &str) -> bool {
    !path.is_empty() && !path.contains('*') && !path.starts_with('~')
}

/// Evaluate the masterplan v2 read-path contract-coverage sweep.
///
/// `corpus` shape (assembled by the caller from the tree):
/// ```jsonc
/// {
///   "surfaces": [
///     { "path": "docs/AGENTS.md", "exists": true,
///       "front_matter": "read_contract:\n  audience:\n…" },          // *.md
///     { "path": "specs/decision-principles.json", "exists": true,
///       "document": { "read_contract": { … } } },                    // *.json
///     { "path": "registry/fixuptasks.jsonl", "exists": true,
///       "opaque_data": true }                                        // data
///   ]
/// }
/// ```
pub fn evaluate_read_path_contract_coverage(
    masterplan: &Value,
    root_hub: &Value,
    corpus: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if root_hub
        .get("read_path_read_contracts")
        .and_then(Value::as_object)
        .is_none()
    {
        findings.insert(missing_contract("<read-path-policy>#missing"));
        return findings;
    }

    let universe = read_path_contract_universe(root_hub);
    if universe.is_empty() {
        findings.insert(missing_contract("<read-path-universe>#empty"));
        return findings;
    }

    let allowlisted_entry_paths = entry_surface_allowlist_paths(root_hub);
    let central_contracts = central_read_contracts(masterplan);

    let mut rows_by_path: BTreeMap<String, &Value> = BTreeMap::new();
    if let Some(surfaces) = corpus.get("surfaces").and_then(Value::as_array) {
        for row in surfaces {
            let Some(path) = non_empty_field(row, "path") else {
                findings.insert(missing_contract("<corpus-row>#missing-path"));
                continue;
            };
            let normalized = normalize_read_path_for_match(path);
            if !universe.contains(&normalized) {
                findings.insert(missing_contract(&format!("{normalized}#corpus-unexpected")));
                continue;
            }
            if rows_by_path.insert(normalized.clone(), row).is_some() {
                findings.insert(missing_contract(&format!("{normalized}#corpus-duplicate")));
            }
        }
    }

    for path in &universe {
        let Some(row) = rows_by_path.get(path) else {
            findings.insert(missing_contract(&format!("{path}#corpus-uncovered")));
            continue;
        };
        evaluate_read_path_row(
            path,
            row,
            &central_contracts,
            &allowlisted_entry_paths,
            &mut findings,
        );
    }

    findings
}

fn evaluate_read_path_row(
    path: &str,
    row: &Value,
    central_contracts: &BTreeMap<String, &Value>,
    allowlisted_entry_paths: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    if row.get("exists").and_then(Value::as_bool) != Some(true) {
        findings.insert(missing_contract(&format!("{path}#surface-missing")));
        return;
    }

    if path.ends_with(".md") {
        let Some(front_matter) = row.get("front_matter").and_then(Value::as_str) else {
            findings.insert(missing_contract(&format!("{path}#missing-facts")));
            return;
        };
        evaluate_markdown_contract(path, front_matter, allowlisted_entry_paths, findings);
        return;
    }

    if path.ends_with(".json") {
        let Some(document) = row.get("document") else {
            findings.insert(missing_contract(&format!("{path}#missing-facts")));
            return;
        };
        if let Some(contract) = embedded_json_contract(document) {
            evaluate_contract_object(path, contract, allowlisted_entry_paths, findings);
            return;
        }
        if let Some(contract) = central_contracts.get(path) {
            evaluate_contract_object(path, contract, allowlisted_entry_paths, findings);
            return;
        }
        findings.insert(missing_contract(&format!("{path}#missing-read-contract")));
    }

    // Opaque data surface (neither doc nor JSON): enumerated by the corpus,
    // contract-exempt per the policy's opaque_data_rule.
}

/// The scopes a JSON artifact may declare its read contract in: the document
/// root plus its `_meta`/`_metadata` header objects.
fn embedded_json_contract(document: &Value) -> Option<&Value> {
    [
        Some(document),
        document.get("_meta"),
        document.get("_metadata"),
    ]
    .into_iter()
    .flatten()
    .filter_map(|scope| scope.get("read_contract"))
    .find(|contract| contract.is_object())
}

fn evaluate_contract_object(
    path: &str,
    contract: &Value,
    allowlisted_entry_paths: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    match contract.get("read_timing_class").and_then(Value::as_str) {
        Some(class) if READ_TIMING_CLASSES.contains(&class) => {
            if class == ENTRY_SURFACE_CLASS && !allowlisted_entry_paths.contains(path) {
                findings.insert(missing_contract(&format!(
                    "{path}#entry-surface-not-allowlisted"
                )));
            }
        }
        _ => {
            findings.insert(missing_contract(&format!(
                "{path}#invalid-read-timing-class"
            )));
        }
    }

    match contract.get("audience").and_then(Value::as_array) {
        Some(audiences) if !audiences.is_empty() => {
            let all_known = audiences.iter().all(|audience| {
                audience
                    .as_str()
                    .is_some_and(|value| AUDIENCE_VOCABULARY.contains(&value))
            });
            if !all_known {
                findings.insert(missing_contract(&format!("{path}#invalid-audience")));
            }
        }
        _ => {
            findings.insert(missing_contract(&format!("{path}#missing-audience")));
        }
    }

    if contract
        .get("freshness_rule")
        .and_then(Value::as_str)
        .is_none_or(|rule| rule.trim().is_empty())
    {
        findings.insert(missing_contract(&format!("{path}#missing-freshness-rule")));
    }
}

/// Front-matter facts for a Markdown surface: exact trimmed-line markers, the
/// same mechanical style the resurrection sweep uses.
fn evaluate_markdown_contract(
    path: &str,
    front_matter: &str,
    allowlisted_entry_paths: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    let lines: Vec<&str> = front_matter.lines().map(str::trim).collect();

    if !lines.contains(&"read_contract:") {
        findings.insert(missing_contract(&format!("{path}#missing-read-contract")));
        return;
    }

    let timing_class = lines.iter().find_map(|line| {
        line.strip_prefix("read_timing_class:")
            .map(|value| value.trim().trim_matches('"').trim_matches('\''))
    });
    match timing_class {
        Some(class) if READ_TIMING_CLASSES.contains(&class) => {
            if class == ENTRY_SURFACE_CLASS && !allowlisted_entry_paths.contains(path) {
                findings.insert(missing_contract(&format!(
                    "{path}#entry-surface-not-allowlisted"
                )));
            }
        }
        _ => {
            findings.insert(missing_contract(&format!(
                "{path}#invalid-read-timing-class"
            )));
        }
    }

    if !lines
        .iter()
        .any(|line| *line == "audience:" || line.starts_with("audience:"))
    {
        findings.insert(missing_contract(&format!("{path}#missing-audience")));
    }

    if !lines.iter().any(|line| {
        line.strip_prefix("freshness_rule:")
            .is_some_and(|value| !value.trim().is_empty())
    }) {
        findings.insert(missing_contract(&format!("{path}#missing-freshness-rule")));
    }
}

fn entry_surface_allowlist_paths(root_hub: &Value) -> BTreeSet<String> {
    root_hub
        .get("agent_entry_surface_allowlist")
        .and_then(|allowlist| allowlist.get("paths"))
        .and_then(Value::as_array)
        .map(|paths| {
            paths
                .iter()
                .filter_map(Value::as_str)
                .map(normalize_read_path_for_match)
                .collect()
        })
        .unwrap_or_default()
}

/// Central single-writer read contracts declared by
/// `/specs/masterplan.json#masterplan_v2.read_contracts`, keyed by
/// normalized path. Used as the equivalent declaration for artifacts whose
/// bytes are mechanically generated and must never be hand-edited.
fn central_read_contracts(masterplan: &Value) -> BTreeMap<String, &Value> {
    let mut contracts = BTreeMap::new();
    let Some(rows) = masterplan
        .get("masterplan_v2")
        .and_then(|v2| v2.get("read_contracts"))
        .and_then(Value::as_array)
    else {
        return contracts;
    };
    for row in rows {
        if let Some(path) = non_empty_field(row, "path") {
            contracts.insert(normalize_read_path_for_match(path), row);
        }
    }
    contracts
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use serde_json::{Value, json};

    use super::*;

    fn policy_root_hub() -> Value {
        json!({
            "agent_entry_surface_allowlist": {
                "paths": ["/specs/masterplan.json"]
            },
            "read_path_read_contracts": {
                "validator": READ_PATH_CONTRACT_VALIDATOR,
                "violation_code": READ_PATH_CONTRACT_CODE,
                "root_markdown": ["AGENTS.md"],
                "read_timing_classes": ["entry-surface", "on-demand", "provenance-archive"],
                "audience_vocabulary": ["agents", "cloud-ci-gates", "humans"]
            },
            "entry_points": {
                "masterplan": {
                    "current_path": "/specs/masterplan.json",
                    "kind": "spec"
                },
                "decision_principles": {
                    "current_path": "/specs/decision-principles.json",
                    "kind": "spec"
                },
                "legacy_sequencing": {
                    "current_path": "/specs/master-plan-sequencing.json",
                    "authority_status": "provenance-archive-not-live-plan-authority"
                },
                "loop_patterns": {
                    "current_path": "registry/loop-recovery-patterns",
                    "kind": "registry"
                },
                "fragment_pointer": {
                    "current_path": "/specs/decision-principles.json#rules",
                    "kind": "spec"
                },
                "glob_store": {
                    "current_path": ".omc/**"
                },
                "home_store": {
                    "current_path": "~/.hermes/kanban/boards/oyatie/kanban.db"
                }
            }
        })
    }

    fn masterplan_with_central_row() -> Value {
        json!({
            "masterplan_v2": {
                "read_contracts": [
                    {
                        "path": "/specs/masterplan.json",
                        "audience": ["agents", "humans", "cloud-ci-gates"],
                        "read_timing_class": "entry-surface",
                        "freshness_rule": "single-writer canonical authority"
                    }
                ]
            }
        })
    }

    fn contract_json() -> Value {
        json!({
            "read_contract": {
                "audience": ["agents", "cloud-ci-gates"],
                "read_timing_class": "on-demand",
                "freshness_rule": "On-demand read; live at this path."
            }
        })
    }

    fn contract_front_matter() -> &'static str {
        "doc_class: Guidance\nread_contract:\n  audience:\n    - agents\n    - humans\n  read_timing_class: on-demand\n  freshness_rule: \"On-demand read; live at this path.\"\n"
    }

    fn green_corpus() -> Value {
        json!({
            "surfaces": [
                { "path": "AGENTS.md", "exists": true, "front_matter": contract_front_matter() },
                { "path": "/specs/masterplan.json", "exists": true, "document": {} },
                { "path": "/specs/decision-principles.json", "exists": true, "document": contract_json() },
                { "path": "registry/loop-recovery-patterns", "exists": true, "opaque_data": true }
            ]
        })
    }

    fn finding_keys(findings: &std::collections::BTreeSet<Finding>) -> Vec<String> {
        findings.iter().map(|finding| finding.key.clone()).collect()
    }

    #[test]
    fn universe_derives_mechanically_from_live_entrypoints_and_root_markdown() {
        let universe = read_path_contract_universe(&policy_root_hub());
        let expected: Vec<&str> = vec![
            "AGENTS.md",
            "registry/loop-recovery-patterns",
            "specs/decision-principles.json",
            "specs/masterplan.json",
        ];
        assert_eq!(
            universe.iter().map(String::as_str).collect::<Vec<_>>(),
            expected,
            "superseded, glob, and home-store entrypoints must stay out; fragments normalize onto the file"
        );
    }

    #[test]
    fn green_corpus_passes_with_embedded_front_matter_json_and_central_contracts() {
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &policy_root_hub(),
            &green_corpus(),
        );
        assert!(findings.is_empty(), "green corpus must pass: {findings:?}");
    }

    #[test]
    fn missing_policy_block_fails_closed() {
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &json!({ "entry_points": {} }),
            &green_corpus(),
        );
        assert_eq!(finding_keys(&findings), vec!["<read-path-policy>#missing"]);
    }

    #[test]
    fn markdown_without_read_contract_block_fails_closed() {
        let mut corpus = green_corpus();
        corpus["surfaces"][0]["front_matter"] = Value::String("doc_class: Guidance\n".to_owned());
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &policy_root_hub(),
            &corpus,
        );
        assert_eq!(
            finding_keys(&findings),
            vec!["AGENTS.md#missing-read-contract"]
        );
    }

    #[test]
    fn markdown_with_invalid_class_or_missing_fields_fails_closed() {
        let mut corpus = green_corpus();
        corpus["surfaces"][0]["front_matter"] =
            Value::String("read_contract:\n  read_timing_class: whenever\n".to_owned());
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &policy_root_hub(),
            &corpus,
        );
        assert_eq!(
            finding_keys(&findings),
            vec![
                "AGENTS.md#invalid-read-timing-class",
                "AGENTS.md#missing-audience",
                "AGENTS.md#missing-freshness-rule"
            ]
        );
    }

    #[test]
    fn json_without_contract_and_without_central_row_fails_closed() {
        let mut corpus = green_corpus();
        corpus["surfaces"][2]["document"] = json!({ "_meta": { "status": "Accepted" } });
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &policy_root_hub(),
            &corpus,
        );
        assert_eq!(
            finding_keys(&findings),
            vec!["specs/decision-principles.json#missing-read-contract"]
        );
    }

    #[test]
    fn json_contract_in_meta_scope_is_accepted() {
        let mut corpus = green_corpus();
        corpus["surfaces"][2]["document"] =
            json!({ "_meta": { "read_contract": contract_json()["read_contract"] } });
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &policy_root_hub(),
            &corpus,
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn json_contract_field_defects_fail_closed_per_field() {
        let mut corpus = green_corpus();
        corpus["surfaces"][2]["document"] = json!({
            "read_contract": {
                "audience": ["agents", "bots"],
                "read_timing_class": "sometimes",
                "freshness_rule": "   "
            }
        });
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &policy_root_hub(),
            &corpus,
        );
        assert_eq!(
            finding_keys(&findings),
            vec![
                "specs/decision-principles.json#invalid-audience",
                "specs/decision-principles.json#invalid-read-timing-class",
                "specs/decision-principles.json#missing-freshness-rule"
            ]
        );
    }

    #[test]
    fn entry_surface_inflation_outside_allowlist_fails_closed() {
        let mut corpus = green_corpus();
        corpus["surfaces"][2]["document"] = json!({
            "read_contract": {
                "audience": ["agents"],
                "read_timing_class": "entry-surface",
                "freshness_rule": "self-promoted entry surface"
            }
        });
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &policy_root_hub(),
            &corpus,
        );
        assert_eq!(
            finding_keys(&findings),
            vec!["specs/decision-principles.json#entry-surface-not-allowlisted"]
        );
    }

    #[test]
    fn central_allowlisted_entry_surface_contract_is_accepted() {
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &policy_root_hub(),
            &green_corpus(),
        );
        assert!(
            findings.is_empty(),
            "masterplan.json entry-surface central row is allowlisted: {findings:?}"
        );
    }

    #[test]
    fn coverage_drift_fails_closed() {
        // Uncovered universe member + unexpected corpus row + duplicate row.
        let corpus = json!({
            "surfaces": [
                { "path": "AGENTS.md", "exists": true, "front_matter": contract_front_matter() },
                { "path": "AGENTS.md", "exists": true, "front_matter": contract_front_matter() },
                { "path": "/specs/masterplan.json", "exists": true, "document": {} },
                { "path": "docs/uninvited.md", "exists": true, "front_matter": contract_front_matter() }
            ]
        });
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &policy_root_hub(),
            &corpus,
        );
        assert_eq!(
            finding_keys(&findings),
            vec![
                "AGENTS.md#corpus-duplicate",
                "docs/uninvited.md#corpus-unexpected",
                "registry/loop-recovery-patterns#corpus-uncovered",
                "specs/decision-principles.json#corpus-uncovered"
            ]
        );
    }

    #[test]
    fn missing_surface_and_missing_facts_fail_closed() {
        let mut corpus = green_corpus();
        corpus["surfaces"][0]["exists"] = Value::Bool(false);
        corpus["surfaces"][2] =
            json!({ "path": "/specs/decision-principles.json", "exists": true });
        let findings = evaluate_read_path_contract_coverage(
            &masterplan_with_central_row(),
            &policy_root_hub(),
            &corpus,
        );
        assert_eq!(
            finding_keys(&findings),
            vec![
                "AGENTS.md#surface-missing",
                "specs/decision-principles.json#missing-facts"
            ]
        );
    }
}
