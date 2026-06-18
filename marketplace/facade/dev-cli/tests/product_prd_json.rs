// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// repository invariants for the product PRD JSON gate.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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
fn product_prd_json_gate_accepts_repo_specs() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args(["gate", "validate", "product-prd-json"])
        .output()
        .expect("product PRD JSON gate runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("product-prd-json validation passed"));
}

#[test]
fn product_prd_json_gate_rejects_unknown_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args(["gate", "validate", "product-prd-json", "--bogus"])
        .output()
        .expect("product PRD JSON gate runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown flag"));
}
