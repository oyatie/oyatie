#[allow(dead_code)]
#[path = "../ci/assert-rust-llvm-coverage-runner-contract.rs"]
mod checker;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap())
}

fn temp_file(label: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "oya-{label}-{}-{nanos}-{counter}.json",
        std::process::id()
    ));
    path
}

fn checked_in_spec() -> String {
    fs::read_to_string(repo_root().join("specs/rust-llvm-coverage-runner-contract.json")).unwrap()
}

fn assert_fails_with(label: &str, expected: &str, replacements: &[(&str, &str)]) {
    let mut text = checked_in_spec();
    for (old, new) in replacements {
        assert!(
            text.contains(old),
            "{label} mutation source not found: {old}"
        );
        text = text.replace(old, new);
    }
    let path = temp_file(label);
    fs::write(&path, text).unwrap();
    let evaluation = checker::evaluate_file(path.to_str().unwrap());
    let _ = fs::remove_file(path);
    assert_eq!(evaluation.verdict, "FAIL", "{label} should fail");
    let joined = evaluation.failures.join("\n");
    assert!(
        joined.contains(expected),
        "{label} missing {expected:?}:\n{joined}"
    );
    assert!(
        checker::evaluate_text(&checked_in_spec())
            .failures
            .is_empty()
    );
}

#[test]
fn checked_in_contract_passes() {
    let spec = checked_in_spec();
    assert!(spec.contains(r#""coverage_runner_contract_proven": true"#));
    assert!(spec.contains(r#""coverage_report_generated": false"#));
    let evaluation = checker::evaluate_text(&checked_in_spec());
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.failures.is_empty());
}

#[test]
fn missing_instrument_flag_fails() {
    assert_fails_with(
        "missing_instrument_flag",
        "missing_instrument_coverage_flag",
        &[(
            "rustc -C instrument-coverage",
            "rustc without coverage instrumentation",
        )],
    );
}

#[test]
fn missing_profile_env_fails() {
    assert_fails_with(
        "missing_profile_env",
        "missing_llvm_profile_file_env",
        &[("LLVM_PROFILE_FILE", "PROFILE_FILE")],
    );
}

#[test]
fn missing_collision_guard_fails() {
    assert_fails_with(
        "missing_collision_guard",
        "missing_profile_collision_guard",
        &[("%m-%p", "%p")],
    );
}

#[test]
fn missing_profdata_fails() {
    assert_fails_with(
        "missing_profdata",
        "missing_llvm_profdata_tool",
        &[("llvm-profdata", "profile-merge-tool")],
    );
}

#[test]
fn missing_llvm_cov_fails() {
    assert_fails_with(
        "missing_llvm_cov",
        "missing_llvm_cov_tool",
        &[("llvm-cov", "coverage-export-tool")],
    );
}

#[test]
fn missing_smoke_target_fails() {
    assert_fails_with(
        "missing_smoke_target",
        "missing_buck2_coverage_smoke_target",
        &[(
            "//:rust-llvm-coverage-smoke-check",
            "//:missing-coverage-smoke",
        )],
    );
}

#[test]
fn ambient_path_required_fails() {
    assert_fails_with(
        "ambient_path_required",
        "ambient_path_llvm_tools_not_forbidden",
        &[(
            "\"ambient_path_llvm_tools_required\": false",
            "\"ambient_path_llvm_tools_required\": true",
        )],
    );
}

#[test]
fn production_report_claim_fails() {
    assert_fails_with(
        "production_report_claim",
        "production_coverage_false_boundary_missing",
        &[(
            "\"production_coverage_report_generated\": false",
            "\"production_coverage_report_generated\": true",
        )],
    );
}

#[test]
fn tarpaulin_boundary_missing_fails() {
    assert_fails_with(
        "tarpaulin_boundary_missing",
        "tarpaulin_noncanonical_boundary_missing",
        &[(
            "Tarpaulin is not required CI/PR coverage evidence for this monorepo",
            "alternative coverage tool is canonical",
        )],
    );
}

#[test]
fn generated_report_claim_fails() {
    assert_fails_with(
        "generated_report_claim",
        "forbidden_true_or_missing_claim_coverage_report_generated",
        &[(
            "\"coverage_report_generated\": false",
            "\"coverage_report_generated\": true",
        )],
    );
}
