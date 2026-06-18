// ADR-0083 Tier 3: integration tests use `.expect()` to assert repository
// invariants for Rust-owned compatibility lint runners.
#![allow(clippy::expect_used, clippy::panic)]

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
fn lint_proto_accepts_audit_event_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args([
            "lint",
            "proto",
            "contracts/proto/platform/audit/v1/audit-event-v1.proto",
        ])
        .output()
        .expect("proto lint runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("proto-lint: ok"));
}

#[test]
fn lint_asyncapi_accepts_audit_event_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args([
            "lint",
            "asyncapi",
            "contracts/asyncapi/platform/audit-events-v1.yaml",
        ])
        .output()
        .expect("asyncapi lint runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("asyncapi-lint: ok"));
}

#[test]
fn lint_adr_shape_accepts_valid_adr() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args([
            "lint",
            "adr-shape",
            "docs/decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md",
        ])
        .output()
        .expect("adr shape lint runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("adr-shape ok"));
}

#[test]
fn lint_foundry_phase00_evidence_accepts_minimal_fixture() {
    let fixture = phase00_fixture_root();
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args([
            "lint",
            "foundry-phase00-evidence",
            fixture.to_str().expect("fixture path is utf8"),
        ])
        .output()
        .expect("phase00 evidence lint runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Phase 00 evidence validator: OK"));
    let _ = fs::remove_dir_all(fixture);
}

fn phase00_fixture_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-lint-phase00-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos()
    ));
    for crate_name in [
        "oya-intelligence-account-kernel",
        "oya-intelligence-account-domain",
        "oya-intelligence-account-app",
        "oya-intelligence-account-adapter-codex-cli",
        "oya-intelligence-account-adapter-claude-code",
        "oya-intelligence-account-adapter-gemini-cli",
        "oya-intelligence-account-adapter-openbao",
        "oya-intelligence-account-runtime",
        "oya-governance-claim-ceiling-kernel",
        "oya-governance-bypass-kernel",
        "oya-governance-pr-traceability-kernel",
        "oya-governance-pre-push-kernel",
        "oya-governance-quality-lane-kernel",
        "oya-governance-cohesion-kernel",
        "oya-intelligence-bypass-ledger-kernel",
    ] {
        let crate_dir = root.join("crates").join(crate_name);
        fs::create_dir_all(crate_dir.join("src")).expect("crate src created");
        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\n",
        )
        .expect("crate manifest written");
        fs::write(crate_dir.join("src/lib.rs"), "pub fn marker() {}\n")
            .expect("crate source written");
    }
    for ip in [
        ".omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/IP-001-phase00-evidence-validator.md",
        ".omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/IP-002-foundry-fitness-lane-ratchet.md",
        ".omc/plans/milestones/M02-foundry-preview/phases/P03-gates-validators-evidence/IP-003-adr-template-bypass-ledger.md",
    ] {
        let path = root.join(ip);
        fs::create_dir_all(path.parent().expect("ip has parent")).expect("ip parent created");
        fs::write(path, "status: complete\n").expect("ip fixture written");
    }
    root
}
