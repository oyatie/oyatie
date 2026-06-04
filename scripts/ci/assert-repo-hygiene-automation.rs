//! P00 repo-hygiene automation gate.
//!
//! This is the Rust/Buck2 replacement for the retired Python hygiene checker.
//! It is deliberately local/static: it validates checked-in contracts, active
//! guidance wording, and backlog anchors without mutating branch protection,
//! Kubernetes, GitHub state, or any deployment surface.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SPEC_PATH: &str = "specs/repo-hygiene-automation.json";
const RETIRED_SUBSTRATE_PATH: &str = "specs/retired-external-substrate-registry.json";
const ROOT_HUB_PATH: &str = "specs/root-hub-pointers.json";
const GITHUB_BRIDGE_PATH: &str = "specs/github-lane-unlocker-bridge.json";
const CANONICAL_PRIMITIVES_PATH: &str = "specs/canonical-primitives.json";
const MASTERPLAN_PATH: &str = "specs/masterplan.json";
const SEQUENCING_PATH: &str = "specs/master-plan-sequencing.json";
const PLANNING_CLOSURE_CONTRACT_PATH: &str = "specs/planning-closure-contract.json";
const PLANNING_CLOSURE_LEDGER_PATH: &str = "specs/planning-closure-status-closure-ledger.json";
const README_PATH: &str = "README.md";
const AGENTS_PATH: &str = "AGENTS.md";
const CLAUDE_PATH: &str = "CLAUDE.md";
const DOC_AGENTS_PATH: &str = "docs/AGENTS.md";
const DOC_CATALOG_PATH: &str = "docs/DOC-CATALOG.md";
const PROCEDURE_PATH: &str = "docs/ci/github-actions-lane-unlocker.md";
const WORKFLOW_PATH: &str = ".github/workflows/github-lane-unlocker-ci-cd.yml";
const BUCK_PATH: &str = "BUCK";
const DOC_STALENESS_MAIN: &str = "tools/oya-doc-staleness-inventory-app/src/main.rs";

const STALE_DOC_INVENTORY_COMMAND: &str =
    "buck2 build //tools/oya-doc-staleness-inventory-app:doc-staleness-inventory-json";
const STALE_DOC_INVENTORY_TEST_COMMAND: &str =
    "buck2 build //tools/oya-doc-staleness-inventory-app:doc-staleness-inventory-unit-tests";
const REQUIRED_BUCK2_AUTHORITY_COMMAND: &str = "buck2 build //:repo-hygiene-automation-check";
const PROWJOB_REGISTRY_COMMAND: &str = "buck2 build //:oya-ci-prowjob-registry-check";
const RETIRED_PLANNING_CLOSURE_COMMAND: &str =
    "cargo run -q -p oya-dev-cli -- gate validate planning-closure";
const RETIRED_OYA_DEV_CLI_PLANNING_CLOSURE_COMMAND: &str =
    "buck2 run //oya/developer-sdk/crates/oya-dev-cli:oya -- gate validate planning-closure";
const RETIRED_PYTHON_COMMAND: &str = "python3 scripts/ci/assert-repo-hygiene-automation.py --json";

const REQUIRED_DOMAINS: &[&str] = &[
    "git_worktree_hygiene",
    "branch_merge_hygiene",
    "repository_publication_hygiene",
    "disk_workspace_hygiene",
    "kubernetes_workload_hygiene",
    "documentation_sprawl_hygiene",
];

const REQUIRED_AUTOMATION_COMMANDS: &[&str] = &[
    "buck2 build //:repo-hygiene-automation-check",
    "buck2 build //:github-lane-unlocker-bridge-check",
    "buck2 build //:third-party-durable-handedits-check",
    "buck2 build //:github-lane-unlocker-bridge-check //:buck2-authority-policy-check //:repo-hygiene-automation-check",
    STALE_DOC_INVENTORY_COMMAND,
    STALE_DOC_INVENTORY_TEST_COMMAND,
    PROWJOB_REGISTRY_COMMAND,
];

const REQUIRED_SOURCE_URLS: &[&str] = &[
    "https://docs.github.com/en/repositories/creating-and-managing-repositories/about-repositories",
    "https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners",
    "https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue",
    "https://docs.github.com/en/actions/sharing-automations/reusing-workflows",
    "https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs",
    "https://docs.prow.k8s.io/docs/",
    "https://docs.prow.k8s.io/docs/overview/architecture/",
    "https://docs.prow.k8s.io/docs/components/core/tide/",
    "https://docs.prow.k8s.io/docs/jobs/",
    "https://sapling-scm.com/docs/introduction/",
    "https://sapling-scm.com/docs/scale/overview/",
    "https://architecture.cncf.io/",
    "https://kubernetes.io/docs/tasks/run-application/scale-deployment/",
    "https://www.nist.gov/publications/zero-trust-architecture",
    "https://csrc.nist.gov/pubs/sp/800/162/upd2/final",
    "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
    "https://kubernetes.io/docs/concepts/security/pod-security-standards/",
    "https://kubernetes.io/docs/tasks/configure-pod-container/security-context/",
    "https://kubernetes.io/docs/concepts/containers/runtime-class/",
    "https://kubernetes.io/docs/tasks/configure-pod-container/configure-service-account/",
    "https://cloud.google.com/kubernetes-engine/docs/concepts/workload-identity",
    "https://docs.aws.amazon.com/eks/latest/best-practices/identity-and-access-management.html",
    "https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/",
    "https://docs.github.com/en/actions/concepts/security/openid-connect",
    "https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions",
    "https://slsa.dev/spec/v1.2/requirements",
    "https://istio.io/latest/docs/concepts/security/",
    "https://docs.aws.amazon.com/organizations/latest/userguide/orgs_manage_policies_scps.html",
];

const REQUIRED_SECURITY_BACKLOG_IDS: &[&str] = &[
    "zero_trust_architecture",
    "privileged_identity_management",
    "abac_beyond_rbac",
    "network_microsegmentation",
    "silo_apis_integrations",
    "silo_ai_automation",
    "encrypt_sensitive_data",
    "purge_redundant_obsolete_data",
    "data_layer_rate_limiting",
    "automated_session_revocation",
    "dual_attribution_audit_logging",
    "isolate_ci_cd_pipelines",
    "tightly_scope_pipeline_secrets",
    "pin_dependencies_private_registries",
    "honeytokens_tripwires",
    "host_based_microsegmentation",
    "hardware_enforced_isolation",
    "multi_account_cloud_strategy",
    "service_control_policies",
    "enforce_workload_identity",
    "block_node_metadata_access",
    "pod_level_runtime_isolation",
    "restrict_container_privileges",
    "immutable_container_filesystems",
    "drop_linux_capabilities",
    "default_deny_network_policies",
    "service_mesh_mtls",
    "disable_default_service_account_token_mounting",
    "cluster_architecture_blast_walls",
    "separate_control_planes",
    "sandboxed_runtimes",
];

const REQUIRED_NON_RUST_SURFACE_NEEDLES: &[(&str, &str)] = &[
    (
        "\"existing_non_rust_surface_inventory\"",
        "repo hygiene spec must classify existing non-Rust surfaces",
    ),
    (
        "\"status\": \"classified_no_durable_authority\"",
        "non-Rust surface inventory must be classified as no durable authority",
    ),
    (
        "\"tracked_typescript_pnpm_mjs_count\": 23",
        "non-Rust surface inventory must record the audited TS/pnpm/MJS count",
    ),
    (
        "\"tracked_nonvendored_python_shell_count\": 55",
        "non-Rust surface inventory must record the audited non-vendored Python/shell count",
    ),
    (
        "\"pnpm_or_package_json_repo_authority\": false",
        "pnpm/package metadata must not be repo authority",
    ),
    (
        "\"typescript_runtime_merge_authority\": false",
        "TypeScript runtime surfaces must not be merge authority",
    ),
    (
        "\"python_shell_durable_gate_authority\": false",
        "Python/shell surfaces must not be durable gate authority",
    ),
    (
        "\"rewrite_active_gate_surfaces_to_rust_buck2\": true",
        "active gate surfaces must retain the Rust/Buck2 rewrite requirement",
    ),
    (
        "\"app_shell_frontend_prototype\"",
        "app-shell TypeScript/pnpm prototype surface must be classified",
    ),
    (
        "\"workflow_studio_sveltekit_templates\"",
        "workflow-studio TypeScript template surface must be classified",
    ),
    (
        "\"feature_flags_reference_clients\"",
        "feature-flag reference clients must be classified",
    ),
    (
        "\"advisory_mjs_doc_contract_tools\"",
        "MJS doc/API helpers must be classified as advisory",
    ),
    (
        "\"bootstrap_host_prelude\"",
        "bootstrap/prelude script surface must be classified",
    ),
    (
        "\"rust_backed_wrappers\"",
        "Rust-backed shell wrappers must be classified",
    ),
    (
        "\"pending_rust_buck2_rewrite\"",
        "pending Python/shell rewrite surface must be classified",
    ),
    (
        "\"not merge authority until rewritten or rehosted as Rust libraries/Buck2 targets\"",
        "pending script surfaces must be denied merge authority until Rust/Buck2 rewrite",
    ),
];

const CLEANUP_BACKLOG_IDS: &[&str] = &[
    "legacy_python_shell_gate_surfaces",
    "shared_ci_workflow_surface",
    "root_hub_masterplan_shared_docs",
    "stale_doc_inventory_followups",
    "retired_external_substrate_residue",
    "temporary_github_bridge_artifacts",
    "retire_oya_cli_governance_authority",
    "typescript_pnpm_retirement_review",
    "prow_job_registry_generation",
    "python_shell_to_rust_buck2_migration",
    "cd_fleet_bootstrap_surface_retirement",
    "retired_external_scm_adapter_retirement",
];

const REQUIRED_FORBIDDEN_PHRASE_IDS: &[&str] = &[
    "cloud-ci/oya-ci required context + reviewer APPROVE gate merge readiness",
    "cloud-ci/oya-ci required context is merge authority",
    "GitHub Actions is retired",
    "legacy_ci_server CI + oya gate run-all",
    "legacy_self_hosted_git_forge required-checks/auto-merge is the substrate target",
    "manual oya-ci-required success statuses to merge bridge PRs",
    "infra/ci/buck2-affected-gate.sh origin/dev HEAD",
    "github-lane-unlocker-required is merge authority",
    "The required temporary context is `github-lane-unlocker-required`",
    "dev requires github-lane-unlocker-required",
    "oya gate is merge authority",
    "oya verify is CI authority",
    "oya gate` / `oya verify` governance evidence",
    "oya-dev-cli:oya -- gate validate planning-closure",
];

const FORBIDDEN_ACTIVE_DOC_PHRASES: &[&str] = &[
    "cloud-ci/oya-ci required context + reviewer APPROVE gate merge readiness",
    "cloud-ci/oya-ci required context is merge authority",
    "GitHub Actions is retired",
    "Jenkins CI + oya gate run-all",
    "self-hosted Forgejo required-checks/auto-merge is the substrate target",
    "manual oya-ci-required success statuses to merge bridge PRs",
    "infra/ci/buck2-affected-gate.sh origin/dev HEAD",
    "github-lane-unlocker-required is merge authority",
    "The required temporary context is `github-lane-unlocker-required`",
    "dev requires github-lane-unlocker-required",
    "oya gate is merge authority",
    "oya verify is CI authority",
];

const ACTIVE_CONTEXT_SCAN_PATHS: &[&str] = &[
    AGENTS_PATH,
    CLAUDE_PATH,
    README_PATH,
    DOC_AGENTS_PATH,
    PROCEDURE_PATH,
    "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
    ".github/branch-protection.yaml",
    "infra/branch-protection/dev.json",
    "docs/MASTERPLAN.md",
];

const ACTIVE_EXACT_NAME_SCAN_PATHS: &[&str] = &[
    AGENTS_PATH,
    CLAUDE_PATH,
    README_PATH,
    DOC_AGENTS_PATH,
    DOC_CATALOG_PATH,
    "docs/MASTERPLAN.md",
    PROCEDURE_PATH,
    "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
    "docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md",
    ".github/branch-protection.yaml",
    "infra/branch-protection/dev.json",
    ROOT_HUB_PATH,
    MASTERPLAN_PATH,
    SEQUENCING_PATH,
    "specs/cloud-toolchain-target.json",
    "specs/bespoke-cloud-toolchain-services.json",
    "specs/gitops-vcs-replacement.json",
    "specs/cloud-strangler-migration-target.json",
    GITHUB_BRIDGE_PATH,
    CANONICAL_PRIMITIVES_PATH,
    SPEC_PATH,
    RETIRED_SUBSTRATE_PATH,
];

const RETIRED_EXACT_NAME_PATTERNS: &[&str] = &[
    "Jenkins",
    "Forgejo",
    "ArgoCD",
    "Argo CD",
    "Argo Workflows/Rollouts",
];

const ALLOWED_EXACT_NAME_CONTEXTS: &[&str] = &[
    "retired-external-substrate-registry.json",
    "ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate",
    "adr_0349_jenkins_argocd_ci_cd_substrate",
    "15-ZE-jenkins-argocd-self-hostable-ci-cd-substrate",
    "ADR-0363-retire-agentic-vcs-foundry-to-intelligence-forgejo-substrate",
];

const ROOT_MARKDOWN_ALLOWLIST: &[&str] = &["AGENTS.md", "CLAUDE.md", "README.md"];
const RETIRED_ROOT_FILES: &[&str] = &["Jenkinsfile"];
const RETIRED_SERVICE_CI_ENTRYPOINT_ROOTS: &[&str] = &["cloud", "oya"];
const RETIRED_ACTIVE_PATHS: &[&str] = &[
    "infra/ci/jenkins",
    "infra/ci/argocd",
    "infra/ci/deploy-local.sh",
    "infra/cilium/cell-boundaries/oya-ci-jenkins-ingress.netpol.yaml",
    "infra/cilium/cell-boundaries/oya-forge-ingress.netpol.yaml",
    "infra/forge",
    "infra/forge/jenkins-forgejo-token.secret.template.yaml",
    "scripts/ci/arm-auto-merge.sh",
    "scripts/tests/forgejo_auto_merge_after_ci.test.sh",
    "docs/ci/forge-of-record.md",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
    pub domains_checked: usize,
    pub security_backlog_count: usize,
    pub active_context_scan_files: usize,
    pub retired_exact_name_scan_files: usize,
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

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&format!(
        "\"{}\":{}",
        key,
        if value { "true" } else { "false" }
    ))
}

fn contains_json_string(text: &str, value: &str) -> bool {
    let escaped = json_escape(value);
    text.contains(&format!("\"{}\"", escaped))
}

fn count_json_key_value(text: &str, key: &str, value: &str) -> usize {
    let compact = compact_json_text(text);
    let needle = format!("\"{}\":\"{}\"", key, json_escape(value));
    compact.matches(&needle).count()
}

fn read(root: &Path, rel: &str, failures: &mut Vec<String>) -> String {
    let path = root.join(rel);
    match fs::read_to_string(&path) {
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

pub fn active_doc_phrase_failures(label: &str, text: &str) -> Vec<String> {
    FORBIDDEN_ACTIVE_DOC_PHRASES
        .iter()
        .filter(|phrase| text.contains(**phrase))
        .map(|phrase| format!("{label}: stale active authority phrase present: {phrase:?}"))
        .collect()
}

pub fn retired_exact_name_failures(rel: &str, text: &str) -> Vec<String> {
    let rel_lower = rel.to_ascii_lowercase();
    let rel_allowed = ALLOWED_EXACT_NAME_CONTEXTS
        .iter()
        .any(|allowed| rel_lower.contains(&allowed.to_ascii_lowercase()));
    let mut failures = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let lowered = line.to_ascii_lowercase();
        if !RETIRED_EXACT_NAME_PATTERNS
            .iter()
            .any(|pattern| lowered.contains(&pattern.to_ascii_lowercase()))
        {
            continue;
        }
        if rel_allowed
            || ALLOWED_EXACT_NAME_CONTEXTS
                .iter()
                .any(|allowed| lowered.contains(&allowed.to_ascii_lowercase()))
        {
            continue;
        }
        failures.push(format!(
            "{}:{}: retired exact-name reference must use generic active-doc term",
            rel,
            index + 1
        ));
    }

    failures
}

fn retired_oya_dev_cli_active_gate_command_failure(label: &str, text: &str) -> Option<String> {
    let compact = compact_json_text(text);
    let retired_active_gate = format!(
        "\"gate_command\":\"{}\"",
        json_escape(RETIRED_OYA_DEV_CLI_PLANNING_CLOSURE_COMMAND)
    );
    if compact.contains(&retired_active_gate) {
        Some(format!(
            "{label}: must not reintroduce retired oya-dev-cli planning-closure command as active gate_command"
        ))
    } else {
        None
    }
}

pub fn spec_failures(spec: &str) -> Vec<String> {
    let mut failures = Vec::new();

    for (needle, message) in [
        (
            "\"status\": \"p00_active_automation_contract\"",
            "repo hygiene spec status must be p00_active_automation_contract",
        ),
        (
            "\"new_parallel_fanout_automation\": \"rust_buck2_first\"",
            "automation language policy must be Rust/Buck2-first",
        ),
        (
            "\"new_python_or_shell_gate_surface\": \"deny_unless_explicit_bootstrap_exception\"",
            "automation language policy must deny new Python/shell gates",
        ),
        (
            "\"pattern\": \"registry_owned_desired_ci_graph_to_generated_consolidation\"",
            "shared-surface mitigation must require generated consolidation",
        ),
        (
            "\"seed_check\": \"buck2 build //:oya-ci-prowjob-registry-check\"",
            "ProwJob registry seed check must be recorded in cleanup backlog",
        ),
        (
            "\"seed_registry\": \"specs/oya-ci-prowjob-registry.json\"",
            "ProwJob registry seed registry path must be recorded in cleanup backlog",
        ),
        (
            "\"generated_controller_config\": \"specs/generated/oya-ci-controller-config.generated.yaml\"",
            "ProwJob registry cleanup backlog must record generated controller config path",
        ),
        (
            "\"required_native_context\": \"oya-ci-required\"",
            "active context drift scan must require oya-ci-required",
        ),
        (
            "\"legacy_shadow_context\": \"github-lane-unlocker-required\"",
            "active context drift scan must preserve github-lane-unlocker-required as legacy/shadow context",
        ),
        (
            "\"new_markdown_default\": \"reject_unless_registered_or_lane_owned\"",
            "documentation sprawl policy must reject unregistered/laneless docs",
        ),
        (
            "\"claim_boundary\": \"inventory_only_no_deletion_no_archive_no_live_mutation\"",
            "stale-doc inventory must remain inventory-only",
        ),
        (
            "\"anti_patterns_guardrails\"",
            "repo hygiene spec must record hyperscaler/cloud/Kubernetes-native anti-pattern guardrails",
        ),
        (
            "\"trusted_controller_owned_oya_ci_required\"",
            "anti-pattern guardrails must require controller-owned oya-ci-required truth",
        ),
        (
            "\"controller_oriented_kubernetes_scale_pause_drain\"",
            "anti-pattern guardrails must require controller-oriented Kubernetes workload handling",
        ),
        (
            "\"pointer_thin_or_generated_shared_surfaces\"",
            "anti-pattern guardrails must require pointer-thin or generated shared surfaces",
        ),
        (
            "\"retired_external_substrate_bridge_authority\"",
            "anti-pattern guardrails must reject retired substrate bridge authority",
        ),
        (
            "\"blind_kubectl_delete_pods\"",
            "anti-pattern guardrails must reject blind pod deletion",
        ),
        (
            "\"github_actions_as_durable_ci_authority\"",
            "anti-pattern guardrails must reject GitHub Actions as durable CI authority",
        ),
    ] {
        require_contains(spec, needle, &mut failures, message);
    }

    for (key, value) in [
        ("local_static_only", true),
        ("live_mutation_performed", false),
        ("buck2_authority", true),
        ("github_bridge_temporary", true),
        ("github_actions_shadow_only", true),
        ("native_scm_requires_github_adapter", true),
        ("cli_revival_allowed", false),
        ("legacy_python_shell_migration_backlog", true),
        ("archive_before_delete", true),
        ("thin_pointer_shared_docs", true),
    ] {
        require(
            has_bool(spec, key, value),
            &mut failures,
            format!("spec missing bool {key}={value}"),
        );
    }

    for (needle, message) in REQUIRED_NON_RUST_SURFACE_NEEDLES {
        require_contains(spec, needle, &mut failures, message);
    }

    for domain in REQUIRED_DOMAINS {
        require(
            count_json_key_value(spec, "id", domain) > 0,
            &mut failures,
            format!("repo hygiene domains missing: {domain}"),
        );
    }

    for command in REQUIRED_AUTOMATION_COMMANDS {
        require(
            contains_json_string(spec, command),
            &mut failures,
            format!("automation_commands missing: {command}"),
        );
    }
    require(
        !contains_json_string(spec, RETIRED_PYTHON_COMMAND),
        &mut failures,
        "automation_commands must not keep retired repo-hygiene Python command",
    );

    for url in REQUIRED_SOURCE_URLS {
        require_contains(spec, url, &mut failures, "official_sources");
    }
    for tool_example in [
        "git worktree add",
        "gh pr create --base dev",
        "buck2 build //:repo-hygiene-automation-check",
        "buck2 build //:buck2-authority-policy-check",
    ] {
        require(
            contains_json_string(spec, tool_example),
            &mut failures,
            format!("active context drift scan missing required tool example {tool_example:?}"),
        );
    }
    for phrase_id in REQUIRED_FORBIDDEN_PHRASE_IDS {
        require(
            contains_json_string(spec, phrase_id),
            &mut failures,
            format!("active context drift scan missing forbidden phrase id {phrase_id:?}"),
        );
    }
    for cleanup_id in CLEANUP_BACKLOG_IDS {
        require(
            count_json_key_value(spec, "id", cleanup_id) > 0,
            &mut failures,
            format!("cleanup_candidate_backlog missing {cleanup_id}"),
        );
    }
    for security_id in REQUIRED_SECURITY_BACKLOG_IDS {
        require(
            count_json_key_value(spec, "id", security_id) > 0,
            &mut failures,
            format!("security_hardening_backlog missing {security_id}"),
        );
    }
    let valid_count = compact_json_text(spec).matches("\"valid\":true").count();
    require(
        valid_count >= REQUIRED_SECURITY_BACKLOG_IDS.len(),
        &mut failures,
        format!(
            "security_hardening_backlog valid=true count too low: expected at least {}, got {}",
            REQUIRED_SECURITY_BACKLOG_IDS.len(),
            valid_count
        ),
    );

    failures
}

fn evaluate_root_markdown(root: &Path, failures: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(root) else {
        failures.push("repo root: read_dir failed".to_owned());
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        require(
            ROOT_MARKDOWN_ALLOWLIST.contains(&name),
            failures,
            format!("root markdown sprawl: {name}"),
        );
    }
}

pub fn retired_root_file_failures(root: &Path) -> Vec<String> {
    RETIRED_ROOT_FILES
        .iter()
        .filter(|rel| root.join(rel).exists())
        .map(|rel| format!("{rel}: retired root CI entrypoint must not exist"))
        .collect()
}

pub fn retired_service_ci_entrypoint_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();

    for service_root in RETIRED_SERVICE_CI_ENTRYPOINT_ROOTS {
        let service_root_path = root.join(service_root);
        let Ok(entries) = fs::read_dir(&service_root_path) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(service_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let rel = format!("{service_root}/{service_name}/ci/Jenkinsfile");
            if root.join(&rel).exists() {
                failures.push(format!(
                    "{rel}: retired service Jenkins CI entrypoint must not exist; use Prow/Kubernetes-native oya-ci and keep GitHub Actions as shadow compatibility only"
                ));
            }
        }
    }

    failures
}

pub fn retired_active_path_failures(root: &Path) -> Vec<String> {
    RETIRED_ACTIVE_PATHS
        .iter()
        .filter(|rel| root.join(rel).exists())
        .map(|rel| {
            format!(
                "{rel}: retired active CI substrate path must not exist; use Prow/Kubernetes-native oya-ci and keep GitHub Actions as shadow compatibility only"
            )
        })
        .collect()
}

pub fn evaluate(root: &Path) -> Evaluation {
    let mut failures = Vec::new();

    let spec = read(root, SPEC_PATH, &mut failures);
    let retired = read(root, RETIRED_SUBSTRATE_PATH, &mut failures);
    let root_hub = read(root, ROOT_HUB_PATH, &mut failures);
    let github_bridge = read(root, GITHUB_BRIDGE_PATH, &mut failures);
    let masterplan = read(root, MASTERPLAN_PATH, &mut failures);
    let sequencing = read(root, SEQUENCING_PATH, &mut failures);
    let planning_contract = read(root, PLANNING_CLOSURE_CONTRACT_PATH, &mut failures);
    let planning_ledger = read(root, PLANNING_CLOSURE_LEDGER_PATH, &mut failures);
    let readme = read(root, README_PATH, &mut failures);
    let agents = read(root, AGENTS_PATH, &mut failures);
    let claude = read(root, CLAUDE_PATH, &mut failures);
    let doc_agents = read(root, DOC_AGENTS_PATH, &mut failures);
    let doc_catalog = read(root, DOC_CATALOG_PATH, &mut failures);
    let procedure = read(root, PROCEDURE_PATH, &mut failures);
    let workflow = read(root, WORKFLOW_PATH, &mut failures);
    let buck = read(root, BUCK_PATH, &mut failures);

    failures.extend(spec_failures(&spec));
    evaluate_root_markdown(root, &mut failures);
    failures.extend(retired_root_file_failures(root));
    failures.extend(retired_service_ci_entrypoint_failures(root));
    failures.extend(retired_active_path_failures(root));

    for item_id in [
        "legacy_ci_server",
        "legacy_self_hosted_git_forge",
        "legacy_gitops_cd_runtime",
        "legacy_workflow_runtime",
    ] {
        require(
            count_json_key_value(&retired, "id", item_id) > 0,
            &mut failures,
            format!("retired external substrate registry missing {item_id}"),
        );
    }
    require_contains(
        &retired,
        "retired external SCM/CI/CD substrates",
        &mut failures,
        RETIRED_SUBSTRATE_PATH,
    );
    require_contains(
        &retired,
        "buck2 build //:repo-hygiene-automation-check",
        &mut failures,
        RETIRED_SUBSTRATE_PATH,
    );
    require(
        !retired.contains(RETIRED_PYTHON_COMMAND),
        &mut failures,
        "retired substrate registry must not point to the retired repo-hygiene Python command",
    );

    for needle in [
        "\"repo_hygiene_automation\"",
        "\"current_path\": \"/specs/repo-hygiene-automation.json\"",
        "\"retired_external_substrate_registry\"",
        "specs/retired-external-substrate-registry.json",
    ] {
        require_contains(&root_hub, needle, &mut failures, ROOT_HUB_PATH);
    }

    for needle in [
        "\"github_public_private_publication\"",
        "\"git_protocol\"",
        "\"github_actions_status_bridge\"",
        "\"repo_hygiene_automation_ref\": \"/specs/repo-hygiene-automation.json\"",
    ] {
        require_contains(&github_bridge, needle, &mut failures, GITHUB_BRIDGE_PATH);
    }

    require_contains(
        &masterplan,
        "\"repo_hygiene_automation\"",
        &mut failures,
        MASTERPLAN_PATH,
    );
    require_contains(
        &masterplan,
        REQUIRED_BUCK2_AUTHORITY_COMMAND,
        &mut failures,
        MASTERPLAN_PATH,
    );
    require(
        !masterplan.contains(RETIRED_PLANNING_CLOSURE_COMMAND),
        &mut failures,
        "masterplan must not reintroduce retired Cargo planning-closure command",
    );
    for (label, text) in [
        (MASTERPLAN_PATH, masterplan.as_str()),
        (SEQUENCING_PATH, sequencing.as_str()),
        (PLANNING_CLOSURE_CONTRACT_PATH, planning_contract.as_str()),
        (PLANNING_CLOSURE_LEDGER_PATH, planning_ledger.as_str()),
    ] {
        if let Some(failure) = retired_oya_dev_cli_active_gate_command_failure(label, text) {
            failures.push(failure);
        }
    }
    require_contains(
        &masterplan,
        "\"retired_gate_command\"",
        &mut failures,
        "masterplan planning-closure tombstone",
    );
    require_contains(
        &sequencing,
        "repo_hygiene_automation",
        &mut failures,
        SEQUENCING_PATH,
    );
    require_contains(
        &sequencing,
        REQUIRED_BUCK2_AUTHORITY_COMMAND,
        &mut failures,
        SEQUENCING_PATH,
    );
    require(
        !sequencing.contains(RETIRED_PLANNING_CLOSURE_COMMAND),
        &mut failures,
        "master-plan sequencing must not reintroduce retired Cargo planning-closure command",
    );
    require(
        !compact_json_text(&sequencing).contains(&format!(
            "\"gate_command\":\"{}\"",
            json_escape(RETIRED_OYA_DEV_CLI_PLANNING_CLOSURE_COMMAND)
        )),
        &mut failures,
        "master-plan sequencing must not reintroduce retired oya-dev-cli planning-closure command",
    );
    require_contains(
        &planning_contract,
        "\"retired_gate_command\"",
        &mut failures,
        "planning-closure contract tombstone",
    );
    require_contains(
        &planning_ledger,
        "\"retired_gate_command\"",
        &mut failures,
        "planning-closure ledger tombstone",
    );

    for (label, text) in [
        (README_PATH, readme.as_str()),
        (AGENTS_PATH, agents.as_str()),
        (CLAUDE_PATH, claude.as_str()),
        (DOC_AGENTS_PATH, doc_agents.as_str()),
        (DOC_CATALOG_PATH, doc_catalog.as_str()),
        (PROCEDURE_PATH, procedure.as_str()),
    ] {
        require_contains(text, "repo-hygiene-automation", &mut failures, label);
        require_contains(
            text,
            "buck2 build //:repo-hygiene-automation-check",
            &mut failures,
            label,
        );
        require(
            !text.contains(RETIRED_PYTHON_COMMAND),
            &mut failures,
            format!("{label}: must not recommend retired repo-hygiene Python command"),
        );
    }

    for (label, text) in [
        (AGENTS_PATH, agents.as_str()),
        (CLAUDE_PATH, claude.as_str()),
    ] {
        for tool_example in [
            "git worktree add",
            "gh pr create --base dev",
            "buck2 build //:repo-hygiene-automation-check",
            "buck2 build //:buck2-authority-policy-check",
        ] {
            require_contains(text, tool_example, &mut failures, label);
        }
    }

    for (label, text) in [
        (README_PATH, readme.as_str()),
        (AGENTS_PATH, agents.as_str()),
        (CLAUDE_PATH, claude.as_str()),
        (DOC_AGENTS_PATH, doc_agents.as_str()),
    ] {
        require_contains(text, "Prow", &mut failures, label);
        require_contains(text, "Kubernetes-native", &mut failures, label);
        require_contains(text, "oya-ci-required", &mut failures, label);
    }

    for (label, text) in [(PROCEDURE_PATH, procedure.as_str())] {
        require_contains(text, "github-lane-unlocker-required", &mut failures, label);
    }

    for rel in ACTIVE_CONTEXT_SCAN_PATHS {
        let text = read(root, rel, &mut failures);
        failures.extend(active_doc_phrase_failures(rel, &text));
    }

    for rel in ACTIVE_EXACT_NAME_SCAN_PATHS {
        let text = read(root, rel, &mut failures);
        failures.extend(retired_exact_name_failures(rel, &text));
    }

    for needle in [
        "repo-hygiene-automation-check",
        "oya-ci-prowjob-registry-check",
        "assert-repo-hygiene-automation.rs",
        "generate-oya-ci-prowjob-registry.rs",
        "repo_hygiene_automation_check.rs",
        "oya_ci_prowjob_registry_check.rs",
        "repo-hygiene-automation.json",
        "oya-ci-prowjob-registry.json",
        "retired-external-substrate-registry.json",
        "oya-doc-staleness-inventory-app",
    ] {
        require_contains(&buck, needle, &mut failures, BUCK_PATH);
    }
    require(
        !buck.contains("assert-repo-hygiene-automation.py"),
        &mut failures,
        "BUCK must not depend on retired repo-hygiene Python checker",
    );
    require(
        root.join(DOC_STALENESS_MAIN).exists(),
        &mut failures,
        format!("{DOC_STALENESS_MAIN} must exist"),
    );

    for needle in [
        "buck2 build //:repo-hygiene-automation-check",
        PROWJOB_REGISTRY_COMMAND,
        STALE_DOC_INVENTORY_COMMAND,
        STALE_DOC_INVENTORY_TEST_COMMAND,
    ] {
        require_contains(&workflow, needle, &mut failures, WORKFLOW_PATH);
    }
    require(
        !workflow.contains(RETIRED_PYTHON_COMMAND),
        &mut failures,
        "GitHub lane unlocker workflow must not invoke retired repo-hygiene Python checker",
    );

    Evaluation {
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned(),
        failures,
        domains_checked: REQUIRED_DOMAINS.len(),
        security_backlog_count: REQUIRED_SECURITY_BACKLOG_IDS.len(),
        active_context_scan_files: ACTIVE_CONTEXT_SCAN_PATHS.len(),
        retired_exact_name_scan_files: ACTIVE_EXACT_NAME_SCAN_PATHS.len(),
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
        "{{\"verdict\":\"{}\",\"spec\":\"{}\",\"local_static_only\":true,\"live_mutation_performed\":false,\"domains_checked\":{},\"security_hardening_backlog_count\":{},\"active_context_scan_files\":{},\"retired_exact_name_scan_files\":{},\"stale_doc_inventory_command\":\"{}\",\"stale_doc_inventory_test_command\":\"{}\",\"checker_language\":\"rust\",\"failures\":[{}]}}",
        evaluation.verdict,
        SPEC_PATH,
        evaluation.domains_checked,
        evaluation.security_backlog_count,
        evaluation.active_context_scan_files,
        evaluation.retired_exact_name_scan_files,
        json_escape(STALE_DOC_INVENTORY_COMMAND),
        json_escape(STALE_DOC_INVENTORY_TEST_COMMAND),
        failures
    )
}

fn config() -> (PathBuf, bool) {
    let mut json = false;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--json" => json = true,
            unknown => {
                eprintln!("assert-repo-hygiene-automation: unknown argument {unknown}");
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
            eprintln!("repo-hygiene-automation: RED");
            for failure in &evaluation.failures {
                eprintln!("- {failure}");
            }
        }
        std::process::exit(1);
    }
}
