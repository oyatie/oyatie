//! Per-sub-check unit tests for the 4 TG2 sub-checks.
//! Resolves CONV-6 from TG2 11-facet debate synthesis (F3 adversarial + F4
//! ergonomic convergence: prior to this file, the canonical-IDs test asserted
//! wiring only; behavior was untested).
//!
//! Each sub-check has 2 tests:
//!   - happy-path  → scan tree shape that the (eventual) armed check would Pass
//!   - failing-path → scan tree shape that the (eventual) armed check would Fail
//!
//! All 4 sub-checks today return NotYetArmed regardless of input (day-1 stubs
//! aligned per CONV-3), so tests assert on FINDINGS content, not status. When
//! the FixupTasks F-LANE-RUST-DEFAULT-ENFORCE / F-NAMING-CONVENTION-ENFORCE /
//! F-LANE-SCORECARD-RENDER / F-LANE-DEBATE-SUBCHECK promote status to Pass/
//! Fail, these tests get extended with status assertions then.
//!
//! Std-only: builds a fresh tmp workspace per test via std::env::temp_dir +
//! std::fs. No tempfile crate dep (matches kernel std-only policy).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use oya_check_dependency_seam::{
    check_consensus_debate_evidence, check_naming_convention, check_rust_default_language,
    check_scorecard_render, SubCheckStatus, WorkspaceContext,
};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Build a fresh tmp workspace dir unique to this test invocation.
fn make_workspace() -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("oya-seam-test-{}-{}-{}", pid, nanos, n));
    fs::create_dir_all(&dir).expect("mkdir tmp workspace");
    dir
}

fn cleanup(ws: &PathBuf) {
    let _ = fs::remove_dir_all(ws);
}

fn findings_contain(result: &oya_check_dependency_seam::SubCheckResult, needle: &str) -> bool {
    result.findings.iter().any(|f| f.contains(needle))
}

// =============== check_rust_default_language ===============

#[test]
fn rust_default_language_happy_path_no_non_rust_scripts() {
    let ws = make_workspace();
    let scripts = ws.join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join("ok.rs"), "fn main() {}").unwrap();
    fs::write(scripts.join("README.md"), "docs").unwrap();

    let result = check_rust_default_language(&WorkspaceContext::new(&ws));
    assert_eq!(result.id, "rust-default-language");
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "non-Rust file count: 0"),
        "happy path should report count: 0, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

#[test]
fn rust_default_language_failing_path_counts_disallowed_extensions() {
    let ws = make_workspace();
    let scripts = ws.join("scripts");
    fs::create_dir_all(&scripts).unwrap();
    fs::write(scripts.join("setup.sh"), "#!/bin/sh\n").unwrap();
    fs::write(scripts.join("helper.py"), "print('x')").unwrap();
    fs::write(scripts.join("lint.mjs"), "// x").unwrap();
    fs::write(scripts.join("ok.rs"), "fn main() {}").unwrap();

    let result = check_rust_default_language(&WorkspaceContext::new(&ws));
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "non-Rust file count: 3"),
        "failing path should report count: 3, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

#[test]
fn rust_default_language_surfaces_io_error_when_scripts_missing() {
    let ws = make_workspace(); // intentionally NO scripts/ dir
    let result = check_rust_default_language(&WorkspaceContext::new(&ws));
    assert!(
        findings_contain(&result, "read_dir") && findings_contain(&result, "failed"),
        "missing scripts/ should surface read_dir failure (CONV-2 fix), got {:?}",
        result.findings
    );
    cleanup(&ws);
}

// =============== check_naming_convention ===============

#[test]
fn naming_convention_happy_path_kebab_case_files() {
    let ws = make_workspace();
    let home = ws.join("specs/cross-cutting");
    fs::create_dir_all(&home).unwrap();
    fs::write(home.join("oyatie-doctrine-v1.0.0.json"), "{}").unwrap();
    fs::write(home.join("fixuptasks.jsonl"), "").unwrap();

    let result = check_naming_convention(&WorkspaceContext::new(&ws));
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "kebab-case violations: 0"),
        "happy path should report 0 violations, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

#[test]
fn naming_convention_failing_path_detects_uppercase_and_snake_case() {
    let ws = make_workspace();
    let home = ws.join("specs/cross-cutting");
    fs::create_dir_all(&home).unwrap();
    fs::write(home.join("BadCase.json"), "{}").unwrap();
    fs::write(home.join("snake_case.json"), "{}").unwrap();
    fs::write(home.join("ok-kebab.json"), "{}").unwrap();

    let result = check_naming_convention(&WorkspaceContext::new(&ws));
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "kebab-case violations: 2"),
        "failing path should report 2 violations, got {:?}",
        result.findings
    );
    assert!(
        findings_contain(&result, "BadCase.json"),
        "violation list should include BadCase.json"
    );
    assert!(
        findings_contain(&result, "snake_case.json"),
        "violation list should include snake_case.json"
    );
    cleanup(&ws);
}

// =============== check_scorecard_render ===============

#[test]
fn scorecard_render_happy_path_evidence_with_required_keys() {
    let ws = make_workspace();
    let dir = ws.join("evidence/multispectrum");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("renderable.json"),
        r#"{"change_class_id": "CC-7", "facets": {}}"#,
    )
    .unwrap();

    let result = check_scorecard_render(&WorkspaceContext::new(&ws));
    assert_eq!(result.status, SubCheckStatus::NotYetArmed);
    assert!(
        findings_contain(&result, "evidence files scanned: 1; minimum-renderable as scorecard: 1"),
        "happy path should report 1/1, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

#[test]
fn scorecard_render_failing_path_missing_facets_key() {
    let ws = make_workspace();
    let dir = ws.join("evidence/multispectrum");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("incomplete.json"),
        r#"{"change_class_id": "CC-5"}"#,
    )
    .unwrap();

    let result = check_scorecard_render(&WorkspaceContext::new(&ws));
    assert!(
        findings_contain(&result, "evidence files scanned: 1; minimum-renderable as scorecard: 0"),
        "failing path: file missing 'facets' should not be renderable, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

// =============== check_consensus_debate_evidence ===============

#[test]
fn consensus_debate_happy_path_meta_triggered_with_matching_synthesis() {
    let ws = make_workspace();
    let multispectrum = ws.join("evidence/multispectrum");
    let debate = ws.join("evidence/debate");
    fs::create_dir_all(&multispectrum).unwrap();
    fs::create_dir_all(&debate).unwrap();
    fs::write(
        multispectrum.join("CHG-X-r1.json"),
        r#"{"meta_review_triggered": true}"#,
    )
    .unwrap();
    fs::write(debate.join("CHG-X-synthesis.json"), r#"{}"#).unwrap();

    let result = check_consensus_debate_evidence(&WorkspaceContext::new(&ws));
    assert!(
        findings_contain(&result, "meta_review_triggered: 1") && findings_contain(&result, "synthesis files present: 1"),
        "happy path should report balanced meta=1/synthesis=1, got {:?}",
        result.findings
    );
    cleanup(&ws);
}

#[test]
fn consensus_debate_failing_path_meta_triggered_no_synthesis() {
    let ws = make_workspace();
    let multispectrum = ws.join("evidence/multispectrum");
    let debate = ws.join("evidence/debate");
    fs::create_dir_all(&multispectrum).unwrap();
    fs::create_dir_all(&debate).unwrap();
    fs::write(
        multispectrum.join("CHG-Y-r1.json"),
        r#"{"meta_review_triggered": true}"#,
    )
    .unwrap();
    // Intentionally NO synthesis file in debate/

    let result = check_consensus_debate_evidence(&WorkspaceContext::new(&ws));
    assert!(
        findings_contain(&result, "meta_review_triggered: 1") && findings_contain(&result, "synthesis files present: 0"),
        "failing path: meta=1, synthesis=0 imbalance, got {:?}",
        result.findings
    );
    cleanup(&ws);
}
