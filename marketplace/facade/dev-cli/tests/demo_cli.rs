// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn demo_can_persist_and_replay_audit_ledger() {
    let audit_path = temp_path("audit-ledger");
    let evidence_path = temp_path("evidence-store");
    let run_path = temp_path("run-ledger");
    let step_path = temp_path("step-ledger");
    let outbox_path = temp_path("outbox-store");
    let secret_path = temp_path("secret-store");
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "demo",
            "--audit-ledger",
            audit_path.to_str().expect("utf8 path"),
            "--evidence-store",
            evidence_path.to_str().expect("utf8 path"),
            "--run-ledger",
            run_path.to_str().expect("utf8 path"),
            "--step-ledger",
            step_path.to_str().expect("utf8 path"),
            "--outbox-store",
            outbox_path.to_str().expect("utf8 path"),
            "--secret-store",
            secret_path.to_str().expect("utf8 path"),
        ])
        .output()
        .expect("demo command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("audit_verified=true"));
    assert!(stdout.contains("mcp_tools=1"));
    assert!(stdout.contains("audit_persisted=true"));
    assert!(stdout.contains("evidence_persisted=true"));
    assert!(stdout.contains("run_persisted=true"));
    assert!(stdout.contains("step_persisted=true"));
    assert!(stdout.contains("outbox_persisted=true"));
    assert!(stdout.contains("secret_persisted=true"));
    assert!(
        fs::read_to_string(&audit_path)
            .expect("ledger written")
            .contains("foundry.capability.invoke")
    );
    assert!(
        fs::read_to_string(&audit_path)
            .expect("ledger written")
            .contains("foundry.mcp.tools.list")
    );
    assert!(
        fs::read_to_string(&evidence_path)
            .expect("evidence store written")
            .contains("cap.demo.readiness")
    );
    assert!(
        fs::read_to_string(&run_path)
            .expect("run ledger written")
            .contains("run_000000000001")
    );
    assert!(
        fs::read_to_string(&step_path)
            .expect("step ledger written")
            .contains("step_000000000001_000001")
    );
    assert!(
        fs::read_to_string(&outbox_path)
            .expect("outbox store written")
            .contains("oya.demo.readiness.v1")
    );
    let secret_store = fs::read_to_string(&secret_path).expect("secret store written");
    assert!(secret_store.contains("cap.demo.readiness"));
    assert!(!secret_store.contains("sk-demo-provider-key"));
    assert!(!secret_store.contains("736b2d64656d6f2d70726f76696465722d6b6579"));

    fs::remove_file(audit_path).ok();
    fs::remove_file(evidence_path).ok();
    fs::remove_file(run_path).ok();
    fs::remove_file(step_path).ok();
    fs::remove_file(outbox_path).ok();
    fs::remove_file(secret_path).ok();
}

fn temp_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "oya-demo-{label}-{}-{nanos}.log",
        std::process::id()
    ))
}
