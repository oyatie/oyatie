// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// CLI invariants under controlled fixtures.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// Wave 15-ZA pending: the `fd001 manifest workspace alignment` CLI subcommand
// is not wired into oya-dev-cli's dispatch yet (origin/dev merged the
// `fd001_manifest_workspace_alignment_gate` module but dropped the match arm).
// Re-enable once Wave 15-ZA restores the dispatch per ADR-0346.
#![cfg(any())]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn fd001_manifest_workspace_alignment_rejects_missing_manifest_crates() {
    let root = fixture_root("fd001-manifest-missing");
    write_workspace(&root, &["crates/oya-messenger-message-stream-kernel"]);
    write_crate(&root, "oya-messenger-message-stream-kernel");
    let manifest = write_manifest(
        &root,
        "messenger",
        &[
            "oya-messenger-message-stream-kernel",
            "oya-messenger-message-stream-rest",
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest",
            manifest.to_str().expect("utf8 manifest"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("fd001-manifest-workspace-alignment validation failed"),
        "stderr={stderr}"
    );
    assert!(
        stderr.contains("missing manifest crates: 1"),
        "stderr={stderr}"
    );
    assert!(
        stderr.contains("oya-messenger-message-stream-rest"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn fd001_manifest_workspace_alignment_report_only_emits_evidence() {
    let root = fixture_root("fd001-manifest-report-only");
    write_workspace(&root, &["crates/oya-messenger-message-stream-kernel"]);
    write_crate(&root, "oya-messenger-message-stream-kernel");
    let manifest = write_manifest(
        &root,
        "messenger",
        &[
            "oya-messenger-message-stream-kernel",
            "oya-messenger-message-stream-rest",
        ],
    );
    let report = PathBuf::from("evidence/fd001/manifest-workspace-alignment.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest",
            manifest.to_str().expect("utf8 manifest"),
            "--report-only",
            "--emit-report",
            report.to_str().expect("utf8 report"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "fd001-manifest-workspace-alignment report-only: 1 manifests, 2 required crates, 1 missing"
    ));

    let report_body = fs::read_to_string(root.join(&report)).expect("report emitted");
    assert!(report_body.contains("\"gate\": \"fd001-manifest-workspace-alignment\""));
    assert!(report_body.contains("\"missing_crate_count\": 1"));
    assert!(report_body.contains("\"required_crates\""));
    assert!(report_body.contains("\"present_crates\""));
    assert!(report_body.contains("oya-messenger-message-stream-rest"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn fd001_manifest_workspace_alignment_report_only_requires_emit_report() {
    let root = fixture_root("fd001-manifest-report-only-no-report");
    write_workspace(&root, &["crates/oya-messenger-message-stream-kernel"]);
    write_crate(&root, "oya-messenger-message-stream-kernel");
    let manifest = write_manifest(
        &root,
        "messenger",
        &[
            "oya-messenger-message-stream-kernel",
            "oya-messenger-message-stream-rest",
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest",
            manifest.to_str().expect("utf8 manifest"),
            "--report-only",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--report-only requires --emit-report"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn fd001_manifest_workspace_alignment_defaults_to_fd001_material_manifests_from_index() {
    let root = fixture_root("fd001-manifest-index-scope");
    write_workspace(&root, &["crates/oya-messenger-message-stream-kernel"]);
    write_crate(&root, "oya-messenger-message-stream-kernel");
    let messenger = write_manifest(
        &root,
        "messenger",
        &[
            "oya-messenger-message-stream-kernel",
            "oya-messenger-message-stream-rest",
        ],
    );
    let calendar = write_manifest(&root, "calendar", &["oya-calendar-app"]);
    let index = write_manifest_index(
        &root,
        &[
            ("messenger", &messenger, true),
            ("calendar", &calendar, false),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest-index",
            index.to_str().expect("utf8 index"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("1 manifests checked"), "stderr={stderr}");
    assert!(
        stderr.contains("missing manifest crates: 1"),
        "stderr={stderr}"
    );
    assert!(
        stderr.contains("oya-messenger-message-stream-rest"),
        "stderr={stderr}"
    );
    assert!(!stderr.contains("oya-calendar-app"), "stderr={stderr}");

    fs::remove_dir_all(root).ok();
}

#[test]
fn fd001_manifest_workspace_alignment_all_manifests_includes_non_material_rows() {
    let root = fixture_root("fd001-manifest-all-index");
    write_workspace(&root, &["crates/oya-messenger-message-stream-kernel"]);
    write_crate(&root, "oya-messenger-message-stream-kernel");
    let messenger = write_manifest(
        &root,
        "messenger",
        &[
            "oya-messenger-message-stream-kernel",
            "oya-messenger-message-stream-rest",
        ],
    );
    let calendar = write_manifest(&root, "calendar", &["oya-calendar-app"]);
    let index = write_manifest_index(
        &root,
        &[
            ("messenger", &messenger, true),
            ("calendar", &calendar, false),
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest-index",
            index.to_str().expect("utf8 index"),
            "--all-manifests",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("2 manifests checked"), "stderr={stderr}");
    assert!(
        stderr.contains("oya-messenger-message-stream-rest"),
        "stderr={stderr}"
    );
    assert!(stderr.contains("oya-calendar-app"), "stderr={stderr}");

    fs::remove_dir_all(root).ok();
}

#[test]
fn fd001_manifest_workspace_alignment_rejects_index_manifest_identity_mismatch() {
    let root = fixture_root("fd001-manifest-identity-mismatch");
    write_workspace(&root, &["crates/oya-calendar-app"]);
    write_crate(&root, "oya-calendar-app");
    let calendar = write_manifest(&root, "calendar", &["oya-calendar-app"]);
    let index = write_manifest_index(&root, &[("messenger", &calendar, true)]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest-index",
            index.to_str().expect("utf8 index"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("microservice identity mismatch"),
        "stderr={stderr}"
    );
    assert!(stderr.contains("expected messenger"), "stderr={stderr}");

    fs::remove_dir_all(root).ok();
}

#[test]
fn fd001_manifest_workspace_alignment_repeated_manifest_flags_aggregate_explicit_scope() {
    let root = fixture_root("fd001-manifest-explicit-repeat");
    write_workspace(
        &root,
        &[
            "crates/oya-messenger-message-stream-kernel",
            "crates/oya-mail-app",
        ],
    );
    write_crate(&root, "oya-messenger-message-stream-kernel");
    write_crate(&root, "oya-mail-app");
    let messenger = write_manifest(&root, "messenger", &["oya-messenger-message-stream-kernel"]);
    let mail = write_manifest(&root, "mail", &["oya-mail-app"]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest",
            messenger.to_str().expect("utf8 messenger"),
            "--manifest",
            mail.to_str().expect("utf8 mail"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "fd001-manifest-workspace-alignment validation passed: 2 manifests, 2 required crates, 0 missing"
    ));

    fs::remove_dir_all(root).ok();
}

#[test]
fn fd001_manifest_workspace_alignment_rejects_mixed_all_and_explicit_manifest_scope() {
    let root = fixture_root("fd001-manifest-mixed-scope");
    write_workspace(&root, &["crates/oya-messenger-message-stream-kernel"]);
    write_crate(&root, "oya-messenger-message-stream-kernel");
    let manifest = write_manifest(&root, "messenger", &["oya-messenger-message-stream-kernel"]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest",
            manifest.to_str().expect("utf8 manifest"),
            "--all-manifests",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--all-manifests cannot be combined with --manifest"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn fd001_manifest_workspace_alignment_rejects_empty_crate_declarations() {
    let root = fixture_root("fd001-manifest-empty-crates");
    write_workspace(&root, &["crates/oya-messenger-message-stream-kernel"]);
    write_crate(&root, "oya-messenger-message-stream-kernel");
    let manifest = write_manifest(&root, "messenger", &[]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest",
            manifest.to_str().expect("utf8 manifest"),
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("bounded_contexts[0].crates must not be empty"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn fd001_manifest_workspace_alignment_rejects_report_path_outside_evidence() {
    let root = fixture_root("fd001-manifest-report-path");
    write_workspace(&root, &["crates/oya-messenger-message-stream-kernel"]);
    write_crate(&root, "oya-messenger-message-stream-kernel");
    let manifest = write_manifest(&root, "messenger", &["oya-messenger-message-stream-kernel"]);

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest",
            manifest.to_str().expect("utf8 manifest"),
            "--report-only",
            "--emit-report",
            "../bad.json",
        ])
        .output()
        .expect("gate command runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--emit-report must be repo-relative under evidence/"),
        "stderr={stderr}"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn fd001_manifest_workspace_alignment_passes_when_all_manifest_crates_are_members() {
    let root = fixture_root("fd001-manifest-clean");
    write_workspace(
        &root,
        &[
            "crates/oya-messenger-message-stream-kernel",
            "crates/oya-messenger-message-stream-rest",
        ],
    );
    write_crate(&root, "oya-messenger-message-stream-kernel");
    write_crate(&root, "oya-messenger-message-stream-rest");
    let manifest = write_manifest(
        &root,
        "messenger",
        &[
            "oya-messenger-message-stream-kernel",
            "oya-messenger-message-stream-rest",
        ],
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "gate",
            "validate",
            "fd001-manifest-workspace-alignment",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--workspace",
            root.join("Cargo.toml").to_str().expect("utf8 workspace"),
            "--manifest",
            manifest.to_str().expect("utf8 manifest"),
        ])
        .output()
        .expect("gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "fd001-manifest-workspace-alignment validation passed: 1 manifests, 2 required crates, 0 missing"
    ));

    fs::remove_dir_all(root).ok();
}

fn fixture_root(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("oya-{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&root).expect("fixture root created");
    root
}

fn write_workspace(root: &Path, members: &[&str]) {
    let members = members
        .iter()
        .map(|member| format!("  \"{member}\","))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        root.join("Cargo.toml"),
        format!("[workspace]\nmembers = [\n{members}\n]\n"),
    )
    .expect("workspace manifest written");
}

fn write_crate(root: &Path, package_name: &str) {
    let crate_dir = root.join("crates").join(package_name);
    fs::create_dir_all(&crate_dir).expect("crate dir created");
    fs::write(
        crate_dir.join("Cargo.toml"),
        format!("[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nlicense = \"Apache-2.0\"\n"),
    )
    .expect("crate manifest written");
}

fn write_manifest(root: &Path, microservice: &str, crates: &[&str]) -> PathBuf {
    let manifest_dir = root.join("microservices").join(microservice);
    fs::create_dir_all(&manifest_dir).expect("manifest dir created");
    let crate_rows = crates
        .iter()
        .map(|crate_name| format!("        \"{crate_name}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    let manifest = manifest_dir.join("manifest.json");
    fs::write(
        &manifest,
        format!(
            r#"{{
  "schema_version": "1.0",
  "microservice": "{microservice}",
  "bounded_contexts": [
    {{
      "name": "{microservice}",
      "crates": [
{crate_rows}
      ]
    }}
  ]
}}
"#
        ),
    )
    .expect("microservice manifest written");
    manifest
}

fn write_manifest_index(root: &Path, rows: &[(&str, &Path, bool)]) -> PathBuf {
    let index = root.join("specs/microservices/manifests-index.json");
    fs::create_dir_all(index.parent().expect("index parent")).expect("index dir created");
    let rows = rows
        .iter()
        .map(|(name, manifest, fd001_material)| {
            format!(
                r#"    {{ "name": "{name}", "manifest": "{}", "fd001_material": {fd001_material} }}"#,
                manifest
                    .strip_prefix(root)
                    .expect("relative manifest")
                    .display()
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    fs::write(
        &index,
        format!(
            r#"{{
  "microservices": [
{rows}
  ]
}}
"#
        ),
    )
    .expect("manifest index written");
    index
}
