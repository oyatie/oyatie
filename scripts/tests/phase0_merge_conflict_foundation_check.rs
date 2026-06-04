#[allow(dead_code)]
#[path = "../ci/assert-phase0-merge-conflict-foundation.rs"]
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
        "oya-phase0-merge-conflict-{label}-{}-{nanos}-{counter}.json",
        std::process::id()
    ));
    path
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn registry_path() -> PathBuf {
    repo_root().join("specs/generated-artifact-registry.json")
}

fn registry_text() -> String {
    read_repo_file("specs/generated-artifact-registry.json")
}

fn fixture_text(path: &str) -> String {
    read_repo_file(path)
}

fn evaluate_registry(path: &Path) -> checker::Evaluation {
    checker::evaluate(&repo_root(), path, &[])
}

fn evaluate_with_fixture(registry: &Path, fixture: &Path) -> checker::Evaluation {
    checker::evaluate(&repo_root(), registry, &[fixture.to_path_buf()])
}

fn replace_first(text: &str, old: &str, new: &str) -> String {
    assert!(text.contains(old), "mutation source not found: {old}");
    text.replacen(old, new, 1)
}

fn assert_fails_with(label: &str, expected: &str, mutate: impl FnOnce(String) -> String) {
    let path = temp_path(label);
    fs::write(&path, mutate(registry_text())).unwrap();
    let evaluation = evaluate_registry(&path);
    let _ = fs::remove_file(path);
    assert_eq!(evaluation.verdict, "FAIL", "{label} should fail");
    let joined = evaluation.failures.join("\n");
    assert!(
        joined.contains(expected),
        "{label} missing {expected:?}:\n{joined}"
    );
    let json = checker::to_json(&evaluation);
    assert!(json.contains(r#""phase1_tide_batching_claimed":false"#));
    assert!(json.contains(r#""p0_0_green":false"#));
    assert!(json.contains(r#""phase0_complete":false"#));
}

#[test]
fn automation_matrix_and_coverage_rows_map_to_buck2_target() {
    let matrix = read_repo_file("specs/phase0-automation-matrix.json");
    let coverage = read_repo_file("specs/phase0-automation-coverage-registry.json");

    assert!(matrix.contains(r#""id": "AC-0.15-merge-conflict-foundation""#));
    assert!(
        matrix.contains(
            r#""target_gate_or_controller": "//:phase0-merge-conflict-foundation-check""#
        )
    );
    assert!(matrix.contains(
        r#""verification_command": "buck2 build //:phase0-merge-conflict-foundation-check""#
    ));
    assert!(matrix.contains("Phase-1 Tide batching"));
    assert!(matrix.contains(r#""no_new_oya_cli_surface": true"#));
    assert!(coverage.contains(r#""id": "P0.9-generated-artifact-registry""#));
    assert!(coverage.contains(
        r#""verification_command": "buck2 build //:phase0-merge-conflict-foundation-check""#
    ));
    assert!(coverage.contains("full-repo generated-artifact coverage remains false"));
    assert!(coverage.contains("live required-context authority"));
}

#[test]
fn checked_in_registry_passes() {
    let evaluation = evaluate_registry(&registry_path());
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.registered_artifact_count, 3);
    assert!(evaluation.taxonomy_count >= 7);
    assert_eq!(evaluation.fixture_count, 5);
    assert_eq!(evaluation.expected_green_fixture_count, 1);
    assert_eq!(evaluation.expected_red_fixture_count, 4);
    assert!(evaluation.failures.is_empty());

    let json = checker::to_json(&evaluation);
    assert!(json.contains(r#""verdict":"PASS""#));
    assert!(json.contains(r#""generated_artifact_registry_published":true"#));
    assert!(json.contains(r#""merge_tree_fixture_contract_measured":true"#));
    assert!(json.contains("TC-0.15-GOOD-clean-merge-tree-generated-registry"));
    assert!(json.contains("TC-0.15-BAD-path-overlap"));
    assert!(json.contains("path_overlap_without_review"));
    assert!(json.contains("generated_artifact_missing_registry"));
    assert!(json.contains("phase1_tide_batched_projection_overclaim"));
    assert!(json.contains("merge_tree_conflict"));
    assert!(json.contains(r#""live_required_context_execution_proven":false"#));
    assert!(json.contains(r#""phase1_tide_batching_claimed":false"#));
    assert!(json.contains(r#""full_repo_generated_artifact_coverage_proven":false"#));
    assert!(json.contains(r#""p0_0_green":false"#));
    assert!(json.contains(r#""phase0_complete":false"#));
}

#[test]
fn p0_green_claim_fails() {
    assert_fails_with(
        "p0-green",
        "forbidden_true_or_missing_claim_p0_0_green",
        |text| replace_first(&text, r#""p0_0_green": false"#, r#""p0_0_green": true"#),
    );
}

#[test]
fn missing_automated_chain_target_fails() {
    assert_fails_with(
        "missing-target",
        "missing_automated_chain_token://:phase0-merge-conflict-foundation-check",
        |text| {
            replace_first(
                &text,
                "    \"buck2 build //:phase0-merge-conflict-foundation-check\",\n",
                "",
            )
        },
    );
}

#[test]
fn missing_source_path_fails() {
    assert_fails_with(
        "missing-source",
        "artifact_source_path_missing_or_invalid",
        |text| {
            replace_first(
                &text,
                r#""source_paths": [
        "Cargo.toml",
        "Cargo.lock",
        "reindeer.toml",
        "third-party/fixups",
        "scripts/ci/third-party-buckify-handedits.patch"
      ]"#,
                r#""source_paths": [
        "missing-generated-source.toml"
      ]"#,
            )
        },
    );
}

#[test]
fn red_fixture_made_clean_fails() {
    let path = temp_path("red-made-clean");
    let text = fixture_text(
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-bad-path-overlap.json",
    );
    fs::write(
        &path,
        replace_first(
            &text,
            r#""owned_paths": [
        "specs/phase0-automation-matrix.json"
      ]"#,
            r#""owned_paths": [
        "specs/phase0-automation-coverage-registry.json"
      ]"#,
        ),
    )
    .unwrap();
    let evaluation = evaluate_with_fixture(&registry_path(), &path);
    let _ = fs::remove_file(path);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .join("\n")
            .contains("RED merge-conflict fixture must produce violations")
    );
}

#[test]
fn green_fixture_now_conflicts_fails() {
    let path = temp_path("green-now-conflicts");
    let text = fixture_text(
        "specs/fixtures/phase0-merge-conflict-foundation/tc-0.15-good-clean-merge-tree-generated-registry.json",
    );
    fs::write(
        &path,
        replace_first(&text, r#""result": "clean""#, r#""result": "conflict""#),
    )
    .unwrap();
    let evaluation = evaluate_with_fixture(&registry_path(), &path);
    let _ = fs::remove_file(path);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .join("\n")
            .contains("GREEN merge-conflict fixture produced violations")
    );
}
