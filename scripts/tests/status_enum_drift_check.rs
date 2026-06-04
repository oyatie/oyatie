#[allow(dead_code)]
#[path = "../ci/assert-status-enum-drift.rs"]
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

fn registry_path() -> &'static str {
    "specs/status-enum-registry.json"
}

fn fixtures_dir() -> &'static str {
    "specs/fixtures/phase0-status-enum-drift"
}

fn default_fixtures() -> Vec<(String, Json)> {
    let mut paths = fs::read_dir(repo_root().join(fixtures_dir()))
        .expect("fixtures dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let relative = path
                .strip_prefix(repo_root())
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let fixture = read_json(&relative);
            (relative, fixture)
        })
        .collect()
}

fn evaluate(registry: &Json) -> checker::Report {
    checker::evaluate_sources(&repo_root(), registry, &default_fixtures())
}

fn evaluate_with_fixtures(registry: &Json, fixtures: Vec<(String, Json)>) -> checker::Report {
    checker::evaluate_sources(&repo_root(), registry, &fixtures)
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
    assert!(!report.status_drift_fixture_contract_measured);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"FAIL""#));
    assert!(rendered.contains(r#""p0_0_green":false"#));
    assert!(rendered.contains(r#""phase0_complete":false"#));
}

#[test]
fn matrix_and_coverage_preserve_status_enum_contract() {
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
        .find(|row| row.get("id").and_then(Json::as_str) == Some("AC-0.2-status-enum-drift"))
        .expect("AC-0.2 status-enum row");
    assert_eq!(
        row.get("target_gate_or_controller").and_then(Json::as_str),
        Some("//:status-enum-drift-check")
    );
    assert_eq!(
        row.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:status-enum-drift-check")
    );
    let claim_boundary = row
        .get("claim_boundary")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(claim_boundary.contains("full manifest/PRD conformance"));
    assert!(claim_boundary.contains("not live required-context authority"));
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
        .find(|subject| subject.get("id").and_then(Json::as_str) == Some("AC-0.2"))
        .expect("AC-0.2 coverage subject");
    assert_eq!(
        subject.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:status-enum-drift-check")
    );
    let coverage_note = subject
        .get("coverage_note")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(coverage_note.contains("//:status-enum-drift-check"));
    assert!(coverage_note.contains("full manifest/PRD conformance"));
    assert!(coverage_note.contains("live required-context authority remain false"));
}

#[test]
fn good_registry_and_fixtures_pass_without_live_claims() {
    let report = evaluate(&read_json(registry_path()));
    assert_eq!(report.verdict, "PASS");
    assert!(report.status_enum_registry_published);
    assert!(report.status_drift_fixture_contract_measured);
    assert_eq!(report.registry_summary.axis_count, 3);
    assert!(report.registry_summary.allowed_value_count >= 15);
    assert_eq!(report.fixture_count, 5);
    assert_eq!(report.expected_green_fixture_count, 1);
    assert_eq!(report.expected_red_fixture_count, 4);
    assert_eq!(
        report.fixture_count,
        report.expected_green_fixture_count + report.expected_red_fixture_count
    );
    for expected in [
        "invalid_status_enum_value",
        "retired_real_token_live_field",
        "spec_code_manifest_mismatch",
        "status_drift_mismatch",
    ] {
        assert!(
            report
                .fixture_results
                .iter()
                .flat_map(|fixture| fixture.observed_violations.iter())
                .any(|violation| violation == expected),
            "missing observed violation {expected}"
        );
    }
    assert!(!report.full_manifest_prd_conformance_proven);
    assert!(!report.status_drift_live_gate_proven);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    assert!(report.failures.is_empty());
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"PASS""#));
    assert!(rendered.contains(r#""status_enum_registry_published":true"#));
    assert!(rendered.contains(r#""status_drift_fixture_contract_measured":true"#));
    assert!(rendered.contains(r#""full_manifest_prd_conformance_proven":false"#));
    assert!(rendered.contains(r#""status_drift_live_gate_proven":false"#));
    assert!(rendered.contains(r#""p0_0_green":false"#));
    assert!(rendered.contains(r#""phase0_complete":false"#));
}

#[test]
fn p0_green_claim_fails_closed() {
    let mut registry = read_json(registry_path());
    let boundary = object_mut(
        object_mut(&mut registry)
            .get_mut("claim_boundary")
            .expect("claim boundary"),
    );
    boundary.insert("p0_0_green".to_string(), Json::Bool(true));
    assert_fails_with(
        evaluate(&registry),
        "forbidden_true_or_missing_claim_p0_0_green",
    );
}

#[test]
fn retired_real_allowed_value_fails_closed() {
    let mut registry = read_json(registry_path());
    let axes = object_mut(object_mut(&mut registry).get_mut("axes").expect("axes"));
    let maturity = object_mut(axes.get_mut("maturity").expect("maturity axis"));
    array_mut(maturity.get_mut("allowed_values").expect("allowed values"))
        .push(Json::String("REAL".to_string()));
    assert_fails_with(evaluate(&registry), "retired_real_token_allowed:maturity");
}

#[test]
fn red_fixture_made_clean_fails_closed() {
    let mut fixture =
        read_json("specs/fixtures/phase0-status-enum-drift/tc-status-enum-bad-status-drift.json");
    let object = object_mut(&mut fixture);
    let pairs = array_mut(object.get_mut("spec_manifest_pairs").expect("pairs"));
    let first = object_mut(pairs.get_mut(0).expect("first pair"));
    let spec_status = first
        .get("spec_status_fields")
        .expect("spec status")
        .clone();
    first.insert("manifest_status_fields".to_string(), spec_status);
    assert_fails_with(
        evaluate_with_fixtures(
            &read_json(registry_path()),
            vec![("in-memory-red-made-clean".to_string(), fixture)],
        ),
        "RED status-enum fixture must produce violations",
    );
}

#[test]
fn green_fixture_invalid_status_fails_closed() {
    let mut fixture =
        read_json("specs/fixtures/phase0-status-enum-drift/tc-status-enum-good-aligned.json");
    let object = object_mut(&mut fixture);
    let fields = object_mut(object.get_mut("status_fields").expect("status fields"));
    fields.insert(
        "maturity_status".to_string(),
        Json::String("REAL".to_string()),
    );
    assert_fails_with(
        evaluate_with_fixtures(
            &read_json(registry_path()),
            vec![("in-memory-good-invalid-status".to_string(), fixture)],
        ),
        "GREEN status-enum fixture produced violations",
    );
}
