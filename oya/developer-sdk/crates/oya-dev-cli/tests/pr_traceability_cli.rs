// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn pr_traceability_gate_accepts_author_pr_shape() {
    let temp = temp_dir("pr-traceability-valid");
    let body = write_pr_body(&temp, valid_pr_body());

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(pr_traceability_args(&body, &[]))
        .output()
        .expect("PR traceability gate runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("PR traceability validation passed: 5 required sections")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn pr_traceability_gate_rejects_missing_required_h2() {
    let temp = temp_dir("pr-traceability-missing-h2");
    let body = write_pr_body(
        &temp,
        valid_pr_body()
            .replace("## Evidence", "## Artifacts")
            .as_str(),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(pr_traceability_args(&body, &[]))
        .output()
        .expect("PR traceability gate runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingSection"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn pr_traceability_gate_enforces_merge_time_code_review_policy() {
    let temp = temp_dir("pr-traceability-code-review");
    let body = write_pr_body(&temp, valid_pr_body());

    let author_output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(pr_traceability_args(&body, &["--forbid-code-review"]))
        .output()
        .expect("PR traceability gate runs");

    assert!(!author_output.status.success());
    assert!(
        String::from_utf8_lossy(&author_output.stderr).contains("CodeReviewForbidden"),
        "stderr={}",
        String::from_utf8_lossy(&author_output.stderr)
    );

    let merge_output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(pr_traceability_args(&body, &["--require-code-review"]))
        .output()
        .expect("PR traceability gate runs");

    assert!(
        merge_output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&merge_output.stdout),
        String::from_utf8_lossy(&merge_output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&merge_output.stdout).contains("code_review_present=true"),
        "stdout={}",
        String::from_utf8_lossy(&merge_output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn pr_traceability_gate_rejects_code_review_without_reviewer_agent_approve_evidence() {
    let temp = temp_dir("pr-traceability-missing-reviewer-evidence");
    let body = write_pr_body(
        &temp,
        valid_pr_body()
            .replace(
                "- Reviewer agent: architect 136-IdentityBridgeReReview\n",
                "",
            )
            .as_str(),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(pr_traceability_args(&body, &["--require-code-review"]))
        .output()
        .expect("PR traceability gate runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("MissingCodeReviewField"),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(temp).ok();
}

fn pr_traceability_args(path: &Path, policy_flags: &[&str]) -> Vec<String> {
    let mut args = vec![
        "gate".into(),
        "validate".into(),
        "pr-traceability".into(),
        "--pr-body".into(),
        path.to_str().expect("utf8 PR body").into(),
    ];
    args.extend(policy_flags.iter().map(|flag| (*flag).into()));
    args
}

fn write_pr_body(root: &Path, contents: &str) -> std::path::PathBuf {
    fs::create_dir_all(root).expect("temp dir created");
    let body = root.join("pr.md");
    fs::write(&body, contents).expect("PR body written");
    body
}

fn valid_pr_body() -> &'static str {
    "## Issue\nCloses #123\n\n## Summary\n- Implemented the thing.\n\n## Verification\n- pass: oya dev check\n\n## Traceability\n- Catalog records touched: oya-intelligence-capability-kernel\n- Cross-axis contracts touched: none\n- ADRs cited: ADR-0001\n\n## Evidence\n- Audit-chain emission: EVT-1\n- Foundation-bypass referenced (if any): none\n- Per-pack regulator-watch impact (if any): none\n\n## Code Review\n- Reviewer agent: architect 136-IdentityBridgeReReview\n- Verdict: APPROVE\n- Resolved items: none\n- Deferred items: none\n"
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{label}-{}-{nanos}", std::process::id()))
}
