use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn repo_root() -> PathBuf {
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

pub(crate) fn temp_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("oya-dependency-automation-test-{nonce}-{count}"));
    fs::create_dir_all(&path).expect("create temp root");
    path
}

pub(crate) fn write_minimal_candidate(root: &Path, pin: &str) {
    fs::create_dir_all(root.join("build/toolchains")).unwrap();
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
    fs::create_dir_all(root.join("build/images")).unwrap();
    fs::write(
        root.join("build/images/Dockerfile.distroless"),
        format!("ARG RUST_VERSION={pin}\n"),
    )
    .unwrap();
    fs::write(
        root.join("build/toolchains/BUCK"),
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
    fs::write(root.join("deps.toml"), oya_deps(pin)).unwrap();
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
drift_guard = "ci/facade/generated-artifact-freshness/src/rust_toolchain_drift.rs"
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
path = "build/images/Dockerfile.distroless"
role = "container-builder-toolchain"
update = "sync-rust-pin"
reason = "fixture"
"#
    )
}
