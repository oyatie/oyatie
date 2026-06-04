#[allow(dead_code)]
#[path = "../ci/assert-required-status-source.rs"]
mod checker;

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap())
}

fn fixture(name: &str) -> PathBuf {
    repo_root()
        .join("specs/fixtures/phase0-required-status-source")
        .join(name)
}

fn summarize_fixture(name: &str, expected_app_id: Option<i64>) -> checker::Summary {
    let text =
        fs::read_to_string(fixture(name)).unwrap_or_else(|error| panic!("read {name}: {error}"));
    let json = checker::parse_json(&text).unwrap_or_else(|error| panic!("parse {name}: {error}"));
    checker::summarize(&json, "oya-ci-required", expected_app_id)
}

fn assert_fails_with_reason(fixture: &str, reason: &str, expected_app_id: Option<i64>) {
    let summary = summarize_fixture(fixture, expected_app_id);
    assert_eq!(summary.verdict, "FAIL", "{fixture} should fail");
    assert_eq!(summary.reason, reason, "{fixture} reason");
    assert!(!summary.p0_0_green);
    assert!(!summary.phase0_complete);
    let json = checker::to_json(&summary);
    assert!(json.contains(r#""verdict":"FAIL""#));
    assert!(json.contains(&format!(r#""reason":"{reason}""#)));
    assert!(json.contains(r#""p0_0_green":false"#));
    assert!(json.contains(r#""phase0_complete":false"#));
}

#[test]
fn good_bound_expected_source_app_passes() {
    let summary = summarize_fixture("good-bound-expected-source-app.json", Some(12345));
    assert_eq!(summary.verdict, "PASS");
    assert_eq!(summary.reason, "required_status_source_app_bound");
    assert_eq!(summary.observed_source_app_id, Some(12345));
    assert!(summary.required_context_source_app_bound);
    assert!(summary.trusted_source_app_proven);
    assert!(!summary.p0_0_green);
    assert!(!summary.phase0_complete);
    let json = checker::to_json(&summary);
    assert!(json.contains(r#""verdict":"PASS""#));
    assert!(json.contains(r#""trusted_source_app_proven":true"#));
}

#[test]
fn good_fixture_without_expected_app_id_fails_closed() {
    assert_fails_with_reason(
        "good-bound-expected-source-app.json",
        "expected_source_app_id_not_configured",
        None,
    );
}

#[test]
fn required_status_source_bad_fixtures_fail_closed() {
    for (fixture, reason) in [
        (
            "bad-contexts-only-no-checks-array.json",
            "missing_required_status_checks_checks_array",
        ),
        (
            "bad-null-source-app.json",
            "missing_required_status_source_app",
        ),
        (
            "bad-wildcard-any-source-app.json",
            "wildcard_required_status_source_app",
        ),
        (
            "bad-wrong-source-app.json",
            "wrong_required_status_source_app",
        ),
        (
            "bad-missing-required-context.json",
            "missing_required_context",
        ),
        (
            "bad-required-context-not-in-checks-array.json",
            "required_context_not_in_checks_array",
        ),
    ] {
        assert_fails_with_reason(fixture, reason, Some(12345));
    }
}

#[test]
fn checker_records_non_mutating_authority_boundary() {
    let source =
        fs::read_to_string(repo_root().join("scripts/ci/assert-required-status-source.rs"))
            .unwrap();
    assert!(source.contains("required-status source binding evidence only; this checker never mutates branch protection or posts statuses"));
    assert!(source.contains("p0_0_green"));
    assert!(source.contains("phase0_complete"));
}
