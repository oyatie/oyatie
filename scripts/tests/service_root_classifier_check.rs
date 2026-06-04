#[allow(dead_code)]
#[path = "../ci/assert-service-root-classifier.rs"]
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

fn inventory_path() -> &'static str {
    "specs/service-inventory.json"
}

fn packets_path() -> &'static str {
    "specs/phase0-structural-packets.json"
}

fn fixtures_dir() -> &'static str {
    "specs/fixtures/phase0-service-root-classifier"
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
            let display = path
                .strip_prefix(repo_root())
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            (
                display,
                read_json(path.strip_prefix(repo_root()).unwrap().to_str().unwrap()),
            )
        })
        .collect()
}

fn evaluate(inventory: &Json, packets: &Json) -> checker::Report {
    checker::evaluate_sources(&repo_root(), inventory, packets, &default_fixtures())
}

fn evaluate_with_fixtures(
    inventory: &Json,
    packets: &Json,
    fixtures: Vec<(String, Json)>,
) -> checker::Report {
    checker::evaluate_sources(&repo_root(), inventory, packets, &fixtures)
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
    assert!(!report.service_root_classifier_measured);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"FAIL""#));
    assert!(rendered.contains(r#""p0_0_green":false"#));
    assert!(rendered.contains(r#""phase0_complete":false"#));
}

#[test]
fn matrix_and_coverage_preserve_service_root_contract() {
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
        .find(|row| row.get("id").and_then(Json::as_str) == Some("AC-0.1-service-inventory"))
        .expect("AC-0.1 service-inventory row");
    assert_eq!(
        row.get("target_gate_or_controller").and_then(Json::as_str),
        Some("//:service-root-classifier-check")
    );
    assert_eq!(
        row.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:service-root-classifier-check")
    );
    let claim_boundary = row
        .get("claim_boundary")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(claim_boundary.contains("full nested crate coverage"));
    assert!(claim_boundary.contains("not live required-context authority"));
    assert!(claim_boundary.contains("post-migration pure split"));
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
        .find(|subject| subject.get("id").and_then(Json::as_str) == Some("AC-0.1"))
        .expect("AC-0.1 coverage subject");
    assert_eq!(
        subject.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:service-root-classifier-check")
    );
    let coverage_note = subject
        .get("coverage_note")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(coverage_note.contains("//:service-root-classifier-check"));
    assert!(coverage_note.contains("full nested crate coverage"));
    assert!(coverage_note.contains("live required-context authority false"));

    for (row_id, claim) in [
        ("P0.6-pack-root-classifier", "authorized shared roots"),
        ("AC-0.7-service-layout-sprawl", "post-migration pure split"),
    ] {
        let row = rows
            .iter()
            .filter_map(Json::as_object)
            .find(|row| row.get("id").and_then(Json::as_str) == Some(row_id))
            .expect("service-root related row");
        assert_eq!(
            row.get("target_gate_or_controller").and_then(Json::as_str),
            Some("//:service-root-classifier-check")
        );
        assert_eq!(
            row.get("verification_command").and_then(Json::as_str),
            Some("buck2 build //:service-root-classifier-check")
        );
        let boundary = row
            .get("claim_boundary")
            .and_then(Json::as_str)
            .unwrap_or_default();
        assert!(boundary.contains(claim));
        assert!(boundary.contains("not live required-context authority"));
        assert_eq!(
            row.get("no_new_oya_cli_surface").and_then(Json::as_bool),
            Some(true)
        );
    }

    for (subject_id, row_id, claim) in [
        (
            "P0.6-pack-root-classifier",
            "P0.6-pack-root-classifier",
            "authorized shared roots",
        ),
        (
            "AC-0.7",
            "AC-0.7-service-layout-sprawl",
            "post-migration pure split remains false",
        ),
    ] {
        let subject = subjects
            .iter()
            .filter_map(Json::as_object)
            .find(|subject| subject.get("id").and_then(Json::as_str) == Some(subject_id))
            .expect("service-root coverage subject");
        let mapped = subject
            .get("mapped_row_ids")
            .and_then(Json::as_array)
            .expect("mapped rows");
        assert!(mapped.iter().any(|item| item.as_str() == Some(row_id)));
        assert_eq!(
            subject.get("verification_command").and_then(Json::as_str),
            Some("buck2 build //:service-root-classifier-check")
        );
        let note = subject
            .get("coverage_note")
            .and_then(Json::as_str)
            .unwrap_or_default();
        assert!(note.contains("//:service-root-classifier-check"));
        assert!(note.contains(claim));
        assert!(note.contains("live required-context authority"));
    }
}

#[test]
fn good_inventory_packets_and_fixtures_pass_without_live_claims() {
    let report = evaluate(&read_json(inventory_path()), &read_json(packets_path()));
    assert_eq!(report.verdict, "PASS");
    assert!(report.service_inventory_published);
    assert!(report.service_root_classifier_measured);
    assert!(report.closed_world_root_classifier_measured);
    assert_eq!(report.inventory_summary.closed_world_root_count, 8);
    assert_eq!(report.fixture_count, 8);
    assert_eq!(report.expected_green_fixture_count, 1);
    assert_eq!(report.expected_red_fixture_count, 7);
    assert_eq!(
        report.inventory_summary.inventory_entry_count,
        report.inventory_summary.observed_direct_child_dir_count
    );
    assert!(report.inventory_summary.inventory_entry_count >= 250);
    assert!(report.packet_summary.structural_packet_count >= 6);
    assert_eq!(
        report.fixture_count,
        report.expected_green_fixture_count + report.expected_red_fixture_count
    );
    assert!(
        report
            .fixture_results
            .iter()
            .flat_map(|fixture| fixture.observed_violations.iter())
            .any(|violation| violation == "service_inventory_entry_missing")
    );
    assert!(
        report
            .fixture_results
            .iter()
            .flat_map(|fixture| fixture.observed_violations.iter())
            .any(|violation| violation == "service_layout_sprawl")
    );
    assert!(
        report
            .fixture_results
            .iter()
            .flat_map(|fixture| fixture.observed_violations.iter())
            .any(|violation| violation == "service_root_outside_closed_world")
    );
    assert!(
        report
            .fixture_results
            .iter()
            .flat_map(|fixture| fixture.observed_violations.iter())
            .any(|violation| violation == "retired_real_token_live_field")
    );
    assert!(
        report
            .fixture_results
            .iter()
            .flat_map(|fixture| fixture.observed_violations.iter())
            .any(|violation| violation == "structural_packet_missing_required_family")
    );
    assert!(
        report
            .fixture_results
            .iter()
            .flat_map(|fixture| fixture.observed_violations.iter())
            .any(|violation| violation == "duplicate_service_across_roots")
    );
    assert!(
        report
            .fixture_results
            .iter()
            .flat_map(|fixture| fixture.observed_violations.iter())
            .any(|violation| violation == "underscore_crate_name")
    );
    assert!(!report.full_service_inventory_coverage_proven);
    assert!(!report.post_migration_pure_split_proven);
    assert!(!report.structural_shards_executed);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    assert!(report.failures.is_empty());
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"PASS""#));
    assert!(rendered.contains(r#""service_inventory_published":true"#));
    assert!(rendered.contains(r#""service_root_classifier_measured":true"#));
    assert!(rendered.contains(r#""full_service_inventory_coverage_proven":false"#));
    assert!(rendered.contains(r#""post_migration_pure_split_proven":false"#));
    assert!(rendered.contains(r#""p0_0_green":false"#));
    assert!(rendered.contains(r#""phase0_complete":false"#));
}

#[test]
fn p0_green_claim_fails_closed() {
    let mut inventory = read_json(inventory_path());
    let boundary = object_mut(
        object_mut(&mut inventory)
            .get_mut("claim_boundary")
            .expect("claim boundary"),
    );
    boundary.insert("p0_0_green".to_string(), Json::Bool(true));
    assert_fails_with(
        evaluate(&inventory, &read_json(packets_path())),
        "forbidden_true_or_missing_claim_p0_0_green",
    );
}

#[test]
fn missing_inventory_entry_fails_closed() {
    let mut inventory = read_json(inventory_path());
    let rows = array_mut(
        object_mut(&mut inventory)
            .get_mut("inventory_entries")
            .expect("inventory entries"),
    );
    let removed = rows[0]
        .as_object()
        .and_then(|row| row.get("source_path"))
        .and_then(Json::as_str)
        .expect("source path")
        .to_string();
    rows.retain(|entry| {
        entry
            .as_object()
            .and_then(|row| row.get("source_path"))
            .and_then(Json::as_str)
            != Some(removed.as_str())
    });
    assert_fails_with(
        evaluate(&inventory, &read_json(packets_path())),
        "service_inventory_entry_missing",
    );
}

#[test]
fn missing_structural_packet_family_fails_closed() {
    let mut packets = read_json(packets_path());
    let rows = array_mut(
        object_mut(&mut packets)
            .get_mut("structural_packets")
            .expect("structural packets"),
    );
    rows.retain(|packet| {
        !packet
            .as_object()
            .and_then(|packet| packet.get("packet_id"))
            .and_then(Json::as_str)
            .unwrap_or_default()
            .starts_with("P0.6d-BNF-")
    });
    assert_fails_with(
        evaluate(&read_json(inventory_path()), &packets),
        "structural_packet_missing_required_family",
    );
}

#[test]
fn red_fixture_made_clean_fails_closed() {
    let mut fixture = read_json(
        "specs/fixtures/phase0-service-root-classifier/tc-service-root-bad-layout-sprawl.json",
    );
    let object = object_mut(&mut fixture);
    let candidates = array_mut(object.get_mut("candidate_paths").expect("candidate paths"));
    let first = object_mut(candidates.get_mut(0).expect("first candidate"));
    first.insert("path".to_string(), Json::String("oya/payments".to_string()));
    first.insert(
        "crate_name".to_string(),
        Json::String("payments".to_string()),
    );
    object.insert(
        "inventory_entry_paths".to_string(),
        string_array(&["oya/payments"]),
    );
    assert_fails_with(
        evaluate_with_fixtures(
            &read_json(inventory_path()),
            &read_json(packets_path()),
            vec![("in-memory-red-made-clean".to_string(), fixture)],
        ),
        "RED service-root fixture must produce violations",
    );
}

#[test]
fn green_fixture_sprawl_and_real_token_fails_closed() {
    let mut fixture =
        read_json("specs/fixtures/phase0-service-root-classifier/tc-service-root-good-seed.json");
    let object = object_mut(&mut fixture);
    let candidates = array_mut(object.get_mut("candidate_paths").expect("candidate paths"));
    let first = object_mut(candidates.get_mut(0).expect("first candidate"));
    first.insert(
        "path".to_string(),
        Json::String("platforms/accounting".to_string()),
    );
    let paths = array_mut(
        object
            .get_mut("inventory_entry_paths")
            .expect("inventory entry paths"),
    );
    paths[0] = Json::String("platforms/accounting".to_string());
    let live_status = object_mut(
        object
            .get_mut("live_status_fields")
            .expect("live status fields"),
    );
    live_status.insert(
        "maturity_status".to_string(),
        Json::String("REAL".to_string()),
    );
    assert_fails_with(
        evaluate_with_fixtures(
            &read_json(inventory_path()),
            &read_json(packets_path()),
            vec![("in-memory-good-sprawl-real".to_string(), fixture)],
        ),
        "GREEN service-root fixture produced violations",
    );
}
