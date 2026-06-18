// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::process::Command;

#[test]
fn cross_tenant_access_fuzz_gate_exercises_isolation_cases() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["gate", "validate", "cross-tenant-access-fuzz"])
        .output()
        .expect("cross-tenant access fuzz gate command runs");

    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("cross-tenant access fuzz validation passed: 7 cases")
    );
}

#[test]
fn cross_tenant_access_fuzz_gate_rejects_unknown_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_oya"))
        .args(["gate", "validate", "cross-tenant-access-fuzz", "--unknown"])
        .output()
        .expect("cross-tenant access fuzz gate command runs");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage: oya demo"));
}
