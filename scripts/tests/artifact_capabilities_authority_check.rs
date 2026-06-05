#![allow(dead_code)]

#[path = "../ci/assert-artifact-capabilities-authority.rs"]
mod gate;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_registry() -> String {
    fs::read_to_string(repo_root().join("registry/artifact-capabilities-registry.json"))
        .expect("read artifact capabilities registry")
}

#[test]
fn checked_in_registry_uses_buck2_command_authority() {
    let evaluation = gate::evaluate(Path::new(&repo_root()));
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert_eq!(evaluation.forbidden_hit_count, 0);
    assert_eq!(evaluation.required_command_count, 4);
}

#[test]
fn rejects_retired_local_gate_command_authority() {
    let registry = read_registry().replace(
        "buck2 build //:artifact-capabilities-authority-check --show-output",
        concat!("oya", " gate validate active-artifact-contract"),
    );
    let evaluation = gate::evaluate_text(&registry);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .iter()
            .any(|failure| failure.contains(concat!("oya", " gate"))),
        "{:?}",
        evaluation.failures
    );
}

#[test]
fn rejects_retired_local_oya_check_command_authority() {
    let registry = read_registry().replace(
        "buck2 build //:artifact-capabilities-authority-check --show-output",
        concat!("oya", " check active-artifact-contract"),
    );
    let evaluation = gate::evaluate_text(&registry);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .iter()
            .any(|failure| failure.contains(concat!("oya", " check"))),
        "{:?}",
        evaluation.failures
    );
}

#[test]
fn rejects_direct_cargo_check_command_authority() {
    let registry = read_registry().replace(
        "buck2 build //oya/intelligence/crates/oya-intelligence-supervisor-kernel:oya-intelligence-supervisor-kernel --show-output",
        concat!("cargo", " check -p oya-intelligence-supervisor-kernel"),
    );
    let evaluation = gate::evaluate_text(&registry);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .iter()
            .any(|failure| failure.contains(concat!("cargo", " check"))),
        "{:?}",
        evaluation.failures
    );
}

#[test]
fn rejects_missing_required_buck2_command() {
    let registry = read_registry().replace(
        "buck2 build //:language-discipline-check --show-output",
        "buck2 build //:language-discipline-check-removed --show-output",
    );
    let evaluation = gate::evaluate_text(&registry);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .iter()
            .any(|failure| failure.contains("missing required Buck2 command")),
        "{:?}",
        evaluation.failures
    );
}
