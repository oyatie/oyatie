// ADR-0083 Tier 3: integration tests use `.expect()` to assert repository
// invariants for the loop-recovery-patterns gate.
#![allow(clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::process::Command;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|candidate| {
            candidate.join("specs/masterplan.json").is_file()
                && candidate.join("HANDOFF.md").is_file()
        })
        .expect("repo root")
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
    // The exact count of executed score-card commands varies by environment:
    // environments with cargo-nextest installed execute 3 (supply-chain +
    // nextest + shell-shebang); hermetic buck2 lanes without nextest execute 2
    // (supply-chain + shell-shebang; the nextest check degrades gracefully).
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("score-card commands"),
        "stdout={}",
        stdout
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

#[test]
fn loop_recovery_patterns_gate_rejects_the_retired_durable_goal_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args([
            "gate",
            "validate",
            "loop-recovery-patterns",
            "--agent-durable-goal",
            "specs/agent-durable-goal.json",
        ])
        .output()
        .expect("loop-recovery-patterns gate runs");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage: oya"));
}

#[test]
fn loop_recovery_patterns_gate_rejects_score_cards_without_its_self_contained_contract() {
    let repo = repo_root();
    let mut source: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("specs/score-cards.json"))
            .expect("score-cards fixture source is readable"),
    )
    .expect("score-cards fixture source is JSON");
    source
        .as_object_mut()
        .expect("score-cards fixture source is an object")
        .remove("score_card_contract");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "oya-score-cards-missing-contract-{}-{unique}.json",
        std::process::id()
    ));
    fs::write(
        &path,
        serde_json::to_string(&source).expect("malformed score-cards fixture serializes"),
    )
    .expect("malformed score-cards fixture written");
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(&repo)
        .args([
            "gate",
            "validate",
            "loop-recovery-patterns",
            "--score-cards",
            path.to_str().expect("temporary fixture path is UTF-8"),
        ])
        .output()
        .expect("loop-recovery-patterns gate runs");
    let _ = fs::remove_file(&path);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("score_card_contract"));
}
