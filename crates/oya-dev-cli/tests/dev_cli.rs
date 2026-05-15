// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn dev_check_runs_check_script_and_reports_text_success() {
    let temp = temp_dir("dev-check-success");
    let script = write_script(&temp, "echo dev-check-ok\n");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "dev",
            "check",
            "--check-script",
            script.to_str().expect("utf8 script"),
        ])
        .output()
        .expect("dev check command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dev-check-ok"), "stdout={stdout}");
    assert!(stdout.contains("dev check passed:"), "stdout={stdout}");

    fs::remove_dir_all(temp).ok();
}

#[test]
fn dev_check_replays_script_stderr_and_fails_closed() {
    let temp = temp_dir("dev-check-failure");
    let script = write_script(&temp, "echo dev-check-bad >&2\nexit 7\n");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "dev",
            "check",
            "--check-script",
            script.to_str().expect("utf8 script"),
        ])
        .output()
        .expect("dev check command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("dev-check-bad"), "stderr={stderr}");
    assert!(
        stderr.contains("dev check failed:") && stderr.contains("exit code 7"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn dev_check_json_mode_emits_structured_evidence_without_replaying_text() {
    let temp = temp_dir("dev-check-json");
    let script = write_script(&temp, "echo json-ok\n");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "dev",
            "check",
            "--check-script",
            script.to_str().expect("utf8 script"),
            "--format",
            "json",
        ])
        .output()
        .expect("dev check command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("{\"command\":\"oya dev check\""),
        "stdout={stdout}"
    );
    assert!(stdout.contains("\"status\":\"passed\""), "stdout={stdout}");
    assert!(
        stdout.contains("\"stdout\":\"json-ok\\n\""),
        "stdout={stdout}"
    );

    fs::remove_dir_all(temp).ok();
}

fn write_script(root: &Path, contents: &str) -> std::path::PathBuf {
    fs::create_dir_all(root).expect("temp dir created");
    let script = root.join("check.sh");
    fs::write(&script, contents).expect("script written");
    script
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{label}-{}-{nanos}", std::process::id()))
}
