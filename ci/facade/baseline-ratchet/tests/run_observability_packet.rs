// GH #1003 cloud-ci run observability packet contract.
//
// The firewall crate is the current required-context gate home. The canonical packet and status
// query contracts live under top-level specs/ schemas; this test target proves a failed gate can be
// diagnosed from typed packet/status data without scraping GitHub Actions logs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use ci_baseline_ratchet::run_observability_packet::{
    PACKET_SCHEMA_PATH, PacketVerdict, STATUS_SCHEMA_PATH, STATUS_VALUES, validate_packet,
    validate_status,
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
fn status_finding_codes(value: &Value) -> BTreeSet<String> {
    validate_status(value)
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

#[test]
fn status_schema_declares_the_canonical_query_surface() {
    let root = repo_root();
    let schema = read_json(&root.join(STATUS_SCHEMA_PATH));

    let required: BTreeSet<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("status schema required array")
        .iter()
        .map(|value| value.as_str().expect("required item string"))
        .collect();
    for expected in [
        "schema_version",
        "status_id",
        "producer",
        "run",
        "subject",
        "status",
        "phase",
        "gate_summary",
        "artifact_refs",
        "diagnostic_refs",
        "correlation",
        "retention",
        "diagnosability",
    ] {
        assert!(
            required.contains(expected),
            "status schema top-level required set must include {expected}; observed {required:#?}"
        );
    }

    let values: BTreeSet<&str> = schema
        .pointer("/properties/status/enum")
        .and_then(Value::as_array)
        .expect("status enum array")
        .iter()
        .map(|value| value.as_str().expect("status enum string"))
        .collect();
    assert_eq!(values, STATUS_VALUES.into_iter().collect());

    assert_eq!(
        schema
            .pointer("/properties/diagnosability/properties/actions_log_scrape_required/const")
            .and_then(Value::as_bool),
        Some(false),
        "status API contract must forbid raw Actions log scraping as first-diagnosis dependency"
    );
    assert_eq!(
        schema
            .pointer("/properties/artifact_refs/items/pattern")
            .and_then(Value::as_str),
        Some(
            "^artifact:(gate-report|step-report|redacted-diagnostics|status-packet):[A-Za-z0-9._:-]+$"
        ),
        "status artifact refs must point at typed packet artifacts, never raw log URLs"
    );
    assert_eq!(
        schema
            .pointer("/properties/diagnostic_refs/items/pattern")
            .and_then(Value::as_str),
        Some("^diag:[A-Za-z0-9._-]+:[A-Za-z0-9._-]+$"),
        "status diagnostic refs must point at typed redacted diagnostic ids"
    );
    assert_eq!(
        schema
            .pointer("/allOf/0/oneOf/0/properties/producer/properties/required_context/const")
            .and_then(Value::as_str),
        Some("cloud-ci-required"),
        "schema must bind status required-context fields for cloud-ci-required"
    );
    assert_eq!(
        schema
            .pointer("/allOf/0/oneOf/1/properties/run/properties/status_context/const")
            .and_then(Value::as_str),
        Some("oya-ci-required"),
        "schema must bind status required-context fields for oya-ci-required"
    );
    assert_eq!(
        schema
            .pointer("/allOf/1/then/properties/artifact_refs/minItems")
            .and_then(Value::as_u64),
        Some(1),
        "failed/timed_out statuses must carry typed artifact refs"
    );
    assert_eq!(
        schema
            .pointer("/allOf/1/then/properties/diagnostic_refs/minItems")
            .and_then(Value::as_u64),
        Some(1),
        "failed/timed_out statuses must carry typed diagnostic refs"
    );
}

#[test]
fn failed_status_fixture_is_diagnosable_without_actions_log_scraping() {
    let root = repo_root();
    let status = fixture(&root, "tc-1003-good-failed-status.json");

    let report = validate_status(&status);
    assert_eq!(report.verdict, PacketVerdict::Green, "{report:#?}");

    assert_eq!(
        status
            .pointer("/diagnosability/first_diagnosis_from_status_api")
            .and_then(Value::as_bool),
        Some(true),
        "failed status projection must be useful to API/console consumers without log scraping"
    );
}

#[test]
fn running_status_does_not_need_failed_gate_diagnostics() {
    let root = repo_root();
    let status = fixture(&root, "tc-1003-good-running-status.json");

    let report = validate_status(&status);
    assert_eq!(report.verdict, PacketVerdict::Green, "{report:#?}");

    assert!(
        status
            .get("diagnostic_refs")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty),
        "running status must not invent failed-gate diagnostics"
    );
}

#[test]
fn invalid_status_value_is_red_with_exact_expected_enum() {
    let root = repo_root();
    let status = fixture(&root, "tc-1003-bad-invalid-status-ready.json");

    let report = validate_status(&status);
    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "status_value_invalid")
        .expect("invalid status finding");
    assert_eq!(
        finding.remediation,
        "invalid cloud-ci run observability status: ready; expected queued|running|passed|failed|cancelled|timed_out"
    );
}

#[test]
fn status_actions_log_scrape_only_fixture_is_red() {
    let root = repo_root();
    let status = fixture(&root, "tc-1003-bad-status-actions-log-scrape-only.json");

    let codes = status_finding_codes(&status);
    for expected in [
        "actions_log_scrape_required",
        "status_api_diagnosis_not_declared",
        "status_failure_artifact_refs_missing",
        "status_failure_diagnostic_refs_missing",
    ] {
        assert!(
            codes.contains(expected),
            "expected status finding code {expected}; observed {codes:#?}"
        );
    }
}

#[test]
fn status_untyped_refs_fixture_is_red() {
    let root = repo_root();
    let status = fixture(&root, "tc-1003-bad-status-untyped-refs.json");

    let codes = status_finding_codes(&status);
    for expected in [
        "status_artifact_ref_raw_actions_log",
        "status_artifact_ref_invalid",
        "status_diagnostic_ref_invalid",
        "status_failure_artifact_refs_missing",
        "status_failure_diagnostic_refs_missing",
    ] {
        assert!(
            codes.contains(expected),
            "expected status finding code {expected}; observed {codes:#?}"
        );
    }
}

#[test]
fn status_context_mismatch_fixture_is_red() {
    let root = repo_root();
    let status = fixture(&root, "tc-1003-bad-status-context-mismatch.json");

    let codes = status_finding_codes(&status);
    assert!(
        codes.contains("context_binding_mismatch"),
        "expected context_binding_mismatch; observed {codes:#?}"
    );
}
