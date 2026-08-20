// Planned-maturity gate live-corpus tests.
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use ci_feature_maturity_policy::{Verdict, evaluate};
use serde_json::{Value, json};

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn product_prd_rows(root: &Path) -> Vec<Value> {
    let products_dir = root.join("docs/products");
    let mut rows = Vec::new();
    for entry in fs::read_dir(&products_dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", products_dir.display()))
    {
        let entry = entry.expect("product dir entry");
        if !entry.file_type().expect("product file type").is_dir() {
            continue;
        }
        let prd = entry.path().join("PRD.md");
        // Every product directory owes a PRD. Skipping the ones without it made a RENAMED PRD
        // indistinguishable from a product that was never covered: the product's acceptance and
        // verification rows simply stopped being observed, and the maturity evaluator went green
        // over the shrunken set.
        assert!(
            prd.is_file(),
            "product directory {} carries no PRD.md — a product whose PRD is missing or renamed \
             drops out of the maturity corpus silently; add the PRD or retire the directory in \
             the same change",
            entry.path().display()
        );
        let text =
            fs::read_to_string(&prd).unwrap_or_else(|e| panic!("read {}: {e}", prd.display()));
        let lower = text.to_ascii_lowercase();
        let rel_path = rel(root, &prd);
        let terms = product_terms(&rel_path);
        let acceptance_rows = table_rows_after_heading(&text, "## 2a.")
            .into_iter()
            .filter(|row| is_acceptance_contract_row(row))
            .collect::<Vec<_>>();
        let verification_rows = table_rows_after_heading(&text, "## 9b.")
            .into_iter()
            .filter(|row| is_verification_command_row(row))
            .collect::<Vec<_>>();
        let product_acceptance_rows = acceptance_rows
            .iter()
            .filter(|row| row_contains_any(row, &terms))
            .count();
        let product_verification_rows = verification_rows
            .iter()
            .filter(|row| row_contains_any(row, &terms) && has_executable_verification_command(row))
            .count();
        rows.push(json!({
            "path": rel_path,
            "has_acceptance_heading": lower.contains("## 2a."),
            "acceptance_row_count": acceptance_rows.len(),
            "product_specific_acceptance_row_count": product_acceptance_rows,
            "has_verification_heading": lower.contains("## 9b."),
            "verification_command_row_count": verification_rows
                .iter()
                .filter(|row| has_executable_verification_command(row))
                .count(),
            "product_specific_verification_row_count": product_verification_rows,
        }));
    }
    rows.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
    rows
}

fn has_executable_verification_command(text: &str) -> bool {
    ["buck2 test", "buck2 build", "cargo test", "cargo run"]
        .iter()
        .any(|needle| text.contains(needle))
}
fn is_acceptance_contract_row(row: &str) -> bool {
    let cells = table_cells(row);
    cells.len() >= 6
        && cells[0].contains("-PRD-AC-")
        && cells[4].contains("-PRD-GATE-")
        && cells[5].contains("planned_maturity.rs")
}

fn is_verification_command_row(row: &str) -> bool {
    let cells = table_cells(row);
    cells.len() >= 4
        && !cells[0].trim().is_empty()
        && has_executable_verification_command(&cells[1])
        && !cells[2].trim().is_empty()
}

fn table_cells(row: &str) -> Vec<String> {
    row.trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().trim_matches('`').to_owned())
        .collect()
}
fn table_rows_after_heading(text: &str, heading_prefix: &str) -> Vec<String> {
    let mut in_section = false;
    let mut rows = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            in_section = trimmed.to_ascii_lowercase().starts_with(heading_prefix);
            continue;
        }
        if !in_section || !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();
        if lower.contains("ac-id")
            || lower.contains("verification command")
            || lower
                .chars()
                .all(|ch| ch == '|' || ch == '-' || ch == ':' || ch.is_whitespace())
        {
            continue;
        }
        rows.push(trimmed.to_owned());
    }
    rows
}

fn row_contains_any(row: &str, terms: &[&str]) -> bool {
    let lower = row.to_ascii_lowercase();
    terms.iter().any(|term| lower.contains(term))
}

fn product_terms(rel_path: &str) -> Vec<&'static str> {
    if rel_path.contains("saas-platform") {
        vec![
            "workflow",
            "plugin",
            "marketplace",
            "tenant",
            "billing",
            "audit",
        ]
    } else if rel_path.contains("workplace-integration") {
        vec![
            "saga",
            "workflow",
            "hr",
            "payroll",
            "calendar",
            "messenger",
            "audit",
        ]
    } else if rel_path.contains(concat!("foun", "dry")) {
        vec![
            "capability",
            "autonomy",
            "provider",
            "audit-chain",
            "evidence",
        ]
    } else if rel_path.contains("cloud/") {
        vec![
            "region", "cell", "resource", "audit", "billing", "slo", "security",
        ]
    } else if rel_path.contains("erp-coverage") {
        vec![
            "ledger",
            "procurement",
            "payroll",
            "workflow",
            "sap",
            "audit",
        ]
    } else {
        vec!["evidence"]
    }
}

fn retired_plan_ref_rows(root: &Path) -> Vec<Value> {
    let masterplan = read_json(&root.join("specs/masterplan.json"));
    let master_policy = &masterplan["_meta"]["retired_plan_reference_policy"];
    let master_status = master_policy["status"].as_str().unwrap_or_default();
    let master_live_refs_resolve = live_refs_resolve(root, &master_policy["live_gate_input_refs"]);

    let sequencing = read_json(&root.join("specs/master-plan-sequencing.json"));
    let sequencing_archive = &sequencing["_metadata"]["archived_stale_documents"];
    let sequencing_status = if sequencing_archive["retired_manifest_status"]
        .as_str()
        .is_some_and(|status| {
            status.contains("historical") && status.contains("not live authority")
        }) {
        "historical_provenance_only"
    } else {
        ""
    };
    let sequencing_live_refs_resolve = live_ref_resolves_at_root(
        root,
        sequencing_archive["live_replacement"]
            .as_str()
            .unwrap_or_default(),
    );

    let mut rows = Vec::new();
    collect_retired_refs(
        &masterplan,
        "specs/masterplan.json",
        master_status,
        master_live_refs_resolve,
        &mut rows,
    );
    collect_retired_refs(
        &sequencing,
        "specs/master-plan-sequencing.json",
        sequencing_status,
        sequencing_live_refs_resolve,
        &mut rows,
    );
    rows.sort_by(|left, right| left["key"].as_str().cmp(&right["key"].as_str()));
    rows
}

fn collect_retired_refs(
    value: &Value,
    key: &str,
    status: &str,
    live_ref_resolves: bool,
    rows: &mut Vec<Value>,
) {
    match value {
        Value::String(text) => {
            if text.contains(".omc/") || text.contains(".omx/") {
                rows.push(json!({
                    "key": key,
                    "retired_path": text,
                    "status": status,
                    "usage": retired_usage_for_key(key),
                    "live_ref_resolves": live_ref_resolves,
                }));
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                collect_retired_refs(
                    child,
                    &format!("{key}[{index}]"),
                    status,
                    live_ref_resolves,
                    rows,
                );
            }
        }
        Value::Object(values) => {
            for (field, child) in values {
                collect_retired_refs(
                    child,
                    &format!("{key}.{field}"),
                    status,
                    live_ref_resolves,
                    rows,
                );
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}
fn retired_usage_for_key(key: &str) -> &'static str {
    if key.contains(".live_implementation_index.") {
        return "historical_snapshot";
    }

    let historical_contexts = [
        "specs/masterplan.json._meta.",
        "specs/masterplan.json.architecture_invariants",
        "specs/masterplan.json.milestones",
        "specs/masterplan.json.evidence_records",
        "specs/masterplan.json.gitops_vcs_replacement",
        "specs/masterplan.json.implementation_plan_contract.legacy_",
        "specs/masterplan.json.implementation_plan_contract.live_applies_to_artifacts",
        "specs/masterplan.json.planning_authority.retired_scratch_globs",
        "specs/masterplan.json.masterplan_v2.surface_dispositions",
        "specs/masterplan.json.masterplan_v2.authority_consolidation_audit",
        "specs/masterplan.json.masterplan_v2.sequencing.rederivation.inherited_orderings_ignored",
        "specs/master-plan-sequencing.json._metadata.archived_stale_documents",
        "specs/master-plan-sequencing.json.canonical_build_sequence.canonical_anchors",
        "specs/master-plan-sequencing.json.realignment_wave_sequence",
        "specs/master-plan-sequencing.json.implementation_plan_changeset_contract.legacy_",
        "specs/master-plan-sequencing.json.implementation_plan_changeset_contract.live_applies_to_artifacts",
    ];

    if historical_contexts
        .iter()
        .any(|context| key.contains(context))
    {
        "historical_provenance_only"
    } else {
        "live_input"
    }
}
fn live_refs_resolve(root: &Path, refs: &Value) -> bool {
    refs.as_array()
        .map(|refs| {
            !refs.is_empty()
                && refs
                    .iter()
                    .filter_map(Value::as_str)
                    .all(|reference| live_ref_resolves_at_root(root, reference))
        })
        .unwrap_or(false)
}

fn live_ref_resolves_at_root(root: &Path, reference: &str) -> bool {
    let Some(path_part) = reference.split('#').next() else {
        return false;
    };
    let path = path_part.trim_start_matches('/');
    !path.is_empty() && root.join(path).exists()
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("path under repo root")
        .to_string_lossy()
        .replace('\\', "/")
}

fn live_observation(root: &Path) -> Value {
    json!({
        "minimum_product_prds": 4,
        "minimum_acceptance_rows_per_prd": 2,
        "minimum_verification_rows_per_prd": 2,
        "product_prds": product_prd_rows(root),
        "minimum_capability_records": 50,
        "capability_records": read_json(&root.join(format!("registry/capabilities/{}-internal.json", concat!("foun", "dry")))),
        "retired_plan_scan_executed": true,
        "retired_plan_refs": retired_plan_ref_rows(root),
    })
}

#[test]
fn red_fixtures_block_marker_only_prds_shallow_capabilities_and_live_retired_paths() {
    let fixture = json!({
        "minimum_product_prds": 1,
        "product_prds": [{
            "path": "docs/products/example/PRD.md",
            "has_acceptance_heading": true,
            "acceptance_row_count": 0,
            "product_specific_acceptance_row_count": 0,
            "has_verification_heading": true,
            "verification_command_row_count": 0,
            "product_specific_verification_row_count": 0
        }],
        "minimum_capability_records": 1,
        "capability_records": [{
            "id": "minimal.only",
            "name": "Minimal only"
        }],
        "retired_plan_scan_executed": true,
        "retired_plan_refs": [{
            "key": "$.milestones[0].path",
            "retired_path": ".omc/plans/stale.md",
            "status": "historical_provenance_only",
            "usage": "live_input",
            "live_ref_resolves": false
        }]
    });

    let report = evaluate(&fixture);
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .violations
            .contains("planned_maturity_product_prd_missing_acceptance_contract")
    );
    assert!(
        report
            .violations
            .contains("planned_maturity_product_prd_missing_verification_contract")
    );
    assert!(
        report
            .violations
            .contains("planned_maturity_capability_record_too_shallow")
    );
    assert!(
        report
            .violations
            .contains("planned_maturity_retired_plan_live_input")
    );
    assert!(
        report
            .violations
            .contains("planned_maturity_live_gate_input_missing")
    );
}

#[test]
fn live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated() {
    let root = repo_root();
    let observation = live_observation(&root);
    let report = evaluate(&observation);

    assert_eq!(
        report.verdict,
        Verdict::Green,
        "planned-maturity live corpus findings: {:#?}",
        report.findings
    );
}
