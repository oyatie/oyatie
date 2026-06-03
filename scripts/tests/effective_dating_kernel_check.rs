#![allow(dead_code)]

#[path = "../ci/assert-effective-dating-kernel.rs"]
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
fn checked_in_registry_source_buck_and_fixtures_pass() {
    let evaluation = gate::evaluate(
        Path::new(&repo_root()),
        "specs/effective-dating-kernel-registry.json",
    )
    .expect("effective-dating kernel evaluation should run");
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.file_results.len(), 5);
    assert_eq!(evaluation.fixture_results.len(), 3);
    assert!(evaluation.failures.is_empty());
}

#[test]
fn kernel_source_rejects_missing_bitemporal_type() {
    let mutated = read_repo_file("oya/ontology/crates/oya-ontology-kernel/src/effective_dating.rs")
        .replace("pub struct BitemporalRange", "pub struct TemporalRangeOnly");
    let failures = gate::kernel_source_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("BitemporalRange")),
        "{:?}",
        failures
    );
}

#[test]
fn kernel_source_rejects_missing_as_of_query() {
    let mutated = read_repo_file("oya/ontology/crates/oya-ontology-kernel/src/effective_dating.rs")
        .replace("pub fn as_of", "pub fn lookup_only");
    let failures = gate::kernel_source_failures(&mutated);
    assert!(
        failures.iter().any(|failure| failure.contains("as_of")),
        "{:?}",
        failures
    );
}

#[test]
fn bad_overlap_fixture_reports_overlap_violation() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-bad-overlapping-valid-time.json",
    );
    let failures = gate::fixture_policy_failures(&fixture);
    assert!(failures.is_empty(), "{:?}", failures);
    assert!(fixture.contains("overlapping_bitemporal_range"));
}

#[test]
fn bad_clock_skew_fixture_reports_missing_determinism() {
    let fixture = read_repo_file(
        "specs/fixtures/phase0-effective-dating-kernel/tc-0.6-bad-clock-skew-nondeterministic.json",
    );
    let failures = gate::fixture_policy_failures(&fixture);
    assert!(failures.is_empty(), "{:?}", failures);
    assert!(fixture.contains("clock_skew_determinism_missing"));
}

#[test]
fn registry_rejects_phase0_completion_claim() {
    let mutated = read_repo_file("specs/effective-dating-kernel-registry.json").replacen(
        "\"phase0_complete\": false",
        "\"phase0_complete\": true",
        1,
    );
    let failures = gate::registry_failures(&mutated);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "forbidden_true_or_missing_claim_phase0_complete"),
        "{:?}",
        failures
    );
}
