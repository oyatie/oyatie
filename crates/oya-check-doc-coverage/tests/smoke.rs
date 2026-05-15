// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Smoke tests for oya-check-doc-coverage against synthetic fixtures.

use std::fs;

#[test]
fn empty_repo_runs_without_panic() {
    let tmp = tempfile::tempdir().unwrap();
    // Minimal Cargo.toml so registry parser doesn't panic
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = []\n[workspace.metadata.oya.microservices]\n",
    )
    .unwrap();
    let report = oya_check_doc_coverage::run(tmp.path()).unwrap();
    // No µservices registered, no packs → no violations
    assert!(report.is_clean());
}

#[test]
fn missing_canonical_artifacts_for_registered_microservice() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = []\n\n[workspace.metadata.oya.microservices]\nhr = {}\n",
    )
    .unwrap();
    let report = oya_check_doc_coverage::run(tmp.path()).unwrap();
    // hr is registered but has no docs/microservices/hr.md, no docs/prds/hr.md, no naming ADR
    assert!(!report.is_clean());
    let kinds: Vec<_> = report
        .violations
        .iter()
        .map(|v| format!("{:?}", v.kind))
        .collect();
    assert!(kinds.iter().any(|k| k == "MissingCanonicalArtifact"));
}
