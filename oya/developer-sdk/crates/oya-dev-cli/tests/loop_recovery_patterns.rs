// ADR-0083 Tier 3: integration tests use `.expect()` to assert repository
// invariants for the loop-recovery-patterns gate.
#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate has workspace parent")
        .parent()
        .expect("workspace has repo root")
        .to_path_buf()
}

#[test]
fn loop_recovery_patterns_gate_accepts_repo_inventory() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args(["gate", "validate", "loop-recovery-patterns"])
        .output()
        .expect("loop-recovery-patterns gate runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("loop-recovery-patterns validation passed")
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("3 score-card commands"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn loop_recovery_patterns_gate_rejects_unknown_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args(["gate", "validate", "loop-recovery-patterns", "--bogus"])
        .output()
        .expect("loop-recovery-patterns gate runs");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("loop-recovery-patterns")
            || String::from_utf8_lossy(&output.stderr).contains("Usage: oya")
    );
}
