//! Local/static cloud-cell elasticity policy guard.
//!
//! This validates source-backed checked-in policy for Oyatie cloud cells and
//! Kubernetes pods. It deliberately performs no live Kubernetes, cloud-provider,
//! Helm, or CUE mutation/execution.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SPEC_PATH: &str = "specs/cloud-cell-elasticity-policy.json";
const ROOT_HUB_PATH: &str = "specs/root-hub-pointers.json";
const MASTERPLAN_PATH: &str = "specs/masterplan.json";
const REPO_HYGIENE_PATH: &str = "specs/repo-hygiene-automation.json";
const KUBERNETES_ANTI_PATTERN_PATH: &str = "specs/kubernetes-native-anti-patterns.json";
const CELL_MANIFEST_PATH: &str = "cloud/cell-lifecycle/manifest.json";
const BUCK_PATH: &str = "BUCK";
const CHECK_COMMAND: &str = "buck2 build //:cloud-cell-elasticity-policy-check";

const REQUIRED_URLS: &[&str] = &[
    "https://cue.dev/docs/getting-started-with-kubernetes-cue/",
    "https://cue.dev/docs/curated-module-kubernetes/",
    "https://helm.sh/docs/topics/charts/",
    "https://helm.sh/docs/chart_template_guide/",
    "https://cloud.google.com/kubernetes-engine/docs/concepts/horizontalpodautoscaler",
    "https://cloud.google.com/kubernetes-engine/docs/concepts/verticalpodautoscaler",
    "https://cloud.google.com/kubernetes-engine/docs/concepts/cluster-autoscaler",
    "https://cloud.google.com/kubernetes-engine/docs/concepts/workload-identity",
    "https://docs.aws.amazon.com/eks/latest/best-practices/karpenter.html",
    "https://docs.aws.amazon.com/eks/latest/best-practices/cas.html",
    "https://docs.aws.amazon.com/eks/latest/best-practices/identity-and-access-management.html",
    "https://aws.amazon.com/builders-library/workload-isolation-using-shuffle-sharding/",
    "https://keda.sh/docs/2.18/concepts/scaling-deployments/",
    "https://kubernetes.io/docs/concepts/workloads/autoscaling/",
    "https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/",
    "https://kubernetes.io/docs/concepts/workloads/controllers/ttlafterfinished/",
    "https://kubernetes.io/docs/concepts/security/pod-security-standards/",
    "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
    "https://architecture.cncf.io/",
];

const REQUIRED_POLICY_IDS: &[&str] = &[
    "hpa_vpa_metric_ownership",
    "node_autoscaler_adapter",
    "scale_to_zero_eligibility_gate",
    "auto_healing_slo_guardrails",
    "cell_rebalance_capacity_solver",
    "workload_identity_metadata_lockdown",
    "cell_level_fairness_and_quota",
    "cue_first_cell_pod_config_authority",
    "helm_adapter_compatibility_wrapper",
    "cue_generated_manifest_buck2_check",
];

const REQUIRED_FORBIDDEN: &[&str] = &[
    "first_party_helm_template_authority",
    "hpa_vpa_same_metric_auto_fight",
    "scale_stateful_or_control_plane_to_zero",
    "blind_pod_delete_as_healing_strategy",
    "node_metadata_credentials",
    "autoscaling_without_quota_or_fairness",
    "cell_rebalance_without_residency_permit_audit_or_rollback",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
    pub official_sources: usize,
    pub policy_ids: usize,
    pub forbidden_anti_patterns: usize,
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

fn read(root: &Path, rel: &str, failures: &mut Vec<String>) -> String {
    match fs::read_to_string(root.join(rel)) {
        Ok(text) => text,
        Err(error) => {
            failures.push(format!("{rel}: read failed: {error}"));
            String::new()
        }
    }
}

pub fn contract_failures(spec: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for (needle, message) in [
        (
            "\"spec_id\": \"CLOUD-CELL-ELASTICITY-POLICY\"",
            "spec_id must identify cloud-cell elasticity policy",
        ),
        (
            "\"status\": \"immediate_planning_contract\"",
            "status must remain immediate_planning_contract",
        ),
        (
            "\"first_party_desired_state\": \"cue\"",
            "first-party desired state must be CUE",
        ),
        (
            "\"helm_role\": \"external_chart_or_temporary_compatibility_adapter_only\"",
            "Helm must be adapter-only",
        ),
        (
            "\"buck2_check\": \"buck2 build //:cloud-cell-elasticity-policy-check\"",
            "spec must publish Buck2 check command",
        ),
    ] {
        require_contains(spec, needle, &mut failures, message);
    }

    for (key, value) in [
        ("local_static_only", true),
        ("live_kubernetes_mutation_performed", false),
        ("provider_live_validation_performed", false),
        ("helm_migration_completed", false),
        ("cue_runtime_dependency_introduced", false),
        ("helm_adapter_only", true),
        ("first_party_helm_template_authority", false),
    ] {
        require(
            has_json_bool(spec, key, value),
            &mut failures,
            format!("spec missing bool {key}={value}"),
        );
    }

    for url in REQUIRED_URLS {
        require_contains(spec, url, &mut failures, "official_sources");
    }
    for policy_id in REQUIRED_POLICY_IDS {
        require(
            count_json_key_value(spec, "id", policy_id) > 0,
            &mut failures,
            format!("policy/backlog id missing {policy_id}"),
        );
    }
    for forbidden in REQUIRED_FORBIDDEN {
        require(
            contains_json_string(spec, forbidden),
            &mut failures,
            format!("forbidden anti-pattern missing {forbidden}"),
        );
    }
    for phrase in [
        "VPA starts recommend-only",
        "Scale-to-zero is valid for stateless, event-driven, preview/dev, batch, or async workloads",
        "Node autoscaling is capacity supply, not workload demand policy",
        "Pods are disposable",
        "Cell rebalancing is a control-plane placement problem",
        "short-lived workload identity",
        "tenant, account, project, cell, and trust-tier quotas",
    ] {
        require_contains(spec, phrase, &mut failures, "policy rationale");
    }

    failures
}

fn cross_artifact_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let root_hub = read(root, ROOT_HUB_PATH, &mut failures);
    let masterplan = read(root, MASTERPLAN_PATH, &mut failures);
    let repo_hygiene = read(root, REPO_HYGIENE_PATH, &mut failures);
    let anti_patterns = read(root, KUBERNETES_ANTI_PATTERN_PATH, &mut failures);
    let manifest = read(root, CELL_MANIFEST_PATH, &mut failures);
    let buck = read(root, BUCK_PATH, &mut failures);

    for (text, label) in [
        (&root_hub, ROOT_HUB_PATH),
        (&masterplan, MASTERPLAN_PATH),
        (&repo_hygiene, REPO_HYGIENE_PATH),
        (&anti_patterns, KUBERNETES_ANTI_PATTERN_PATH),
        (&manifest, CELL_MANIFEST_PATH),
        (&buck, BUCK_PATH),
    ] {
        require_contains(text, SPEC_PATH, &mut failures, label);
        require_contains(text, CHECK_COMMAND, &mut failures, label);
    }

    for (text, label) in [
        (&repo_hygiene, REPO_HYGIENE_PATH),
        (&anti_patterns, KUBERNETES_ANTI_PATTERN_PATH),
        (&manifest, CELL_MANIFEST_PATH),
        (&masterplan, MASTERPLAN_PATH),
    ] {
        require_contains(
            text,
            "cue_first_cell_pod_config_authority",
            &mut failures,
            label,
        );
        require_contains(
            text,
            "helm_adapter_compatibility_wrapper",
            &mut failures,
            label,
        );
        require_contains(text, "scale_to_zero_eligibility_gate", &mut failures, label);
    }

    failures
}

pub fn evaluate(root: &Path) -> Evaluation {
    let mut failures = Vec::new();
    let spec = read(root, SPEC_PATH, &mut failures);
    failures.extend(contract_failures(&spec));
    failures.extend(cross_artifact_failures(root));

    let verdict = if failures.is_empty() { "PASS" } else { "FAIL" }.to_string();
    Evaluation {
        verdict,
        failures,
        official_sources: REQUIRED_URLS.len(),
        policy_ids: REQUIRED_POLICY_IDS.len(),
        forbidden_anti_patterns: REQUIRED_FORBIDDEN.len(),
    }
}

fn json_array(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("\"{}\"", json_escape(item)))
        .collect::<Vec<_>>()
        .join(",")
}

fn repo_root_from_env() -> PathBuf {
    env::var_os("OYA_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn main() {
    let json = env::args().any(|arg| arg == "--json");
    let evaluation = evaluate(&repo_root_from_env());
    if json {
        println!(
            "{{\"verdict\":\"{}\",\"spec\":\"{}\",\"local_static_only\":true,\"live_kubernetes_mutation_performed\":false,\"official_sources\":{},\"policy_ids\":{},\"forbidden_anti_patterns\":{},\"configuration_authority\":\"cue_first_helm_adapter_only\",\"failures\":[{}]}}",
            evaluation.verdict,
            SPEC_PATH,
            evaluation.official_sources,
            evaluation.policy_ids,
            evaluation.forbidden_anti_patterns,
            json_array(&evaluation.failures)
        );
    } else if evaluation.failures.is_empty() {
        println!("PASS {SPEC_PATH}");
    } else {
        eprintln!("FAIL {SPEC_PATH}");
        for failure in &evaluation.failures {
            eprintln!("- {failure}");
        }
    }
    if !evaluation.failures.is_empty() {
        std::process::exit(1);
    }
}
