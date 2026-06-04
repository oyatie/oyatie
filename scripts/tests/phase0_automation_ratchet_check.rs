#[allow(dead_code)]
#[path = "../ci/assert-automation-ratchet.rs"]
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
        "oya-phase0-automation-ratchet-{label}-{}-{nanos}-{counter}.json",
        std::process::id()
    ));
    path
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn evaluate_default() -> checker::Evaluation {
    checker::evaluate(&checker::default_config(repo_root()))
}

fn evaluate_with(
    matrix: Option<&Path>,
    coverage: Option<&Path>,
    fixtures: &[PathBuf],
) -> checker::Evaluation {
    let root = repo_root();
    checker::evaluate(&checker::Config {
        matrix: matrix
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("specs/phase0-automation-matrix.json")),
        coverage_registry: coverage
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("specs/phase0-automation-coverage-registry.json")),
        fixtures: fixtures.to_vec(),
        repo_root: root,
    })
}

fn replace_first(text: &str, old: &str, new: &str) -> String {
    assert!(text.contains(old), "mutation source not found: {old}");
    text.replacen(old, new, 1)
}

fn assert_fails_with(label: &str, expected: &str, evaluation: checker::Evaluation) {
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
fn automation_ratchet_rows_map_to_buck2_target() {
    let matrix = read_repo_file("specs/phase0-automation-matrix.json");
    let coverage = read_repo_file("specs/phase0-automation-coverage-registry.json");
    assert!(matrix.contains(r#""id": "AC-0.16-automation-ratchet""#));
    assert!(
        matrix.contains(r#""target_gate_or_controller": "//:phase0-automation-ratchet-check""#)
    );
    assert!(
        matrix.contains(
            r#""verification_command": "buck2 build //:phase0-automation-ratchet-check""#
        )
    );
    assert!(matrix.contains("not live required-context authority"));
    assert!(matrix.contains(r#""no_new_oya_cli_surface": true"#));
    assert!(coverage.contains(r#""id": "AC-0.16""#));
    assert!(
        coverage.contains(
            r#""verification_command": "buck2 build //:phase0-automation-ratchet-check""#
        )
    );
    assert!(coverage.contains("//:phase0-automation-ratchet-check"));
    assert!(coverage.contains("not live required-context authority"));
}

#[test]
fn checked_in_fixture_contract_passes() {
    let evaluation = evaluate_default();
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.failures.is_empty());
    assert_eq!(evaluation.matrix_summary.violations, Vec::<String>::new());
    assert_eq!(
        evaluation.coverage_registry_summary.unmapped_row_ids,
        Vec::<String>::new()
    );
    assert_eq!(
        evaluation.coverage_registry_summary.missing_mapped_row_ids,
        Vec::<String>::new()
    );
    assert!(evaluation.local_fixture_contract_proven);
    assert!(evaluation.coverage_registry_local_static_proven);
    assert!(!evaluation.automation_ratchet_live);
    assert!(!evaluation.protected_branch_authority_proven);
    assert!(!evaluation.status_mutation_performed);
    assert!(!evaluation.p0_0_green);
    assert!(!evaluation.phase0_complete);
    let json = checker::to_json(&evaluation);
    assert!(json.contains(r#""verdict":"PASS""#));
    assert!(json.contains("TC-0.16-BAD-oya-cli-authority"));
    assert!(json.contains(r#""coverage_registry_local_static_proven":true"#));
}

#[test]
fn missing_required_row_fails() {
    let matrix = read_repo_file("specs/phase0-automation-matrix.json");
    let path = temp_path("missing-required-row");
    fs::write(
        &path,
        replace_first(
            &matrix,
            "    \"AC-0.16-automation-ratchet\",\n",
            "    \"AC-0.16-automation-ratchet\",\n    \"MISSING-row-id\",\n",
        ),
    )
    .unwrap();
    assert_fails_with(
        "missing-required-row",
        "missing_required_row_id",
        evaluate_with(Some(&path), None, &[]),
    );
    let _ = fs::remove_file(path);
}

#[test]
fn duplicate_unknown_and_missing_field_fail() {
    let matrix = read_repo_file("specs/phase0-automation-matrix.json");
    let path = temp_path("duplicate-unknown-missing");
    let mutated = replace_first(
        &replace_first(
            &replace_first(
                &matrix,
                "\"id\": \"AC-0.0-trusted-target-inventory\"",
                "\"id\": \"AC-0.0-cloud-ci-required-context\"",
            ),
            "\"classification\": \"automated_blocking_now\"",
            "\"classification\": \"automated_some_day\"",
        ),
        "\"owner\": \"platform-toolchain\"",
        "\"owner\": \"\"",
    );
    fs::write(&path, mutated).unwrap();
    let evaluation = evaluate_with(Some(&path), None, &[]);
    assert_fails_with("duplicate", "duplicate_row_id", evaluation.clone());
    assert_fails_with("unknown", "unknown_classification", evaluation.clone());
    assert_fails_with("missing", "missing_or_empty_required_field", evaluation);
    let _ = fs::remove_file(path);
}

#[test]
fn oya_cli_authority_references_fail() {
    let matrix = read_repo_file("specs/phase0-automation-matrix.json");
    let target_path = temp_path("target-oya-cli");
    fs::write(
        &target_path,
        replace_first(
            &replace_first(
                &matrix,
                "\"target_gate_or_controller\": \"cloud-ci-required / oya-ci-required branch-protection context + //:phase0-ci-enforcement-baseline-catalog-check + //:phase0-required-status-source-check + //:phase0-trusted-target-inventory-check + //:phase0-result-bundle-output-check\"",
                "\"target_gate_or_controller\": \"oya gate run-all --ci-required\"",
            ),
            "\"no_new_oya_cli_surface\": true",
            "\"no_new_oya_cli_surface\": false",
        ),
    )
    .unwrap();
    assert_fails_with(
        "target-oya-cli",
        "blocking_invariant_mapped_to_oya_cli",
        evaluate_with(Some(&target_path), None, &[]),
    );

    let requirement_path = temp_path("requirement-oya-cli");
    fs::write(
        &requirement_path,
        replace_first(
            &matrix,
            "Protected branch merge/exit authority must be a Prow-shaped cloud-ci/oya-ci required context sourced from trusted trunk/controller state; affected-only/buck2-only/local oya output is not authority.",
            "Protected branch required context is satisfied by oya gate run-all --ci-required.",
        ),
    )
    .unwrap();
    assert_fails_with(
        "requirement-oya-cli",
        "blocking_invariant_mapped_to_oya_cli",
        evaluate_with(Some(&requirement_path), None, &[]),
    );

    let coverage = read_repo_file("specs/phase0-automation-coverage-registry.json");
    let coverage_path = temp_path("coverage-oya-cli");
    fs::write(
        &coverage_path,
        replace_first(
            &coverage,
            "Maps all AC-0.0 local/static and live-read RED gate rows, including trusted target inventory, structured result bundles, override/kill-switch, tenant isolation, required context, auto-merge, and Buck2 authority. This is coverage mapping only, not P0.0 green, plus the upstream Prow parity registry for the Rust oya-ci reimplementation/improvement. This is coverage mapping only, not P0.0 green.",
            "Operator may use oya gate run-all --ci-required as required-context evidence.",
        ),
    )
    .unwrap();
    assert_fails_with(
        "coverage-oya-cli",
        "blocking_invariant_mapped_to_oya_cli",
        evaluate_with(None, Some(&coverage_path), &[]),
    );
    let _ = fs::remove_file(target_path);
    let _ = fs::remove_file(requirement_path);
    let _ = fs::remove_file(coverage_path);
}

#[test]
fn human_judgment_without_reason_fails() {
    let matrix = read_repo_file("specs/phase0-automation-matrix.json");
    let path = temp_path("human-judgment-no-reason");
    fs::write(
        &path,
        replace_first(
            &matrix,
            "      \"human_judgment_reason\": \"Final reviewer/architect risk judgment is irreducible human judgment; automatable subparts remain evidence packet completeness, required-status production, and reviewer-status producer cutover.\",\n",
            "",
        ),
    )
    .unwrap();
    assert_fails_with(
        "human-judgment-no-reason",
        "human_judgment_missing_irreducible_reason",
        evaluate_with(Some(&path), None, &[]),
    );
    let _ = fs::remove_file(path);
}

#[test]
fn coverage_registry_mapping_failures_are_detected() {
    let coverage = read_repo_file("specs/phase0-automation-coverage-registry.json");
    let unmapped_path = temp_path("coverage-unmapped-row");
    fs::write(
        &unmapped_path,
        replace_first(
            &coverage,
            "        \"AC-0.0-structured-result-bundle\",\n",
            "",
        ),
    )
    .unwrap();
    assert_fails_with(
        "coverage-unmapped-row",
        "coverage_row_unmapped",
        evaluate_with(None, Some(&unmapped_path), &[]),
    );

    let unknown_path = temp_path("coverage-unknown-row");
    fs::write(
        &unknown_path,
        replace_first(
            &coverage,
            "        \"AC-0.0-tenant-pipeline-isolation\",\n",
            "        \"AC-0.0-tenant-pipeline-isolation\",\n        \"MISSING-row-id\",\n",
        ),
    )
    .unwrap();
    assert_fails_with(
        "coverage-unknown-row",
        "coverage_mapped_row_missing",
        evaluate_with(None, Some(&unknown_path), &[]),
    );

    let green_claim_path = temp_path("coverage-green-claim");
    fs::write(
        &green_claim_path,
        replace_first(&coverage, "\"p0_0_green\": false", "\"p0_0_green\": true"),
    )
    .unwrap();
    assert_fails_with(
        "coverage-green-claim",
        "green_claim_boundary_without_live_authority",
        evaluate_with(None, Some(&green_claim_path), &[]),
    );
    let _ = fs::remove_file(unmapped_path);
    let _ = fs::remove_file(unknown_path);
    let _ = fs::remove_file(green_claim_path);
}

#[test]
fn red_fixture_that_becomes_good_fails() {
    let source = read_repo_file(
        "specs/fixtures/phase0-automation-ratchet/tc-0.16-bad-oya-cli-authority.json",
    );
    let path = temp_path("red-fixture-made-good");
    let mutated = replace_first(
        &replace_first(
            &replace_first(
                &source,
                "\"target_gate_or_controller\": \"oya gate run-all --ci-required\"",
                "\"target_gate_or_controller\": \"cloud-ci-automation-ratchet\"",
            ),
            "\"evidence_path\": \"local oya gate output\"",
            "\"evidence_path\": \"specs/phase0-automation-matrix.json\"",
        ),
        "\"no_new_oya_cli_surface\": false",
        "\"no_new_oya_cli_surface\": true",
    );
    fs::write(&path, mutated).unwrap();
    assert_fails_with(
        "red-fixture-made-good",
        "RED automation-ratchet fixture must produce violations",
        evaluate_with(None, None, &[path.clone()]),
    );
    let _ = fs::remove_file(path);
}
