// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// repository invariants for the deployment ops contract gate.
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
fn deployment_ops_contract_gate_accepts_repo_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args(["gate", "validate", "deployment-ops-contract"])
        .output()
        .expect("deployment ops contract gate runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("deployment-ops-contract validation passed")
    );
}
