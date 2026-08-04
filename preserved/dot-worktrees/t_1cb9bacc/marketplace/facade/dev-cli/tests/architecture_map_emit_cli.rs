// ADR-0083 Tier 3: integration tests use unwrap/expect for fixture assertions.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn architecture_map_emit_cli_writes_provenance_and_complete_coverage() {
    let root = temp_dir("architecture-map-emit");
    fs::create_dir_all(root.join("crates/a")).expect("crate dir");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\n",
    )
    .expect("workspace manifest");
    fs::write(
        root.join("crates/a/Cargo.toml"),
        "[package]\nname = \"a\"\nversion = \"0.1.0\"\n",
    )
    .expect("crate manifest");
    let out = root.join("registry/graph/architecture-map.json");

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["gate", "emit", "architecture-map", "--workspace-root"])
        .arg(&root)
        .arg("--out")
        .arg(&out)
        .output()
        .expect("architecture-map emit command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let body = fs::read_to_string(&out).expect("emitted artifact");
    for expected in [
        "\"producer_version\"",
        "\"source_digest_sha256\"",
        "\"resolved_workspace_crate_count\": 1",
        "\"represented_workspace_crate_count\": 1",
        "\"coverage_ratio\": 1.0000",
        "\"missing_workspace_crate_ids\": []",
        "\"orphan_crate_ids\": []",
    ] {
        assert!(body.contains(expected), "missing {expected} in {body}");
    }

    fs::remove_dir_all(root).ok();
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("oya-{label}-{}-{nanos}", std::process::id()))
}
