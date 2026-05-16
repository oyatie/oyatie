// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// repository invariants for the Rust-owned ops command surface.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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
fn ops_a1_capacity_retry_dry_run_is_opentofu_owned() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args([
            "ops",
            "oci-a1-capacity-retry",
            "--dry-run",
            "--format",
            "json",
            "--tofu",
            "tofu-test",
            "--infra-dir",
            "infra/oci",
            "--max-attempts",
            "2",
            "--sleep-secs",
            "0",
        ])
        .output()
        .expect("ops dry-run executes");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"command\":\"oya ops oci-a1-capacity-retry\""));
    assert!(stdout.contains("\"authority\":\"opentofu\""));
    assert!(stdout.contains("tofu-test"));
}

#[test]
fn ops_oci_readiness_probe_dry_run_is_read_only() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args([
            "ops",
            "oci-readiness-probe",
            "--dry-run",
            "--format",
            "json",
            "--oci",
            "oci-test",
            "--compartment-id",
            "ocid1.compartment.example",
        ])
        .output()
        .expect("ops readiness dry-run executes");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"command\":\"oya ops oci-readiness-probe\""));
    assert!(stdout.contains("\"mutation_authority\":\"none\""));
    assert!(stdout.contains("compute-instances"));
}

#[test]
fn ops_onprem_bring_up_dry_run_routes_to_makefile_and_ops() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args([
            "ops",
            "onprem-bring-up",
            "--dry-run",
            "--format",
            "json",
            "--repo-root",
            ".",
        ])
        .output()
        .expect("ops onprem bring-up dry-run executes");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"command\":\"oya ops onprem-bring-up\""));
    assert!(stdout.contains("\"manual_ssh_troubleshooting_allowed\":false"));
    assert!(stdout.contains("make bootstrap"));
    assert!(stdout.contains("https://ops.oyatie.com"));
}

#[test]
fn root_deployment_scripts_are_rust_ops_shims() {
    let root = repo_root();
    for (script, command) in [
        (
            "scripts/oci-a1-capacity-retry.sh",
            "ops oci-a1-capacity-retry",
        ),
        ("scripts/oci-readiness-probe.sh", "ops oci-readiness-probe"),
        ("scripts/onprem-bring-up.sh", "ops onprem-bring-up"),
    ] {
        let text = std::fs::read_to_string(root.join(script)).expect("script readable");
        assert!(
            text.contains(command),
            "{script} must dispatch to {command}"
        );
        assert!(
            !text.contains("sudo bash"),
            "{script} must not hand off sudo bash"
        );
        assert!(
            !text.contains("apt-get"),
            "{script} must not own package installation logic"
        );
    }
}
