//! Local/static OyaCIControllerConfig schema/admission guard.
//!
//! This validates the generated controller config emitted from lane-owned
//! ProwJob shards. It deliberately does not apply Kubernetes resources, deploy
//! admission webhooks, post statuses, or claim live oya-ci parity.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CONTRACT_PATH: &str = "specs/oya-ci-controller-config-contract.json";
const CONFIG_PATH: &str = "specs/generated/oya-ci-controller-config.generated.yaml";
const REGISTRY_PATH: &str = "specs/oya-ci-prowjob-registry.json";
const GENERATED_REGISTRY_PATH: &str = "specs/generated/oya-ci-prowjob-registry.generated.yaml";
const ROOT_HUB_PATH: &str = "specs/root-hub-pointers.json";
const CHECK_COMMAND: &str = "buck2 build //:oya-ci-controller-config-check";
const GENERATED_MARKER: &str =
    "# @generated controller config by scripts/ci/generate-oya-ci-prowjob-registry.rs";
const REQUIRED_CONTEXT: &str = "oya-ci-required";
const SHADOW_CONTEXT: &str = "github-lane-unlocker-required";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub id: String,
    pub prow_job_kind: String,
    pub context: String,
    pub required: bool,
    pub controller_owned: bool,
    pub security_profile: String,
    pub service_account: String,
    pub runtime_class_name: String,
    pub workload_identity: bool,
    pub node_metadata_access: String,
    pub automount_service_account_token: bool,
    pub allow_privilege_escalation: bool,
    pub read_only_root_filesystem: bool,
    pub run_as_non_root: bool,
    pub default_deny_network_policy: bool,
    pub drop_capabilities: Vec<String>,
    pub buck2_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
    pub contract: String,
    pub config: String,
    pub jobs_checked: usize,
    pub required_context: String,
    pub local_static_only: bool,
    pub live_kubernetes_mutated: bool,
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

fn has_json_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&format!(
        "\"{}\":{}",
        key,
        if value { "true" } else { "false" }
    ))
}

fn has_json_string(text: &str, value: &str) -> bool {
    text.contains(&format!("\"{}\"", json_escape(value)))
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

fn yaml_value(line: &str, key: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix(&format!("{key}:"))?.trim();
    Some(rest.trim_matches('"').to_owned())
}

fn yaml_bool(line: &str, key: &str) -> Option<bool> {
    match yaml_value(line, key)?.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn string_field(block: &str, key: &str) -> Option<String> {
    block.lines().find_map(|line| yaml_value(line, key))
}

fn bool_field(block: &str, key: &str) -> Option<bool> {
    block.lines().find_map(|line| yaml_bool(line, key))
}

fn string_array_after(block: &str, heading: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut in_section = false;
    let mut section_indent = 0_usize;
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed == format!("{heading}:") {
            in_section = true;
            section_indent = line.len() - line.trim_start().len();
            continue;
        }
        if !in_section {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent <= section_indent && !trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("- ") {
            values.push(value.trim().trim_matches('"').to_owned());
        }
    }
    values
}

pub fn parse_jobs(config: &str) -> Vec<Job> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    for line in config.lines() {
        if line.starts_with("    - id: ") {
            if !current.is_empty() {
                blocks.push(current);
                current = String::new();
            }
        }
        if !current.is_empty() || line.starts_with("    - id: ") {
            current.push_str(line);
            current.push('\n');
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }

    blocks
        .into_iter()
        .map(|block| Job {
            id: string_field(&block, "- id").unwrap_or_default(),
            prow_job_kind: string_field(&block, "prowJobKind").unwrap_or_default(),
            context: string_field(&block, "context").unwrap_or_default(),
            required: bool_field(&block, "required").unwrap_or(false),
            controller_owned: bool_field(&block, "controllerOwned").unwrap_or(false),
            security_profile: string_field(&block, "securityProfile").unwrap_or_default(),
            service_account: string_field(&block, "serviceAccount").unwrap_or_default(),
            runtime_class_name: string_field(&block, "runtimeClassName").unwrap_or_default(),
            workload_identity: bool_field(&block, "workloadIdentity").unwrap_or(false),
            node_metadata_access: string_field(&block, "nodeMetadataAccess").unwrap_or_default(),
            automount_service_account_token: bool_field(&block, "automountServiceAccountToken")
                .unwrap_or(true),
            allow_privilege_escalation: bool_field(&block, "allowPrivilegeEscalation")
                .unwrap_or(true),
            read_only_root_filesystem: bool_field(&block, "readOnlyRootFilesystem")
                .unwrap_or(false),
            run_as_non_root: bool_field(&block, "runAsNonRoot").unwrap_or(false),
            default_deny_network_policy: bool_field(&block, "defaultDenyNetworkPolicy")
                .unwrap_or(false),
            drop_capabilities: string_array_after(&block, "dropCapabilities"),
            buck2_commands: string_array_after(&block, "buck2Commands"),
        })
        .collect()
}

pub fn contract_failures(contract: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for (needle, message) in [
        (
            "\"status\": \"p00_local_static_schema_contract\"",
            "contract status must be p00_local_static_schema_contract",
        ),
        (
            "\"generated_config\": \"specs/generated/oya-ci-controller-config.generated.yaml\"",
            "contract must name generated controller config",
        ),
        (
            "\"source_registry\": \"specs/oya-ci-prowjob-registry.json\"",
            "contract must name source ProwJob registry",
        ),
        (
            "\"buck2_check\": \"buck2 build //:oya-ci-controller-config-check\"",
            "contract must publish Buck2 check command",
        ),
        (
            "\"api_version\": \"oyatie.dev/v1alpha1\"",
            "contract must pin api version",
        ),
        (
            "\"kind\": \"OyaCIControllerConfig\"",
            "contract must pin config kind",
        ),
    ] {
        require(contract.contains(needle), &mut failures, message);
    }
    for source in [
        "https://kubernetes.io/docs/tasks/extend-kubernetes/custom-resources/custom-resource-definitions/",
        "https://kubernetes.io/docs/concepts/security/pod-security-standards/",
        "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
        "https://docs.prow.k8s.io/docs/jobs/",
    ] {
        require(
            has_json_string(contract, source),
            &mut failures,
            format!("contract official_sources missing {source}"),
        );
    }
    for (key, value) in [
        ("local_static_only", true),
        ("live_kubernetes_mutated", false),
        ("crd_applied", false),
        ("admission_webhook_deployed", false),
        ("full_prow_parity_claimed", false),
        ("github_actions_shadow_only", true),
        ("candidate_owned_truth_allowed", false),
        ("github_adapter_merge_authority", false),
        ("github_adapter_shadow_only", true),
        ("workload_identity_required", true),
        ("static_cloud_secrets_allowed", false),
        ("automount_service_account_token", false),
        ("default_deny_network_policy", true),
        ("allow_privilege_escalation", false),
        ("read_only_root_filesystem", true),
        ("run_as_non_root", true),
        ("runtime_class_required_for_untrusted", true),
        ("controller_owned", true),
        ("buck2_commands_only", true),
        ("forbid_live_kubectl_mutation_commands", true),
    ] {
        require(
            has_json_bool(contract, key, value),
            &mut failures,
            format!("contract missing {key}={value}"),
        );
    }
    for required in [
        REQUIRED_CONTEXT,
        SHADOW_CONTEXT,
        "blocked",
        "RuntimeDefault",
        "ALL",
        "restricted-untrusted-pr",
        "oya-ci-untrusted-runner",
        "sandboxed",
        "trusted-controller-rollup",
        "oya-ci-controller-runner",
    ] {
        require(
            has_json_string(contract, required),
            &mut failures,
            format!("contract missing required value {required}"),
        );
    }
    failures
}

pub fn config_failures(config: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for (needle, message) in [
        (GENERATED_MARKER, "config must carry generated marker"),
        (
            "apiVersion: oyatie.dev/v1alpha1",
            "config must pin apiVersion",
        ),
        ("kind: OyaCIControllerConfig", "config must pin kind"),
        (
            "name: oya-ci-controller-config",
            "config must pin metadata.name",
        ),
        (
            "oyatie.dev/local-static-only: \"true\"",
            "config must declare local-static-only annotation",
        ),
        (
            "oyatie.dev/live-authority-claimed: \"false\"",
            "config must declare live-authority-claimed false annotation",
        ),
        (
            "registry: \"specs/oya-ci-prowjob-registry.json\"",
            "config must point at source registry",
        ),
        (
            "generatedRegistry: \"specs/generated/oya-ci-prowjob-registry.generated.yaml\"",
            "config must point at generated registry",
        ),
        (
            "shadowOnly: true",
            "GitHub publication adapter must be shadow-only",
        ),
        (
            "mergeAuthority: false",
            "GitHub publication adapter must not be merge authority",
        ),
        (
            "requiredContext: \"oya-ci-required\"",
            "rollup must use oya-ci-required",
        ),
        (
            "owner: \"trusted-controller\"",
            "rollup must be trusted-controller owned",
        ),
        (
            "candidateOwnedTruthAllowed: false",
            "rollup must reject candidate-owned truth",
        ),
        (
            "workloadIdentityRequired: true",
            "security defaults must require workload identity",
        ),
        (
            "nodeMetadataAccess: \"blocked\"",
            "security defaults must block node metadata",
        ),
        (
            "staticCloudSecretsAllowed: false",
            "security defaults must forbid static cloud secrets",
        ),
        (
            "automountServiceAccountToken: false",
            "security defaults must disable service-account token automount",
        ),
        (
            "defaultDenyNetworkPolicy: true",
            "security defaults must require default-deny NetworkPolicy",
        ),
        (
            "serviceMeshMtls: \"required-for-service-to-service\"",
            "security defaults must require service mesh mTLS intent",
        ),
        (
            "allowPrivilegeEscalation: false",
            "security defaults must forbid privilege escalation",
        ),
        (
            "readOnlyRootFilesystem: true",
            "security defaults must require immutable root filesystem",
        ),
        (
            "runAsNonRoot: true",
            "security defaults must require non-root execution",
        ),
        (
            "seccompProfile: \"RuntimeDefault\"",
            "security defaults must require RuntimeDefault seccomp",
        ),
        (
            "runtimeClassRequiredForUntrusted: true",
            "security defaults must require sandboxed RuntimeClass for untrusted jobs",
        ),
    ] {
        require(config.contains(needle), &mut failures, message);
    }
    require(
        config.contains("dropCapabilities:\n      - \"ALL\""),
        &mut failures,
        "security defaults must drop all Linux capabilities",
    );
    require(
        !config.contains("kubectl apply")
            && !config.contains("kubectl delete")
            && !config.contains("kubectl scale"),
        &mut failures,
        "generated controller config must not contain live kubectl mutation instructions",
    );

    let jobs = parse_jobs(config);
    require(
        jobs.len() >= 7,
        &mut failures,
        format!("config must list at least 7 jobs, got {}", jobs.len()),
    );
    let mut ids = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    let mut contexts = BTreeSet::new();
    for job in &jobs {
        let label = if job.id.is_empty() {
            "<missing-job-id>"
        } else {
            &job.id
        };
        require(
            !job.id.is_empty(),
            &mut failures,
            "job missing id".to_owned(),
        );
        require(
            ids.insert(job.id.clone()),
            &mut failures,
            format!("duplicate job id {}", job.id),
        );
        kinds.insert(job.prow_job_kind.clone());
        contexts.insert(job.context.clone());
        require(
            job.controller_owned,
            &mut failures,
            format!("{label}: controllerOwned must be true"),
        );
        require(
            job.workload_identity,
            &mut failures,
            format!("{label}: workloadIdentity must be true"),
        );
        require(
            job.node_metadata_access == "blocked",
            &mut failures,
            format!("{label}: nodeMetadataAccess must be blocked"),
        );
        require(
            !job.automount_service_account_token,
            &mut failures,
            format!("{label}: automountServiceAccountToken must be false"),
        );
        require(
            !job.allow_privilege_escalation,
            &mut failures,
            format!("{label}: allowPrivilegeEscalation must be false"),
        );
        require(
            job.read_only_root_filesystem,
            &mut failures,
            format!("{label}: readOnlyRootFilesystem must be true"),
        );
        require(
            job.run_as_non_root,
            &mut failures,
            format!("{label}: runAsNonRoot must be true"),
        );
        require(
            job.default_deny_network_policy,
            &mut failures,
            format!("{label}: defaultDenyNetworkPolicy must be true"),
        );
        require(
            job.drop_capabilities
                .iter()
                .any(|capability| capability == "ALL"),
            &mut failures,
            format!("{label}: dropCapabilities must include ALL"),
        );
        require(
            !job.buck2_commands.is_empty(),
            &mut failures,
            format!("{label}: missing buck2Commands"),
        );
        for command in &job.buck2_commands {
            require(
                command.starts_with("buck2 build //"),
                &mut failures,
                format!("{label}: command must be Buck2 build authority: {command}"),
            );
            require(
                !command.contains("kubectl ")
                    && !command.contains("python ")
                    && !command.contains("python3 ")
                    && !command.ends_with(".sh"),
                &mut failures,
                format!(
                    "{label}: command must not reintroduce ad-hoc script/live mutation: {command}"
                ),
            );
        }
        if job.security_profile == "restricted-untrusted-pr" {
            require(
                job.service_account == "oya-ci-untrusted-runner",
                &mut failures,
                format!("{label}: untrusted jobs must use oya-ci-untrusted-runner"),
            );
            require(
                job.runtime_class_name == "sandboxed",
                &mut failures,
                format!("{label}: untrusted jobs must use sandboxed runtime"),
            );
        }
        if job.context == REQUIRED_CONTEXT {
            require(
                job.required,
                &mut failures,
                format!("{label}: rollup must be required"),
            );
            require(
                job.security_profile == "trusted-controller-rollup",
                &mut failures,
                format!("{label}: rollup must use trusted-controller-rollup profile"),
            );
            require(
                job.service_account == "oya-ci-controller-runner",
                &mut failures,
                format!("{label}: rollup must use controller service account"),
            );
        }
        require(
            job.context != SHADOW_CONTEXT,
            &mut failures,
            format!("{label}: GitHub shadow context must not become a controller job context"),
        );
    }
    for kind in ["presubmit", "postsubmit", "periodic", "batch"] {
        require(
            kinds.contains(kind),
            &mut failures,
            format!("config missing Prow job kind {kind}"),
        );
    }
    require(
        contexts.contains(REQUIRED_CONTEXT),
        &mut failures,
        "config missing oya-ci-required rollup context",
    );
    failures
}

pub fn evaluate(root: &Path) -> Evaluation {
    let mut failures = Vec::new();
    let contract = read(root, CONTRACT_PATH, &mut failures);
    let config = read(root, CONFIG_PATH, &mut failures);
    let registry = read(root, REGISTRY_PATH, &mut failures);
    let generated_registry = read(root, GENERATED_REGISTRY_PATH, &mut failures);
    let root_hub = read(root, ROOT_HUB_PATH, &mut failures);

    failures.extend(contract_failures(&contract));
    failures.extend(config_failures(&config));
    require(
        registry.contains("\"generated_controller_config\": \"specs/generated/oya-ci-controller-config.generated.yaml\""),
        &mut failures,
        "source registry must name generated controller config",
    );
    require(
        generated_registry.contains("kind: OyaCIProwJobRegistry"),
        &mut failures,
        "generated ProwJob registry must remain present",
    );
    require(
        root_hub.contains("\"oya_ci_controller_config_contract\""),
        &mut failures,
        "root hub must expose oya_ci_controller_config_contract",
    );
    require(
        root_hub.contains("\"current_path\": \"/specs/oya-ci-controller-config-contract.json\""),
        &mut failures,
        "root hub controller config contract current_path must point at spec",
    );
    require(
        root_hub.contains(
            "\"oya_ci_controller_config_contract\": \"specs/oya-ci-controller-config-contract.json\"",
        ),
        &mut failures,
        "root hub pointers must include oya_ci_controller_config_contract",
    );
    require(
        root_hub.contains(
            "\"oya_ci_controller_config_generated\": \"specs/generated/oya-ci-controller-config.generated.yaml\"",
        ),
        &mut failures,
        "root hub pointers must include generated controller config",
    );

    Evaluation {
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned(),
        failures,
        contract: CONTRACT_PATH.to_owned(),
        config: CONFIG_PATH.to_owned(),
        jobs_checked: parse_jobs(&config).len(),
        required_context: REQUIRED_CONTEXT.to_owned(),
        local_static_only: true,
        live_kubernetes_mutated: false,
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
        "{{\"verdict\":\"{}\",\"contract\":\"{}\",\"config\":\"{}\",\"jobs_checked\":{},\"required_context\":\"{}\",\"local_static_only\":{},\"live_kubernetes_mutated\":{},\"buck2_check\":\"{}\",\"failures\":[{}]}}",
        evaluation.verdict,
        json_escape(&evaluation.contract),
        json_escape(&evaluation.config),
        evaluation.jobs_checked,
        json_escape(&evaluation.required_context),
        evaluation.local_static_only,
        evaluation.live_kubernetes_mutated,
        json_escape(CHECK_COMMAND),
        failures
    )
}

fn config() -> (PathBuf, bool) {
    let mut json = false;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--json" => json = true,
            unknown => {
                eprintln!("assert-oya-ci-controller-config: unknown argument {unknown}");
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
            eprintln!("oya-ci-controller-config: RED");
            for failure in &evaluation.failures {
                eprintln!("- {failure}");
            }
        }
        std::process::exit(1);
    }
}
