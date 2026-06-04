#![allow(dead_code)]

#[path = "../ci/assert-oya-ci-controller-config.rs"]
mod gate;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|error| {
        panic!("read {}: {}", path, error);
    })
}

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "oyatie-oya-ci-controller-config-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap_or_else(|error| {
        panic!("create temp dir {}: {}", path.display(), error);
    });
    path
}

fn copy_fixture(root: &Path) {
    for rel in [
        "specs/oya-ci-controller-config-contract.json",
        "specs/generated/oya-ci-controller-config.generated.yaml",
        "specs/oya-ci-prowjob-registry.json",
        "specs/generated/oya-ci-prowjob-registry.generated.yaml",
        "specs/root-hub-pointers.json",
    ] {
        let destination = root.join(rel);
        fs::create_dir_all(destination.parent().unwrap()).unwrap_or_else(|error| {
            panic!("create fixture parent {}: {}", destination.display(), error);
        });
        fs::write(&destination, read_repo_file(rel)).unwrap_or_else(|error| {
            panic!("write fixture {}: {}", destination.display(), error);
        });
    }
}

#[test]
fn checked_in_controller_config_contract_passes() {
    let evaluation = gate::evaluate(&repo_root());
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.failures.is_empty(), "{:?}", evaluation.failures);
    assert_eq!(
        evaluation.contract,
        "specs/oya-ci-controller-config-contract.json"
    );
    assert_eq!(
        evaluation.config,
        "specs/generated/oya-ci-controller-config.generated.yaml"
    );
    assert_eq!(evaluation.required_context, "oya-ci-required");
    assert!(evaluation.jobs_checked >= 7);
    assert!(evaluation.local_static_only);
    assert!(!evaluation.live_kubernetes_mutated);
}

#[test]
fn contract_rejects_live_authority_or_missing_buck2_check() {
    let contract = read_repo_file("specs/oya-ci-controller-config-contract.json")
        .replace("\"live_kubernetes_mutated\": false", "\"live_kubernetes_mutated\": true")
        .replace(
            "\"buck2_check\": \"buck2 build //:oya-ci-controller-config-check\"",
            "\"buck2_check\": \"kubectl apply -f specs/generated/oya-ci-controller-config.generated.yaml\"",
        );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("live_kubernetes_mutated=false")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("publish Buck2 check command")),
        "{:?}",
        failures
    );
}

#[test]
fn config_rejects_weakened_security_defaults() {
    let config = read_repo_file("specs/generated/oya-ci-controller-config.generated.yaml")
        .replace(
            "workloadIdentityRequired: true",
            "workloadIdentityRequired: false",
        )
        .replace(
            "nodeMetadataAccess: \"blocked\"",
            "nodeMetadataAccess: \"allowed\"",
        )
        .replace(
            "defaultDenyNetworkPolicy: true",
            "defaultDenyNetworkPolicy: false",
        )
        .replace(
            "runtimeClassRequiredForUntrusted: true",
            "runtimeClassRequiredForUntrusted: false",
        );
    let failures = gate::config_failures(&config);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("require workload identity")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("block node metadata")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("default-deny NetworkPolicy")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("sandboxed RuntimeClass")),
        "{:?}",
        failures
    );
}

#[test]
fn config_rejects_untrusted_job_without_sandbox() {
    let config = read_repo_file("specs/generated/oya-ci-controller-config.generated.yaml")
        .replacen(
            "runtimeClassName: \"sandboxed\"",
            "runtimeClassName: \"default\"",
            1,
        );
    let failures = gate::config_failures(&config);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("untrusted jobs must use sandboxed runtime")),
        "{:?}",
        failures
    );
}

#[test]
fn config_rejects_non_buck2_or_live_mutation_job_command() {
    let config = read_repo_file("specs/generated/oya-ci-controller-config.generated.yaml")
        .replacen(
            "buck2 build //:repo-hygiene-automation-check",
            "kubectl delete pods --all",
            1,
        );
    let failures = gate::config_failures(&config);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("command must be Buck2 build authority")),
        "{:?}",
        failures
    );
}

#[test]
fn root_hub_pointer_is_required() {
    let root = temp_dir("missing-root-hub-pointer");
    copy_fixture(&root);
    let path = root.join("specs/root-hub-pointers.json");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read root hub fixture: {}", error))
        .replace(
            "oya_ci_controller_config_contract",
            "removed_controller_config_contract",
        );
    fs::write(&path, text).unwrap_or_else(|error| panic!("write root hub fixture: {}", error));

    let evaluation = gate::evaluate(&root);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(evaluation.verdict, "FAIL");
    assert!(
        evaluation
            .failures
            .iter()
            .any(|failure| failure.contains("root hub must expose")),
        "{:?}",
        evaluation.failures
    );
}
