use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn runbook_freshness_gate_accepts_fresh_scoped_and_unscoped_runbooks() {
    let temp = temp_dir("runbook-freshness-valid");
    fs::create_dir_all(&temp).expect("runbooks dir created");
    write_runbook(&temp.join("sev1.md"), Some("Sev 1"), Some("2026-05-09"));
    write_runbook(&temp.join("stub.md"), None, Some("2026-05-09"));

    let output = run_gate(&temp, "2026-05-10");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "runbook freshness validation passed: 2 runbooks, 1 severity-scoped, 1 unscoped"
    ));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn runbook_freshness_gate_rejects_stale_sev1_runbook() {
    let temp = temp_dir("runbook-freshness-stale-sev1");
    fs::create_dir_all(&temp).expect("runbooks dir created");
    write_runbook(&temp.join("incident.md"), Some("Sev 1"), Some("2026-01-01"));

    let output = run_gate(&temp, "2026-05-10");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("StaleRunbook"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn runbook_freshness_gate_rejects_missing_last_verified() {
    let temp = temp_dir("runbook-freshness-missing-date");
    fs::create_dir_all(&temp).expect("runbooks dir created");
    write_runbook(&temp.join("missing-date.md"), Some("Sev 2"), None);

    let output = run_gate(&temp, "2026-05-10");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("MissingLastVerified"));

    fs::remove_dir_all(temp).ok();
}

#[test]
fn runbook_freshness_gate_rejects_unknown_severity() {
    let temp = temp_dir("runbook-freshness-unknown-severity");
    fs::create_dir_all(&temp).expect("runbooks dir created");
    write_runbook(
        &temp.join("critical.md"),
        Some("Critical"),
        Some("2026-05-09"),
    );

    let output = run_gate(&temp, "2026-05-10");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("UnknownSeverity"));

    fs::remove_dir_all(temp).ok();
}

fn run_gate(runbooks_dir: &Path, today: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "runbook-freshness",
            "--runbooks-dir",
            runbooks_dir.to_str().expect("utf8 runbooks dir"),
            "--today",
            today,
        ])
        .output()
        .expect("runbook freshness gate command runs")
}

fn write_runbook(path: &Path, severity_scope: Option<&str>, last_verified: Option<&str>) {
    let severity_line = severity_scope
        .map(|severity| format!("> **Severity scope:** {severity}\n"))
        .unwrap_or_default();
    let last_verified_line = last_verified
        .map(|date| format!("> **Last verified:** {date}\n"))
        .unwrap_or_default();
    fs::write(
        path,
        format!(
            "# Test runbook\n\n> **Status:** Stub\n> **Owner:** ops-sre-reliability\n{severity_line}{last_verified_line}\n## Procedure\n\n- Exercise the fixture.\n"
        ),
    )
    .expect("runbook written");
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{name}-{nonce}"))
}
