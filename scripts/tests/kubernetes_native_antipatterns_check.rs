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
    assert_eq!(evaluation.required_patterns, 38);
    assert_eq!(evaluation.forbidden_anti_patterns, 38);
    assert_eq!(evaluation.official_sources, 41);
    assert_eq!(evaluation.first_party_helm_evidence_clean_files, 10);
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
fn contract_rejects_missing_cue_first_cell_pod_config_authority() {
    let contract = read_repo_file("specs/kubernetes-native-anti-patterns.json").replace(
        "\"id\": \"cue_first_cell_pod_config_authority\"",
        "\"id\": \"cue_first_cell_pod_config_authority_removed\"",
    );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures
            .iter()
            .any(|failure| failure
                == "required_patterns missing cue_first_cell_pod_config_authority"),
        "{:?}",
        failures
    );
}

#[test]
fn contract_rejects_missing_hyperscaler_adoption_fitness() {
    let contract = read_repo_file("specs/kubernetes-native-anti-patterns.json").replace(
        "\"id\": \"hyperscaler_adoption_fitness_for_everything\"",
        "\"id\": \"hyperscaler_adoption_fitness_for_everything_removed\"",
    );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures.iter().any(|failure| failure
            == "required_patterns missing hyperscaler_adoption_fitness_for_everything"),
        "{:?}",
        failures
    );
}

#[test]
fn contract_rejects_missing_rust_cue_conformance_lane() {
    let contract = read_repo_file("specs/kubernetes-native-anti-patterns.json").replace(
        "\"id\": \"rust_cue_compatible_desired_state_engine_conformance_lane\"",
        "\"id\": \"rust_cue_compatible_desired_state_engine_conformance_lane_removed\"",
    );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures.iter().any(|failure| failure
            == "required_patterns missing rust_cue_compatible_desired_state_engine_conformance_lane"),
        "{:?}",
        failures
    );
}

#[test]
fn contract_rejects_missing_non_hyperscaler_fit_antipattern() {
    let contract = read_repo_file("specs/kubernetes-native-anti-patterns.json").replace(
        "\"id\": \"non_hyperscaler_fit_decision_or_cargo_cult_tooling\"",
        "\"id\": \"non_hyperscaler_fit_decision_or_cargo_cult_tooling_removed\"",
    );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures.iter().any(|failure| failure
            == "forbidden_anti_patterns missing non_hyperscaler_fit_decision_or_cargo_cult_tooling"),
        "{:?}",
        failures
    );
}

#[test]
fn desired_state_standard_rejects_helm_first_canonical_wording() {
    let mut standard = read_repo_file("docs/standards/kubernetes-desired-state-authority.md");
    standard.push_str("\n# Helm chart convention (canonical)\n");
    let failures = gate::desired_state_authority_failures(
        &standard,
        &read_repo_file("docs/standards/helm-chart-convention.md"),
        &read_repo_file("docs/README.md"),
        &read_repo_file("docs/standards/INDEX.md"),
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("Helm chart convention (canonical)")),
        "{:?}",
        failures
    );
}

#[test]
fn desired_state_standard_rejects_missing_cue_first_source() {
    let standard = read_repo_file("docs/standards/kubernetes-desired-state-authority.md").replace(
        "CUE packages are the first-party source of truth",
        "CUE packages",
    );
    let failures = gate::desired_state_authority_failures(
        &standard,
        &read_repo_file("docs/standards/helm-chart-convention.md"),
        &read_repo_file("docs/README.md"),
        &read_repo_file("docs/standards/INDEX.md"),
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("CUE packages are the first-party source of truth")),
        "{:?}",
        failures
    );
}

#[test]
fn desired_state_standard_rejects_missing_rust_engine_conformance_boundary() {
    let standard = read_repo_file("docs/standards/kubernetes-desired-state-authority.md").replace(
        "not first-party authority until conformance is proven",
        "production authority",
    );
    let failures = gate::desired_state_authority_failures(
        &standard,
        &read_repo_file("docs/standards/helm-chart-convention.md"),
        &read_repo_file("docs/README.md"),
        &read_repo_file("docs/standards/INDEX.md"),
    );
    assert!(
        failures.iter().any(
            |failure| failure.contains("not first-party authority until conformance is proven")
        ),
        "{:?}",
        failures
    );
}

#[test]
fn first_party_helm_evidence_scan_rejects_active_helm_paths() {
    let failures = gate::first_party_helm_evidence_text_failures(
        "example-scorecard.json",
        "evidence_pattern: microservices/<ms>/iac/helm/<chart>/values.yaml",
    );
    for expected in ["microservices/<ms>/iac/helm", "CUE", "generated KRM"] {
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "missing {expected:?} in {failures:?}"
        );
    }
}

#[test]
fn helm_redirect_rejects_current_guidance() {
    let mut redirect = read_repo_file("docs/standards/helm-chart-convention.md");
    redirect.push_str("\nEvery µservice depends on a shared helper chart.\n");
    let failures = gate::desired_state_authority_failures(
        &read_repo_file("docs/standards/kubernetes-desired-state-authority.md"),
        &redirect,
        &read_repo_file("docs/README.md"),
        &read_repo_file("docs/standards/INDEX.md"),
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("Every µservice depends on")),
        "{:?}",
        failures
    );
}

#[test]
fn docs_readme_rejects_active_helm_standard_pointer() {
    let mut docs_readme = read_repo_file("docs/README.md");
    docs_readme.push_str("\n[helm-chart-convention.md](standards/helm-chart-convention.md)\n");
    let failures = gate::desired_state_authority_failures(
        &read_repo_file("docs/standards/kubernetes-desired-state-authority.md"),
        &read_repo_file("docs/standards/helm-chart-convention.md"),
        &docs_readme,
        &read_repo_file("docs/standards/INDEX.md"),
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("standards/helm-chart-convention.md")),
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
fn contract_rejects_missing_stateless_cache_and_runner_antipatterns() {
    let contract = read_repo_file("specs/kubernetes-native-anti-patterns.json")
        .replace(
            "\"id\": \"content_addressed_remote_cache_not_workspace_state\"",
            "\"id\": \"content_addressed_remote_cache_not_workspace_state_removed\"",
        )
        .replace(
            "\"id\": \"stateful_runner_workspace_pool\"",
            "\"id\": \"stateful_runner_workspace_pool_removed\"",
        );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures.iter().any(|failure| failure
            == "required_patterns missing content_addressed_remote_cache_not_workspace_state"),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure
                == "forbidden_anti_patterns missing stateful_runner_workspace_pool"),
        "{:?}",
        failures
    );
}

#[test]
fn contract_rejects_missing_supply_chain_and_fairness_guardrails() {
    let contract = read_repo_file("specs/kubernetes-native-anti-patterns.json")
        .replace(
            "\"id\": \"hermetic_provenance_signed_artifacts\"",
            "\"id\": \"hermetic_provenance_signed_artifacts_removed\"",
        )
        .replace(
            "\"id\": \"unbounded_parallelism_without_fairness\"",
            "\"id\": \"unbounded_parallelism_without_fairness_removed\"",
        );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures
            .iter()
            .any(|failure| failure
                == "required_patterns missing hermetic_provenance_signed_artifacts"),
        "{:?}",
        failures
    );
    assert!(
        failures.iter().any(|failure| failure
            == "forbidden_anti_patterns missing unbounded_parallelism_without_fairness"),
        "{:?}",
        failures
    );
}

#[test]
fn contract_rejects_missing_controller_coordination_and_rbac_guardrails() {
    let contract = read_repo_file("specs/kubernetes-native-anti-patterns.json")
        .replace(
            "\"id\": \"lease_based_controller_coordination\"",
            "\"id\": \"lease_based_controller_coordination_removed\"",
        )
        .replace(
            "\"id\": \"cluster_admin_or_wildcard_rbac_for_runners\"",
            "\"id\": \"cluster_admin_or_wildcard_rbac_for_runners_removed\"",
        );
    let failures = gate::contract_failures(&contract);
    assert!(
        failures
            .iter()
            .any(|failure| failure
                == "required_patterns missing lease_based_controller_coordination"),
        "{:?}",
        failures
    );
    assert!(
        failures.iter().any(|failure| failure
            == "forbidden_anti_patterns missing cluster_admin_or_wildcard_rbac_for_runners"),
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
fn generated_config_rejects_stateful_or_cross_region_cache_policy() {
    let config = read_repo_file("specs/generated/oya-ci-controller-config.generated.yaml")
        .replace("statelessJobPods: true", "statelessJobPods: false")
        .replace(
            "localDiskCachePersistsAfterPod: false",
            "localDiskCachePersistsAfterPod: true",
        )
        .replace(
            "hotPathCrossRegionIoAllowed: false",
            "hotPathCrossRegionIoAllowed: true",
        );
    let failures = gate::controller_config_failures(&config);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("statelessJobPods: true")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("localDiskCachePersistsAfterPod: false")),
        "{:?}",
        failures
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("hotPathCrossRegionIoAllowed: false")),
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
fn active_promotion_spec_rejects_retired_substrate_authority() {
    let mut spec = read_repo_file("specs/agentic-slo-gated-promotion.json");
    spec.push_str("\n{\"promotion_authority\":\"Jenkins/Forgejo via oya-dev-cli\"}\n");
    let failures = gate::active_promotion_spec_failures(&spec);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("forbidden retired promotion authority")),
        "{:?}",
        failures
    );
}

#[test]
fn vcs_registry_tombstone_rejects_stale_bridge_wording() {
    let mut readme = read_repo_file("registry/vcs/README.md");
    readme.push_str("\nretired in favour of git + Jenkins + self-hosted Forgejo\n");
    let failures = gate::vcs_registry_tombstone_failures(
        &readme,
        &read_repo_file("registry/vcs/event-router.yaml"),
        &read_repo_file("registry/vcs/concurrent-safe-paths.yaml"),
    );
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("git + Jenkins + self-hosted Forgejo")),
        "{:?}",
        failures
    );
}

#[test]
fn vcs_registry_tombstone_rejects_active_loader_language() {
    let mut safe_paths = read_repo_file("registry/vcs/concurrent-safe-paths.yaml");
    safe_paths.push_str("\n# Loader: retired-app/src/projected_merge_state.rs\n");
    let failures = gate::vcs_registry_tombstone_failures(
        &read_repo_file("registry/vcs/README.md"),
        &read_repo_file("registry/vcs/event-router.yaml"),
        &safe_paths,
    );
    assert!(
        failures.iter().any(|failure| failure.contains("Loader:")),
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
