// GH #1003 cloud-ci run observability packet contract.
//
// The firewall crate is the current required-context gate home. The canonical status packet
// contract is the top-level specs/cloud-ci-run-observability-packet.schema.json schema; this
// test target proves a failed gate can be diagnosed from typed packet data without scraping
// GitHub Actions logs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use oya_cloud_ci_firewall_app::run_observability_packet::{
    PACKET_SCHEMA_PATH, PacketVerdict, validate_packet,
};
use serde_json::Value;

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

fn fixture(root: &Path, name: &str) -> Value {
    read_json(
        &root
            .join("specs/fixtures/cloud-ci-run-observability")
            .join(name),
    )
}

fn finding_codes(value: &Value) -> BTreeSet<String> {
    validate_packet(value)
        .findings
        .iter()
        .map(|finding| finding.code.clone())
        .collect()
}

#[test]
fn failed_gate_fixture_is_diagnosable_without_actions_log_scraping() {
    let root = repo_root();
    let packet = fixture(&root, "tc-1003-good-failed-gate-diagnosable.json");

    let report = validate_packet(&packet);
    assert_eq!(report.verdict, PacketVerdict::Green, "{report:#?}");

    assert_eq!(
        packet
            .pointer("/diagnosability/actions_log_scrape_required")
            .and_then(Value::as_bool),
        Some(false),
        "the good fixture must make first diagnosis possible without GitHub Actions log scraping"
    );
    assert_eq!(
        packet
            .pointer("/diagnostics/0/redaction/applied")
            .and_then(Value::as_bool),
        Some(true),
        "diagnostic payloads must be redacted before becoming status API data"
    );
}

#[test]
fn actions_log_scrape_only_fixture_is_red() {
    let root = repo_root();
    let packet = fixture(&root, "tc-1003-bad-actions-log-scrape-only.json");

    let codes = finding_codes(&packet);
    for expected in [
        "actions_log_scrape_required",
        "raw_actions_log_artifact_forbidden",
        "failure_artifact_refs_missing",
        "failure_diagnostic_refs_missing",
        "diagnosability_diagnostic_refs_missing",
        "diagnostic_redaction_not_applied",
        "diagnostic_secret_unredacted",
    ] {
        assert!(
            codes.contains(expected),
            "expected finding code {expected}; observed {codes:#?}"
        );
    }
}

#[test]
fn passed_packet_does_not_need_failed_gate_diagnostics() {
    let root = repo_root();
    let packet = fixture(&root, "tc-1003-good-passed-no-failure-diagnostics.json");

    let report = validate_packet(&packet);
    assert_eq!(report.verdict, PacketVerdict::Green, "{report:#?}");

    assert!(
        packet
            .get("diagnostics")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty),
        "passed fixture must not invent failed-gate diagnostics"
    );
}

#[test]
fn failed_packet_without_diagnostics_is_red() {
    let root = repo_root();
    let packet = fixture(&root, "tc-1003-bad-failed-without-diagnostics.json");

    let codes = finding_codes(&packet);
    for expected in [
        "diagnostics_empty",
        "failure_diagnostic_refs_missing",
        "diagnosability_diagnostic_refs_missing",
    ] {
        assert!(
            codes.contains(expected),
            "expected finding code {expected}; observed {codes:#?}"
        );
    }
}

#[test]
fn context_mismatch_fixture_is_red() {
    let root = repo_root();
    let packet = fixture(&root, "tc-1003-bad-context-mismatch.json");

    let codes = finding_codes(&packet);
    assert!(
        codes.contains("context_binding_mismatch"),
        "expected context_binding_mismatch; observed {codes:#?}"
    );
}

#[test]
fn unstable_packet_id_fixture_is_red() {
    let root = repo_root();
    let packet = fixture(&root, "tc-1003-bad-unstable-packet-id.json");

    let codes = finding_codes(&packet);
    assert!(
        codes.contains("packet_id_run_mismatch"),
        "expected packet_id_run_mismatch; observed {codes:#?}"
    );
}

#[test]
fn unstable_nested_ids_fixture_is_red() {
    let root = repo_root();
    let packet = fixture(&root, "tc-1003-bad-unstable-nested-ids.json");

    let codes = finding_codes(&packet);
    for expected in [
        "transition_id_unstable",
        "artifact_id_unstable",
        "artifact_id_run_mismatch",
        "diagnostic_id_unstable",
    ] {
        assert!(
            codes.contains(expected),
            "expected finding code {expected}; observed {codes:#?}"
        );
    }
}

#[test]
fn schema_declares_the_canonical_status_packet_surface() {
    let root = repo_root();
    let schema = read_json(&root.join(PACKET_SCHEMA_PATH));

    let required: BTreeSet<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("schema required array")
        .iter()
        .map(|value| value.as_str().expect("required item string"))
        .collect();
    for expected in [
        "schema_version",
        "packet_id",
        "producer",
        "run",
        "subject",
        "transitions",
        "artifacts",
        "diagnostics",
        "retention",
        "diagnosability",
    ] {
        assert!(
            required.contains(expected),
            "schema top-level required set must include {expected}; observed {required:#?}"
        );
    }

    assert_eq!(
        schema
            .pointer("/properties/diagnosability/properties/actions_log_scrape_required/const")
            .and_then(Value::as_bool),
        Some(false),
        "the schema must forbid status packets that require raw Actions log scraping"
    );

    let taxonomy: BTreeSet<&str> = schema
        .pointer("/$defs/failure/properties/taxonomy/enum")
        .and_then(Value::as_array)
        .expect("failure taxonomy enum")
        .iter()
        .map(|value| value.as_str().expect("taxonomy enum string"))
        .collect();
    for expected in [
        "code_regression",
        "policy_violation",
        "infra_red",
        "operator_waiver_required",
        "cancelled",
        "flake_suspected",
    ] {
        assert!(
            taxonomy.contains(expected),
            "failure taxonomy must include {expected}; observed {taxonomy:#?}"
        );
    }
}
