#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use oya_gen_microservice_manifests_app::{MICROSERVICES, build_manifests_index};
use serde_json::Value;

const MANIFESTS_INDEX_GENERATED_AT: &str = "2026-05-19";

fn temp_repo() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "oya-gen-microservice-manifests-check-mode-{}-{nonce}",
        std::process::id()
    ))
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write fixture");
}

fn write_current_index_fixture(repo: &Path) -> Value {
    let index = build_manifests_index(MANIFESTS_INDEX_GENERATED_AT, MICROSERVICES);
    write(
        &repo.join("specs/microservices/manifests-index.json"),
        &(serde_json::to_string_pretty(&index).unwrap() + "\n"),
    );

    for row in index["microservices"].as_array().unwrap() {
        let Some(manifest) = row.get("manifest").and_then(Value::as_str) else {
            continue;
        };
        let name = row["name"].as_str().unwrap();
        let manifest_microservice = if name == "foundry" {
            "intelligence"
        } else {
            name
        };
        let payload = serde_json::json!({
            "schema_version": "1.0",
            "microservice": manifest_microservice,
            "version": "0.1.0"
        });
        write(
            &repo.join(manifest),
            &(serde_json::to_string_pretty(&payload).unwrap() + "\n"),
        );
    }

    index
}

#[test]
fn check_mode_accepts_current_manifest_index_contract() {
    let repo = temp_repo();
    write_current_index_fixture(&repo);

    let bin = env!("CARGO_BIN_EXE_oya-gen-microservice-manifests");
    let check = Command::new(bin)
        .arg("--repo-root")
        .arg(&repo)
        .arg("--check")
        .output()
        .expect("run check");
    assert!(
        check.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let stdout = String::from_utf8_lossy(&check.stdout);
    assert!(stdout.contains("retired writer guard passed"), "{stdout}");
    assert!(stdout.contains("rows=37"), "{stdout}");

    fs::remove_dir_all(repo).ok();
}

#[test]
fn check_mode_rejects_legacy_microservices_manifest_index_rows() {
    let repo = temp_repo();
    let mut index = write_current_index_fixture(&repo);
    for row in index["microservices"].as_array_mut().unwrap() {
        let Some(name) = row
            .get("name")
            .and_then(Value::as_str)
            .map(ToString::to_string)
        else {
            continue;
        };
        if row.get("manifest").is_some() {
            row["manifest"] = Value::String(format!("microservices/{name}/manifest.json"));
        }
    }
    write(
        &repo.join("specs/microservices/manifests-index.json"),
        &(serde_json::to_string_pretty(&index).unwrap() + "\n"),
    );

    let bin = env!("CARGO_BIN_EXE_oya-gen-microservice-manifests");
    let check = Command::new(bin)
        .arg("--repo-root")
        .arg(&repo)
        .arg("--check")
        .output()
        .expect("run check");
    assert!(
        !check.status.success(),
        "check unexpectedly passed: stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(stderr.contains("[diff]"), "{stderr}");
    assert!(stderr.contains("[legacy-path]"), "{stderr}");
    assert!(
        stderr.contains("microservices/application/manifest.json"),
        "{stderr}"
    );

    fs::remove_dir_all(repo).ok();
}

#[test]
fn non_check_mode_refuses_to_write_retired_generator_output() {
    let repo = temp_repo();
    fs::create_dir_all(&repo).expect("create repo");

    let bin = env!("CARGO_BIN_EXE_oya-gen-microservice-manifests");
    let run = Command::new(bin)
        .arg("--repo-root")
        .arg(&repo)
        .output()
        .expect("run retired writer");
    assert!(
        !run.status.success(),
        "retired writer unexpectedly succeeded: stdout={} stderr={}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("retired/provenance-only"), "{stderr}");
    assert!(
        !repo
            .join("specs/microservices/manifests-index.json")
            .exists(),
        "retired writer must not create manifests-index.json"
    );

    fs::remove_dir_all(repo).ok();
}
