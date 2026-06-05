//! Local/static hyperscaler/cloud/Kubernetes-native anti-pattern guard.
//!
//! This validates checked-in policy contracts and generated oya-ci controller
//! config. It deliberately does not mutate Kubernetes, GitHub branch
//! protection, CI statuses, or deployment state.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CONTRACT_PATH: &str = "specs/kubernetes-native-anti-patterns.json";
const ROOT_HUB_PATH: &str = "specs/root-hub-pointers.json";
const MASTERPLAN_PATH: &str = "specs/masterplan.json";
const REPO_HYGIENE_PATH: &str = "specs/repo-hygiene-automation.json";
const CONTROLLER_CONTRACT_PATH: &str = "specs/oya-ci-controller-config-contract.json";
const CONTROLLER_CONFIG_PATH: &str = "specs/generated/oya-ci-controller-config.generated.yaml";
const AGENTIC_SLO_PROMOTION_PATH: &str = "specs/agentic-slo-gated-promotion.json";
const AGENTS_PATH: &str = "AGENTS.md";
const CLAUDE_PATH: &str = "CLAUDE.md";
const BUCK_PATH: &str = "BUCK";
const VCS_REGISTRY_README_PATH: &str = "registry/vcs/README.md";
const VCS_EVENT_ROUTER_PATH: &str = "registry/vcs/event-router.yaml";
const VCS_CONCURRENT_SAFE_PATHS_PATH: &str = "registry/vcs/concurrent-safe-paths.yaml";
const CHECK_COMMAND: &str = "buck2 build //:kubernetes-native-anti-pattern-check";

const OFFICIAL_SOURCES: &[&str] = &[
    "https://architecture.cncf.io/",
    "https://kubernetes.io/docs/concepts/architecture/controller/",
    "https://kubernetes.io/docs/concepts/security/pod-security-standards/",
    "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
    "https://kubernetes.io/docs/tasks/configure-pod-container/security-context/",
    "https://kubernetes.io/docs/concepts/containers/runtime-class/",
    "https://kubernetes.io/docs/tasks/configure-pod-container/configure-service-account/",
    "https://kubernetes.io/docs/tasks/administer-cluster/safely-drain-node/",
    "https://docs.prow.k8s.io/docs/jobs/",
    "https://docs.prow.k8s.io/docs/life-of-a-prow-job/",
    "https://docs.prow.k8s.io/docs/components/pod-utilities/",
    "https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/",
    "https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/#local-ephemeral-storage",
    "https://buck2.build/docs/users/remote_execution/",
    "https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions",
    "https://docs.github.com/en/actions/concepts/security/openid-connect",
    "https://slsa.dev/spec/v1.2/",
    "https://kubernetes.io/docs/reference/access-authn-authz/validating-admission-policy/",
    "https://kubernetes.io/docs/concepts/policy/resource-quotas/",
    "https://kubernetes.io/docs/concepts/scheduling-eviction/pod-priority-preemption/",
    "https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue",
    "https://docs.prow.k8s.io/docs/components/core/tide/",
    "https://opentelemetry.io/docs/concepts/signals/",
    "https://kubernetes.io/docs/concepts/architecture/leases/",
    "https://kubernetes.io/docs/concepts/workloads/controllers/ttlafterfinished/",
    "https://kubernetes.io/docs/concepts/scheduling-eviction/topology-spread-constraints/",
    "https://kubernetes.io/docs/tasks/run-application/configure-pdb/",
    "https://kubernetes.io/docs/concepts/cluster-administration/flow-control/",
    "https://kubernetes.io/docs/reference/access-authn-authz/rbac/",
    "https://kubernetes.io/docs/concepts/configuration/secret/#information-security-for-secrets",
    "https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration/",
    "https://cue.dev/docs/getting-started-with-kubernetes-cue/",
    "https://helm.sh/docs/topics/charts/",
];

const REQUIRED_PATTERNS: &[&str] = &[
    "controller_reconciliation_over_manual_mutation",
    "trusted_controller_owned_oya_ci_required",
    "buck2_job_authority",
    "workload_identity_metadata_blocked",
    "restricted_pod_security_defaults",
    "default_deny_network_policy",
    "sandboxed_untrusted_runtime",
    "service_mesh_mtls_intent",
    "pointer_thin_or_generated_shared_surfaces",
    "cue_first_cell_pod_config_authority",
    "helm_adapter_compatibility_wrapper",
    "scale_to_zero_eligibility_gate",
    "shadow_adapters_not_authority",
    "native_scm_ci_cd_service_seams",
    "cloud_auth_product_auth_decoupled_until_rewire",
    "disposable_prowjob_pods_remote_state",
    "content_addressed_remote_cache_not_workspace_state",
    "regional_cell_local_cache_and_artifact_store",
    "trusted_cache_promotion_and_cold_cache_probes",
    "bounded_ephemeral_storage_and_io_metrics",
    "hermetic_provenance_signed_artifacts",
    "least_privilege_oidc_and_job_permissions",
    "policy_as_code_admission_guardrails",
    "quota_priority_and_fair_scheduling",
    "trusted_target_inventory_before_candidate_checkout",
    "generated_lane_shards_with_merge_queue",
    "slo_gated_progressive_delivery",
    "observability_first_control_loops",
    "lease_based_controller_coordination",
    "ttl_and_owner_reference_cleanup",
    "topology_spread_and_dedicated_node_pools",
    "pdb_backed_graceful_disruption",
    "api_priority_fairness_for_control_planes",
    "watch_cache_informers_over_polling",
    "rbac_minimal_roles_per_job_and_controller",
    "kms_encrypted_secrets_and_external_secret_boundary",
];

const FORBIDDEN_ANTI_PATTERNS: &[&str] = &[
    "blind_kubectl_delete_pods",
    "candidate_owned_manual_status_or_merge_truth",
    "github_actions_as_durable_ci_authority",
    "retired_external_substrate_bridge_authority",
    "single_hand_edited_workflow_bottleneck",
    "new_python_or_shell_gate_sprawl",
    "oya_cli_revival",
    "cargo_or_tarpaulin_as_monorepo_authority_over_buck2",
    "first_party_helm_template_authority",
    "node_metadata_access_enabled",
    "default_service_account_token_mount",
    "privileged_or_mutable_container_defaults",
    "missing_runtime_class_for_untrusted_jobs",
    "missing_default_deny_network_policy",
    "stateful_runner_workspace_pool",
    "pod_local_cache_as_correctness_or_state_authority",
    "cross_trust_cache_poisoning",
    "cross_region_hot_path_ci_io",
    "mutable_or_overwritten_ci_artifacts",
    "unbounded_ephemeral_storage",
    "privileged_dind_or_host_socket_runner",
    "floating_unpinned_actions_images_or_toolchains",
    "static_long_lived_pipeline_secrets",
    "candidate_controlled_check_scope",
    "shared_trust_domain_cluster_namespace_or_account",
    "direct_deploy_from_ci_job",
    "kubectl_as_cd_or_admission_bypass",
    "kubernetes_api_as_application_database",
    "unbounded_parallelism_without_fairness",
    "process_local_leader_or_sticky_controller_lock",
    "finished_jobs_and_orphaned_resources_left_forever",
    "unconstrained_topology_or_node_pool_placement",
    "force_delete_or_pdb_bypass_for_service_workloads",
    "api_server_polling_flood_or_unbounded_watch_clients",
    "cluster_admin_or_wildcard_rbac_for_runners",
    "kubernetes_secret_as_long_lived_app_database",
    "manual_cleanup_sweeps_as_primary_lifecycle",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
    pub required_patterns: usize,
    pub forbidden_anti_patterns: usize,
    pub official_sources: usize,
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn compact_json_text(input: &str) -> String {
    input.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn contains_json_string(text: &str, value: &str) -> bool {
    text.contains(&format!("\"{}\"", json_escape(value)))
}

fn has_json_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&format!(
        "\"{}\":{}",
        key,
        if value { "true" } else { "false" }
    ))
}

fn count_json_key_value(text: &str, key: &str, value: &str) -> usize {
    compact_json_text(text)
        .matches(&format!("\"{}\":\"{}\"", key, json_escape(value)))
        .count()
}

fn read(root: &Path, rel: &str, failures: &mut Vec<String>) -> String {
    match fs::read_to_string(root.join(rel)) {
        Ok(text) => text,
        Err(error) => {
            failures.push(format!("{rel}: read failed: {error}"));
            String::new()
        }
    }
}

fn require(condition: bool, failures: &mut Vec<String>, message: impl Into<String>) {
    if !condition {
        failures.push(message.into());
    }
}

fn require_contains(text: &str, needle: &str, failures: &mut Vec<String>, label: &str) {
    require(
        text.contains(needle),
        failures,
        format!("{label}: missing {needle:?}"),
    );
}

fn require_not_contains(text: &str, needle: &str, failures: &mut Vec<String>, label: &str) {
    require(
        !text.contains(needle),
        failures,
        format!("{label}: forbidden stale active-authority wording present: {needle:?}"),
    );
}

pub fn contract_failures(contract: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for (needle, message) in [
        (
            "\"status\": \"p00_local_static_policy_contract\"",
            "contract status must be p00_local_static_policy_contract",
        ),
        (
            "\"spec_id\": \"P00-KUBERNETES-NATIVE-ANTI-PATTERNS\"",
            "contract spec_id must be P00-KUBERNETES-NATIVE-ANTI-PATTERNS",
        ),
        (
            "\"buck2_check\": \"buck2 build //:kubernetes-native-anti-pattern-check\"",
            "contract must publish Buck2 check command",
        ),
        (
            "\"root_hub_pointer\": \"/specs/root-hub-pointers.json#entry_points.kubernetes_native_anti_patterns\"",
            "contract must require root hub pointer",
        ),
        (
            "\"generated_controller_config\": \"/specs/generated/oya-ci-controller-config.generated.yaml\"",
            "contract must bind generated controller config",
        ),
    ] {
        require_contains(contract, needle, &mut failures, message);
    }

    for (key, value) in [
        ("local_static_only", true),
        ("live_mutation_performed", false),
        ("buck2_authority", true),
        ("github_actions_shadow_only", true),
        ("github_actions_durable_authority", false),
        ("candidate_owned_truth_allowed", false),
        ("oya_cli_revival_allowed", false),
        ("live_kubernetes_mutated", false),
        ("branch_protection_mutated", false),
        ("github_required_context_changed", false),
        ("production_readiness_claimed", false),
        ("phase0_green_claimed", false),
    ] {
        require(
            has_json_bool(contract, key, value),
            &mut failures,
            format!("contract missing bool {key}={value}"),
        );
    }

    for source in OFFICIAL_SOURCES {
        require(
            contains_json_string(contract, source),
            &mut failures,
            format!("contract official_sources missing {source}"),
        );
    }

    for id in REQUIRED_PATTERNS {
        require(
            count_json_key_value(contract, "id", id) > 0,
            &mut failures,
            format!("required_patterns missing {id}"),
        );
    }
    for id in FORBIDDEN_ANTI_PATTERNS {
        require(
            count_json_key_value(contract, "id", id) > 0,
            &mut failures,
            format!("forbidden_anti_patterns missing {id}"),
        );
    }
    let valid_count = compact_json_text(contract)
        .matches("\"valid\":true")
        .count();
    require(
        valid_count >= REQUIRED_PATTERNS.len() + FORBIDDEN_ANTI_PATTERNS.len(),
        &mut failures,
        format!(
            "valid=true count too low: expected at least {}, got {}",
            REQUIRED_PATTERNS.len() + FORBIDDEN_ANTI_PATTERNS.len(),
            valid_count
        ),
    );

    failures
}

pub fn controller_config_failures(config: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for needle in [
        "kind: OyaCIControllerConfig",
        "requiredContext: \"oya-ci-required\"",
        "owner: \"trusted-controller\"",
        "candidateOwnedTruthAllowed: false",
        "shadowOnly: true",
        "mergeAuthority: false",
        "workloadIdentityRequired: true",
        "nodeMetadataAccess: \"blocked\"",
        "staticCloudSecretsAllowed: false",
        "automountServiceAccountToken: false",
        "defaultDenyNetworkPolicy: true",
        "serviceMeshMtls: \"required-for-service-to-service\"",
        "allowPrivilegeEscalation: false",
        "readOnlyRootFilesystem: true",
        "runAsNonRoot: true",
        "seccompProfile: \"RuntimeDefault\"",
        "runtimeClassRequiredForUntrusted: true",
        "- \"ALL\"",
        "securityProfile: \"restricted-untrusted-pr\"",
        "serviceAccount: \"oya-ci-untrusted-runner\"",
        "runtimeClassName: \"sandboxed\"",
        "statelessJobPods: true",
        "podLocalStateAuthority: false",
        "workspaceDestructionRequired: true",
        "remoteCacheAllowed: true",
        "localDiskCachePersistsAfterPod: false",
        "cacheAuthoritativeForCorrectness: false",
        "contentAddressedOnly: true",
        "trustDomainSeparated: true",
        "untrustedWritesQuarantined: true",
        "trustedPostsubmitPromotionRequired: true",
        "coldCacheProbeRequired: true",
        "cacheTopology: \"regional-cell-local-remote-cache\"",
        "artifactStore: \"immutable-object-store\"",
        "hotPathCrossRegionIoAllowed: false",
        "ephemeralStorageRequestsLimitsRequired: true",
        "remoteExecutionCasRequired: true",
    ] {
        require_contains(config, needle, &mut failures, CONTROLLER_CONFIG_PATH);
    }
    for forbidden in [
        "kubectl delete pod",
        "kubectl delete pods",
        "kubectl apply",
        "python3 ",
        ".sh",
    ] {
        require(
            !config.contains(forbidden),
            &mut failures,
            format!("{CONTROLLER_CONFIG_PATH}: forbidden live/ad-hoc command present: {forbidden}"),
        );
    }
    failures
}

pub fn active_promotion_spec_failures(spec: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for needle in [
        "Prow/Kubernetes-native oya-ci promotion-readiness Buck2 lane",
        "native release-conveyor promotion controllers",
        "release-conveyor dev-to-staging promotion controller",
        "release-conveyor staging-to-production promotion controller",
        "\"actor\": \"release-conveyor-controller\"",
        "Prow/Kubernetes-native oya-ci publishes source-bound oya-ci-required evidence",
        "audited replay request reconciled by the release-conveyor promotion controller",
    ] {
        require_contains(spec, needle, &mut failures, AGENTIC_SLO_PROMOTION_PATH);
    }

    for forbidden in [
        "Jenkins",
        "Forgejo",
        "oya-dev-cli",
        "cargo run -p",
        "oya gate",
        "\"actor\": \"jenkins\"",
    ] {
        require(
            !spec.contains(forbidden),
            &mut failures,
            format!(
                "{AGENTIC_SLO_PROMOTION_PATH}: forbidden retired promotion authority present: {forbidden}"
            ),
        );
    }
    failures
}

pub fn vcs_registry_tombstone_failures(
    readme: &str,
    event_router: &str,
    concurrent_safe_paths: &str,
) -> Vec<String> {
    let mut failures = Vec::new();

    for needle in [
        "Status: **retired historical evidence only**.",
        "This directory is not an active SCM",
        "promotion substrate",
        "/specs/gitops-vcs-replacement.json",
        "/specs/masterplan.json",
        "/specs/oya-ci-prow-capability-parity.json",
        "/specs/retired-external-substrate-registry.json",
        "buck2 build //:repo-hygiene-automation-check",
        "buck2 build //:kubernetes-native-anti-pattern-check",
        "Do not add rows or consumers under this directory.",
        "Do not revive the retired CLI wrapper",
        "Rust libraries/services, Buck2 targets, Prow",
        "jobs, and Git/GitHub adapters",
    ] {
        require_contains(readme, needle, &mut failures, VCS_REGISTRY_README_PATH);
    }

    for needle in [
        "Retired historical evidence only.",
        "not active routing, merge, CI",
        "agent-dispatch",
        "/specs/gitops-vcs-replacement.json",
        "/specs/masterplan.json",
        "/specs/oya-ci-prow-capability-parity.json",
        "Do not add rows.",
        "Do not wire new readers to this file.",
        "Do not treat the historical agent names below as executable surfaces.",
    ] {
        require_contains(event_router, needle, &mut failures, VCS_EVENT_ROUTER_PATH);
    }

    for needle in [
        "Retired historical evidence only.",
        "must not be used as active admission",
        "lease, or merge authority",
        "native SCM service/control-plane lease",
        "Git/GitHub adapter publication",
        "Rust/Buck2/Prow checks",
        "No active loader is allowed for this retired seed.",
    ] {
        require_contains(
            concurrent_safe_paths,
            needle,
            &mut failures,
            VCS_CONCURRENT_SAFE_PATHS_PATH,
        );
    }

    for (label, text) in [
        (VCS_REGISTRY_README_PATH, readme),
        (VCS_EVENT_ROUTER_PATH, event_router),
        (VCS_CONCURRENT_SAFE_PATHS_PATH, concurrent_safe_paths),
    ] {
        for forbidden in [
            "git + Jenkins + self-hosted Forgejo",
            "Jenkins CI",
            "canonical self-hostable CI/CD",
            "oya gate validate",
            "cargo run -q -p",
            "current file, runs the monotonic-event-log validator",
            "No silent additions",
            "Loader:",
            "Schema (consumed by",
        ] {
            require_not_contains(text, forbidden, &mut failures, label);
        }
    }

    failures
}

pub fn evaluate(root: &Path) -> Evaluation {
    let mut failures = Vec::new();
    let contract = read(root, CONTRACT_PATH, &mut failures);
    let root_hub = read(root, ROOT_HUB_PATH, &mut failures);
    let masterplan = read(root, MASTERPLAN_PATH, &mut failures);
    let repo_hygiene = read(root, REPO_HYGIENE_PATH, &mut failures);
    let controller_contract = read(root, CONTROLLER_CONTRACT_PATH, &mut failures);
    let controller_config = read(root, CONTROLLER_CONFIG_PATH, &mut failures);
    let agentic_slo_promotion = read(root, AGENTIC_SLO_PROMOTION_PATH, &mut failures);
    let agents = read(root, AGENTS_PATH, &mut failures);
    let claude = read(root, CLAUDE_PATH, &mut failures);
    let buck = read(root, BUCK_PATH, &mut failures);
    let vcs_readme = read(root, VCS_REGISTRY_README_PATH, &mut failures);
    let vcs_event_router = read(root, VCS_EVENT_ROUTER_PATH, &mut failures);
    let vcs_concurrent_safe_paths = read(root, VCS_CONCURRENT_SAFE_PATHS_PATH, &mut failures);

    failures.extend(contract_failures(&contract));
    failures.extend(controller_config_failures(&controller_config));
    failures.extend(active_promotion_spec_failures(&agentic_slo_promotion));
    failures.extend(vcs_registry_tombstone_failures(
        &vcs_readme,
        &vcs_event_router,
        &vcs_concurrent_safe_paths,
    ));

    for needle in [
        "\"kubernetes_native_anti_patterns\"",
        "\"current_path\": \"/specs/kubernetes-native-anti-patterns.json\"",
        "\"kubernetes_native_anti_patterns\": \"specs/kubernetes-native-anti-patterns.json\"",
    ] {
        require_contains(&root_hub, needle, &mut failures, ROOT_HUB_PATH);
    }

    for needle in [
        "kubernetes_native_antipatterns_green",
        CHECK_COMMAND,
        "/specs/kubernetes-native-anti-patterns.json",
        "controller_reconciliation_over_manual_mutation",
        "blind_kubectl_delete_pods",
        "native_scm_ci_cd_service_seams",
        "disposable_prowjob_pods_remote_state",
        "content_addressed_remote_cache_not_workspace_state",
        "stateful_runner_workspace_pool",
        "cross_trust_cache_poisoning",
        "lease_based_controller_coordination",
        "topology_spread_and_dedicated_node_pools",
        "cluster_admin_or_wildcard_rbac_for_runners",
    ] {
        require_contains(&masterplan, needle, &mut failures, MASTERPLAN_PATH);
    }

    for needle in [
        "kubernetes_native_anti_pattern_contract",
        "specs/kubernetes-native-anti-patterns.json",
        CHECK_COMMAND,
        "controller_reconciliation_over_manual_mutation",
        "blind_kubectl_delete_pods",
        "github_actions_as_durable_ci_authority",
        "lease_based_controller_coordination",
        "manual_cleanup_sweeps_as_primary_lifecycle",
    ] {
        require_contains(&repo_hygiene, needle, &mut failures, REPO_HYGIENE_PATH);
    }

    for needle in [
        "workload_identity_required",
        "node_metadata_access",
        "automount_service_account_token",
        "default_deny_network_policy",
        "runtime_class_required_for_untrusted",
        "forbid_live_kubectl_mutation_commands",
    ] {
        require_contains(
            &controller_contract,
            needle,
            &mut failures,
            CONTROLLER_CONTRACT_PATH,
        );
    }

    for (label, text) in [
        (AGENTS_PATH, agents.as_str()),
        (CLAUDE_PATH, claude.as_str()),
    ] {
        require_contains(text, "Prow/Kubernetes-native", &mut failures, label);
        require_contains(
            text,
            "buck2 build //:repo-hygiene-automation-check",
            &mut failures,
            label,
        );
        require(
            !text.contains("kubectl delete pods"),
            &mut failures,
            format!("{label}: must not recommend blind pod deletion"),
        );
    }

    for needle in [
        "kubernetes-native-anti-pattern-check",
        "scripts/ci/assert-kubernetes-native-antipatterns.rs",
        "scripts/tests/kubernetes_native_antipatterns_check.rs",
        "specs/kubernetes-native-anti-patterns.json",
        "specs/agentic-slo-gated-promotion.json",
        "registry/vcs/README.md",
        "registry/vcs/event-router.yaml",
        "registry/vcs/concurrent-safe-paths.yaml",
    ] {
        require_contains(&buck, needle, &mut failures, BUCK_PATH);
    }

    Evaluation {
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned(),
        failures,
        required_patterns: REQUIRED_PATTERNS.len(),
        forbidden_anti_patterns: FORBIDDEN_ANTI_PATTERNS.len(),
        official_sources: OFFICIAL_SOURCES.len(),
    }
}

fn render_json(evaluation: &Evaluation) -> String {
    let failures = evaluation
        .failures
        .iter()
        .map(|failure| format!("\"{}\"", json_escape(failure)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"verdict\":\"{}\",\"contract\":\"{}\",\"buck2_check\":\"{}\",\"local_static_only\":true,\"live_mutation_performed\":false,\"required_patterns\":{},\"forbidden_anti_patterns\":{},\"official_sources\":{},\"failures\":[{}]}}",
        evaluation.verdict,
        CONTRACT_PATH,
        CHECK_COMMAND,
        evaluation.required_patterns,
        evaluation.forbidden_anti_patterns,
        evaluation.official_sources,
        failures
    )
}

fn config() -> (PathBuf, bool) {
    let mut json = false;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--json" => json = true,
            unknown => {
                eprintln!("assert-kubernetes-native-antipatterns: unknown argument {unknown}");
                std::process::exit(2);
            }
        }
    }
    let root = env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    (root, json)
}

fn main() {
    let (root, json) = config();
    let evaluation = evaluate(&root);
    if json || evaluation.failures.is_empty() {
        println!("{}", render_json(&evaluation));
    }
    if !evaluation.failures.is_empty() {
        if !json {
            eprintln!("kubernetes-native-anti-patterns: RED");
            for failure in &evaluation.failures {
                eprintln!("- {failure}");
            }
        }
        std::process::exit(1);
    }
}
