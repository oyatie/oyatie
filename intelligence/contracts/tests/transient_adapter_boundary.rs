#![allow(clippy::expect_used, clippy::panic)]

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn assert_omits(label: &str, haystack: &str, forbidden: &[&str]) {
    for phrase in forbidden {
        assert!(
            !haystack.contains(phrase),
            "{label} must use owned policy-engine / secret-provider wording, not direct transient engine target phrase: {phrase}"
        );
    }
}

#[test]
fn production_plan_targets_owned_policy_engine_and_secret_provider_ports() {
    let ip001 = read("intelligence/IP-001-cloud-intelligence-design.md");
    for required in [
        "owned secret-provider/KMS",
        "owned policy-engine",
        "transient adapter",
    ] {
        assert!(ip001.contains(required), "IP-001 missing {required}");
    }

    assert_omits(
        "IP-001",
        &ip001,
        &[
            "vault-only secrets",
            "OpenBao KV",
            "Cedar realm policies",
            "OpenBao unreachable",
        ],
    );
}

#[test]
fn capability_catalog_uses_policy_action_not_concrete_engine_action() {
    let capabilities =
        read("intelligence/capabilities/cloud-intelligence.capabilities.yaml");
    assert!(capabilities.contains("policy_action:"));
    assert_omits("capability catalog", &capabilities, &["cedar_action"]);
}

#[test]
fn kernel_and_core_rest_tests_do_not_name_transient_secret_or_policy_engines() {
    let kernel_lib =
        read("intelligence/core/kernel/src/lib.rs");
    let kernel_d7 = read(
        "intelligence/core/kernel/tests/d7_cross_tenant_forbid.rs",
    );
    let kernel_refresh = read(
        "intelligence/core/kernel/tests/refresh_failed_outcome.rs",
    );
    let rest_cargo = read("intelligence/adapters/rest/Cargo.toml");
    let rest_d8 = read(
        "intelligence/adapters/rest/tests/d8_secret_provider_envelope_encryption.rs",
    );

    assert_omits(
        "kernel lib",
        &kernel_lib,
        &["Cedar adapter consumes", "The Cedar adapter implements"],
    );
    assert_omits(
        "kernel D7 test",
        &kernel_d7,
        &["Cedar", "cedar", "openbao://"],
    );
    assert_omits(
        "kernel refresh test",
        &kernel_refresh,
        &["vault", "openbao://"],
    );
    assert_omits(
        "rest Cargo metadata",
        &rest_cargo,
        &["OpenBao envelope-encrypted"],
    );
    assert_omits(
        "rest D8 generic contract test",
        &rest_d8,
        &[
            "OpenBao",
            "real OpenBao adapter",
            "SealedVaultStore",
            "vault sealed",
        ],
    );
}
