// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// repository invariants for the Rust-owned ADR-0039 supply-chain runner.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
fn adr0039_dry_run_emits_rust_owned_plan() {
    let root = repo_root();
    let temp = root.join("target/oya-test/supply-chain-adr0039");
    fs::create_dir_all(&temp).expect("temp dir created");
    let manifest = temp.join("images.yaml");
    fs::write(
        &manifest,
        "images:\n  - ref: ghcr.io/acme/app@sha256:abc123\n",
    )
    .expect("manifest written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(root)
        .args([
            "supply-chain",
            "adr0039",
            "--manifest",
            manifest.to_str().expect("utf8 manifest"),
            "--artifacts-dir",
            temp.join("artifacts").to_str().expect("utf8 artifacts"),
            "--dry-run",
            "--format",
            "json",
        ])
        .output()
        .expect("adr0039 dry-run runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"command\":\"oya supply-chain adr0039\""));
    assert!(stdout.contains("trivy fs --severity HIGH,CRITICAL --exit-code 1 ."));
    assert!(stdout.contains("cosign verify --rekor-url"));
}

#[test]
fn adr0039_runner_rejects_empty_release_manifest() {
    let root = repo_root();
    let temp = root.join("target/oya-test/supply-chain-adr0039-empty");
    fs::create_dir_all(&temp).expect("temp dir created");
    let manifest = temp.join("images.yaml");
    fs::write(&manifest, "# release_state: pre-release\nimages: []\n").expect("manifest written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(root)
        .args([
            "supply-chain",
            "adr0039",
            "--manifest",
            manifest.to_str().expect("utf8 manifest"),
            "--dry-run",
        ])
        .output()
        .expect("adr0039 dry-run runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("has no image refs"));
}

#[test]
fn install_trivy_dry_run_emits_rust_owned_plan() {
    let root = repo_root();
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(root)
        .args([
            "supply-chain",
            "install-trivy",
            "--dry-run",
            "--format",
            "json",
        ])
        .output()
        .expect("install-trivy dry-run runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"command\":\"oya supply-chain install-trivy\""));
    assert!(stdout.contains("trivy_0.70.0_Linux-64bit.tar.gz"));
    assert!(stdout.contains("sha256sum -c"));
}
