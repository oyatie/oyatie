use std::process::Command;

use crate::support::{materialized_fixture, pinned_source_root};

#[test]
#[ignore = "requires the exact upstream Reindeer source snapshot"]
fn adapted_source_builds_as_the_pinned_provider() {
    let (_, fixture) = materialized_fixture(&pinned_source_root());
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let check = Command::new(&cargo)
        .args(["check", "--locked", "--offline", "--all-targets"])
        .current_dir(fixture.path())
        .env("CARGO_TARGET_DIR", fixture.path().join("target"))
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "adapted provider failed to build:\n{}\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr),
    );

    let fault_tests = Command::new(cargo)
        .args(["test", "--locked", "--offline", "artifact_"])
        .current_dir(fixture.path())
        .env("CARGO_TARGET_DIR", fixture.path().join("target"))
        .output()
        .unwrap();
    assert!(
        fault_tests.status.success(),
        "adapted provider fault tests failed:\n{}\n{}",
        String::from_utf8_lossy(&fault_tests.stdout),
        String::from_utf8_lossy(&fault_tests.stderr),
    );
}
