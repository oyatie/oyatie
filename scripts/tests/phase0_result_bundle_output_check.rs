#[allow(dead_code)]
#[path = "../ci/assert-result-bundle-output.rs"]
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

fn object(entries: &[(&str, Json)]) -> Json {
    Json::Object(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect(),
    )
}

fn schema_path() -> &'static str {
    "specs/phase0-ci-enforcement-result-schema.json"
}

fn current_red_path() -> &'static str {
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0-current-red-gap-result.json"
}

fn false_green_path() -> &'static str {
    "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.4-bad-result-bundle-false-green.json"
}

fn evaluate(schema: &Json, current_red: &Json, false_green: &Json) -> checker::Report {
    checker::evaluate_sources(
        schema,
        current_red,
        false_green,
        schema_path(),
        current_red_path(),
        false_green_path(),
    )
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
fn matrix_and_coverage_preserve_result_bundle_contract() {
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
        .find(|row| row.get("id").and_then(Json::as_str) == Some("AC-0.0-structured-result-bundle"))
        .expect("structured result-bundle row");
    let target = row
        .get("target_gate_or_controller")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(target.contains("cloud-ci/oya-ci result bundle emitter"));
    assert!(target.contains("//:phase0-result-bundle-output-check"));
    assert_eq!(
        row.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:phase0-result-bundle-output-check")
    );
    let claim_boundary = row
        .get("claim_boundary")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(claim_boundary.contains("not live status output"));
    assert!(claim_boundary.contains("not protected-branch authority"));
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
            .any(|item| item.as_str() == Some("AC-0.0-structured-result-bundle"))
    );
    let commands = subject
        .get("verification_commands")
        .and_then(Json::as_object)
        .expect("verification commands");
    assert_eq!(
        commands
            .get("AC-0.0-structured-result-bundle")
            .and_then(Json::as_str),
        Some("buck2 build //:phase0-result-bundle-output-check")
    );
    let coverage_note = subject
        .get("coverage_note")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(coverage_note.contains("structured result bundles"));
    assert!(coverage_note.contains("not P0.0 green"));
}

#[test]
fn good_schema_and_result_fixtures_pass_without_live_claims() {
    let report = evaluate(
        &read_json(schema_path()),
        &read_json(current_red_path()),
        &read_json(false_green_path()),
    );
    assert_eq!(report.verdict, "PASS");
    assert!(report.local_fixture_contract_proven);
    assert!(!report.structured_result_bundle_live);
    assert!(!report.trusted_status_producer_live);
    assert!(!report.protected_branch_authority_proven);
    assert!(!report.status_mutation_performed);
    assert!(!report.live_required_context_execution_proven);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"PASS""#));
    assert!(rendered.contains(r#""local_fixture_contract_proven":true"#));
    assert!(rendered.contains(r#""structured_result_bundle_live":false"#));
    assert!(rendered.contains(r#""trusted_status_producer_live":false"#));
    assert!(rendered.contains(r#""protected_branch_authority_proven":false"#));
    assert!(rendered.contains(r#""status_mutation_performed":false"#));
    assert!(rendered.contains(r#""live_required_context_execution_proven":false"#));
    assert!(rendered.contains("structured result-bundle fixture evidence only; this checker never posts statuses or claims live required-context authority"));
}

#[test]
fn schema_missing_producer_fails_closed() {
    let mut schema = read_json(schema_path());
    object_mut(&mut schema).insert(
        "required".to_string(),
        string_array(&[
            "candidate_sha",
            "required_context",
            "fixture_results",
            "observed_verdict",
            "provenance",
            "claim_boundary",
        ]),
    );
    assert_fails_with(
        evaluate(
            &schema,
            &read_json(current_red_path()),
            &read_json(false_green_path()),
        ),
        "schema.required missing producer",
    );
}

#[test]
fn schema_context_enum_drift_fails_closed() {
    let mut schema = read_json(schema_path());
    let properties = object_mut(
        object_mut(&mut schema)
            .get_mut("properties")
            .expect("properties"),
    );
    let required_context = object_mut(
        properties
            .get_mut("required_context")
            .expect("required_context"),
    );
    array_mut(required_context.get_mut("enum").expect("enum"))
        .push(Json::String("legacy-oya-verify".to_string()));
    assert_fails_with(
        evaluate(
            &schema,
            &read_json(current_red_path()),
            &read_json(false_green_path()),
        ),
        "schema.required_context enum must be exactly",
    );
}

#[test]
fn schema_sha_pattern_missing_fails_closed() {
    let mut schema = read_json(schema_path());
    let properties = object_mut(
        object_mut(&mut schema)
            .get_mut("properties")
            .expect("properties"),
    );
    object_mut(properties.get_mut("candidate_sha").expect("candidate_sha")).remove("pattern");
    assert_fails_with(
        evaluate(
            &schema,
            &read_json(current_red_path()),
            &read_json(false_green_path()),
        ),
        "schema.candidate_sha must require 40 hexadecimal characters",
    );
}

#[test]
fn current_red_extra_top_level_field_fails_closed() {
    let mut current_red = read_json(current_red_path());
    object_mut(&mut current_red).insert(
        "candidate_authored_summary".to_string(),
        Json::String("should fail local schema-shape guard".to_string()),
    );
    assert_fails_with(
        evaluate(
            &read_json(schema_path()),
            &current_red,
            &read_json(false_green_path()),
        ),
        "unexpected top-level fields",
    );
}

#[test]
fn current_red_invalid_sha_fails_closed() {
    let mut current_red = read_json(current_red_path());
    object_mut(&mut current_red).insert(
        "candidate_sha".to_string(),
        Json::String("not-a-sha".to_string()),
    );
    assert_fails_with(
        evaluate(
            &read_json(schema_path()),
            &current_red,
            &read_json(false_green_path()),
        ),
        "candidate_sha must be a 40-character hexadecimal SHA",
    );
}

#[test]
fn current_red_empty_provenance_sources_fails_closed() {
    let mut current_red = read_json(current_red_path());
    let provenance = object_mut(
        object_mut(&mut current_red)
            .get_mut("provenance")
            .expect("provenance"),
    );
    provenance.insert("sources".to_string(), Json::Array(Vec::new()));
    assert_fails_with(
        evaluate(
            &read_json(schema_path()),
            &current_red,
            &read_json(false_green_path()),
        ),
        "provenance.sources must be a non-empty string array",
    );
}

#[test]
fn current_red_claims_green_fails_closed() {
    let mut current_red = read_json(current_red_path());
    let claim_boundary = object_mut(
        object_mut(&mut current_red)
            .get_mut("claim_boundary")
            .expect("claim_boundary"),
    );
    claim_boundary.insert("p0_0_green".to_string(), Json::Bool(true));
    assert_fails_with(
        evaluate(
            &read_json(schema_path()),
            &current_red,
            &read_json(false_green_path()),
        ),
        "current RED result bundle must keep p0_0_green=false and phase0_complete=false",
    );
}

#[test]
fn current_red_live_looking_producer_fails_closed() {
    let mut current_red = read_json(current_red_path());
    let current_red_object = object_mut(&mut current_red);
    current_red_object.insert(
        "required_context".to_string(),
        Json::String("oya-ci-required".to_string()),
    );
    current_red_object.insert(
        "producer".to_string(),
        object(&[
            ("context", Json::String("oya-ci-required".to_string())),
            ("kind", Json::String("oya-ci-controller".to_string())),
            ("trusted_control_state", Json::Bool(true)),
            (
                "candidate_bytes_policy",
                Json::String("untrusted_input_only".to_string()),
            ),
            (
                "gate_definition_source",
                Json::String("trusted_dev_or_controller_state".to_string()),
            ),
        ]),
    );
    assert_fails_with(
        evaluate(
            &read_json(schema_path()),
            &current_red,
            &read_json(false_green_path()),
        ),
        "current RED result bundle must expose missing-context, untrusted-producer, candidate-bytes, and candidate-sourced violations",
    );
}

#[test]
fn false_green_boundary_not_exercised_fails_closed() {
    let mut false_green = read_json(false_green_path());
    object_mut(&mut false_green).insert(
        "claim_boundary".to_string(),
        object(&[
            ("p0_0_green", Json::Bool(false)),
            ("phase0_complete", Json::Bool(false)),
        ]),
    );
    assert_fails_with(
        evaluate(
            &read_json(schema_path()),
            &read_json(current_red_path()),
            &false_green,
        ),
        "false-green result bundle must exercise p0_0_green=true and phase0_complete=true",
    );
}

#[test]
fn current_red_empty_fixture_results_fails_closed() {
    let mut current_red = read_json(current_red_path());
    object_mut(&mut current_red).insert("fixture_results".to_string(), Json::Array(Vec::new()));
    assert_fails_with(
        evaluate(
            &read_json(schema_path()),
            &current_red,
            &read_json(false_green_path()),
        ),
        "current RED result bundle must remain schema-shaped and non-empty",
    );
}

#[test]
fn false_green_fixture_result_matches_red_fails_closed() {
    let mut false_green = read_json(false_green_path());
    let fixtures = array_mut(
        object_mut(&mut false_green)
            .get_mut("fixture_results")
            .expect("fixture_results"),
    );
    object_mut(&mut fixtures[0]).insert(
        "observed_verdict".to_string(),
        Json::String("RED".to_string()),
    );
    assert_fails_with(
        evaluate(
            &read_json(schema_path()),
            &read_json(current_red_path()),
            &false_green,
        ),
        "false-green result bundle must expose all required false-green violation classes",
    );
}

#[test]
fn false_green_red_expected_nonempty_violations_fails_closed() {
    let mut false_green = read_json(false_green_path());
    let fixtures = array_mut(
        object_mut(&mut false_green)
            .get_mut("fixture_results")
            .expect("fixture_results"),
    );
    object_mut(&mut fixtures[0]).insert(
        "violations".to_string(),
        string_array(&["missing_cloud_ci_required_context"]),
    );
    assert_fails_with(
        evaluate(
            &read_json(schema_path()),
            &read_json(current_red_path()),
            &false_green,
        ),
        "false-green result bundle must expose all required false-green violation classes",
    );
}
