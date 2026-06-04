#[allow(dead_code)]
#[path = "../ci/assert-trusted-target-inventory.rs"]
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

fn array_mut(value: &mut Json) -> &mut Vec<Json> {
    match value {
        Json::Array(array) => array,
        _ => panic!("expected array"),
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
    "specs/phase0-trusted-target-inventory-schema.json"
}

fn good_path() -> &'static str {
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-good-trusted-target-inventory.json"
}

fn bad_path() -> &'static str {
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1a-bad-candidate-sourced-target-inventory.json"
}

fn evaluate(schema: &Json, good: &Json, bad: &Json) -> checker::Report {
    checker::evaluate_sources(schema, good, bad, schema_path())
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
    assert!(!report.candidate_pr_bytes_are_data_only_locally_proven);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"FAIL""#));
    assert!(rendered.contains(r#""p0_0_green":false"#));
    assert!(rendered.contains(r#""phase0_complete":false"#));
}

#[test]
fn matrix_and_coverage_preserve_trusted_target_contract() {
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
        .find(|row| row.get("id").and_then(Json::as_str) == Some("AC-0.0-trusted-target-inventory"))
        .expect("trusted target-inventory row");
    let target = row
        .get("target_gate_or_controller")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(target.contains("cloud-ci/oya-ci trusted target inventory controller"));
    assert!(target.contains("//:phase0-trusted-target-inventory-check"));
    assert_eq!(
        row.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:phase0-trusted-target-inventory-check")
    );
    let claim_boundary = row
        .get("claim_boundary")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(claim_boundary.contains("not live controller target authority"));
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
            .any(|item| item.as_str() == Some("AC-0.0-trusted-target-inventory"))
    );
    let commands = subject
        .get("verification_commands")
        .and_then(Json::as_object)
        .expect("verification commands");
    assert_eq!(
        commands
            .get("AC-0.0-trusted-target-inventory")
            .and_then(Json::as_str),
        Some("buck2 build //:phase0-trusted-target-inventory-check")
    );
    let coverage_note = subject
        .get("coverage_note")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(coverage_note.contains("trusted target inventory"));
    assert!(coverage_note.contains("not P0.0 green"));
}

#[test]
fn good_schema_and_inventory_fixtures_pass_without_live_claims() {
    let report = evaluate(
        &read_json(schema_path()),
        &read_json(good_path()),
        &read_json(bad_path()),
    );
    assert_eq!(report.verdict, "PASS");
    assert!(report.local_fixture_contract_proven);
    assert!(report.candidate_pr_bytes_are_data_only_locally_proven);
    assert!(!report.trusted_target_inventory_live_authority_proven);
    assert!(!report.trusted_controller_inventory_live);
    assert!(!report.live_required_context_execution_proven);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"PASS""#));
    assert!(rendered.contains(r#""local_fixture_contract_proven":true"#));
    assert!(rendered.contains(r#""candidate_pr_bytes_are_data_only_locally_proven":true"#));
    assert!(rendered.contains(r#""trusted_target_inventory_live_authority_proven":false"#));
    assert!(rendered.contains("trusted target-inventory fixture evidence only; this checker never claims live cloud-ci execution or protected-branch authority"));
}

#[test]
fn schema_missing_inventory_source_fails_closed() {
    let mut schema = read_json(schema_path());
    let required = array_mut(
        object_mut(&mut schema)
            .get_mut("required")
            .expect("required"),
    );
    required.retain(|item| item.as_str() != Some("inventory_source"));
    assert_fails_with(
        evaluate(&schema, &read_json(good_path()), &read_json(bad_path())),
        "schema.required missing inventory_source",
    );
}

#[test]
fn schema_extra_inventory_source_fails_closed() {
    let mut schema = read_json(schema_path());
    let properties = object_mut(
        object_mut(&mut schema)
            .get_mut("properties")
            .expect("properties"),
    );
    let inventory_source = object_mut(
        properties
            .get_mut("inventory_source")
            .expect("inventory_source"),
    );
    array_mut(inventory_source.get_mut("enum").expect("enum")).push(Json::String(
        "candidate_supplied_controller_state".to_string(),
    ));
    assert_fails_with(
        evaluate(&schema, &read_json(good_path()), &read_json(bad_path())),
        "schema.inventory_source enum must be exactly",
    );
}

#[test]
fn good_fixture_extra_top_level_field_fails_closed() {
    let mut good = read_json(good_path());
    object_mut(&mut good).insert(
        "candidate_discovered_targets".to_string(),
        string_array(&["root//:candidate-owned"]),
    );
    assert_fails_with(
        evaluate(&read_json(schema_path()), &good, &read_json(bad_path())),
        "unexpected top-level fields",
    );
}

#[test]
fn good_fixture_short_candidate_sha_fails_closed() {
    let mut good = read_json(good_path());
    object_mut(&mut good).insert(
        "candidate_sha".to_string(),
        Json::String("abc123".to_string()),
    );
    assert_fails_with(
        evaluate(&read_json(schema_path()), &good, &read_json(bad_path())),
        "candidate_sha must be a 40-character hexadecimal SHA",
    );
}

#[test]
fn good_fixture_candidate_source_fails_closed() {
    let mut good = read_json(good_path());
    object_mut(&mut good).insert(
        "inventory_source".to_string(),
        Json::String("candidate_pr_bytes".to_string()),
    );
    assert_fails_with(
        evaluate(&read_json(schema_path()), &good, &read_json(bad_path())),
        "target_inventory_not_trusted",
    );
}

#[test]
fn good_fixture_malformed_target_fails_closed() {
    let mut good = read_json(good_path());
    object_mut(&mut good).insert(
        "test_targets".to_string(),
        string_array(&["not-a-buck2-target"]),
    );
    assert_fails_with(
        evaluate(&read_json(schema_path()), &good, &read_json(bad_path())),
        "malformed_buck2_target",
    );
}

#[test]
fn good_fixture_false_green_boundary_fails_closed() {
    let mut good = read_json(good_path());
    let claim_boundary = object_mut(
        object_mut(&mut good)
            .get_mut("claim_boundary")
            .expect("claim_boundary"),
    );
    claim_boundary.insert("p0_0_green".to_string(), Json::Bool(true));
    assert_fails_with(
        evaluate(&read_json(schema_path()), &good, &read_json(bad_path())),
        "green_claim_boundary_without_live_authority",
    );
}

#[test]
fn red_fixture_missing_expected_violation_fails_closed() {
    let mut bad = read_json(bad_path());
    let expected = array_mut(
        object_mut(&mut bad)
            .get_mut("expected_violations")
            .expect("expected_violations"),
    );
    expected.retain(|item| item.as_str() != Some("candidate_can_author_target_inventory"));
    assert_fails_with(
        evaluate(&read_json(schema_path()), &read_json(good_path()), &bad),
        "RED fixture expected_violations must include all trusted-target violation classes",
    );
}
