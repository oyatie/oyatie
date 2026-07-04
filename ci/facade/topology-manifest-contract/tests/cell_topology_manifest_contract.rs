// CELL-001R cloud-ci Rust gate for the cell/residency/multi-region topology
// manifest contract. The test reads committed JSON surfaces directly and refuses
// retired/local CLI authority strings in the new contract.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use ci_topology_manifest_contract::GATE_ID;
use serde_json::Value;

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current dir");
    loop {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        assert!(dir.pop(), "repo root marker not found from current dir");
    }
}

fn read_text(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap_or_else(|err| panic!("read {rel}: {err}"))
}

fn read_json(root: &Path, rel: &str) -> Value {
    serde_json::from_str(&read_text(root, rel)).unwrap_or_else(|err| panic!("parse {rel}: {err}"))
}

fn as_strings<'a>(value: &'a Value, field: &str) -> Vec<&'a str> {
    value[field]
        .as_array()
        .unwrap_or_else(|| panic!("{field} must be an array"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("{field} entries must be strings"))
        })
        .collect()
}

fn recursively_contains_oya_gate(value: &Value) -> bool {
    match value {
        Value::String(text) => text.contains("oya gate") || text.contains("oya-dev-cli"),
        Value::Array(items) => items.iter().any(recursively_contains_oya_gate),
        Value::Object(map) => map.values().any(recursively_contains_oya_gate),
        _ => false,
    }
}

#[test]
fn manifest_contract_declares_atomic_fields_adr_sources_and_cloud_ci_authority() {
    let root = repo_root();
    let spec = read_json(&root, "specs/cell-topology-manifest-contract.json");

    assert_eq!(spec["spec_id"], "cell-topology-manifest-contract");
    assert_eq!(spec["spec_kind"], "manifest-contract");
    assert_eq!(spec["authoring_task"], "CELL-001R");
    assert_eq!(spec["cloud_ci_gate"], GATE_ID);
    assert_eq!(
        spec["manifest_field"]["validator"],
        "cloud-ci Rust gate: oya-cloud-ci-cell-topology-manifest-contract-app"
    );
    assert!(
        !recursively_contains_oya_gate(&spec),
        "CELL-001R must not introduce retired/local CLI gate authority"
    );

    let adrs = as_strings(&spec, "related_adrs");
    for adr in ["ADR-0009", "ADR-0049", "ADR-0158", "ADR-0161"] {
        assert!(adrs.contains(&adr), "missing related ADR {adr}");
    }

    let required = as_strings(&spec, "required_contract_fields");
    for field in [
        "service_id",
        "tenant_id",
        "cell_topology.cell_id",
        "cell_topology.region",
        "cell_topology.cell_tier",
        "cell_topology.residency_class",
        "cell_topology.region_disposition",
        "cell_topology.storage_class",
        "cell_topology.quarterly_isolation_evidence.quarter",
        "cell_topology.quarterly_isolation_evidence.network",
        "cell_topology.quarterly_isolation_evidence.storage",
        "cell_topology.quarterly_isolation_evidence.crypto",
        "cell_topology.quarterly_isolation_evidence.compute",
        "cell_topology.quarterly_isolation_evidence.audit",
    ] {
        assert!(required.contains(&field), "missing required field {field}");
    }

    assert!(
        spec["cell_tier_sources"]["dedicated"]
            .as_str()
            .unwrap()
            .contains("ADR-0009")
    );
    assert!(
        spec["residency_class_sources"]["strict_kr"]
            .as_str()
            .unwrap()
            .contains("ADR-0049")
    );
    assert!(
        spec["region_disposition_sources"]["active_passive"]
            .as_str()
            .unwrap()
            .contains("ADR-0158")
    );
    assert!(
        spec["storage_class_source"]
            .as_str()
            .unwrap()
            .contains("ADR-0161")
    );

    let proposed_scope = spec["authority"]["proposed_adr_scope"].as_str().unwrap();
    assert!(proposed_scope.contains("ADR-0009"));
    assert!(proposed_scope.contains("ADR-0049"));
    assert!(proposed_scope.contains("planning/context"));
    assert!(proposed_scope.contains("no runtime"));

    assert_eq!(
        spec["path_conflicts"]["conflict_class"],
        "repo-governance-specs-docs"
    );
    let serialized_paths = as_strings(&spec["path_conflicts"], "serialized_paths");
    assert!(serialized_paths.contains(&"specs/root-hub-pointers.json"));
}

#[test]
fn fixture_satisfies_atomic_contract_without_live_evidence_or_runtime_claims() {
    let root = repo_root();
    let spec = read_json(&root, "specs/cell-topology-manifest-contract.json");
    let fixture = read_json(
        &root,
        "specs/fixtures/cell-topology-manifest/tenancy-kr-strict.json",
    );
    assert!(
        !recursively_contains_oya_gate(&fixture),
        "fixture must not cite retired/local CLI gate authority"
    );

    assert_eq!(
        fixture["authority"]["spec"],
        "specs/cell-topology-manifest-contract.json"
    );
    assert_eq!(fixture["authority"]["cloud_ci_gate"], GATE_ID);
    let fixture_adrs = as_strings(&fixture["authority"], "source_adrs");
    for adr in ["ADR-0009", "ADR-0049", "ADR-0158", "ADR-0161"] {
        assert!(fixture_adrs.contains(&adr), "missing fixture ADR {adr}");
    }

    let row = &fixture["fixtures"]
        .as_array()
        .expect("fixtures array")
        .first()
        .expect("one fixture row");
    assert_eq!(row["service_id"], "tenancy");
    assert_eq!(row["tenant_id"], "ten_kr_healthcare_alpha");

    let topology = &row["cell_topology"];
    assert!(as_strings(&spec, "cell_tier_enum").contains(&topology["cell_tier"].as_str().unwrap()));
    assert!(
        as_strings(&spec, "residency_class_enum")
            .contains(&topology["residency_class"].as_str().unwrap())
    );
    assert!(
        as_strings(&spec, "region_disposition_enum")
            .contains(&topology["region_disposition"].as_str().unwrap())
    );
    assert!(
        as_strings(&spec, "storage_class_enum")
            .contains(&topology["storage_class"].as_str().unwrap())
    );

    let evidence = &topology["quarterly_isolation_evidence"];
    assert_eq!(evidence["quarter"], "2026-Q3");
    for kind in ["network", "storage", "crypto", "compute", "audit"] {
        let handle = evidence[kind].as_str().unwrap_or_default();
        assert!(
            handle.starts_with("evidence://cell/"),
            "bad {kind} evidence handle: {handle}"
        );
    }

    let non_claims = as_strings(&fixture, "non_claims").join("\n");
    assert!(non_claims.contains("fixture only"));
    assert!(non_claims.contains("not fetched evidence payloads"));
}

#[test]
fn root_hub_registers_contract_as_serialized_spec_surface() {
    let root = repo_root();
    let root_hub = read_json(&root, "specs/root-hub-pointers.json");
    let entry = &root_hub["entry_points"]["spec_cell_topology_manifest_contract"];
    assert_eq!(
        entry["current_path"],
        "/specs/cell-topology-manifest-contract.json"
    );
    assert_eq!(entry["kind"], "spec");
    assert_eq!(entry["conflict_class"], "repo-governance-specs-docs");
    assert!(
        entry["purpose"]
            .as_str()
            .unwrap_or_default()
            .contains("cell tier, residency class, region disposition, storage class")
    );
}
