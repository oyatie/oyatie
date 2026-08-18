//! Proves Cargo builds the concrete producer resources before workspace tests execute.

#![allow(clippy::expect_used)]

use std::path::Path;
use std::process::Command;

const ADAPTER: &str = env!("CARGO_BIN_EXE_oya-cloud-ci-cargo-test-producer-adapter");
const PRODUCER: &str = env!("CARGO_BIN_EXE_oya-cloud-ci-accounting-registry-app");

#[test]
fn cargo_exposes_regular_adapter_and_producer_artifacts() {
    for (label, path) in [("adapter", ADAPTER), ("producer", PRODUCER)] {
        let metadata = Path::new(path)
            .metadata()
            .unwrap_or_else(|error| panic!("{label} artifact {path}: {error}"));
        assert!(metadata.is_file(), "{label} artifact is not a regular file");
    }
}

#[test]
fn adapter_fails_closed_without_declared_arguments() {
    let output = Command::new(ADAPTER)
        .output()
        .expect("execute Cargo test-resource adapter");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing required --repo-root argument")
    );
}
