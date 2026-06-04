#[allow(dead_code)]
#[path = "../ci/assert-red-green-fixture-contract.rs"]
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

fn temp_spec(label: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "oya-red-green-{label}-{}-{nanos}-{counter}.json",
        std::process::id()
    ));
    path
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn checked_in_spec_path() -> PathBuf {
    repo_root().join("specs/red-green-fixture-contract.json")
}

fn checked_in_spec() -> String {
    read_repo_file("specs/red-green-fixture-contract.json")
}

fn evaluate_spec(path: &Path) -> checker::Evaluation {
    checker::evaluate(&repo_root(), path)
}

fn replace_first(text: &str, old: &str, new: &str) -> String {
    assert!(text.contains(old), "mutation source not found: {old}");
    text.replacen(old, new, 1)
}

fn assert_fails_with(label: &str, expected: &str, mutate: impl FnOnce(String) -> String) {
    let path = temp_spec(label);
    fs::write(&path, mutate(checked_in_spec())).unwrap();
    let evaluation = evaluate_spec(&path);
    let _ = fs::remove_file(path);
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
fn automation_matrix_row_maps_to_buck2_target() {
    let matrix = read_repo_file("specs/phase0-automation-matrix.json");
    assert!(matrix.contains(r#""id": "AC-0.14-red-green-fixture-contract""#));
    assert!(
        matrix.contains(
            r#""target_gate_or_controller": "//:phase0-red-green-fixture-contract-check""#
        )
    );
    assert!(matrix.contains(
        r#""verification_command": "buck2 build //:phase0-red-green-fixture-contract-check""#
    ));
    assert!(matrix.contains("live cloud-ci authority"));
    assert!(matrix.contains(r#""no_new_oya_cli_surface": true"#));
}

#[test]
fn checked_in_contract_passes() {
    let evaluation = evaluate_spec(&checked_in_spec_path());
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.entry_count, 26);
    assert_eq!(evaluation.buck2_target_count, evaluation.entry_count);
    assert_eq!(evaluation.green_marker_count, 27);
    assert_eq!(evaluation.red_marker_count, 40);
    assert_eq!(evaluation.non_claim_marker_count, 32);
    assert!(evaluation.failures.is_empty());

    let json = checker::to_json(&evaluation);
    assert!(json.contains(r#""verdict":"PASS""#));
    assert!(json.contains(r#""red_green_fixture_contract_measured":true"#));
    assert!(json.contains(r#""live_required_context_execution_proven":false"#));
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
fn remove_red_marker_fails() {
    assert_fails_with("remove-red-marker", "missing_red_markers", |text| {
        replace_first(
            &text,
            concat!(
                r#""red_markers": ["#,
                "\n        {\n",
                r#"          "path": "specs/fixtures/phase0-ci-enforcement-baseline/tc-0.0.1-bad-missing-required-context.json","#,
                "\n",
                r#"          "contains": "\"expected_verdict\": \"RED\"""#,
                "\n        }\n      ]"
            ),
            r#""red_markers": []"#,
        )
    });
}

#[test]
fn stale_marker_text_fails() {
    assert_fails_with("stale-marker-text", "marker_text_missing", |text| {
        replace_first(
            &text,
            r#""contains": "\"expected_verdict\": \"GREEN\"""#,
            r#""contains": "definitely-not-present""#,
        )
    });
}

#[test]
fn missing_target_fails() {
    assert_fails_with("missing-target", "buck2_target_missing", |text| {
        replace_first(
            &text,
            r#""buck2_target": "//:phase0-ci-enforcement-baseline-catalog-check""#,
            r#""buck2_target": "//:missing-red-green-target""#,
        )
    });
}
