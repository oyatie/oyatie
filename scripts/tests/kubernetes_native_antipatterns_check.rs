#![allow(dead_code)]

#[path = "../ci/assert-kubernetes-native-antipatterns.rs"]
mod gate;

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    std::env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_repo_file(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path)).unwrap_or_else(|error| {
        panic!("read {}: {}", path, error);
    })
}

#[test]
fn checked_in_kubernetes_native_antipattern_contract_passes() {
    let evaluation = gate::evaluate(Path::new(&repo_root()));
    assert_eq!(evaluation.verdict, "PASS", "{:?}", evaluation.failures);
    assert!(evaluation.failures.is_empty());
    assert_eq!(evaluation.required_patterns, 12);
    assert_eq!(evaluation.forbidden_anti_patterns, 13);
    assert_eq!(evaluation.official_sources, 9);
}

#[test]
fn contract_rejects_github_actions_as_durable_authority() {
    let contract = read_repo_file("specs/kubernetes-native-anti-patterns.json").replace(
        "\"github_actions_durable_authority\": false",
        "\"github_actions_durable_authority\": true",
    );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("github_actions_durable_authority=false")),
        "{:?}",
        failures
    );
}

#[test]
fn contract_rejects_missing_untrusted_runtime_guard() {
    let contract = read_repo_file("specs/kubernetes-native-anti-patterns.json").replace(
        "\"id\": \"sandboxed_untrusted_runtime\"",
        "\"id\": \"sandboxed_untrusted_runtime_removed\"",
    );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "required_patterns missing sandboxed_untrusted_runtime"),
        "{:?}",
        failures
    );
}

#[test]
fn contract_rejects_missing_blind_pod_deletion_antipattern() {
    let contract = read_repo_file("specs/kubernetes-native-anti-patterns.json").replace(
        "\"id\": \"blind_kubectl_delete_pods\"",
        "\"id\": \"blind_kubectl_delete_pods_removed\"",
    );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures
            .iter()
            .any(|failure| failure == "forbidden_anti_patterns missing blind_kubectl_delete_pods"),
        "{:?}",
        failures
    );
}

#[test]
fn generated_config_rejects_unsafe_pod_security_defaults() {
    let config = read_repo_file("specs/generated/oya-ci-controller-config.generated.yaml")
        .replace(
            "automountServiceAccountToken: false",
            "automountServiceAccountToken: true",
        )
        .replace(
            "allowPrivilegeEscalation: false",
            "allowPrivilegeEscalation: true",
        )
        .replace(
            "runtimeClassName: \"sandboxed\"",
            "runtimeClassName: \"default\"",
        );
    let failures = gate::controller_config_failures(&config);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("automountServiceAccountToken: false")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("allowPrivilegeEscalation: false")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("runtimeClassName: \\\"sandboxed\\\"")),
        "{:?}",
        failures
    );
}

#[test]
fn generated_config_rejects_live_mutation_commands() {
    let mut config = read_repo_file("specs/generated/oya-ci-controller-config.generated.yaml");
    config.push_str("\n      buck2Commands:\n        - \"kubectl delete pods --all\"\n");
    let failures = gate::controller_config_failures(&config);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("forbidden live/ad-hoc command")),
        "{:?}",
        failures
    );
}

#[test]
fn root_hub_and_masterplan_pointer_are_required() {
    let evaluation = gate::evaluate(Path::new(&repo_root()));
    assert!(
        evaluation
            .failures
            .iter()
            .all(|failure| !failure.contains("kubernetes_native_anti_patterns")),
        "{:?}",
        evaluation.failures
    );
}
