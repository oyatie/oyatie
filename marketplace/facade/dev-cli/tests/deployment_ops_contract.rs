// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to assert
// repository invariants for the deployment ops contract gate.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::process::Command;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

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

#[test]
fn deployment_ops_contract_gate_rejects_malformed_successor_contract() {
    let repo = repo_root();
    let mut contract: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("specs/deployment-ops-contract.json"))
            .expect("deployment contract source is readable"),
    )
    .expect("deployment contract source is JSON");
    contract["deployment_authority"]["primary"] = serde_json::json!("terraform");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "oya-malformed-deployment-contract-{}-{unique}.json",
        std::process::id()
    ));
    fs::write(
        &path,
        serde_json::to_string(&contract).expect("fixture serializes"),
    )
    .expect("malformed contract fixture written");
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(&repo)
        .args([
            "gate",
            "validate",
            "deployment-ops-contract",
            "--contract",
            path.to_str().expect("temporary fixture path is UTF-8"),
        ])
        .output()
        .expect("deployment ops contract gate runs");
    let _ = fs::remove_file(&path);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("deployment_authority.primary must be exactly opentofu")
    );
}
