// ADR-0535 dependency automation gate: live-tree GREEN plus RED fixtures proving the gate rejects
// missing policy, closed-schema drift, Rust pin split-brain, and external updater config residue.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use oya_cloud_ci_dependency_automation_app::{Verdict, evaluate_repo};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("oya-dependency-automation-test-{nonce}-{count}"));
    fs::create_dir_all(&path).expect("create temp root");
    path
}

fn write_minimal_candidate(root: &Path, pin: &str) {
    fs::create_dir_all(root.join("toolchains")).unwrap();
    fs::create_dir_all(root.join("docs/standards")).unwrap();
    fs::write(
        root.join("rust-toolchain.toml"),
        format!("[toolchain]\nchannel = \"{pin}\"\n"),
    )
    .unwrap();
    fs::write(
        root.join("Cargo.toml"),
        format!("[workspace.package]\nrust-version = \"{pin}\"\n"),
    )
    .unwrap();
    fs::write(
        root.join("Dockerfile.distroless"),
        format!("ARG RUST_VERSION={pin}\n"),
    )
    .unwrap();
    fs::write(
        root.join("toolchains/BUCK"),
        format!("# Rust {pin} toolchain\n"),
    )
    .unwrap();
    fs::write(
        root.join("docs/standards/dependency-policy.md"),
        format!("Rust toolchain | {pin} stable\n"),
    )
    .unwrap();
    fs::write(root.join("deny.toml"), "[licenses]\n").unwrap();
    fs::create_dir_all(root.join("specs")).unwrap();
    fs::write(root.join("specs/oss-stewardship-registry.json"), "{}\n").unwrap();
    fs::write(root.join("oya-deps.toml"), oya_deps(pin)).unwrap();
}

fn oya_deps(pin: &str) -> String {
    format!(
        r#"schema_version = "1.0.0"

[metadata]
purpose = "fixture"
owner = "cloud-ci-platform"
decision = "ADR-0535"
status = "accepted"

[automation]
engine = "owned-rust-bump-bot"
changeset_transport = "scm-facts"
github_actions = "adapter-only"
external_bots = "disabled"
merge_authority = "oya-ci-required"

[rust]
channel = "stable"
pin = "{pin}"
update_policy = "latest-stable"
drift_guard = "cloud/cloud-ci/gates/oya-cloud-ci-freshness-app/src/rust_toolchain_drift.rs"
exclusions = ["cloud/cloud-kernel/"]

[supply_chain]
license_policy = "deny.toml"
advisory_policy = "cargo-deny"
audit_policy = "cargo-vet"
stewardship_registry = "specs/oss-stewardship-registry.json"
bot_gate = "cloud-ci-dependency-automation"

[[managed_file]]
path = "rust-toolchain.toml"
role = "rust-toolchain-pin"
update = "sync-rust-pin"
reason = "fixture"

[[managed_file]]
path = "Cargo.toml"
role = "workspace-msrv"
update = "sync-rust-pin"
reason = "fixture"

[[managed_file]]
path = "Dockerfile.distroless"
role = "container-builder-toolchain"
update = "sync-rust-pin"
reason = "fixture"
"#
    )
}

#[test]
fn live_tree_has_valid_owned_dependency_automation_contract() {
    let root = repo_root();
    let report = evaluate_repo(&root).expect("evaluate live repo");
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "live oya-deps contract should be green: {:#?}",
        report.findings
    );
}

#[test]
fn missing_oya_deps_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    fs::remove_file(root.join("oya-deps.toml")).unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "DEP-AUTO-MISSING-CONFIG")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn external_bot_config_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    fs::write(root.join("renovate.json"), "{}\n").unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "DEP-AUTO-EXTERNAL-BOT-CONFIG")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn dependabot_yaml_variant_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    fs::create_dir_all(root.join(".github")).unwrap();
    fs::write(root.join(".github/dependabot.yaml"), "version: 2\n").unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "DEP-AUTO-EXTERNAL-BOT-CONFIG"
                && finding.path == ".github/dependabot.yaml")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unknown_config_key_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    let mut text = fs::read_to_string(root.join("oya-deps.toml")).unwrap();
    text.push_str("\nunknown = true\n");
    fs::write(root.join("oya-deps.toml"), text).unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "DEP-AUTO-UNKNOWN-KEY")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rust_pin_split_brain_is_red() {
    let root = temp_root();
    write_minimal_candidate(&root, "1.96.0");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace.package]\nrust-version = \"1.95.0\"\n",
    )
    .unwrap();
    let report = evaluate_repo(&root).expect("evaluate fixture");
    assert_eq!(report.verdict, Verdict::Red);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.code == "DEP-AUTO-RUST-PIN-DRIFT")
    );
    fs::remove_dir_all(root).unwrap();
}
