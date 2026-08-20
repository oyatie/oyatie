// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// CLI invariants against disposable fixture trees.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn client_stack_gate_scans_crate_shaped_active_manifest() {
    let temp = temp_dir("client-stack-crate-shape");
    let manifest = temp.join("app/application/core/shell-frontend/client-manifest.json");
    write_manifest(
        &manifest,
        r#"{
  "surface": "application-shell-control-center",
  "stack": { "framework": "Leptos" },
  "api_contract_codegen": { "recipe": "progenitor", "openapi_version": "3.2.0" }
}"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "client-stack-discipline",
            "--microservices-root",
            temp.join("app").to_str().expect("utf8 root"),
        ])
        .output()
        .expect("client-stack gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("client-stack-discipline passed: 1 client-manifests, 1 surfaces"),
        "stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn client_stack_gate_rejects_solidjs_in_crate_shaped_manifest() {
    let temp = temp_dir("client-stack-solidjs-crate-shape");
    let manifest = temp.join("app/application/core/shell-frontend/client-manifest.json");
    write_manifest(
        &manifest,
        r#"{
  "surface": "application-shell-control-center",
  "stack": { "framework": "SolidStart", "retired_stack": "solidjs" },
  "api_contract_codegen": { "recipe": "progenitor", "openapi_version": "3.2.0" }
}"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "client-stack-discipline",
            "--microservices-root",
            temp.join("app").to_str().expect("utf8 root"),
        ])
        .output()
        .expect("client-stack gate command runs");

    assert!(
        !output.status.success(),
        "SolidJS/SolidStart residue must fail; stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("client-stack-discipline FAILED")
            && stderr.contains("SupersededStackReference"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(temp).ok();
}

fn write_manifest(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("manifest parent")).expect("manifest dir created");
    fs::write(path, contents).expect("client manifest written");
}

fn temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oyatie-{prefix}-{nanos}"))
}
