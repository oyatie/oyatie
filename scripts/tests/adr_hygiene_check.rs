#[allow(dead_code)]
#[path = "../ci/assert-adr-hygiene.rs"]
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
    "specs/adr-hygiene-registry.json"
}

fn fixtures_dir() -> &'static str {
    "specs/fixtures/phase0-adr-hygiene"
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
    assert!(!report.adr_hygiene_fixture_contract_measured);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"FAIL""#));
    assert!(rendered.contains(r#""p0_0_green":false"#));
    assert!(rendered.contains(r#""phase0_complete":false"#));
}

#[test]
fn matrix_and_coverage_preserve_adr_hygiene_contract() {
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
        .find(|row| row.get("id").and_then(Json::as_str) == Some("AC-0.3-adr-hygiene"))
        .expect("AC-0.3 ADR hygiene row");
    assert_eq!(
        row.get("target_gate_or_controller").and_then(Json::as_str),
        Some("//:adr-hygiene-check")
    );
    assert_eq!(
        row.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:adr-hygiene-check")
    );
    let claim_boundary = row
        .get("claim_boundary")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(claim_boundary.contains("full ADR index regeneration"));
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
        .find(|subject| subject.get("id").and_then(Json::as_str) == Some("AC-0.3"))
        .expect("AC-0.3 coverage subject");
    assert_eq!(
        subject.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:adr-hygiene-check")
    );
    let coverage_note = subject
        .get("coverage_note")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(coverage_note.contains("//:adr-hygiene-check"));
    assert!(coverage_note.contains("full ADR index regeneration"));
    assert!(coverage_note.contains("live required-context authority remain false"));
}

#[test]
fn checked_in_contract_passes() {
    let report = evaluate(&read_json(registry_path()));
    assert_eq!(report.verdict, "PASS", "{:?}", report.failures);
    assert!(report.adr_hygiene_registry_published);
    assert!(report.adr_hygiene_fixture_contract_measured);
    assert_eq!(report.fixture_count, 4);
    assert_eq!(report.expected_green_fixture_count, 1);
    assert_eq!(report.expected_red_fixture_count, 3);
    assert!(report.registry_summary.decision_record_count >= 300);
    assert!(report.registry_summary.active_doc_scan_count >= 10);
    assert_eq!(
        report.registry_summary.superseded_reference_pattern_count,
        3
    );
    assert!(report.failures.is_empty());
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"PASS""#));
    assert!(rendered.contains(r#""adr_hygiene_registry_published":true"#));
    assert!(rendered.contains(r#""adr_hygiene_fixture_contract_measured":true"#));
    assert!(rendered.contains("duplicate_adr_number"));
    assert!(rendered.contains("adr_0511_missing_superseded_by_adr_0513"));
    assert!(rendered.contains("superseded_reference_active_doc"));
    assert!(rendered.contains(r#""full_adr_index_regenerated":false"#));
    assert!(rendered.contains(r#""p0_0_green":false"#));
    assert!(rendered.contains(r#""phase0_complete":false"#));
}

#[test]
fn live_decision_files_preserve_renumbering_and_supersession_contract() {
    assert!(
        repo_root()
            .join("docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md")
            .is_file()
    );
    assert!(
        !repo_root()
            .join("docs/decisions/ADR-0377-kafka-to-pulsar-via-kop.md")
            .exists()
    );
    let adr_0520 =
        fs::read_to_string(repo_root().join("docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md"))
            .expect("ADR-0520");
    assert!(adr_0520.contains("id: ADR-0520"));
    assert!(adr_0520.contains("renumbered_from: ADR-0377"));
    let adr_0511 = fs::read_to_string(
        repo_root()
            .join("docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md"),
    )
    .expect("ADR-0511");
    assert!(adr_0511.contains("superseded_by: [ADR-0513]"));
}

#[test]
fn p0_green_claim_fails() {
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
fn red_fixture_made_clean_fails() {
    let registry = read_json(registry_path());
    let mut fixture =
        read_json("specs/fixtures/phase0-adr-hygiene/tc-adr-hygiene-bad-duplicate-adr-number.json");
    let records = array_mut(
        object_mut(&mut fixture)
            .get_mut("decision_records")
            .expect("records"),
    );
    let second = object_mut(records.get_mut(1).expect("second record"));
    second.insert("id".to_string(), Json::String("ADR-0520".to_string()));
    second.insert(
        "path".to_string(),
        Json::String("docs/decisions/ADR-0520-kafka-to-pulsar-via-kop.md".to_string()),
    );
    second.insert(
        "renumbered_from".to_string(),
        Json::String("ADR-0377".to_string()),
    );
    assert_fails_with(
        evaluate_with_fixtures(
            &registry,
            vec![("bad-red-made-clean.json".to_string(), fixture)],
        ),
        "RED ADR hygiene fixture must produce violations",
    );
}

#[test]
fn green_fixture_with_duplicate_fails() {
    let registry = read_json(registry_path());
    let mut fixture = read_json(
        "specs/fixtures/phase0-adr-hygiene/tc-adr-hygiene-good-renumbered-superseded-clean.json",
    );
    let records = array_mut(
        object_mut(&mut fixture)
            .get_mut("decision_records")
            .expect("records"),
    );
    object_mut(records.get_mut(1).expect("second record"))
        .insert("id".to_string(), Json::String("ADR-0377".to_string()));
    assert_fails_with(
        evaluate_with_fixtures(
            &registry,
            vec![("bad-good-duplicate.json".to_string(), fixture)],
        ),
        "GREEN ADR hygiene fixture produced violations",
    );
}

#[test]
fn green_fixture_with_stale_active_reference_fails() {
    let registry = read_json(registry_path());
    let mut fixture = read_json(
        "specs/fixtures/phase0-adr-hygiene/tc-adr-hygiene-good-renumbered-superseded-clean.json",
    );
    let docs = array_mut(
        object_mut(&mut fixture)
            .get_mut("active_documents")
            .expect("active docs"),
    );
    object_mut(docs.get_mut(0).expect("doc")).insert(
        "content".to_string(),
        Json::String("VictoriaMetrics for metrics remains canonical.".to_string()),
    );
    assert_fails_with(
        evaluate_with_fixtures(
            &registry,
            vec![("bad-good-stale-reference.json".to_string(), fixture)],
        ),
        "GREEN ADR hygiene fixture produced violations",
    );
}
