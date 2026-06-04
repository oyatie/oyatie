#[allow(dead_code)]
#[path = "../ci/assert-pr-required-context.rs"]
mod checker;

use checker::Json;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap())
}

fn fixture_path(name: &str) -> PathBuf {
    repo_root()
        .join("specs/fixtures/phase0-required-context-rollup")
        .join(name)
}

fn read_json(name: &str) -> Json {
    let text = fs::read_to_string(fixture_path(name)).unwrap_or_else(|error| {
        panic!("read {name}: {error}");
    });
    checker::parse_json(&text).unwrap_or_else(|error| panic!("parse {name}: {error}"))
}

fn summarize(name: &str) -> checker::Report {
    checker::summarize(
        &read_json(name),
        "github-lane-unlocker-required",
        "github-lane-unlocker-ci-cd",
    )
}

fn assert_fails_with_reason(name: &str, reason: &str) {
    let report = summarize(name);
    assert_eq!(report.verdict, "FAIL", "{name}");
    assert_eq!(report.reason, reason, "{name}");
    assert!(!report.required_context_proven);
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"FAIL""#));
    assert!(rendered.contains(&format!(r#""reason":"{reason}""#)));
    assert!(rendered.contains(r#""p0_0_green":false"#));
    assert!(rendered.contains(r#""phase0_complete":false"#));
}

#[test]
fn direct_github_lane_unlocker_required_context_passes() {
    let report = summarize("good-github-lane-unlocker-required-success.json");
    assert_eq!(report.verdict, "PASS");
    assert_eq!(report.reason, "required_context_success");
    assert!(report.required_context_proven);
    assert_eq!(report.required_context_trusted_producer, Some(true));
    assert_eq!(report.required_context_status, "success");
    assert_eq!(
        report.required_context_producer_values,
        vec!["github-lane-unlocker-ci-cd".to_string()]
    );
    assert!(!report.p0_0_green);
    assert!(!report.phase0_complete);
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""verdict":"PASS""#));
    assert!(rendered.contains(r#""required_context_proven":true"#));
    assert!(rendered.contains(r#""required_context_trusted_producer":true"#));
    assert!(rendered.contains("status-rollup evidence only; this checker never posts statuses"));
}

#[test]
fn nested_github_lane_unlocker_required_context_passes() {
    let report = summarize("good-nested-github-lane-unlocker-required-success.json");
    assert_eq!(report.verdict, "PASS");
    assert_eq!(report.reason, "required_context_success");
    assert!(report.required_context_proven);
    assert_eq!(report.required_context_trusted_producer, Some(true));
    assert_eq!(report.required_context_status, "success");
    assert_eq!(
        report.required_context_producer_values,
        vec![
            "github-lane-unlocker-ci-cd".to_string(),
            "GitHub Actions".to_string()
        ]
    );
    let rendered = checker::to_json(&report);
    assert!(rendered.contains(r#""contexts":["github-lane-unlocker-required"]"#));
    assert!(rendered.contains(r#""required_context_trusted_producer":true"#));
}

#[test]
fn no_checks_and_missing_required_context_fail_closed() {
    assert_fails_with_reason("bad-no-checks-reported.json", "no_status_checks_reported");
    let missing = summarize("bad-missing-github-lane-unlocker-required.json");
    assert_eq!(missing.verdict, "FAIL");
    assert_eq!(missing.reason, "missing_required_context");
    assert_eq!(
        missing.legacy_contexts_present,
        vec!["buck2-affected-only".to_string()]
    );
    assert!(!missing.required_context_proven);
    let rendered = checker::to_json(&missing);
    assert!(rendered.contains(r#""legacy_contexts_present":["buck2-affected-only"]"#));
}

#[test]
fn failed_required_context_and_completed_failure_fail_closed() {
    assert_fails_with_reason(
        "bad-github-lane-unlocker-required-failure.json",
        "required_context_not_success",
    );
    let completed_failure = summarize("bad-github-lane-unlocker-required-completed-failure.json");
    assert_eq!(completed_failure.verdict, "FAIL");
    assert_eq!(completed_failure.reason, "required_context_not_success");
    assert_eq!(completed_failure.required_context_status, "failure");
    assert_eq!(
        completed_failure.required_context_producer_values,
        vec!["github-lane-unlocker-ci-cd".to_string()]
    );
}

#[test]
fn missing_and_untrusted_producer_fail_closed() {
    let missing = summarize("bad-github-lane-unlocker-required-success-missing-producer.json");
    assert_eq!(missing.verdict, "FAIL");
    assert_eq!(missing.reason, "missing_required_context_producer");
    assert_eq!(missing.required_context_trusted_producer, Some(false));
    assert!(missing.required_context_producer_values.is_empty());
    assert!(!missing.required_context_proven);

    let untrusted = summarize("bad-github-lane-unlocker-required-success-untrusted-producer.json");
    assert_eq!(untrusted.verdict, "FAIL");
    assert_eq!(untrusted.reason, "untrusted_required_context_producer");
    assert_eq!(untrusted.required_context_trusted_producer, Some(false));
    assert_eq!(
        untrusted.required_context_producer_values,
        vec!["local-bridge".to_string()]
    );
    let rendered = checker::to_json(&untrusted);
    assert!(rendered.contains(r#""required_context_trusted_producer":false"#));
}

#[test]
fn cli_json_contains_contract_markers() {
    let report = checker::summarize_path(
        &fixture_path("good-github-lane-unlocker-required-success.json").to_string_lossy(),
        "github-lane-unlocker-required",
        "github-lane-unlocker-ci-cd",
    )
    .expect("summarize path");
    let rendered = checker::to_json(&report);
    for expected in [
        "github-lane-unlocker-required",
        "github-lane-unlocker-ci-cd",
        "required_context_success",
        "required_context_trusted_producer",
        "status-rollup evidence only; this checker never posts statuses",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
