use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn audit_chain_replay_gate_accepts_demo_shard() {
    let temp = temp_dir("audit-chain-replay-valid");
    fs::create_dir_all(&temp).expect("shards dir created");
    let shard = temp.join("tenant-demo.log");
    write_demo_audit_shard(&shard);

    let output = run_gate(&temp);

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("audit chain replay validation passed: 1 shards, 28 events")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn audit_chain_replay_gate_rejects_tampered_shard() {
    let temp = temp_dir("audit-chain-replay-tampered");
    fs::create_dir_all(&temp).expect("shards dir created");
    let shard = temp.join("tenant-demo.log");
    write_demo_audit_shard(&shard);
    let tampered = fs::read_to_string(&shard)
        .expect("shard readable")
        .replace("foundry.capability.invoke", "foundry.capability.tamper");
    fs::write(&shard, tampered).expect("tampered shard written");

    let output = run_gate(&temp);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("audit chain replay validation failed")
    );

    fs::remove_dir_all(temp).ok();
}

fn write_demo_audit_shard(path: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "demo",
            "--audit-ledger",
            path.to_str().expect("utf8 audit shard"),
        ])
        .output()
        .expect("demo command runs");
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_gate(shards_dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "audit-chain-replay",
            "--shards-dir",
            shards_dir.to_str().expect("utf8 shards dir"),
        ])
        .output()
        .expect("audit chain replay gate command runs")
}

fn temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{name}-{nonce}"))
}
