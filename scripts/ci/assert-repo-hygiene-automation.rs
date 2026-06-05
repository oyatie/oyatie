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
const AGENT_OPERATING_CONTRACT_PATH: &str = "specs/agent-operating-contract.json";
const PHASE0_AUTO_MERGE_AFTER_CI_PATH: &str = "specs/phase0-auto-merge-after-ci.json";
const TENANT_RBAC_SPEC_PATH: &str = "specs/microservices/tenant-rbac.json";
const TENANT_RBAC_PACKAGING_PATH: &str = "specs/tenant-rbac-packaging.json";
const MASTERPLAN_PATH: &str = "specs/masterplan.json";
const SEQUENCING_PATH: &str = "specs/master-plan-sequencing.json";
const DOC_MASTERPLAN_PATH: &str = "docs/MASTERPLAN.md";
const PLANNING_CLOSURE_CONTRACT_PATH: &str = "specs/planning-closure-contract.json";
const PLANNING_CLOSURE_LEDGER_PATH: &str = "specs/planning-closure-status-closure-ledger.json";
const README_PATH: &str = "README.md";
const AGENTS_PATH: &str = "AGENTS.md";
const CLAUDE_PATH: &str = "CLAUDE.md";
const DOC_AGENTS_PATH: &str = "docs/AGENTS.md";
const DOC_CATALOG_PATH: &str = "docs/DOC-CATALOG.md";
const AGENTS_OPERATING_CONTRACT_DOC_PATH: &str = "docs/AGENTS-OPERATING-CONTRACT.md";
const CANONICAL_PRD_PATH: &str = "docs/PRD-OYATIE-FROM-SCRATCH-CANONICAL.md";
const QA_TEST_STRATEGY_PATH: &str = "docs/QA-TEST-STRATEGY.md";
const RELEASE_MANAGEMENT_PATH: &str = "docs/RELEASE-MANAGEMENT.md";
const STANDARDS_TEMPLATES_PATH: &str = "docs/STANDARDS-AND-TEMPLATES.md";
const DOCUMENTATION_PATH: &str = "docs/DOCUMENTATION.md";
const RACI_OWNERSHIP_PATH: &str = "docs/RACI-OWNERSHIP.md";
const VENDOR_PARTNER_LEDGER_PATH: &str = "docs/VENDOR-PARTNER-LEDGER.md";
const BRIEF_TEMPLATE_PATH: &str = "docs/standards/brief-template.md";
const AGENTIC_DEV_TEAM_STANDARD_PATH: &str = "docs/standards/agentic-dev-team-optimization.md";
const CI_LANES_STANDARD_PATH: &str = "docs/standards/ci-lanes.md";
const TOOLS_AGENT_SKILLS_AGENTS_PATH: &str = "tools/agent-skills/AGENTS.md";
const PROCEDURE_PATH: &str = "docs/ci/github-actions-lane-unlocker.md";
const AUTO_MERGE_FLOW_PATH: &str = "docs/ci/auto-merge-flow.md";
const OPENBAO_ESO_RUNBOOK_PATH: &str = "docs/ci/openbao-eso-runbook.md";
const WORKFLOW_PATH: &str = ".github/workflows/github-lane-unlocker-ci-cd.yml";
const GITHUB_ACTIONS_BOOTSTRAP_PATH: &str = "scripts/ci/github-actions-lane-unlocker-bootstrap.sh";
const BUCK_PATH: &str = "BUCK";
const CARGO_TOML_PATH: &str = "Cargo.toml";
const RUST_TOOLCHAIN_PATH: &str = "rust-toolchain.toml";
const BUCK2_AUTHORITY_POLICY_PATH: &str = "specs/buck2-authority-policy.json";
const DEPENDENCY_RATIONALES_PATH: &str = "registry/dependency-rationales.json";
const DEPENDENCY_BLESSED_ALLOWLIST_PATH: &str = "registry/dependency-blessed-allowlist.json";
const TYPESCRIPT_PNPM_SURFACE_INVENTORY_PATH: &str =
    "registry/repo-hygiene/typescript-pnpm-surface-inventory.json";
const PYTHON_SHELL_SURFACE_INVENTORY_PATH: &str =
    "registry/repo-hygiene/python-shell-surface-inventory.json";
const CODE_STYLE_RUST_PATH: &str = "docs/standards/code-style-rust.md";
const DEPENDENCY_POLICY_PATH: &str = "docs/standards/dependency-policy.md";
const LTS_VERSIONS_VERIFIED_PATH: &str = "docs/standards/lts-versions-verified.md";
const OBSERVABILITY_SLO_PATH: &str = "docs/standards/observability-slo.md";
const DOC_STALENESS_MAIN: &str = "tools/oya-doc-staleness-inventory-app/src/main.rs";
const TOOLCHAIN_PIN_UPDATER_PATH: &str = "scripts/ci/sync-latest-toolchain-pins.rs";
const WORKSPACE_HYGIENE_SPEC_PATH: &str = "specs/workspace-hygiene.json";
const FEATURE_FLAG_SUBSTRATE_PATH: &str = "specs/feature-flag-substrate-canonical.json";
const MULTI_REGION_DISPOSITION_PATH: &str = "specs/multi-region-disposition-canonical.json";
const MICROSERVICE_MIGRATION_TOOLING_PATH: &str = "specs/microservice-migration-tooling.json";
const RETIRED_VOCABULARY_PATH: &str = "registry/vocabulary/retired.yaml";
const DOCS_PIPELINE_REGISTRY_PATH: &str = "registry/docs/pipeline.tsv";
const DOCUMENTATION_SYSTEM_KERNEL_PATH: &str = "libs/oya-check-documentation-system/src/lib.rs";
const GATE_CATALOG_DOMAIN_PATH: &str = "libs/oya-governance-gate-catalog-domain/src/lib.rs";
const QUALITY_LANE_KERNEL_PATH: &str = "libs/oya-check-quality-lane/src/lib.rs";

const REQUIRED_RUST_STABLE_VERSION: &str = "1.96.0";
const REQUIRED_RUST_EDITION: &str = "2024";
const REQUIRED_BUCK2_RELEASE: &str = "2026-06-01";
const EXPECTED_TYPESCRIPT_PNPM_MJS_COUNT: usize = 0;
const EXPECTED_NONVENDORED_PYTHON_SHELL_COUNT: usize = 37;

const STALE_DOC_INVENTORY_COMMAND: &str =
    "buck2 build //tools/oya-doc-staleness-inventory-app:doc-staleness-inventory-json";
const STALE_DOC_INVENTORY_TEST_COMMAND: &str =
    "buck2 build //tools/oya-doc-staleness-inventory-app:doc-staleness-inventory-unit-tests";
const REQUIRED_BUCK2_AUTHORITY_COMMAND: &str = "buck2 build //:repo-hygiene-automation-check";
const PROWJOB_REGISTRY_COMMAND: &str = "buck2 build //:oya-ci-prowjob-registry-check";
const OYA_CI_CONTROLLER_CONFIG_COMMAND: &str = "buck2 build //:oya-ci-controller-config-check";
const KUBERNETES_NATIVE_ANTI_PATTERN_COMMAND: &str =
    "buck2 build //:kubernetes-native-anti-pattern-check";
const CLOUD_CELL_ELASTICITY_POLICY_COMMAND: &str =
    "buck2 build //:cloud-cell-elasticity-policy-check";
const QUALITY_LANE_REGISTRY_AUTHORITY_COMMAND: &str =
    "buck2 build //:quality-lane-registry-authority-check";
const TOOLCHAIN_PIN_UPDATER_COMPILE_COMMAND: &str =
    "buck2 build //:latest-toolchain-pin-updater-check";
const APPEND_MISSING_RUST_UNIT_TEST_TARGETS_COMMAND: &str =
    "buck2 build //:append-missing-rust-unit-test-targets-check";
const GENERATE_FIRST_PARTY_BUCK_COMMAND: &str = "buck2 build //:generate-first-party-buck-check";
const RETIRED_GROUPING_WORDING_COMMAND: &str = "buck2 build //:retired-grouping-wording-check";
const NO_GROUPING_KERNEL_CHECK_COMMAND: &str =
    "buck2 build //libs/oya-check-no-grouping:no-grouping-kernel-check";
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
    OYA_CI_CONTROLLER_CONFIG_COMMAND,
    KUBERNETES_NATIVE_ANTI_PATTERN_COMMAND,
    CLOUD_CELL_ELASTICITY_POLICY_COMMAND,
    QUALITY_LANE_REGISTRY_AUTHORITY_COMMAND,
    TOOLCHAIN_PIN_UPDATER_COMPILE_COMMAND,
    APPEND_MISSING_RUST_UNIT_TEST_TARGETS_COMMAND,
    GENERATE_FIRST_PARTY_BUCK_COMMAND,
    RETIRED_GROUPING_WORDING_COMMAND,
    NO_GROUPING_KERNEL_CHECK_COMMAND,
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
    "https://cue.dev/docs/getting-started-with-kubernetes-cue/",
    "https://helm.sh/docs/topics/charts/",
    "https://cloud.google.com/kubernetes-engine/docs/concepts/horizontalpodautoscaler",
    "https://cloud.google.com/kubernetes-engine/docs/concepts/verticalpodautoscaler",
    "https://docs.aws.amazon.com/eks/latest/best-practices/karpenter.html",
    "https://keda.sh/docs/2.18/concepts/scaling-deployments/",
    "https://kubernetes.io/docs/tasks/run-application/scale-deployment/",
    "https://www.nist.gov/publications/zero-trust-architecture",
    "https://csrc.nist.gov/pubs/sp/800/162/upd2/final",
    "https://csrc.nist.gov/glossary/term/policy_based_access_control",
    "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
    "https://kubernetes.io/docs/concepts/security/pod-security-standards/",
    "https://kubernetes.io/docs/tasks/configure-pod-container/security-context/",
    "https://kubernetes.io/docs/concepts/containers/runtime-class/",
    "https://kubernetes.io/docs/tasks/configure-pod-container/configure-service-account/",
    "https://kubernetes.io/docs/reference/access-authn-authz/admission-controllers/",
    "https://cloud.google.com/kubernetes-engine/docs/concepts/workload-identity",
    "https://docs.aws.amazon.com/eks/latest/best-practices/identity-and-access-management.html",
    "https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/",
    "https://docs.github.com/en/actions/concepts/security/openid-connect",
    "https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions",
    "https://slsa.dev/spec/v1.2/requirements",
    "https://istio.io/latest/docs/concepts/security/",
    "https://docs.aws.amazon.com/organizations/latest/userguide/orgs_manage_policies_scps.html",
    "https://www.nist.gov/publications/nist-definition-cloud-computing",
    "https://www.openpolicyagent.org/docs/latest/",
    "https://github.com/ComplianceAsCode/content",
    "https://kubernetes.io/docs/concepts/security/multi-tenancy/",
    "https://docs.cedarpolicy.com/",
    "https://openfga.dev/docs/modeling",
];

const REQUIRED_SECURITY_BACKLOG_IDS: &[&str] = &[
    "zero_trust_architecture",
    "privileged_identity_management",
    "abac_beyond_rbac",
    "pbac_policy_based_access_control",
    "mature_policy_engine",
    "policy_as_code_service_productization",
    "compliance_as_code_service_productization",
    "policy_as_a_service_productization",
    "platform_as_a_service_productization",
    "containers_as_a_service_productization",
    "compliance_as_a_service_productization",
    "oyatie_dogfood_tenant",
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
        "\"status\": \"zero_product_frontend_typescript_pnpm_surface_enforced\"",
        "TypeScript/pnpm product/frontend surface inventory must be zero and policy-enforced",
    ),
    (
        "\"typescript_pnpm_mjs_surface_inventory\": \"registry/repo-hygiene/typescript-pnpm-surface-inventory.json\"",
        "non-Rust surface inventory must point to the disjoint TS/pnpm surface inventory",
    ),
    (
        "\"tracked_typescript_pnpm_mjs_count\": 0",
        "non-Rust surface inventory must record zero tracked TS/pnpm/MJS surfaces",
    ),
    (
        "\"typescript_pnpm_mjs_count_source\": \"filesystem_scan_excluding_vendored_agent_skills\"",
        "non-Rust surface inventory must record the filesystem count source",
    ),
    (
        "\"python_shell_surface_inventory\": \"registry/repo-hygiene/python-shell-surface-inventory.json\"",
        "non-Rust surface inventory must point to the disjoint Python/shell surface inventory",
    ),
    (
        "\"tracked_nonvendored_python_shell_count\": 37",
        "non-Rust surface inventory must record the audited non-vendored Python/shell count",
    ),
    (
        "\"python_shell_count_source\": \"filesystem_scan_excluding_vendored_surfaces\"",
        "non-Rust surface inventory must record the Python/shell filesystem count source",
    ),
    (
        "\"pnpm_or_package_json_repo_authority\": false",
        "pnpm/package metadata must not be repo authority",
    ),
    (
        "\"typescript_runtime_merge_authority\": false",
        "TypeScript runtime surfaces must not exist or be merge authority",
    ),
    (
        "\"strict_typescript_tooling_exception_allowed\": true",
        "strict TypeScript tooling exception rule must be explicit",
    ),
    (
        "\"typescript_exception_requires_buck2_target\": true",
        "strict TypeScript tooling exceptions must require Buck2 authority",
    ),
    (
        "\"typescript_exception_requires_registry_row\": true",
        "strict TypeScript tooling exceptions must require a registry row",
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
        "\"current_typescript_pnpm_mjs_surface_groups\": []",
        "TypeScript/pnpm surface groups must remain empty",
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

const REQUIRED_RUST_TOOLCHAIN_POLICY_NEEDLES: &[(&str, &str)] = &[
    (
        "\"required_rust_stable\": \"1.96.0\"",
        "repo hygiene spec must record the latest stable Rust pin",
    ),
    (
        "\"required_rust_edition\": \"2024\"",
        "repo hygiene spec must record the Rust 2024 edition requirement",
    ),
    (
        "\"pin_policy\": \"latest_official_stable_immediately\"",
        "repo hygiene spec must require the latest official stable Rust release",
    ),
    (
        "\"enforced_by\": \"buck2 build //:repo-hygiene-automation-check\"",
        "Rust toolchain pin policy must be enforced by the repo-hygiene Buck2 target",
    ),
];

const REQUIRED_DEPENDENCY_REGISTRY_POLICY_NEEDLES: &[(&str, &str)] = &[
    (
        "\"dependency_registry_policy\"",
        "repo hygiene spec must record dependency registry policy",
    ),
    (
        "\"coverage_rule\": \"all_workspace_dependencies_tracked\"",
        "dependency registry policy must track all workspace dependencies",
    ),
    (
        "\"version_policy\": \"latest_upstream_stable_or_lts\"",
        "dependency registry policy must require latest stable/LTS dependencies",
    ),
    (
        "\"in_house_policy\": \"in_house_first_oya_rust_libraries\"",
        "dependency registry policy must prefer in-house Oyatie Rust libraries",
    ),
    (
        "\"exception_policy\": \"explicit_waiver_required_for_non_latest_or_non_in_house_dependency\"",
        "dependency registry policy must require explicit waivers for exceptions",
    ),
    (
        "\"replacement_strategy_required\": true",
        "dependency registry policy must require replacement/wrapper strategy",
    ),
];

const REQUIRED_BUCK2_RELEASE_POLICY_NEEDLES: &[(&str, &str)] = &[
    (
        "\"required_buck2_release\": \"2026-06-01\"",
        "repo hygiene spec must record the current Buck2 release pin",
    ),
    (
        "\"buck2_release_source\": \"https://github.com/facebook/buck2/releases/tag/2026-06-01\"",
        "repo hygiene spec must cite the current Buck2 release tag",
    ),
    (
        "\"buck2_release_policy\": \"latest_upstream_date_tag_immediately\"",
        "Buck2 release policy must require latest upstream date tag",
    ),
    (
        "\"compile_check\": \"buck2 build //:latest-toolchain-pin-updater-check\"",
        "toolchain updater compile check must be Buck2-owned",
    ),
];

const CLEANUP_BACKLOG_IDS: &[&str] = &[
    "legacy_python_shell_gate_surfaces",
    "shared_ci_workflow_surface",
    "root_hub_masterplan_shared_docs",
    "shared_surface_substrate_migration_audit",
    "cue_first_cell_pod_config_authority",
    "helm_adapter_compatibility_wrapper",
    "scale_to_zero_eligibility_gate",
    "stale_doc_inventory_followups",
    "retired_external_substrate_residue",
    "temporary_github_bridge_artifacts",
    "retire_oya_cli_governance_authority",
    "retired_vcs_cli_admission_surface_retirement",
    "typescript_pnpm_retirement_review",
    "product_sdk_language_policy_review",
    "quoted_path_filename_normalization",
    "active_policy_context_name_normalization",
    "active_dotdir_state_surface_review",
    "single_file_top_level_root_review",
    "prow_job_registry_generation",
    "python_shell_to_rust_buck2_migration",
    "cd_fleet_bootstrap_surface_retirement",
    "retired_external_scm_adapter_retirement",
    "in_house_dependency_library_substitution",
    "latest_toolchain_dependency_pin_updater",
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
    "Local `oya verify`, local `oya gate`",
    "local oya verify output",
    "local oya gate output",
    "retired `oya gate` / `oya verify`",
    "the `oya git`, `oya vcs`, `oya gate`, and `oya verify` CLI surfaces",
    "claim with oya vcs before edits",
    "done and promote through oya vcs",
    "oya vcs status",
    "oya vcs verify evidence strings",
    "oya-git",
    "oya-vcs",
    "oya-vcs-admission",
    "oya-dev-cli:oya -- gate validate planning-closure",
    "oya gate validate planning-closure",
    "oya gate validate product-prd-json",
    "legacy CI governance lifecycle",
    "legacy CI contexts",
    "retired local verify command",
    "retired local gate run-all command",
    "oya gate validate",
    "oya gate run-all",
    "oya-dev-cli",
    "oya doc ",
    "oya verify command",
    "`oya gate` / `oya verify`",
    "reviewer/governance approval",
    "reviewer/governance lifecycle",
    "bacon",
    "cargo-machete",
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
    "Local `oya verify`, local `oya gate`",
    "local oya verify output",
    "local oya gate output",
    "retired `oya gate` / `oya verify`",
    "the `oya git`, `oya vcs`, `oya gate`, and `oya verify` CLI surfaces",
    "claim with oya vcs before edits",
    "done and promote through oya vcs",
    "oya vcs status",
    "oya vcs verify evidence strings",
    "oya-git",
    "oya-vcs",
    "oya-vcs-admission",
    "oya gate validate planning-closure",
    "oya gate validate product-prd-json",
    "Jenkins governance lifecycle",
    "Jenkins contexts",
    "./bin/oya verify --ci-required",
    "./bin/oya gate run-all",
    "oya gate validate",
    "oya gate run-all",
    "oya-dev-cli",
    "oya doc ",
    "oya verify command",
    "`oya gate` / `oya verify`",
    "reviewer/governance approval",
    "reviewer/governance lifecycle",
    "bacon",
    "cargo-machete",
];

const FORBIDDEN_ACTIVE_TEMPLATE_PHRASES: &[&str] = &[
    "oya gate validate",
    "oya verify",
    "`oya git`",
    "pnpm",
    "Node 20",
    "typescript-reviewer",
    "python-reviewer",
    "bacon",
    "cargo-machete",
    "cargo nextest run",
    "cargo clippy",
    "cargo deny check",
    "cargo-semver-checks",
    "cargo public-api",
    "cargo vet",
    "cargo run -p",
    "cargo metadata diff",
    "npm --prefix",
    "grit claim",
    "grit done",
    "grit_claim",
    "grit-status",
    "oya-tooling-agent-read",
    "icm store",
    "Icm-store",
];

const ACTIVE_CONTEXT_SCAN_PATHS: &[&str] = &[
    AGENTS_PATH,
    CLAUDE_PATH,
    README_PATH,
    DOC_AGENTS_PATH,
    DOC_CATALOG_PATH,
    AGENTS_OPERATING_CONTRACT_DOC_PATH,
    CANONICAL_PRD_PATH,
    QA_TEST_STRATEGY_PATH,
    RELEASE_MANAGEMENT_PATH,
    STANDARDS_TEMPLATES_PATH,
    DOCUMENTATION_PATH,
    RACI_OWNERSHIP_PATH,
    VENDOR_PARTNER_LEDGER_PATH,
    BRIEF_TEMPLATE_PATH,
    AGENTIC_DEV_TEAM_STANDARD_PATH,
    CI_LANES_STANDARD_PATH,
    TOOLS_AGENT_SKILLS_AGENTS_PATH,
    PROCEDURE_PATH,
    AUTO_MERGE_FLOW_PATH,
    "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
    ".github/branch-protection.yaml",
    "infra/branch-protection/dev.json",
    AGENT_OPERATING_CONTRACT_PATH,
    PHASE0_AUTO_MERGE_AFTER_CI_PATH,
    TENANT_RBAC_SPEC_PATH,
    "docs/MASTERPLAN.md",
];

const ACTIVE_TEMPLATE_SCAN_PATHS: &[&str] = &[
    "templates/INDEX.md",
    "templates/adr-template.md",
    "templates/capability-record-template.yaml",
    "templates/checklists/agent-completion-checklist.md",
    "templates/checklists/agent-kickoff-checklist.md",
    "templates/checklists/cross-axis-contract-change-checklist.md",
    "templates/checklists/doc-freshness-checklist.md",
    "templates/checklists/done-definition-checklist.md",
    "templates/checklists/escalation-checklist.md",
    "templates/checklists/inventory-update-checklist.md",
    "templates/checklists/per-implementation-plan-checklist.md",
    "templates/checklists/per-phase-completion-checklist.md",
    "templates/checklists/pr-review-checklist.md",
    "templates/checklists/pre-flight-checklist.md",
    "templates/checklists/pre-pr-multispectrum.json",
    "templates/checklists/pre-pr-multispectrum.md",
    "templates/checklists/release-readiness-checklist.md",
    "templates/design-doc-template.md",
    "templates/evidence-bundle-template.json",
    "templates/foundry-supervisor/claude.toml",
    "templates/foundry-supervisor/codex.toml",
    "templates/foundry-supervisor/gemini.toml",
    "templates/implementation-plan-template.md",
    "templates/milestone-index-template.md",
    "templates/mistakes-ledger-row-template.md",
    "templates/phase-index-template.md",
    "templates/postmortem-template.md",
    "templates/prfaq-template.md",
    "templates/pull-request-template.md",
    "templates/runbook-template.md",
];

const ACTIVE_EXACT_NAME_SCAN_PATHS: &[&str] = &[
    AGENTS_PATH,
    CLAUDE_PATH,
    README_PATH,
    DOC_AGENTS_PATH,
    DOC_CATALOG_PATH,
    AGENTS_OPERATING_CONTRACT_DOC_PATH,
    CANONICAL_PRD_PATH,
    QA_TEST_STRATEGY_PATH,
    RELEASE_MANAGEMENT_PATH,
    STANDARDS_TEMPLATES_PATH,
    DOCUMENTATION_PATH,
    RACI_OWNERSHIP_PATH,
    VENDOR_PARTNER_LEDGER_PATH,
    BRIEF_TEMPLATE_PATH,
    AGENTIC_DEV_TEAM_STANDARD_PATH,
    CI_LANES_STANDARD_PATH,
    TOOLS_AGENT_SKILLS_AGENTS_PATH,
    "docs/MASTERPLAN.md",
    PROCEDURE_PATH,
    AUTO_MERGE_FLOW_PATH,
    "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
    "docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md",
    ".github/branch-protection.yaml",
    "infra/branch-protection/dev.json",
    ROOT_HUB_PATH,
    AGENT_OPERATING_CONTRACT_PATH,
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
const RUNBOOK_STALE_PROMOTION_GATE_PHRASES: &[&str] = &[
    "Jenkins + `oya gate run-all --ci-required` required",
    "Jenkins green, `oya gate run-all --ci-required` and `oya verify --ci-required` evidence attached",
    "green Jenkins CI and `oya gate run-all --ci-required`",
    "require Jenkins + `oya gate run-all --ci-required` before merge",
];
const CLOUD_NETWORK_DNS_STALE_AUTHORITY_PHRASES: &[&str] = &[
    "canonical local pre-push verifier",
    "Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates",
    "Jenkins (LTS) and ArgoCD are the two canonical self-hostable CI/CD substrates",
    "ArgoCD is the canonical GitOps CD orchestrator",
    "Jenkins build id",
    "ArgoCD sync id",
    "`./bin/oya git worktree-add",
    "VCS CLAIM: `./bin/oya vcs claim",
    "Treat Jenkins LTS as the self-hostable CI substrate",
    "Verify Jenkins/GitHub Actions parity evidence exists",
    "argocd app get",
];
const CLOUD_NETWORK_STALE_AUTHORITY_PHRASES: &[&str] = &[
    "canonical local pre-push verifier",
    "Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates",
    "Jenkins (LTS) and ArgoCD are the two canonical self-hostable CI/CD substrates",
    "ArgoCD is the canonical GitOps CD orchestrator",
    "ArgoCD is the GitOps CD orchestrator",
    "Jenkins LTS and ArgoCD are the canonical self-hostable CI/CD substrates",
    "cargo run -p oya-dev-cli",
    "cargo run -q -p oya-dev-cli",
    "cargo run --release",
    "cargo test --features hermetic",
    "`./bin/oya git worktree-add",
    "`./bin/oya vcs verify",
    "Jenkins/GitHub Actions parity under ADR-0349",
    "Jenkins + `oya gate run-all --ci-required`",
    "argocd app get",
    "Jenkins build id",
    "ArgoCD sync id",
    "app.kubernetes.io/managed-by: Helm",
    "oyatie.com/adr-0349: argocd-managed",
];
const WORKPLACE_INTEGRATION_STALE_AUTHORITY_PHRASES: &[&str] = &[
    "canonical local pre-push verifier",
    "Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates",
    "Jenkins (LTS) and ArgoCD are the two canonical self-hostable CI/CD substrates",
    "ArgoCD is the canonical GitOps CD orchestrator",
    "`./bin/oya git worktree-add",
    "cargo run -p oya-dev-cli -- gate validate",
    "cargo run -q -p oya-dev-cli -- gate validate",
    "cargo run -q -p oya-dev-cli -- doc inventory",
    "VCS CLAIM: `./bin/oya vcs claim",
    "`./bin/oya vcs verify",
    "Jenkins plus ArgoCD substrate expectations",
    "Jenkins/GitHub Actions parity under ADR-0349",
    "helm/templates/deployment.yaml",
    "app.kubernetes.io/managed-by: Helm",
    "oyatie.com/adr-0349: argocd-managed",
];
const CELL_LIFECYCLE_STALE_AUTHORITY_PHRASES: &[&str] = &[
    "canonical local pre-push verifier",
    "Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates",
    "Jenkins (LTS) and ArgoCD are the two canonical self-hostable CI/CD substrates",
    "ArgoCD is the canonical GitOps CD orchestrator",
    "cargo run -q -p oya-dev-cli -- gate validate",
    "cargo run -q -p oya-dev-cli -- doc inventory",
    "Jenkins/GitHub Actions parity under ADR-0349",
    "helm/templates/deployment.yaml",
    "app.kubernetes.io/managed-by: Helm",
    "oyatie.com/adr-0349: argocd-managed",
];
const CLOUD_BILLING_TAX_STALE_AUTHORITY_PHRASES: &[&str] = &[
    "canonical local pre-push verifier",
    "Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates",
    "Jenkins (LTS) and ArgoCD are the two canonical self-hostable CI/CD substrates",
    "ArgoCD is the canonical GitOps CD orchestrator",
    "ArgoCD is the GitOps CD orchestrator",
    "Jenkins LTS and ArgoCD are the canonical self-hostable CI/CD substrates",
    "cargo run -q -p oya-dev-cli -- gate validate",
    "cargo run -q -p oya-dev-cli -- doc inventory",
    "cargo run --bin oya-dev-cli",
    "`./bin/oya git worktree-add",
    "`./bin/oya vcs verify",
    "Jenkins/GitHub Actions parity under ADR-0349",
    "argocd app get",
    "Jenkins build id",
    "ArgoCD sync id",
    "app.kubernetes.io/managed-by: Helm",
    "oyatie.com/adr-0349: argocd-managed",
];
const CLOUD_IAM_STALE_AUTHORITY_PHRASES: &[&str] = &[
    "canonical local pre-push verifier",
    "Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates",
    "Jenkins (LTS) and ArgoCD are the two canonical self-hostable CI/CD substrates",
    "ArgoCD is the canonical GitOps CD orchestrator",
    "ArgoCD is the GitOps CD orchestrator",
    "Jenkins LTS and ArgoCD are the canonical self-hostable CI/CD substrates",
    "cargo run -p oya-dev-cli",
    "cargo run -q -p oya-dev-cli",
    "cargo build --workspace",
    "cargo run --release",
    "cargo test --features hermetic",
    "`./bin/oya git worktree-add",
    "`./bin/oya vcs verify",
    "Jenkins/GitHub Actions parity under ADR-0349",
    "Jenkins + `oya gate run-all --ci-required`",
    "argocd app get",
    "Jenkins build id",
    "ArgoCD sync id",
    "app.kubernetes.io/managed-by: Helm",
    "oyatie.com/adr-0349: argocd-managed",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
    pub domains_checked: usize,
    pub security_backlog_count: usize,
    pub tracked_typescript_pnpm_mjs_count: usize,
    pub tracked_nonvendored_python_shell_count: usize,
    pub active_context_scan_files: usize,
    pub active_template_scan_files: usize,
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

fn workspace_dependency_names(cargo_toml: &str) -> Vec<String> {
    let mut in_workspace_dependencies = false;
    let mut names = Vec::new();

    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.dependencies]" {
            in_workspace_dependencies = true;
            continue;
        }
        if in_workspace_dependencies && trimmed.starts_with('[') {
            break;
        }
        if !in_workspace_dependencies || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            names.push(name.to_owned());
        }
    }

    names.sort();
    names.dedup();
    names
}

fn json_object_keys(text: &str, object_key: &str) -> Vec<String> {
    let marker = format!("\"{}\"", object_key);
    let Some(marker_index) = text.find(&marker) else {
        return Vec::new();
    };
    let Some(relative_start) = text[marker_index..].find('{') else {
        return Vec::new();
    };
    let start = marker_index + relative_start;
    let mut keys = Vec::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    let mut string_start = None;
    let mut last_string_at_depth_one = None::<String>;

    for (index, ch) in text[start..].char_indices() {
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
                if depth == 1 {
                    if let Some(start_index) = string_start {
                        last_string_at_depth_one =
                            Some(text[start + start_index..start + index].to_owned());
                    }
                }
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                string_start = Some(index + 1);
            }
            '{' => depth += 1,
            '}' => {
                if depth == 1 {
                    break;
                }
                depth = depth.saturating_sub(1);
            }
            ':' if depth == 1 => {
                if let Some(key) = last_string_at_depth_one.take() {
                    keys.push(key);
                }
            }
            _ => {}
        }
    }

    keys.sort();
    keys.dedup();
    keys
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

fn path_to_repo_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn is_typescript_pnpm_surface_file(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if matches!(
        file_name,
        "package.json" | "pnpm-lock.yaml" | "pnpm-workspace.yaml" | ".npmrc"
    ) {
        return true;
    }
    if file_name.starts_with("tsconfig") && file_name.ends_with(".json") {
        return true;
    }
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "mjs" | "cjs" | "js" | "jsx" | "svelte")
    )
}

fn is_excluded_typescript_pnpm_dir(rel: &Path) -> bool {
    let rel_string = path_to_repo_string(rel);
    let Some(file_name) = rel.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(
        file_name,
        ".git" | "buck-out" | "node_modules" | "target" | "third-party" | "vendor"
    ) || rel_string == "tools/agent-skills"
        || rel_string.starts_with("tools/agent-skills/")
}

fn collect_typescript_pnpm_surfaces(
    root: &Path,
    rel: &Path,
    output: &mut Vec<String>,
) -> Result<(), String> {
    if is_excluded_typescript_pnpm_dir(rel) {
        return Ok(());
    }
    let dir = root.join(rel);
    for entry in
        fs::read_dir(&dir).map_err(|error| format!("read dir {}: {}", dir.display(), error))?
    {
        let entry =
            entry.map_err(|error| format!("read dir entry {}: {}", dir.display(), error))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("file type {}: {}", entry.path().display(), error))?;
        let entry_rel = rel.join(entry.file_name());
        if file_type.is_dir() {
            collect_typescript_pnpm_surfaces(root, &entry_rel, output)?;
        } else if file_type.is_file() && is_typescript_pnpm_surface_file(&entry.path()) {
            output.push(path_to_repo_string(&entry_rel));
        }
    }
    Ok(())
}

pub fn tracked_typescript_pnpm_mjs_files(root: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    collect_typescript_pnpm_surfaces(root, Path::new(""), &mut files)?;
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn typescript_pnpm_surface_failures(root: &Path, inventory: &str) -> (usize, Vec<String>) {
    let mut failures = Vec::new();
    let files = match tracked_typescript_pnpm_mjs_files(root) {
        Ok(files) => files,
        Err(error) => {
            failures.push(format!("TypeScript/pnpm surface scan failed: {error}"));
            Vec::new()
        }
    };

    if files.len() != EXPECTED_TYPESCRIPT_PNPM_MJS_COUNT {
        failures.push(format!(
            "TypeScript/pnpm surface count drift: expected {}, found {}",
            EXPECTED_TYPESCRIPT_PNPM_MJS_COUNT,
            files.len()
        ));
    }

    if !compact_json_text(inventory).contains(&format!(
        "\"tracked_file_count\":{}",
        EXPECTED_TYPESCRIPT_PNPM_MJS_COUNT
    )) {
        failures.push(format!(
            "{} must record tracked_file_count {}",
            TYPESCRIPT_PNPM_SURFACE_INVENTORY_PATH, EXPECTED_TYPESCRIPT_PNPM_MJS_COUNT
        ));
    }

    for needle in [
        "\"status\": \"zero_product_frontend_surface_registered_tooling_exception\"",
        "\"count_source\": \"filesystem_scan_excluding_vendored_agent_skills\"",
        "\"pnpm_or_package_json_repo_authority\": false",
        "\"typescript_runtime_merge_authority\": false",
        "\"durable_gate_authority\": false",
        "\"strict_typescript_tooling_exception_allowed\": true",
        "\"typescript_exception_requires_buck2_target\": true",
        "\"typescript_exception_requires_registry_row\": true",
        "\"buck2_remains_build_test_check_authority\": true",
        "\"tools/agent-skills/\"",
    ] {
        if !inventory.contains(needle) {
            failures.push(format!(
                "{} missing required anchor {}",
                TYPESCRIPT_PNPM_SURFACE_INVENTORY_PATH, needle
            ));
        }
    }

    for path in &files {
        if !inventory.contains(path) {
            failures.push(format!(
                "{} missing tracked TypeScript/pnpm surface {}",
                TYPESCRIPT_PNPM_SURFACE_INVENTORY_PATH, path
            ));
        }
    }

    (files.len(), failures)
}

fn is_python_shell_surface_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("py" | "sh" | "bash" | "zsh")
    )
}

fn is_excluded_python_shell_dir(rel: &Path) -> bool {
    let rel_string = path_to_repo_string(rel);
    let Some(file_name) = rel.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(
        file_name,
        ".git" | "buck-out" | "node_modules" | "target" | "third-party" | "vendor"
    ) || rel_string == "tools/agent-skills"
        || rel_string.starts_with("tools/agent-skills/")
}

fn collect_python_shell_surfaces(
    root: &Path,
    rel: &Path,
    output: &mut Vec<String>,
) -> Result<(), String> {
    if is_excluded_python_shell_dir(rel) {
        return Ok(());
    }
    let dir = root.join(rel);
    for entry in
        fs::read_dir(&dir).map_err(|error| format!("read dir {}: {}", dir.display(), error))?
    {
        let entry =
            entry.map_err(|error| format!("read dir entry {}: {}", dir.display(), error))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("file type {}: {}", entry.path().display(), error))?;
        let entry_rel = rel.join(entry.file_name());
        if file_type.is_dir() {
            collect_python_shell_surfaces(root, &entry_rel, output)?;
        } else if file_type.is_file() && is_python_shell_surface_file(&entry.path()) {
            output.push(path_to_repo_string(&entry_rel));
        }
    }
    Ok(())
}

pub fn tracked_python_shell_files(root: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    collect_python_shell_surfaces(root, Path::new(""), &mut files)?;
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn python_shell_surface_failures(root: &Path, inventory: &str) -> (usize, Vec<String>) {
    let mut failures = Vec::new();
    let files = match tracked_python_shell_files(root) {
        Ok(files) => files,
        Err(error) => {
            failures.push(format!("Python/shell surface scan failed: {error}"));
            Vec::new()
        }
    };

    if files.len() != EXPECTED_NONVENDORED_PYTHON_SHELL_COUNT {
        failures.push(format!(
            "Python/shell surface count drift: expected {}, found {}",
            EXPECTED_NONVENDORED_PYTHON_SHELL_COUNT,
            files.len()
        ));
    }

    if !compact_json_text(inventory).contains(&format!(
        "\"tracked_file_count\":{}",
        EXPECTED_NONVENDORED_PYTHON_SHELL_COUNT
    )) {
        failures.push(format!(
            "{} must record tracked_file_count {}",
            PYTHON_SHELL_SURFACE_INVENTORY_PATH, EXPECTED_NONVENDORED_PYTHON_SHELL_COUNT
        ));
    }

    for needle in [
        "\"status\": \"classified_no_durable_authority\"",
        "\"count_source\": \"filesystem_scan_excluding_vendored_surfaces\"",
        "\"python_shell_durable_gate_authority\": false",
        "\"new_python_or_shell_gate_surface\": \"deny_unless_explicit_bootstrap_exception\"",
        "\"rewrite_active_gate_surfaces_to_rust_buck2\": true",
        "\"buck2_remains_build_test_check_authority\": true",
        "\"tools/agent-skills/\"",
    ] {
        if !inventory.contains(needle) {
            failures.push(format!(
                "{} missing required anchor {}",
                PYTHON_SHELL_SURFACE_INVENTORY_PATH, needle
            ));
        }
    }

    for path in &files {
        if !inventory.contains(path) {
            failures.push(format!(
                "{} missing tracked Python/shell surface {}",
                PYTHON_SHELL_SURFACE_INVENTORY_PATH, path
            ));
        }
    }

    (files.len(), failures)
}

fn collect_cedar_policy_files(
    root: &Path,
    rel: &Path,
    output: &mut Vec<String>,
) -> Result<(), String> {
    let dir = root.join(rel);
    if !dir.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(&dir).map_err(|error| format!("read dir {}: {}", dir.display(), error))?
    {
        let entry =
            entry.map_err(|error| format!("read dir entry {}: {}", dir.display(), error))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("file type {}: {}", entry.path().display(), error))?;
        let entry_rel = rel.join(entry.file_name());
        if file_type.is_dir() {
            collect_cedar_policy_files(root, &entry_rel, output)?;
        } else if file_type.is_file()
            && entry_rel.file_name().and_then(|name| name.to_str()) == Some("policies.cedar")
            && entry_rel
                .components()
                .any(|component| component.as_os_str() == "cedar")
        {
            output.push(path_to_repo_string(&entry_rel));
        }
    }
    Ok(())
}

pub fn active_policy_context_name_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let mut policy_files = Vec::new();
    for rel in [Path::new("cloud"), Path::new("oya")] {
        if let Err(error) = collect_cedar_policy_files(root, rel, &mut policy_files) {
            failures.push(format!("active Cedar policy scan failed: {error}"));
        }
    }
    policy_files.sort();
    policy_files.dedup();

    for rel in policy_files {
        let path = root.join(&rel);
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!(
                "{rel}: read failed during active policy context scan"
            ));
            continue;
        };
        for forbidden in [
            "context.doctrine.adr_",
            "context.doctrine.prd_",
            "context.doctrine.phase_",
            "context.doctrine.ip_",
        ] {
            if text.contains(forbidden) {
                failures.push(format!(
                    "{rel}: active policy context field must be capability-named, not provenance-token-named ({forbidden})"
                ));
            }
        }
    }

    failures
}

fn collect_runbook_markdown_files(
    root: &Path,
    rel: &Path,
    output: &mut Vec<String>,
) -> Result<(), String> {
    let dir = root.join(rel);
    if !dir.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(&dir).map_err(|error| format!("read dir {}: {}", dir.display(), error))?
    {
        let entry =
            entry.map_err(|error| format!("read dir entry {}: {}", dir.display(), error))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("file type {}: {}", entry.path().display(), error))?;
        let entry_rel = rel.join(entry.file_name());
        if file_type.is_dir() {
            collect_runbook_markdown_files(root, &entry_rel, output)?;
        } else if file_type.is_file()
            && entry_rel
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("md")
            && entry_rel
                .components()
                .any(|component| component.as_os_str() == "runbooks")
        {
            output.push(path_to_repo_string(&entry_rel));
        }
    }
    Ok(())
}

pub fn runbook_promotion_gate_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let mut runbooks = Vec::new();

    if let Err(error) = collect_runbook_markdown_files(root, Path::new("oya"), &mut runbooks) {
        failures.push(format!("runbook promotion-gate scan failed: {error}"));
    }
    runbooks.sort();
    runbooks.dedup();

    for rel in runbooks {
        let path = root.join(&rel);
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!(
                "{rel}: read failed during runbook promotion-gate scan"
            ));
            continue;
        };
        for phrase in RUNBOOK_STALE_PROMOTION_GATE_PHRASES {
            if text.contains(phrase) {
                failures.push(format!(
                    "{rel}: stale runbook promotion gate references retired Jenkins/oya-gate authority: {phrase:?}; use `oya-ci-required` + Buck2 evidence"
                ));
            }
        }
    }

    failures
}

fn collect_markdown_yaml_json_files(
    root: &Path,
    rel: &Path,
    output: &mut Vec<String>,
) -> Result<(), String> {
    let dir = root.join(rel);
    if !dir.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(&dir).map_err(|error| format!("read dir {}: {}", dir.display(), error))?
    {
        let entry =
            entry.map_err(|error| format!("read dir entry {}: {}", dir.display(), error))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("file type {}: {}", entry.path().display(), error))?;
        let entry_rel = rel.join(entry.file_name());
        if file_type.is_dir() {
            collect_markdown_yaml_json_files(root, &entry_rel, output)?;
        } else if file_type.is_file()
            && matches!(
                entry_rel
                    .extension()
                    .and_then(|extension| extension.to_str()),
                Some("md" | "yaml" | "yml" | "json")
            )
        {
            output.push(path_to_repo_string(&entry_rel));
        }
    }
    Ok(())
}

pub fn cloud_network_dns_authority_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let mut files = Vec::new();

    if let Err(error) =
        collect_markdown_yaml_json_files(root, Path::new("cloud/cloud-network-dns"), &mut files)
    {
        failures.push(format!("cloud-network-dns authority scan failed: {error}"));
    }
    files.sort();
    files.dedup();

    for rel in files {
        let path = root.join(&rel);
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!(
                "{rel}: read failed during cloud-network-dns authority scan"
            ));
            continue;
        };
        for phrase in CLOUD_NETWORK_DNS_STALE_AUTHORITY_PHRASES {
            if text.contains(phrase) {
                failures.push(format!(
                    "{rel}: stale cloud-network-dns active authority phrase present: {phrase:?}; use ADR-0513 Buck2/Prow `oya-ci-required` plus native release-conveyor wording"
                ));
            }
        }
    }

    failures
}

pub fn cloud_network_authority_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let mut files = Vec::new();

    if let Err(error) =
        collect_markdown_yaml_json_files(root, Path::new("cloud/cloud-network"), &mut files)
    {
        failures.push(format!("cloud-network authority scan failed: {error}"));
    }
    files.sort();
    files.dedup();

    for rel in files {
        let path = root.join(&rel);
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!(
                "{rel}: read failed during cloud-network authority scan"
            ));
            continue;
        };
        for phrase in CLOUD_NETWORK_STALE_AUTHORITY_PHRASES {
            if text.contains(phrase) {
                failures.push(format!(
                    "{rel}: stale cloud-network active authority phrase present: {phrase:?}; use ADR-0513 Buck2/Prow `oya-ci-required`, CUE/KRM desired state, and native release-conveyor wording"
                ));
            }
        }
    }

    let helm_dir = root.join("cloud/cloud-network/iac/k8s/helm");
    if helm_dir.exists() {
        failures.push(
            "cloud/cloud-network/iac/k8s/helm: first-party Helm chart directory must not exist; cloud-network desired state is CUE/KRM plus Buck2/Prow evidence"
                .to_string(),
        );
    }

    failures
}

pub fn workplace_integration_authority_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let mut files = Vec::new();

    if let Err(error) =
        collect_markdown_yaml_json_files(root, Path::new("oya/workplace-integration"), &mut files)
    {
        failures.push(format!(
            "workplace-integration authority scan failed: {error}"
        ));
    }
    files.sort();
    files.dedup();

    for rel in files {
        let path = root.join(&rel);
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!(
                "{rel}: read failed during workplace-integration authority scan"
            ));
            continue;
        };
        for phrase in WORKPLACE_INTEGRATION_STALE_AUTHORITY_PHRASES {
            if text.contains(phrase) {
                failures.push(format!(
                    "{rel}: stale workplace-integration active authority phrase present: {phrase:?}; use ADR-0513 Buck2/Prow `oya-ci-required`, CUE/KRM desired state, and native release-conveyor wording"
                ));
            }
        }
    }

    let helm_dir = root.join("oya/workplace-integration/iac/k8s/helm");
    if helm_dir.exists() {
        failures.push(
            "oya/workplace-integration/iac/k8s/helm: first-party Helm chart directory must not exist; workplace-integration desired state is CUE/KRM plus Buck2/Prow evidence"
                .to_string(),
        );
    }

    failures
}

pub fn cell_lifecycle_authority_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let mut files = Vec::new();

    if let Err(error) =
        collect_markdown_yaml_json_files(root, Path::new("cloud/cell-lifecycle"), &mut files)
    {
        failures.push(format!("cell-lifecycle authority scan failed: {error}"));
    }
    files.sort();
    files.dedup();

    for rel in files {
        let path = root.join(&rel);
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!(
                "{rel}: read failed during cell-lifecycle authority scan"
            ));
            continue;
        };
        for phrase in CELL_LIFECYCLE_STALE_AUTHORITY_PHRASES {
            if text.contains(phrase) {
                failures.push(format!(
                    "{rel}: stale cell-lifecycle active authority phrase present: {phrase:?}; use ADR-0513 Buck2/Prow `oya-ci-required`, CUE/KRM desired state, and native release-conveyor wording"
                ));
            }
        }
    }

    let helm_dir = root.join("cloud/cell-lifecycle/iac/k8s/helm");
    if helm_dir.exists() {
        failures.push(
            "cloud/cell-lifecycle/iac/k8s/helm: first-party Helm chart directory must not exist; cell-lifecycle desired state is CUE/KRM plus Buck2/Prow evidence"
                .to_string(),
        );
    }

    failures
}

pub fn cloud_billing_tax_authority_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let mut files = Vec::new();

    if let Err(error) =
        collect_markdown_yaml_json_files(root, Path::new("cloud/cloud-billing-tax"), &mut files)
    {
        failures.push(format!("cloud-billing-tax authority scan failed: {error}"));
    }
    files.sort();
    files.dedup();

    for rel in files {
        let path = root.join(&rel);
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!(
                "{rel}: read failed during cloud-billing-tax authority scan"
            ));
            continue;
        };
        for phrase in CLOUD_BILLING_TAX_STALE_AUTHORITY_PHRASES {
            if text.contains(phrase) {
                failures.push(format!(
                    "{rel}: stale cloud-billing-tax active authority phrase present: {phrase:?}; use ADR-0513 Buck2/Prow `oya-ci-required`, CUE/KRM desired state, and native release-conveyor wording"
                ));
            }
        }
    }

    let helm_dir = root.join("cloud/cloud-billing-tax/iac/k8s/helm");
    if helm_dir.exists() {
        failures.push(
            "cloud/cloud-billing-tax/iac/k8s/helm: first-party Helm chart directory must not exist; cloud-billing-tax desired state is CUE/KRM plus Buck2/Prow evidence"
                .to_string(),
        );
    }

    failures
}

pub fn cloud_iam_authority_failures(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let mut files = Vec::new();

    if let Err(error) =
        collect_markdown_yaml_json_files(root, Path::new("cloud/cloud-iam"), &mut files)
    {
        failures.push(format!("cloud-iam authority scan failed: {error}"));
    }
    files.sort();
    files.dedup();

    for rel in files {
        let path = root.join(&rel);
        let Ok(text) = fs::read_to_string(&path) else {
            failures.push(format!(
                "{rel}: read failed during cloud-iam authority scan"
            ));
            continue;
        };
        for phrase in CLOUD_IAM_STALE_AUTHORITY_PHRASES {
            if text.contains(phrase) {
                failures.push(format!(
                    "{rel}: stale cloud-iam active authority phrase present: {phrase:?}; use ADR-0513 Buck2/Prow `oya-ci-required`, CUE/KRM desired state, and native release-conveyor wording"
                ));
            }
        }
    }

    let helm_dir = root.join("cloud/cloud-iam/iac/k8s/helm");
    if helm_dir.exists() {
        failures.push(
            "cloud/cloud-iam/iac/k8s/helm: first-party Helm chart directory must not exist; cloud-iam desired state is CUE/KRM plus Buck2/Prow evidence"
                .to_string(),
        );
    }

    failures
}

pub fn active_doc_phrase_failures(label: &str, text: &str) -> Vec<String> {
    let lowered_text = text.to_ascii_lowercase();
    FORBIDDEN_ACTIVE_DOC_PHRASES
        .iter()
        .filter(|phrase| lowered_text.contains(&phrase.to_ascii_lowercase()))
        .map(|phrase| format!("{label}: stale active authority phrase present: {phrase:?}"))
        .collect()
}

pub fn active_template_phrase_failures(label: &str, text: &str) -> Vec<String> {
    let lowered_text = text.to_ascii_lowercase();
    FORBIDDEN_ACTIVE_TEMPLATE_PHRASES
        .iter()
        .filter(|phrase| lowered_text.contains(&phrase.to_ascii_lowercase()))
        .map(|phrase| format!("{label}: stale active template phrase present: {phrase:?}"))
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

fn require_absent(text: &str, needle: &str, failures: &mut Vec<String>, label: &str) {
    require(
        !text.contains(needle),
        failures,
        format!("{label}: stale retired CLI authority present: {needle:?}"),
    );
}

pub fn retired_cli_registry_spec_failures(
    workspace_hygiene: &str,
    feature_flag: &str,
    multi_region: &str,
    microservice_migration: &str,
    retired_vocabulary: &str,
    docs_pipeline: &str,
    documentation_system_kernel: &str,
) -> Vec<String> {
    let mut failures = Vec::new();

    require_contains(
        workspace_hygiene,
        "\"command\": \"buck2 build //:repo-hygiene-automation-check\"",
        &mut failures,
        WORKSPACE_HYGIENE_SPEC_PATH,
    );
    require(
        compact_json_text(workspace_hygiene).contains("\"cleanup_commands\":[]"),
        &mut failures,
        format!(
            "{WORKSPACE_HYGIENE_SPEC_PATH}: cleanup commands must stay empty until a Rust/Buck2/Prow cleanup job owns side effects"
        ),
    );
    require_contains(
        feature_flag,
        "planned Rust/Buck2/Prow validator",
        &mut failures,
        FEATURE_FLAG_SUBSTRATE_PATH,
    );
    require_contains(
        multi_region,
        "planned Rust/Buck2/Prow validator: multi-region-disposition",
        &mut failures,
        MULTI_REGION_DISPOSITION_PATH,
    );
    require_contains(
        multi_region,
        "planned Rust/Buck2/Prow validator: sovereign-tenant-pin",
        &mut failures,
        MULTI_REGION_DISPOSITION_PATH,
    );
    require_contains(
        microservice_migration,
        "future Rust/Buck2/Prow migration job",
        &mut failures,
        MICROSERVICE_MIGRATION_TOOLING_PATH,
    );
    require_contains(
        retired_vocabulary,
        "retired CLI remains tombstone/provenance only",
        &mut failures,
        RETIRED_VOCABULARY_PATH,
    );
    require_contains(
        docs_pipeline,
        "documentation capability: openapi",
        &mut failures,
        DOCS_PIPELINE_REGISTRY_PATH,
    );
    require_contains(
        documentation_system_kernel,
        "documentation capability: ",
        &mut failures,
        DOCUMENTATION_SYSTEM_KERNEL_PATH,
    );

    for (label, text, needle) in [
        (
            WORKSPACE_HYGIENE_SPEC_PATH,
            workspace_hygiene,
            "oya gate validate workspace-hygiene",
        ),
        (
            FEATURE_FLAG_SUBSTRATE_PATH,
            feature_flag,
            "oya gate validate feature-flag-lifecycle",
        ),
        (
            MULTI_REGION_DISPOSITION_PATH,
            multi_region,
            "oya gate validate multi-region-disposition",
        ),
        (
            MULTI_REGION_DISPOSITION_PATH,
            multi_region,
            "oya gate validate sovereign-tenant-pin",
        ),
        (
            MICROSERVICE_MIGRATION_TOOLING_PATH,
            microservice_migration,
            "oya dev migrate-microservice --rollback",
        ),
        (
            RETIRED_VOCABULARY_PATH,
            retired_vocabulary,
            "do not revive oya vcs",
        ),
        (DOCS_PIPELINE_REGISTRY_PATH, docs_pipeline, "\toya doc "),
        (
            DOCUMENTATION_SYSTEM_KERNEL_PATH,
            documentation_system_kernel,
            "documented_command must name an oya doc subcommand",
        ),
    ] {
        require_absent(text, needle, &mut failures, label);
    }

    failures
}

pub fn retired_compatibility_catalog_failures(
    gate_catalog_domain: &str,
    quality_lane_kernel: &str,
) -> Vec<String> {
    let mut failures = Vec::new();

    for needle in [
        "pub const CATALOG_STATUS: &str = \"retired_compatibility_catalog\";",
        "historical compatibility only; not CI/merge authority",
        "not CI, SCM, merge, or governance authority",
    ] {
        require_contains(
            gate_catalog_domain,
            needle,
            &mut failures,
            GATE_CATALOG_DOMAIN_PATH,
        );
    }

    for needle in [
        "retired compatibility wired-commands corpus",
        "compatibility/provenance only",
    ] {
        require_contains(
            quality_lane_kernel,
            needle,
            &mut failures,
            QUALITY_LANE_KERNEL_PATH,
        );
    }

    for (label, text, needle) in [
        (
            GATE_CATALOG_DOMAIN_PATH,
            gate_catalog_domain,
            "Foundry gate-catalog canonical domain",
        ),
        (
            GATE_CATALOG_DOMAIN_PATH,
            gate_catalog_domain,
            "single source of truth",
        ),
        (
            GATE_CATALOG_DOMAIN_PATH,
            gate_catalog_domain,
            "required merge substrate",
        ),
        (
            GATE_CATALOG_DOMAIN_PATH,
            gate_catalog_domain,
            "governance now rides plain git plus oya gate/verify",
        ),
        (
            QUALITY_LANE_KERNEL_PATH,
            quality_lane_kernel,
            "canonical wired-commands catalog",
        ),
    ] {
        require_absent(text, needle, &mut failures, label);
    }

    failures
}

pub fn active_foundry_shared_surface_failures(
    readme: &str,
    root_hub: &str,
    sequencing: &str,
    doc_agents: &str,
) -> Vec<String> {
    let mut failures = Vec::new();

    for (label, text, required) in [
        (
            README_PATH,
            readme,
            "SaaS, Workspace, Vertical, Intelligence, Cloud",
        ),
        (
            ROOT_HUB_PATH,
            root_hub,
            "\"owner_team\": \"council-architecture + platform-governance\"",
        ),
        (
            SEQUENCING_PATH,
            sequencing,
            "\"owner_team\": \"council-architecture + platform-governance\"",
        ),
        (
            DOC_AGENTS_PATH,
            doc_agents,
            "intelligence/governance capabilities",
        ),
        (
            DOC_AGENTS_PATH,
            doc_agents,
            "Capability records + metering events consumed by capability runtimes.",
        ),
    ] {
        require_contains(text, required, &mut failures, label);
    }

    for (label, text, forbidden) in [
        (
            README_PATH,
            readme,
            "SaaS, Workspace, Vertical, Foundry, Cloud",
        ),
        (
            ROOT_HUB_PATH,
            root_hub,
            "\"owner_team\": \"council-architecture + axis-foundry\"",
        ),
        (
            SEQUENCING_PATH,
            sequencing,
            "\"owner_team\": \"council-architecture + axis-foundry\"",
        ),
        (DOC_AGENTS_PATH, doc_agents, "Foundry capabilities"),
        (DOC_AGENTS_PATH, doc_agents, "Foundry-consumed"),
    ] {
        require_absent(text, forbidden, &mut failures, label);
    }

    failures
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
            "\"self_explanatory_active_name_rule\"",
            "documentation sprawl hygiene must record the active self-explanatory naming rule",
        ),
        (
            "context.doctrine.buck2_prow_ci_authority",
            "repo hygiene spec must include the capability-named context example",
        ),
        (
            "\"seed_check\": \"buck2 build //:oya-ci-prowjob-registry-check\"",
            "ProwJob registry seed check must be recorded in cleanup backlog",
        ),
        (
            "\"schema_check\": \"buck2 build //:oya-ci-controller-config-check\"",
            "ProwJob registry cleanup backlog must record controller config schema check",
        ),
        (
            "\"seed_registry\": \"specs/oya-ci-prowjob-registry.json\"",
            "ProwJob registry seed registry path must be recorded in cleanup backlog",
        ),
        (
            "\"controller_config_contract\": \"specs/oya-ci-controller-config-contract.json\"",
            "ProwJob registry cleanup backlog must record controller config contract path",
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
            "\"contract\": \"specs/kubernetes-native-anti-patterns.json\"",
            "anti-pattern guardrails must point at disjoint Kubernetes-native anti-pattern contract",
        ),
        (
            "\"check\": \"buck2 build //:kubernetes-native-anti-pattern-check\"",
            "anti-pattern guardrails must publish Kubernetes-native anti-pattern Buck2 check",
        ),
        (
            "\"controller_reconciliation_over_manual_mutation\"",
            "anti-pattern guardrails must require controller reconciliation over manual mutation",
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
    for (needle, message) in REQUIRED_RUST_TOOLCHAIN_POLICY_NEEDLES {
        require_contains(spec, needle, &mut failures, message);
    }
    for (needle, message) in REQUIRED_DEPENDENCY_REGISTRY_POLICY_NEEDLES {
        require_contains(spec, needle, &mut failures, message);
    }
    for (needle, message) in REQUIRED_BUCK2_RELEASE_POLICY_NEEDLES {
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
        KUBERNETES_NATIVE_ANTI_PATTERN_COMMAND,
        CLOUD_CELL_ELASTICITY_POLICY_COMMAND,
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

pub fn tenant_rbac_packaging_failures(packaging: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for command in [
        RETIRED_GROUPING_WORDING_COMMAND,
        NO_GROUPING_KERNEL_CHECK_COMMAND,
    ] {
        require(
            contains_json_string(packaging, command),
            &mut failures,
            format!("tenant RBAC packaging static_gates missing {command}"),
        );
    }
    let retired_shell_gate = ["scripts/", "reject-retired-grouping-wording.sh ."].concat();
    require(
        !contains_json_string(packaging, &retired_shell_gate),
        &mut failures,
        "tenant RBAC packaging must not keep retired shell grouping-wording gate",
    );
    let retired_local_runner = ["cargo", " test -p oya-check-no-grouping"].concat();
    require(
        !contains_json_string(packaging, &retired_local_runner),
        &mut failures,
        "tenant RBAC packaging must use Buck2 no-grouping kernel check, not a retired local runner",
    );
    failures
}

pub fn rust_toolchain_policy_failures(
    rust_toolchain: &str,
    cargo_toml: &str,
    buck: &str,
    github_bridge: &str,
    buck2_policy: &str,
    code_style: &str,
    lts_versions: &str,
    dependency_policy: &str,
    observability_slo: &str,
) -> Vec<String> {
    let mut failures = Vec::new();
    let rust_channel = format!("channel = \"{REQUIRED_RUST_STABLE_VERSION}\"");
    let rust_version = format!("rust-version = \"{REQUIRED_RUST_STABLE_VERSION}\"");
    let edition = format!("edition = \"{REQUIRED_RUST_EDITION}\"");

    require_contains(
        rust_toolchain,
        &rust_channel,
        &mut failures,
        RUST_TOOLCHAIN_PATH,
    );
    require_contains(cargo_toml, &rust_version, &mut failures, CARGO_TOML_PATH);
    require_contains(cargo_toml, &edition, &mut failures, CARGO_TOML_PATH);
    require_contains(
        buck,
        "--edition=2024",
        &mut failures,
        "BUCK Rust compilation commands",
    );
    require(
        !buck.contains("--edition=2021"),
        &mut failures,
        "BUCK must not compile Rust checks with edition 2021",
    );
    require_contains(
        github_bridge,
        "\"rust_toolchain\": \"1.96.0\"",
        &mut failures,
        GITHUB_BRIDGE_PATH,
    );
    require_contains(
        buck2_policy,
        "1.96.0",
        &mut failures,
        BUCK2_AUTHORITY_POLICY_PATH,
    );
    for (label, text) in [
        (CODE_STYLE_RUST_PATH, code_style),
        (LTS_VERSIONS_VERIFIED_PATH, lts_versions),
        (DEPENDENCY_POLICY_PATH, dependency_policy),
        (OBSERVABILITY_SLO_PATH, observability_slo),
    ] {
        require_contains(text, "1.96.0", &mut failures, label);
        require_contains(text, "2024", &mut failures, label);
    }
    require_contains(
        lts_versions,
        "current Rust stable channel as soon as the official Rust release is published",
        &mut failures,
        LTS_VERSIONS_VERIFIED_PATH,
    );
    require_contains(
        dependency_policy,
        "immediately for Rust and Buck2 release",
        &mut failures,
        DEPENDENCY_POLICY_PATH,
    );
    require_contains(
        observability_slo,
        "bump immediately through the Rust/Buck2 toolchain updater lane",
        &mut failures,
        OBSERVABILITY_SLO_PATH,
    );

    failures
}

pub fn dependency_registry_policy_failures(
    cargo_toml: &str,
    dependency_rationales: &str,
    blessed_allowlist: &str,
    dependency_policy: &str,
    masterplan: &str,
) -> Vec<String> {
    let mut failures = Vec::new();
    let workspace_deps = workspace_dependency_names(cargo_toml);
    let rationale_keys = json_object_keys(dependency_rationales, "entries");
    let blessed_keys = json_object_keys(blessed_allowlist, "blessed");

    for dependency in &workspace_deps {
        require(
            rationale_keys.contains(dependency),
            &mut failures,
            format!("{DEPENDENCY_RATIONALES_PATH}: missing workspace dependency {dependency}"),
        );
        require(
            blessed_keys.contains(dependency),
            &mut failures,
            format!(
                "{DEPENDENCY_BLESSED_ALLOWLIST_PATH}: missing workspace dependency {dependency}"
            ),
        );
    }
    for (label, text) in [
        (DEPENDENCY_RATIONALES_PATH, dependency_rationales),
        (DEPENDENCY_BLESSED_ALLOWLIST_PATH, blessed_allowlist),
    ] {
        for needle in [
            "latest_upstream_stable_or_lts",
            "all_workspace_dependencies_tracked",
            "explicit_waiver_required_for_non_latest_or_non_in_house_dependency",
            "in_house_first_oya_rust_libraries",
        ] {
            require_contains(text, needle, &mut failures, label);
        }
    }
    for (label, text) in [
        (DEPENDENCY_POLICY_PATH, dependency_policy),
        (MASTERPLAN_PATH, masterplan),
    ] {
        require_contains(
            text,
            "in-house",
            &mut failures,
            &format!("{label} in-house dependency posture"),
        );
        require_contains(
            text,
            "latest",
            &mut failures,
            &format!("{label} latest dependency posture"),
        );
        require_contains(
            text,
            "registry/dependency-rationales.json",
            &mut failures,
            &format!("{label} dependency rationales pointer"),
        );
    }

    failures
}

pub fn buck2_release_policy_failures(
    spec: &str,
    workflow: &str,
    bootstrap: &str,
    toolchain_updater: &str,
    buck: &str,
) -> Vec<String> {
    let mut failures = Vec::new();
    require_contains(
        spec,
        &format!("\"required_buck2_release\": \"{REQUIRED_BUCK2_RELEASE}\""),
        &mut failures,
        SPEC_PATH,
    );
    require_contains(
        workflow,
        &format!("BUCK2_RELEASE: \"{REQUIRED_BUCK2_RELEASE}\""),
        &mut failures,
        WORKFLOW_PATH,
    );
    require_contains(
        bootstrap,
        &format!("BUCK2_RELEASE:={REQUIRED_BUCK2_RELEASE}"),
        &mut failures,
        GITHUB_ACTIONS_BOOTSTRAP_PATH,
    );
    require_contains(
        toolchain_updater,
        "https://github.com/facebook/buck2.git",
        &mut failures,
        TOOLCHAIN_PIN_UPDATER_PATH,
    );
    require_contains(
        buck,
        "latest-toolchain-pin-updater-check",
        &mut failures,
        BUCK_PATH,
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
    let agent_operating_contract = read(root, AGENT_OPERATING_CONTRACT_PATH, &mut failures);
    let tenant_rbac_packaging = read(root, TENANT_RBAC_PACKAGING_PATH, &mut failures);
    let masterplan = read(root, MASTERPLAN_PATH, &mut failures);
    let sequencing = read(root, SEQUENCING_PATH, &mut failures);
    let doc_masterplan = read(root, DOC_MASTERPLAN_PATH, &mut failures);
    let planning_contract = read(root, PLANNING_CLOSURE_CONTRACT_PATH, &mut failures);
    let planning_ledger = read(root, PLANNING_CLOSURE_LEDGER_PATH, &mut failures);
    let readme = read(root, README_PATH, &mut failures);
    let agents = read(root, AGENTS_PATH, &mut failures);
    let claude = read(root, CLAUDE_PATH, &mut failures);
    let doc_agents = read(root, DOC_AGENTS_PATH, &mut failures);
    let tools_agent_skills_agents = read(root, TOOLS_AGENT_SKILLS_AGENTS_PATH, &mut failures);
    let doc_catalog = read(root, DOC_CATALOG_PATH, &mut failures);
    let procedure = read(root, PROCEDURE_PATH, &mut failures);
    let openbao_eso_runbook = read(root, OPENBAO_ESO_RUNBOOK_PATH, &mut failures);
    let workflow = read(root, WORKFLOW_PATH, &mut failures);
    let buck = read(root, BUCK_PATH, &mut failures);
    let cargo_toml = read(root, CARGO_TOML_PATH, &mut failures);
    let rust_toolchain = read(root, RUST_TOOLCHAIN_PATH, &mut failures);
    let buck2_policy = read(root, BUCK2_AUTHORITY_POLICY_PATH, &mut failures);
    let dependency_rationales = read(root, DEPENDENCY_RATIONALES_PATH, &mut failures);
    let blessed_allowlist = read(root, DEPENDENCY_BLESSED_ALLOWLIST_PATH, &mut failures);
    let typescript_pnpm_inventory =
        read(root, TYPESCRIPT_PNPM_SURFACE_INVENTORY_PATH, &mut failures);
    let python_shell_inventory = read(root, PYTHON_SHELL_SURFACE_INVENTORY_PATH, &mut failures);
    let code_style = read(root, CODE_STYLE_RUST_PATH, &mut failures);
    let dependency_policy = read(root, DEPENDENCY_POLICY_PATH, &mut failures);
    let lts_versions = read(root, LTS_VERSIONS_VERIFIED_PATH, &mut failures);
    let observability_slo = read(root, OBSERVABILITY_SLO_PATH, &mut failures);
    let github_actions_bootstrap = read(root, GITHUB_ACTIONS_BOOTSTRAP_PATH, &mut failures);
    let toolchain_updater = read(root, TOOLCHAIN_PIN_UPDATER_PATH, &mut failures);
    let workspace_hygiene = read(root, WORKSPACE_HYGIENE_SPEC_PATH, &mut failures);
    let feature_flag = read(root, FEATURE_FLAG_SUBSTRATE_PATH, &mut failures);
    let multi_region = read(root, MULTI_REGION_DISPOSITION_PATH, &mut failures);
    let microservice_migration = read(root, MICROSERVICE_MIGRATION_TOOLING_PATH, &mut failures);
    let retired_vocabulary = read(root, RETIRED_VOCABULARY_PATH, &mut failures);
    let docs_pipeline = read(root, DOCS_PIPELINE_REGISTRY_PATH, &mut failures);
    let documentation_system_kernel = read(root, DOCUMENTATION_SYSTEM_KERNEL_PATH, &mut failures);
    let gate_catalog_domain = read(root, GATE_CATALOG_DOMAIN_PATH, &mut failures);
    let quality_lane_kernel = read(root, QUALITY_LANE_KERNEL_PATH, &mut failures);

    failures.extend(spec_failures(&spec));
    failures.extend(retired_cli_registry_spec_failures(
        &workspace_hygiene,
        &feature_flag,
        &multi_region,
        &microservice_migration,
        &retired_vocabulary,
        &docs_pipeline,
        &documentation_system_kernel,
    ));
    failures.extend(retired_compatibility_catalog_failures(
        &gate_catalog_domain,
        &quality_lane_kernel,
    ));
    failures.extend(active_foundry_shared_surface_failures(
        &readme,
        &root_hub,
        &sequencing,
        &doc_agents,
    ));
    failures.extend(tenant_rbac_packaging_failures(&tenant_rbac_packaging));
    failures.extend(rust_toolchain_policy_failures(
        &rust_toolchain,
        &cargo_toml,
        &buck,
        &github_bridge,
        &buck2_policy,
        &code_style,
        &lts_versions,
        &dependency_policy,
        &observability_slo,
    ));
    failures.extend(dependency_registry_policy_failures(
        &cargo_toml,
        &dependency_rationales,
        &blessed_allowlist,
        &dependency_policy,
        &masterplan,
    ));
    failures.extend(buck2_release_policy_failures(
        &spec,
        &workflow,
        &github_actions_bootstrap,
        &toolchain_updater,
        &buck,
    ));
    evaluate_root_markdown(root, &mut failures);
    failures.extend(retired_root_file_failures(root));
    failures.extend(retired_service_ci_entrypoint_failures(root));
    failures.extend(retired_active_path_failures(root));
    let (tracked_typescript_pnpm_mjs_count, typescript_pnpm_surface_failures) =
        typescript_pnpm_surface_failures(root, &typescript_pnpm_inventory);
    failures.extend(typescript_pnpm_surface_failures);
    let (tracked_nonvendored_python_shell_count, python_shell_surface_failures) =
        python_shell_surface_failures(root, &python_shell_inventory);
    failures.extend(python_shell_surface_failures);
    failures.extend(active_policy_context_name_failures(root));
    failures.extend(runbook_promotion_gate_failures(root));
    failures.extend(cloud_network_dns_authority_failures(root));
    failures.extend(cloud_network_authority_failures(root));
    failures.extend(workplace_integration_authority_failures(root));
    failures.extend(cell_lifecycle_authority_failures(root));
    failures.extend(cloud_billing_tax_authority_failures(root));
    failures.extend(cloud_iam_authority_failures(root));

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
    for needle in [
        "\"policy_compliance_productization\"",
        "policy_as_code_service_productization",
        "compliance_as_code_service_productization",
        "policy_as_a_service_productization",
        "platform_as_a_service_productization",
        "containers_as_a_service_productization",
        "compliance_as_a_service_productization",
        "oyatie_dogfood_tenant",
        "Adopt proven hyperscaler patterns first",
    ] {
        require_contains(&masterplan, needle, &mut failures, MASTERPLAN_PATH);
    }
    for needle in [
        "Policy/Compliance-as-a-Service productization",
        "PaC, CaC, PBAC",
        "Containers-as-a-Service",
        "Compliance-as-a-Service",
        "Oyatie-as-tenant dogfood",
        "The rule is to adopt proven",
        "reimplement in Rust/Oyatie-native seams only",
    ] {
        require_contains(&doc_masterplan, needle, &mut failures, DOC_MASTERPLAN_PATH);
    }
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
        (README_PATH, readme.as_str()),
        (AGENTS_PATH, agents.as_str()),
        (CLAUDE_PATH, claude.as_str()),
        (DOC_AGENTS_PATH, doc_agents.as_str()),
    ] {
        require_contains(
            text,
            KUBERNETES_NATIVE_ANTI_PATTERN_COMMAND,
            &mut failures,
            label,
        );
    }

    for needle in [
        "\"git\"",
        "\"gh\"",
        "\"buck2\"",
        "Prow/Kubernetes-native oya-ci-required",
        "Buck2/Prow PR evidence or explicit blocker",
    ] {
        require_contains(
            &agent_operating_contract,
            needle,
            &mut failures,
            AGENT_OPERATING_CONTRACT_PATH,
        );
    }

    for needle in [
        "Legacy adapter-secret compatibility note",
        "not SCM/CI authority",
        "retired_external_scm_adapter_retirement",
    ] {
        require_contains(
            &openbao_eso_runbook,
            needle,
            &mut failures,
            OPENBAO_ESO_RUNBOOK_PATH,
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
            KUBERNETES_NATIVE_ANTI_PATTERN_COMMAND,
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

    for needle in [
        "root `CLAUDE.md`,\n`docs/AGENTS.md`, and `specs/root-hub-pointers.json` overlay",
        "Build/test/check authority is Buck2 build/test/check targets",
        "Prow/Kubernetes-native `oya-ci-required` context",
        "Coordination uses plain `git`, isolated worktrees, PRs against `dev`",
        "GitHub flows are compatibility/shadow publication",
        "CUE packages own first-party Kubernetes desired state",
        "Helm is adapter compatibility only",
    ] {
        require_contains(
            &tools_agent_skills_agents,
            needle,
            &mut failures,
            TOOLS_AGENT_SKILLS_AGENTS_PATH,
        );
    }

    for rel in ACTIVE_CONTEXT_SCAN_PATHS {
        let text = read(root, rel, &mut failures);
        failures.extend(active_doc_phrase_failures(rel, &text));
    }

    for rel in ACTIVE_TEMPLATE_SCAN_PATHS {
        let text = read(root, rel, &mut failures);
        failures.extend(active_doc_phrase_failures(rel, &text));
        failures.extend(active_template_phrase_failures(rel, &text));
    }

    for rel in ACTIVE_EXACT_NAME_SCAN_PATHS {
        let text = read(root, rel, &mut failures);
        failures.extend(retired_exact_name_failures(rel, &text));
    }

    for needle in [
        "repo-hygiene-automation-check",
        "latest-toolchain-pin-updater-check",
        "oya-ci-prowjob-registry-check",
        "oya-ci-controller-config-check",
        "kubernetes-native-anti-pattern-check",
        "quality-lane-registry-authority-check",
        "assert-repo-hygiene-automation.rs",
        "sync-latest-toolchain-pins.rs",
        "generate-oya-ci-prowjob-registry.rs",
        "assert-oya-ci-controller-config.rs",
        "assert-kubernetes-native-antipatterns.rs",
        "assert-quality-lane-registry-authority.rs",
        "repo_hygiene_automation_check.rs",
        "oya_ci_prowjob_registry_check.rs",
        "oya_ci_controller_config_check.rs",
        "kubernetes_native_antipatterns_check.rs",
        "quality_lane_registry_authority_check.rs",
        "repo-hygiene-automation.json",
        "dependency-rationales.json",
        "dependency-blessed-allowlist.json",
        "typescript-pnpm-surface-inventory.json",
        "python-shell-surface-inventory.json",
        "oya-ci-prowjob-registry.json",
        "oya-ci-controller-config-contract.json",
        "kubernetes-native-anti-patterns.json",
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
        tracked_typescript_pnpm_mjs_count,
        tracked_nonvendored_python_shell_count,
        active_context_scan_files: ACTIVE_CONTEXT_SCAN_PATHS.len(),
        active_template_scan_files: ACTIVE_TEMPLATE_SCAN_PATHS.len(),
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
        "{{\"verdict\":\"{}\",\"spec\":\"{}\",\"local_static_only\":true,\"live_mutation_performed\":false,\"domains_checked\":{},\"security_hardening_backlog_count\":{},\"tracked_typescript_pnpm_mjs_count\":{},\"tracked_nonvendored_python_shell_count\":{},\"active_context_scan_files\":{},\"active_template_scan_files\":{},\"retired_exact_name_scan_files\":{},\"stale_doc_inventory_command\":\"{}\",\"stale_doc_inventory_test_command\":\"{}\",\"checker_language\":\"rust\",\"failures\":[{}]}}",
        evaluation.verdict,
        SPEC_PATH,
        evaluation.domains_checked,
        evaluation.security_backlog_count,
        evaluation.tracked_typescript_pnpm_mjs_count,
        evaluation.tracked_nonvendored_python_shell_count,
        evaluation.active_context_scan_files,
        evaluation.active_template_scan_files,
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
