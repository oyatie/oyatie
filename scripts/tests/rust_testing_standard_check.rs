#[allow(dead_code)]
#[path = "../ci/assert-rust-testing-standard.rs"]
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
        "oya-{label}-{}-{nanos}-{counter}.md",
        std::process::id()
    ));
    path
}

fn checked_in_doc() -> String {
    fs::read_to_string(repo_root().join("docs/standards/testing.md")).unwrap()
}

fn assert_fails_with(label: &str, expected: &str, replacements: &[(&str, &str)]) {
    let mut text = checked_in_doc();
    for (old, new) in replacements {
        assert!(
            text.contains(old),
            "{label} mutation source not found: {old}"
        );
        text = text.replace(old, new);
    }
    let path = temp_file(label);
    fs::write(&path, text).unwrap();
    let evaluation = checker::evaluate_file(&path);
    let _ = fs::remove_file(path);
    assert_eq!(evaluation.verdict, "FAIL", "{label} should fail");
    let joined = evaluation.failures.join("\n");
    assert!(
        joined.contains(expected),
        "{label} missing {expected:?}:\n{joined}"
    );
    assert!(
        !evaluation.standard_contract_proven,
        "{label} should not prove the standard contract"
    );
    assert!(
        checker::evaluate_text("checked-in", &checked_in_doc())
            .failures
            .is_empty()
    );
}

#[test]
fn checked_in_testing_standard_passes() {
    let evaluation = checker::evaluate_text("docs/standards/testing.md", &checked_in_doc());
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.standard_contract_proven);
    assert_eq!(evaluation.failures, Vec::<String>::new());
    assert_eq!(evaluation.anchor_count, evaluation.anchors_present);

    let json = checker::to_json(&evaluation);
    assert!(json.contains(r#""verdict":"PASS""#));
    assert!(json.contains(r#""standard_contract_proven":true"#));
    assert!(json.contains(r#""coverage_runner_implemented":false"#));
    assert!(json.contains(r#""mutation_lane_implemented":false"#));
    assert!(json.contains(r#""live_required_context_execution_proven":false"#));
    assert!(json.contains(r#""protected_branch_authority_proven":false"#));
    assert!(json.contains(r#""p0_0_green":false"#));
    assert!(json.contains(r#""phase0_complete":false"#));
    assert!(json.contains(r#""production_ready":false"#));
    assert!(json.contains(r#""hyperscaler_grade":false"#));
}

#[test]
fn missing_buck2_native_llvm_policy_fails() {
    assert_fails_with(
        "missing_buck2_llvm",
        "missing_buck2_native_llvm_coverage_policy",
        &[("Buck2-native LLVM source-based coverage", "ad-hoc coverage")],
    );
}

#[test]
fn tarpaulin_canonicalization_fails() {
    assert_fails_with(
        "tarpaulin_canonicalized",
        "tarpaulin_canonicalized",
        &[(
            "Tarpaulin is not the canonical coverage surface",
            "Tarpaulin is the canonical coverage surface",
        )],
    );
}

#[test]
fn missing_profile_file_fails() {
    assert_fails_with(
        "missing_profile_file",
        "missing_llvm_profile_file",
        &[("LLVM_PROFILE_FILE", "PROFILE_FILE")],
    );
}

#[test]
fn local_mutation_authority_fails() {
    assert_fails_with(
        "local_mutation_not_advisory",
        "local_cargo_mutation_not_advisory",
        &[(
            "Local Cargo mutation output is advisory",
            "Local Cargo mutation output is authoritative",
        )],
    );
}

#[test]
fn missing_reindeer_generated_buck_fails() {
    assert_fails_with(
        "missing_reindeer_generated_buck",
        "missing_reindeer_generated_buck",
        &[
            ("reindeer-style generation", "manual vendoring"),
            ("generated-BUCK path", "hand-written path"),
        ],
    );
}

#[test]
fn forbidden_green_claim_fails() {
    assert_fails_with(
        "forbidden_green_claim",
        "forbidden_true_claim_p0_0_green",
        &[(
            "## 12. Sources scanned",
            "p0_0_green=true\n\n## 12. Sources scanned",
        )],
    );
}
