use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn catalog_validate_cli_checks_workspace_members_against_registry_records() {
    let temp = temp_dir("catalog-valid");
    fs::create_dir_all(temp.join("registry/catalog")).expect("registry dir created");
    fs::create_dir_all(temp.join("crates/oya-foundry-capability-kernel"))
        .expect("crate dir created");
    fs::write(
        temp.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/oya-foundry-capability-kernel"]
"#,
    )
    .expect("workspace manifest written");
    fs::write(
        temp.join("registry/catalog/oya-foundry-capability-kernel.yaml"),
        "context: foundry\nrole: kernel\ncapability: capability\nplane: control\ndata_classes_owned: [INTERNAL_ONLY]\napi_stability: preview\nsecurity_review: unreviewed\nsupply_chain: source-only\n",
    )
    .expect("catalog record written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "catalog",
            "validate",
            "--workspace",
            temp.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--registry",
            temp.join("registry/catalog")
                .to_str()
                .expect("utf8 registry"),
        ])
        .output()
        .expect("catalog command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("catalog validation passed: 1 records")
    );

    fs::remove_dir_all(temp).ok();
}

#[test]
fn catalog_validate_cli_rejects_missing_workspace_record() {
    let temp = temp_dir("catalog-missing");
    fs::create_dir_all(temp.join("registry/catalog")).expect("registry dir created");
    fs::create_dir_all(temp.join("crates/oya-foundry-capability-kernel"))
        .expect("crate dir created");
    fs::write(
        temp.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/oya-foundry-capability-kernel"]
"#,
    )
    .expect("workspace manifest written");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "catalog",
            "validate",
            "--workspace",
            temp.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--registry",
            temp.join("registry/catalog")
                .to_str()
                .expect("utf8 registry"),
        ])
        .output()
        .expect("catalog command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("catalog validation failed"));

    fs::remove_dir_all(temp).ok();
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{label}-{}-{nanos}", std::process::id()))
}
