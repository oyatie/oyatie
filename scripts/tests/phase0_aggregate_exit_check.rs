#[allow(dead_code)]
#[path = "../ci/assert-phase0-aggregate-exit.rs"]
mod checker;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap())
}

fn temp_path(label: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "oya-phase0-aggregate-exit-{label}-{}-{nanos}-{counter}.json",
        std::process::id()
    ));
    path
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn fixture(path: &str) -> PathBuf {
    repo_root().join(path)
}

fn evaluate_default() -> checker::Evaluation {
    checker::evaluate(&repo_root(), &[])
}

fn evaluate_fixture(path: &Path) -> checker::Evaluation {
    checker::evaluate(&repo_root(), &[path.to_path_buf()])
}

fn replace_first(text: &str, old: &str, new: &str) -> String {
    assert!(text.contains(old), "mutation source not found: {old}");
    text.replacen(old, new, 1)
}

fn assert_fails_with(label: &str, expected: &str, path: &Path) {
    let evaluation = evaluate_fixture(path);
    assert_eq!(evaluation.verdict, "FAIL", "{label} should fail");
    let joined = evaluation.failures.join("\n");
    assert!(
        joined.contains(expected),
        "{label} missing {expected:?}:\n{joined}"
    );
    let json = checker::to_json(&evaluation);
    assert!(json.contains(r#""p0_0_green":false"#));
    assert!(json.contains(r#""phase0_complete":false"#));
}

#[test]
fn automation_matrix_and_coverage_rows_map_to_buck2_target() {
    let matrix = read_repo_file("specs/phase0-automation-matrix.json");
    let coverage = read_repo_file("specs/phase0-automation-coverage-registry.json");
    assert!(matrix.contains(r#""id": "AC-0.12-aggregate-exit-gate""#));
    assert!(matrix.contains("cloud-ci-phase0-aggregate-exit"));
    assert!(matrix.contains("//:phase0-aggregate-exit-check"));
    assert!(
        matrix.contains(r#""verification_command": "buck2 build //:phase0-aggregate-exit-check""#)
    );
    assert!(matrix.contains("not live required-context authority"));
    assert!(matrix.contains("not P0.0 green"));
    assert!(matrix.contains(r#""no_new_oya_cli_surface": true"#));

    assert!(coverage.contains(r#""id": "AC-0.12""#));
    assert!(coverage.contains("AC-0.12-aggregate-exit-gate"));
    assert!(
        coverage
            .contains(r#""verification_command": "buck2 build //:phase0-aggregate-exit-check""#)
    );
    assert!(coverage.contains("//:phase0-aggregate-exit-check"));
    assert!(coverage.contains("cloud-ci-phase0-aggregate-exit"));
    assert!(coverage.contains("not Phase-0 completion"));
}

#[test]
fn checked_in_fixture_contract_passes() {
    let evaluation = evaluate_default();
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.required_subcondition_count, 32);
    assert_eq!(evaluation.single_false_case_count, 32);
    assert_eq!(evaluation.fixture_results.len(), 4);
    assert!(evaluation.failures.is_empty());
    assert!(evaluation.local_fixture_contract_proven);
    assert!(!evaluation.aggregate_exit_live);
    assert!(!evaluation.live_required_context_execution_proven);
    assert!(!evaluation.p0_0_green);
    assert!(!evaluation.phase0_complete);
    assert!(!evaluation.production_ready);
    assert!(!evaluation.hyperscaler_grade);

    let json = checker::to_json(&evaluation);
    assert!(json.contains(r#""verdict":"PASS""#));
    assert!(json.contains(r#""local_fixture_contract_proven":true"#));
    assert!(json.contains(r#""aggregate_exit_live":false"#));
    assert!(json.contains(r#""p0_0_green":false"#));
    assert!(json.contains(r#""phase0_complete":false"#));
}

#[test]
fn current_red_and_missing_required_fixtures_pass_as_red_fixtures() {
    let current_red = evaluate_fixture(&fixture(
        "specs/fixtures/phase0-exit-gate/tc-0.12-current-red-p0-0-live-context-missing.json",
    ));
    assert_eq!(current_red.verdict, "PASS", "{:?}", current_red.failures);
    assert_eq!(current_red.fixture_results[0].observed_verdict, "RED");
    assert!(
        current_red.fixture_results[0]
            .violations
            .contains(&"false_or_non_true_subcondition".to_string())
    );

    let missing_required = evaluate_fixture(&fixture(
        "specs/fixtures/phase0-exit-gate/tc-0.12-bad-missing-required-subcondition.json",
    ));
    assert_eq!(
        missing_required.verdict, "PASS",
        "{:?}",
        missing_required.failures
    );
    assert_eq!(missing_required.fixture_results[0].observed_verdict, "RED");
    assert!(
        missing_required.fixture_results[0]
            .violations
            .contains(&"missing_required_subcondition".to_string())
    );
}

#[test]
fn missing_required_subcondition_fails_for_green_fixture() {
    let source =
        read_repo_file("specs/fixtures/phase0-exit-gate/tc-0.12-good-all-subconditions-green.json");
    let path = temp_path("missing-ac00");
    fs::write(
        &path,
        replace_first(&source, "    \"AC-0.0_green\": true,\n", ""),
    )
    .unwrap();
    assert_fails_with("missing-ac00", "missing_required_subcondition", &path);
    let _ = fs::remove_file(path);
}

#[test]
fn live_context_false_fails_for_green_fixture() {
    let source =
        read_repo_file("specs/fixtures/phase0-exit-gate/tc-0.12-good-all-subconditions-green.json");
    let path = temp_path("live-context-false");
    fs::write(
        &path,
        replace_first(
            &source,
            "    \"p0_0_full_required_context_proven\": true,",
            "    \"p0_0_full_required_context_proven\": false,",
        ),
    )
    .unwrap();
    assert_fails_with(
        "live-context-false",
        "false_or_non_true_subcondition",
        &path,
    );
    let _ = fs::remove_file(path);
}

#[test]
fn unknown_subcondition_fails() {
    let source =
        read_repo_file("specs/fixtures/phase0-exit-gate/tc-0.12-good-all-subconditions-green.json");
    let path = temp_path("unknown-subcondition");
    fs::write(
        &path,
        replace_first(
            &source,
            "    \"service_inventory_published\": true\n",
            "    \"service_inventory_published\": true,\n    \"unregistered_phase0_shortcut\": true\n",
        ),
    )
    .unwrap();
    assert_fails_with("unknown-subcondition", "unknown_subcondition", &path);
    let _ = fs::remove_file(path);
}

#[test]
fn single_false_missing_case_fails() {
    let source = read_repo_file(
        "specs/fixtures/phase0-exit-gate/tc-0.12-bad-single-false-subconditions.json",
    );
    let path = temp_path("missing-case");
    fs::write(
        &path,
        replace_first(
            &source,
            r#"    {
      "case_id": "BAD-AC-0.17_claim_ceiling_green",
      "expected_verdict": "RED",
      "forced_false": "AC-0.17_claim_ceiling_green",
      "subconditions": {
        "AC-0.0_green": true,
        "AC-0.10_green": true,
        "AC-0.10b_xfail_marker_present_and_classified": true,
        "AC-0.11_green": true,
        "AC-0.12_green_self_check": true,
        "AC-0.13_phase0_coverage_equality_and_divergence_measured": true,
        "AC-0.14_every_gate_has_bad_good_fixtures_passing": true,
        "AC-0.15_merge_conflict_foundation_and_tide_placement_live": true,
        "AC-0.16_automation_ratchet_green": true,
        "AC-0.17_claim_ceiling_green": false,
        "AC-0.1_green": true,
        "AC-0.2_green": true,
        "AC-0.3_green": true,
        "AC-0.4_green": true,
        "AC-0.5_green": true,
        "AC-0.6_green": true,
        "AC-0.7_green": true,
        "AC-0.8_green": true,
        "AC-0.9_green": true,
        "affected_only_checks_not_exit_authority": true,
        "branch_protection_enforcement_claims_use_live_non_501_producers_only": true,
        "cross_artifact_agreement_gate_live": true,
        "every_phase0_pr_has_multispectrum_evidence": true,
        "every_phase0_pr_has_reviewer_verdict": true,
        "language_discipline_gate_live": true,
        "p0_0_full_required_context_proven": true,
        "p0_0_tenant_pipeline_isolation_proven": true,
        "p0_6_packet_manifests_closed_when_applicable": true,
        "p0_6_structural_migrations_landed_with_trunk_green": true,
        "p0_7_decision_propagation_packets_closed_when_applicable": true,
        "p0_9_generated_artifact_registry_and_merge_readiness_closed_when_applicable": true,
        "service_inventory_published": true
      }
    },
"#,
            "",
        ),
    )
    .unwrap();
    assert_fails_with(
        "missing-case",
        "missing_case_for_required_subcondition",
        &path,
    );
    let _ = fs::remove_file(path);
}

#[test]
fn single_false_multi_false_case_fails() {
    let source = read_repo_file(
        "specs/fixtures/phase0-exit-gate/tc-0.12-bad-single-false-subconditions.json",
    );
    let path = temp_path("multi-false-case");
    fs::write(
        &path,
        replace_first(
            &source,
            r#"        "AC-0.17_claim_ceiling_green": true,"#,
            r#"        "AC-0.17_claim_ceiling_green": false,"#,
        ),
    )
    .unwrap();
    assert_fails_with(
        "multi-false-case",
        "single_false_case_not_exactly_one_false_subcondition",
        &path,
    );
    let _ = fs::remove_file(path);
}

#[test]
fn red_claims_green_fails() {
    let source = read_repo_file(
        "specs/fixtures/phase0-exit-gate/tc-0.12-current-red-p0-0-live-context-missing.json",
    );
    let path = temp_path("red-claims-green");
    fs::write(
        &path,
        replace_first(
            &source,
            r#"  "claim_boundary": {
    "p0_0_green": false,
    "phase0_complete": false
  }"#,
            r#"  "claim_boundary": {
    "p0_0_green": true,
    "phase0_complete": true
  }"#,
        ),
    )
    .unwrap();
    assert_fails_with(
        "red-claims-green",
        "fixture_claims_current_phase0_green",
        &path,
    );
    let _ = fs::remove_file(path);
}
