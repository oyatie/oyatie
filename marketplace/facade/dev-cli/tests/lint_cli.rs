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
fn lint_asyncapi_accepts_non_audit_local_protobuf_ref() {
    let fixture = asyncapi_fixture_root(
        "../../proto/cloud/billing/v1/cloud-billing-event-v1.proto#/cloud.billing.v1.CloudBillingEventIngest",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "lint",
            "asyncapi",
            fixture
                .join("contracts/asyncapi/cloud/cloud-billing-events-v1.yaml")
                .to_str()
                .expect("fixture path is utf8"),
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
    let _ = fs::remove_dir_all(fixture);
}

#[test]
fn lint_asyncapi_fails_closed_on_missing_local_protobuf_path() {
    let fixture = asyncapi_fixture_root(
        "../../proto/cloud/billing/v1/missing.proto#/cloud.billing.v1.CloudBillingEventIngest",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "lint",
            "asyncapi",
            fixture
                .join("contracts/asyncapi/cloud/cloud-billing-events-v1.yaml")
                .to_str()
                .expect("fixture path is utf8"),
        ])
        .output()
        .expect("asyncapi lint runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("asyncapi-lint: payload $ref target does not exist")
            && stderr.contains("missing.proto"),
        "stderr={stderr}"
    );
    let _ = fs::remove_dir_all(fixture);
}

#[test]
fn lint_asyncapi_fails_closed_on_missing_local_protobuf_fragment() {
    let fixture = asyncapi_fixture_root(
        "../../proto/cloud/billing/v1/cloud-billing-event-v1.proto#/cloud.billing.v1.MissingMessage",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args([
            "lint",
            "asyncapi",
            fixture
                .join("contracts/asyncapi/cloud/cloud-billing-events-v1.yaml")
                .to_str()
                .expect("fixture path is utf8"),
        ])
        .output()
        .expect("asyncapi lint runs");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "asyncapi-lint: payload proto message cloud.billing.v1.MissingMessage not found"
        ),
        "stderr={stderr}"
    );
    let _ = fs::remove_dir_all(fixture);
}

#[test]
fn lint_adr_shape_accepts_valid_adr() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .current_dir(repo_root())
        .args([
            "lint",
            "adr-shape",
            "docs/decisions/ADR-0709-general-live-apex.md",
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

fn asyncapi_fixture_root(proto_ref: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-lint-asyncapi-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos()
    ));
    let asyncapi_dir = root.join("contracts").join("asyncapi").join("cloud");
    let proto_dir = root
        .join("contracts")
        .join("proto")
        .join("cloud")
        .join("billing")
        .join("v1");
    fs::create_dir_all(&asyncapi_dir).expect("asyncapi fixture dir created");
    fs::create_dir_all(&proto_dir).expect("proto fixture dir created");
    fs::write(
        proto_dir.join("cloud-billing-event-v1.proto"),
        r#"syntax = "proto3";

package cloud.billing.v1;

message CloudBillingEventIngest {
  string id = 1;
}
"#,
    )
    .expect("proto fixture written");
    fs::write(
        asyncapi_dir.join("cloud-billing-events-v1.yaml"),
        format!(
            r#"asyncapi: 3.1.0
info:
  title: Billing Fixture
  version: 1.0.0
defaultContentType: application/cloudevents+protobuf
channels:
  billing:
    address: oya.cloud.billing
    messages:
      BillingEvent:
        contentType: application/cloudevents+protobuf
        payload:
          schemaFormat: application/vnd.google.protobuf;version=3
          schema:
            $ref: '{proto_ref}'
operations:
  ingestBillingEvent:
    action: send
    channel:
      $ref: '#/channels/billing'
"#
        ),
    )
    .expect("asyncapi fixture written");
    root
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
