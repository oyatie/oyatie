#[allow(dead_code)]
#[path = "../ci/assert-claim-ceiling.rs"]
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

fn claim_map_path() -> &'static str {
    "specs/phase0-claim-evidence-map.json"
}

fn contract_path() -> &'static str {
    "specs/hyperscaler-production-readiness-claim-contract.json"
}

fn fixture_paths(claim_map: &Json) -> Vec<String> {
    claim_map
        .as_object()
        .and_then(|object| object.get("fixture_set"))
        .and_then(Json::as_object)
        .and_then(|fixture_set| fixture_set.get("all_fixture_paths"))
        .and_then(Json::as_array)
        .expect("fixture paths")
        .iter()
        .filter_map(Json::as_str)
        .map(str::to_string)
        .collect()
}

fn default_fixtures(claim_map: &Json) -> Vec<(String, Json)> {
    fixture_paths(claim_map)
        .into_iter()
        .map(|path| {
            let fixture = read_json(&path);
            (path, fixture)
        })
        .collect()
}

fn evaluate(claim_map: &Json, contract: &Json, extra_text: &str) -> checker::Report {
    let fixtures = default_fixtures(claim_map);
    checker::evaluate_sources(
        claim_map,
        contract,
        &fixtures,
        extra_text,
        claim_map_path(),
        contract_path(),
    )
}

fn evaluate_with_fixtures(
    claim_map: &Json,
    contract: &Json,
    fixtures: Vec<(String, Json)>,
) -> checker::Report {
    checker::evaluate_sources(
        claim_map,
        contract,
        &fixtures,
        "",
        claim_map_path(),
        contract_path(),
    )
}

fn row_mut(claim_map: &mut Json, index: usize) -> &mut BTreeMap<String, Json> {
    let rows = array_mut(
        object_mut(claim_map)
            .get_mut("seed_claim_rows")
            .expect("seed claim rows"),
    );
    object_mut(rows.get_mut(index).expect("claim row"))
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
fn matrix_and_coverage_preserve_claim_ceiling_contract() {
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
        .find(|row| row.get("id").and_then(Json::as_str) == Some("AC-0.17-claim-ceiling"))
        .expect("AC-0.17 claim-ceiling row");
    assert_eq!(
        row.get("target_gate_or_controller").and_then(Json::as_str),
        Some("//:phase0-claim-ceiling-check")
    );
    assert_eq!(
        row.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:phase0-claim-ceiling-check")
    );
    let claim_boundary = row
        .get("claim_boundary")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(claim_boundary.contains("not live required-context authority"));
    assert!(claim_boundary.contains("hyperscaler-grade readiness"));
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
        .find(|subject| subject.get("id").and_then(Json::as_str) == Some("AC-0.17"))
        .expect("AC-0.17 coverage subject");
    assert_eq!(
        subject.get("verification_command").and_then(Json::as_str),
        Some("buck2 build //:phase0-claim-ceiling-check")
    );
    let coverage_note = subject
        .get("coverage_note")
        .and_then(Json::as_str)
        .unwrap_or_default();
    assert!(coverage_note.contains("//:phase0-claim-ceiling-check"));
    assert!(coverage_note.contains("not live required-context authority"));
    assert!(coverage_note.contains("hyperscaler-grade readiness"));
}

#[test]
fn good_claim_map_and_fixtures_pass_without_live_claims() {
    let report = evaluate(
        &read_json(claim_map_path()),
        &read_json(contract_path()),
        "",
    );
    assert_eq!(report.verdict, "PASS");
    assert!(report.local_fixture_contract_proven);
    assert!(report.claim_evidence_map_local_static_proven);
    assert!(!report.claim_ceiling_live);
    assert!(!report.protected_branch_authority_proven);
    assert!(!report.status_mutation_performed);
    assert!(!report.live_required_context_execution_proven);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    assert!(!report.production_ready);
    assert!(!report.hyperscaler_grade);
    assert!(
        report
            .fixture_results
            .iter()
            .any(|fixture| fixture.fixture_id == "TC-0.17-BAD-local-oya-authority-claim")
    );
    assert!(report.claim_map_summary.violations.is_empty());
    assert!(report.claim_map_summary.row_count >= 6);
    assert!(report.claim_map_summary.regulated_vocabulary_count >= 20);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"PASS""#));
    assert!(rendered.contains(r#""local_fixture_contract_proven":true"#));
    assert!(rendered.contains(r#""claim_ceiling_live":false"#));
    assert!(rendered.contains(r#""protected_branch_authority_proven":false"#));
    assert!(rendered.contains(r#""status_mutation_performed":false"#));
    assert!(rendered.contains(r#""live_required_context_execution_proven":false"#));
    assert!(rendered.contains(r#""p0_0_green":false"#));
    assert!(rendered.contains(r#""phase0_complete":false"#));
    assert!(rendered.contains(r#""production_ready":false"#));
    assert!(rendered.contains(r#""hyperscaler_grade":false"#));
    assert!(rendered.contains("TC-0.17-BAD-local-oya-authority-claim"));
}

#[test]
fn missing_claim_row_fails_closed() {
    let mut claim_map = read_json(claim_map_path());
    object_mut(&mut claim_map).insert("seed_claim_rows".to_string(), Json::Array(Vec::new()));
    assert_fails_with(
        evaluate(
            &claim_map,
            &read_json(contract_path()),
            "The platform is production-ready and secure.",
        ),
        "regulated_vocabulary_without_claim_row",
    );
}

#[test]
fn unknown_tier_and_missing_owner_fail_closed() {
    let mut claim_map = read_json(claim_map_path());
    let row = row_mut(&mut claim_map, 0);
    row.insert(
        "claim_tier".to_string(),
        Json::String("aspirational_ready".to_string()),
    );
    row.insert("owner".to_string(), Json::String(String::new()));
    let report = evaluate(&claim_map, &read_json(contract_path()), "");
    assert_fails_with(report.clone(), "unknown_claim_tier");
    assert_fails_with(report, "missing_or_empty_required_field");
}

#[test]
fn mechanical_local_oya_evidence_fails_closed() {
    let mut claim_map = read_json(claim_map_path());
    let row = row_mut(&mut claim_map, 1);
    row.insert(
        "claim_tier".to_string(),
        Json::String("mechanically_enforced".to_string()),
    );
    row.insert(
        "claim_text".to_string(),
        Json::String("mechanically enforced by oya verify --ci-required".to_string()),
    );
    row.insert(
        "allowed_language_now".to_string(),
        Json::String("mechanically enforced".to_string()),
    );
    row.insert(
        "current_evidence".to_string(),
        string_array(&["local-only command", "legacy oya CLI invocation"]),
    );
    row.insert("missing_for_next_tier".to_string(), Json::Array(Vec::new()));
    assert_fails_with(
        evaluate(&claim_map, &read_json(contract_path()), ""),
        "forbidden_local_or_oya_evidence_for_mechanical_claim",
    );
}

#[test]
fn production_ready_without_budget_or_result_fails_closed() {
    let mut claim_map = read_json(claim_map_path());
    let last_index = object_mut(&mut claim_map)
        .get("seed_claim_rows")
        .and_then(Json::as_array)
        .expect("rows")
        .len()
        - 1;
    let row = row_mut(&mut claim_map, last_index);
    row.insert(
        "claim_tier".to_string(),
        Json::String("production_ready".to_string()),
    );
    row.insert(
        "allowed_language_now".to_string(),
        Json::String("production-ready".to_string()),
    );
    row.insert(
        "current_evidence".to_string(),
        string_array(&["advisory benchmark planned"]),
    );
    row.insert("missing_for_next_tier".to_string(), Json::Array(Vec::new()));
    assert_fails_with(
        evaluate(&claim_map, &read_json(contract_path()), ""),
        "performance_claim_without_budget_or_measured_result",
    );
}

#[test]
fn production_ready_budget_only_fails_closed() {
    let mut claim_map = read_json(claim_map_path());
    let last_index = object_mut(&mut claim_map)
        .get("seed_claim_rows")
        .and_then(Json::as_array)
        .expect("rows")
        .len()
        - 1;
    let row = row_mut(&mut claim_map, last_index);
    row.insert(
        "claim_tier".to_string(),
        Json::String("production_ready".to_string()),
    );
    row.insert(
        "allowed_language_now".to_string(),
        Json::String("production-ready".to_string()),
    );
    row.insert(
        "current_evidence".to_string(),
        string_array(&["PERF-CAPACITY", "performance_budget", "p95 budget declared"]),
    );
    row.insert("missing_for_next_tier".to_string(), Json::Array(Vec::new()));
    assert_fails_with(
        evaluate(&claim_map, &read_json(contract_path()), ""),
        "performance_claim_without_budget_or_measured_result",
    );
}

#[test]
fn production_ready_domain_only_fails_closed() {
    let mut claim_map = read_json(claim_map_path());
    let row = row_mut(&mut claim_map, 3);
    row.insert(
        "claim_tier".to_string(),
        Json::String("production_ready".to_string()),
    );
    row.insert(
        "claim_text".to_string(),
        Json::String("secure control plane".to_string()),
    );
    row.insert(
        "allowed_language_now".to_string(),
        Json::String("secure".to_string()),
    );
    row.insert("regulated_terms".to_string(), string_array(&["secure"]));
    row.insert(
        "current_evidence".to_string(),
        string_array(&["security review planned"]),
    );
    row.insert(
        "missing_for_next_tier".to_string(),
        string_array(&["budget plus measured-result evidence missing"]),
    );
    assert_fails_with(
        evaluate(&claim_map, &read_json(contract_path()), ""),
        "performance_claim_without_budget_or_measured_result",
    );
}

#[test]
fn hyperscaler_tier_only_fails_closed() {
    let mut claim_map = read_json(claim_map_path());
    let row = row_mut(&mut claim_map, 3);
    row.insert(
        "claim_tier".to_string(),
        Json::String("hyperscaler_grade".to_string()),
    );
    row.insert(
        "claim_text".to_string(),
        Json::String("secure control plane".to_string()),
    );
    row.insert(
        "allowed_language_now".to_string(),
        Json::String("secure".to_string()),
    );
    row.insert("regulated_terms".to_string(), string_array(&["secure"]));
    row.insert(
        "current_evidence".to_string(),
        string_array(&["security review planned"]),
    );
    row.insert(
        "missing_for_next_tier".to_string(),
        string_array(&["budget plus measured-result evidence missing"]),
    );
    assert_fails_with(
        evaluate(&claim_map, &read_json(contract_path()), ""),
        "performance_claim_without_budget_or_measured_result",
    );
}

#[test]
fn capacity_breakpoint_only_fails_closed() {
    let mut claim_map = read_json(claim_map_path());
    let last_index = object_mut(&mut claim_map)
        .get("seed_claim_rows")
        .and_then(Json::as_array)
        .expect("rows")
        .len()
        - 1;
    let row = row_mut(&mut claim_map, last_index);
    row.insert(
        "claim_tier".to_string(),
        Json::String("production_ready".to_string()),
    );
    row.insert(
        "allowed_language_now".to_string(),
        Json::String("production-ready".to_string()),
    );
    row.insert(
        "current_evidence".to_string(),
        string_array(&["capacity breakpoint"]),
    );
    row.insert(
        "missing_for_next_tier".to_string(),
        string_array(&["budget evidence missing"]),
    );
    assert_fails_with(
        evaluate(&claim_map, &read_json(contract_path()), ""),
        "performance_claim_without_budget_or_measured_result",
    );
}

#[test]
fn combined_budget_and_result_entry_fails_closed() {
    let mut claim_map = read_json(claim_map_path());
    let last_index = object_mut(&mut claim_map)
        .get("seed_claim_rows")
        .and_then(Json::as_array)
        .expect("rows")
        .len()
        - 1;
    let row = row_mut(&mut claim_map, last_index);
    row.insert(
        "claim_tier".to_string(),
        Json::String("production_ready".to_string()),
    );
    row.insert(
        "allowed_language_now".to_string(),
        Json::String("production-ready".to_string()),
    );
    row.insert(
        "current_evidence".to_string(),
        string_array(&["p95 budget and load result"]),
    );
    row.insert(
        "missing_for_next_tier".to_string(),
        string_array(&[
            "separate budget evidence entry and separate measured-result evidence entry required",
        ]),
    );
    assert_fails_with(
        evaluate(&claim_map, &read_json(contract_path()), ""),
        "performance_claim_without_budget_or_measured_result",
    );
}

#[test]
fn unknown_regulated_term_fails_closed() {
    let mut claim_map = read_json(claim_map_path());
    row_mut(&mut claim_map, 0).insert("regulated_terms".to_string(), string_array(&["magic-fast"]));
    assert_fails_with(
        evaluate(&claim_map, &read_json(contract_path()), ""),
        "unknown_regulated_term",
    );
}

#[test]
fn red_fixture_made_good_fails_closed() {
    let mut fixture = read_json(
        "specs/fixtures/phase0-claim-ceiling/tc-0.17-bad-ungrounded-production-ready.json",
    );
    let object = object_mut(&mut fixture);
    object.insert(
        "text".to_string(),
        Json::String("Claim ceiling target/non-claim only.".to_string()),
    );
    let mut repaired_row = BTreeMap::new();
    repaired_row.insert(
        "id".to_string(),
        Json::String("GOOD-repaired-row".to_string()),
    );
    repaired_row.insert(
        "artifact".to_string(),
        Json::String("in-memory-red-fixture-made-good".to_string()),
    );
    repaired_row.insert(
        "claim_text".to_string(),
        Json::String("target/non-claim only".to_string()),
    );
    repaired_row.insert(
        "claim_tier".to_string(),
        Json::String("target_non_claim".to_string()),
    );
    repaired_row.insert(
        "allowed_language_now".to_string(),
        Json::String("target/non-claim only".to_string()),
    );
    repaired_row.insert(
        "regulated_terms".to_string(),
        string_array(&["hyperscaler-grade"]),
    );
    repaired_row.insert(
        "current_evidence".to_string(),
        string_array(&[
            "owner",
            "phase",
            "source_decision",
            "specific_gap_list",
            "blocking_path_to_next_tier",
        ]),
    );
    repaired_row.insert(
        "missing_for_next_tier".to_string(),
        string_array(&["live evidence"]),
    );
    repaired_row.insert(
        "owner".to_string(),
        Json::String("platform-sre".to_string()),
    );
    object.insert(
        "claim_rows".to_string(),
        Json::Array(vec![Json::Object(repaired_row)]),
    );
    assert_fails_with(
        evaluate_with_fixtures(
            &read_json(claim_map_path()),
            &read_json(contract_path()),
            vec![("in-memory-red-fixture-made-good".to_string(), fixture)],
        ),
        "RED claim-ceiling fixture must produce violations",
    );
}
