#![allow(dead_code)]

#[path = "../ci/assert-language-discipline.rs"]
mod gate;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|err| {
        panic!("read {}: {}", path, err);
    })
}

#[test]
fn checked_in_registry_and_fixtures_pass() {
    let evaluation = gate::evaluate(
        Path::new(&repo_root()),
        "specs/language-discipline-registry.json",
    )
    .expect("registry evaluation should run");
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.fixture_count, 4);
    assert_eq!(evaluation.backlog_offender_count, 7);
    assert!(evaluation.failures.is_empty());
}

#[test]
fn registry_rejects_p0_green_mutation() {
    let mutated = read_repo_file("specs/language-discipline-registry.json").replacen(
        "\"p0_0_green\": false",
        "\"p0_0_green\": true",
        1,
    );
    let failures = gate::registry_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "forbidden_true_or_missing_claim_p0_0_green"),
        "{:?}",
        failures
    );
}

#[test]
fn registry_rejects_missing_cloud_backlog_offender() {
    let mutated = read_repo_file("specs/language-discipline-registry.json").replace(
        "scripts/tests/cloud_observability_slo_evidence_check.py",
        "scripts/tests/cloud_observability_slo_evidence_check.rs",
    );
    let failures = gate::registry_failures(&mutated);
    assert!(
        failures.iter().any(|failure| failure.contains(
            "missing_required_backlog_offender:scripts/tests/cloud_observability_slo_evidence_check.py"
        )),
        "{:?}",
        failures
    );
}

#[test]
fn good_allowlisted_fixture_rejects_disallowed_shell_mutation() {
    let mutated = read_repo_file(
        "specs/fixtures/phase0-language-discipline/tc-0.4-good-allowlisted-bootstrap-shell-edit.json",
    )
    .replace("tools/hook-bootstrap/install.sh", "scripts/tests/new_language_gate.test.sh")
    .replace("\"status\": \"M\"", "\"status\": \"A\"");
    let failures = gate::fixture_policy_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure.starts_with("new_shell_outside_allowlist")),
        "{:?}",
        failures
    );
}

#[test]
fn bad_python_fixture_reports_python_sprawl_failure() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-language-discipline/tc-0.4-bad-new-python-under-scripts.json",
    );
    let failures = gate::fixture_policy_failures(&fixture);
    assert!(
        failures
            .iter()
            .any(|failure| failure.starts_with("new_python_outside_allowlist")),
        "{:?}",
        failures
    );
}
