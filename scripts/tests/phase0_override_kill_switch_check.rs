#[allow(dead_code)]
#[path = "../ci/assert-override-kill-switch.rs"]
mod checker;

use checker::Json;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap())
}

fn read_json(path: &str) -> Json {
    let text = fs::read_to_string(repo_root().join(path)).unwrap_or_else(|error| {
        panic!("read {path}: {error}");
    });
    checker::parse_json(&text).unwrap_or_else(|error| panic!("parse {path}: {error}"))
}

fn object_mut(value: &mut Json) -> &mut BTreeMap<String, Json> {
    match value {
        Json::Object(object) => object,
        _ => panic!("expected object"),
    }
}

fn string_array(values: &[&str]) -> Json {
    Json::Array(
        values
            .iter()
            .map(|value| Json::String((*value).to_string()))
            .collect(),
    )
}

fn schema_path() -> &'static str {
    "specs/phase0-override-packet-schema.json"
}

fn good_path() -> &'static str {
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-good-cloud-ci-required-and-isolated.json"
}

fn bad_path() -> &'static str {
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.2-bad-override-without-ttl-audit.json"
}

fn evaluate(schema: &Json, good: &Json, bad: &Json) -> checker::Report {
    checker::evaluate_sources(schema, good, bad, schema_path(), good_path(), bad_path())
}

fn assert_fails_with(report: checker::Report, expected: &str) {
    assert_eq!(report.verdict, "FAIL");
    assert!(
        report
            .failures
            .iter()
            .any(|failure| failure.contains(expected)),
        "expected failure containing {expected:?}, got {:?}",
        report.failures
    );
    assert!(!report.local_fixture_contract_proven);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"FAIL""#));
    assert!(rendered.contains(r#""p0_0_green":false"#));
    assert!(rendered.contains(r#""phase0_complete":false"#));
}

#[test]
fn matrix_and_coverage_preserve_override_gate_contract() {
    let matrix = read_json("specs/phase0-automation-matrix.json");
    let coverage = read_json("specs/phase0-automation-coverage-registry.json");
    let rows = matrix
        .as_object()
        .and_then(|object| object.get("seed_rows"))
        .and_then(Json::as_array)
        .expect("matrix seed rows");
    let row = rows
        .iter()
        .filter_map(Json::as_object)
        .find(|row| row.get("id").and_then(Json::as_str) == Some("AC-0.0-override-kill-switch"))
        .expect("override row");
    let target = row
        .get("target_gate_or_controller")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(target.contains("cloud-ci/oya-ci override controller"));
    assert!(target.contains("//:phase0-override-kill-switch-check"));
    assert_eq!(
        row.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:phase0-override-kill-switch-check")
    );
    let claim_boundary = row
        .get("claim_boundary")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(claim_boundary.contains("not live protected-flow override authority"));
    assert!(claim_boundary.contains("not P0.0 green"));
    assert_eq!(
        row.get("no_new_oya_cli_surface").and_then(Json::as_bool),
        Some(true)
    );

    let subjects = coverage
        .as_object()
        .and_then(|object| object.get("coverage_subjects"))
        .and_then(Json::as_array)
        .expect("coverage subjects");
    let subject = subjects
        .iter()
        .filter_map(Json::as_object)
        .find(|subject| subject.get("id").and_then(Json::as_str) == Some("AC-0.0"))
        .expect("AC-0.0 coverage subject");
    let mapped = subject
        .get("mapped_row_ids")
        .and_then(Json::as_array)
        .expect("mapped row ids");
    assert!(
        mapped
            .iter()
            .any(|item| item.as_str() == Some("AC-0.0-override-kill-switch"))
    );
    let commands = subject
        .get("verification_commands")
        .and_then(Json::as_object)
        .expect("verification commands");
    assert_eq!(
        commands
            .get("AC-0.0-override-kill-switch")
            .and_then(Json::as_str),
        Some("buck2 build //:phase0-override-kill-switch-check")
    );
    let coverage_note = subject
        .get("coverage_note")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(coverage_note.contains("override/kill-switch"));
    assert!(coverage_note.contains("not P0.0 green"));
}

#[test]
fn good_schema_and_baseline_fixtures_pass_without_live_claims() {
    let report = evaluate(
        &read_json(schema_path()),
        &read_json(good_path()),
        &read_json(bad_path()),
    );
    assert_eq!(report.verdict, "PASS");
    assert!(report.local_fixture_contract_proven);
    assert!(!report.live_required_context_execution_proven);
    assert!(!report.protected_flow_override_live);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"PASS""#));
    assert!(rendered.contains(r#""local_fixture_contract_proven":true"#));
    assert!(rendered.contains(r#""live_required_context_execution_proven":false"#));
    assert!(rendered.contains(r#""protected_flow_override_live":false"#));
    assert!(rendered.contains("override/kill-switch fixture evidence only; this checker never claims live protected-flow override authority or status mutation"));
}

#[test]
fn schema_missing_no_new_oya_cli_required_field_fails_closed() {
    let mut schema = read_json(schema_path());
    object_mut(&mut schema).insert(
        "required".to_string(),
        string_array(&[
            "action",
            "affected_contexts",
            "degraded_gate_ids",
            "ttl_expires_at",
            "reviewer_acknowledgment",
            "audit_chain_event",
            "owner",
            "blast_radius_statement",
            "revert_or_fix_follow_up",
        ]),
    );
    assert_fails_with(
        evaluate(&schema, &read_json(good_path()), &read_json(bad_path())),
        "schema.required missing no_new_oya_cli_surface",
    );
}

#[test]
fn good_fixture_missing_ttl_fails_closed() {
    let mut good = read_json(good_path());
    let packet = object_mut(&mut good)
        .get_mut("override_packet")
        .expect("override packet");
    object_mut(packet).remove("ttl_expires_at");
    assert_fails_with(
        evaluate(&read_json(schema_path()), &good, &read_json(bad_path())),
        "GOOD override packet has violations",
    );
}

#[test]
fn good_fixture_new_oya_cli_surface_fails_closed() {
    let mut good = read_json(good_path());
    let packet = object_mut(&mut good)
        .get_mut("override_packet")
        .expect("override packet");
    object_mut(packet).insert("no_new_oya_cli_surface".to_string(), Json::Bool(false));
    assert_fails_with(
        evaluate(&read_json(schema_path()), &good, &read_json(bad_path())),
        "override_new_oya_cli_surface",
    );
}

#[test]
fn red_fixture_missing_expected_override_violation_fails_closed() {
    let mut bad = read_json(bad_path());
    object_mut(&mut bad).insert(
        "expected_violations".to_string(),
        string_array(&[
            "missing_cloud_ci_required_context",
            "untrusted_or_legacy_status_producer",
            "candidate_bytes_can_weaken_gate",
            "candidate_sourced_gate_definition",
            "override_missing_ttl_reviewer_audit_or_revert",
            "override_missing_context_or_gate",
        ]),
    );
    assert_fails_with(
        evaluate(&read_json(schema_path()), &read_json(good_path()), &bad),
        "RED fixture expected_violations must include all override violation classes",
    );
}
