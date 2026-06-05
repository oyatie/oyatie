//! P00 GitHub/GitHub Actions lane-unlocker bridge gate.
//!
//! This is the Rust/Buck2 replacement for the retired Python bridge checker.
//! It is deliberately local/static evidence only: it validates checked-in bridge
//! contracts, workflow wiring, branch-protection shadows, and docs without
//! mutating GitHub, Kubernetes, branch protection, or deployment state.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SPEC_PATH: &str = "specs/github-lane-unlocker-bridge.json";
const WORKFLOW_PATH: &str = ".github/workflows/github-lane-unlocker-ci-cd.yml";
const BOOTSTRAP_PATH: &str = "scripts/ci/github-actions-lane-unlocker-bootstrap.sh";
const RUST_TOOLCHAIN_PATH: &str = "rust-toolchain.toml";
const BRANCH_PROTECTION_JSON: &str = "infra/branch-protection/dev.json";
const BRANCH_PROTECTION_YAML: &str = ".github/branch-protection.yaml";
const ROOT_HUB_PATH: &str = "specs/root-hub-pointers.json";
const BUCK2_POLICY_PATH: &str = "specs/buck2-authority-policy.json";
const ADR_PATH: &str = "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md";
const PROCEDURE_PATH: &str = "docs/ci/github-actions-lane-unlocker.md";
const BUCK_PATH: &str = "BUCK";
const REPO_HYGIENE_CHECKER_PATH: &str = "scripts/ci/assert-repo-hygiene-automation.rs";
const REPO_HYGIENE_SPEC_PATH: &str = "specs/repo-hygiene-automation.json";
const MASTERPLAN_PATH: &str = "specs/masterplan.json";

const RETIRED_PYTHON_BRIDGE_COMMAND: &str =
    "python3 scripts/ci/assert-github-lane-unlocker-bridge.py --json";
const RETIRED_PYTHON_BRIDGE_PATH: &str = "scripts/ci/assert-github-lane-unlocker-bridge.py";
const BRIDGE_BUCK2_COMMAND: &str = "buck2 build //:github-lane-unlocker-bridge-check";
const THIRD_PARTY_HAND_EDITS_BUCK2_COMMAND: &str =
    "buck2 build //:third-party-durable-handedits-check";
const LEGACY_SHADOW_CONTEXT: &str = "github-lane-unlocker-required";
const REQUIRED_NATIVE_CONTEXT: &str = "oya-ci-required";

const REQUIRED_NATIVE_SEAMS: &[&str] = &[
    "oyatie_scm",
    "cloud_workspace_service",
    "rust_prow_oya_ci",
    "buck2_execution",
    "llvm_source_based_coverage",
    "release_conveyor_cd",
];

const REQUIRED_PATTERN_SOURCES: &[&str] = &[
    "prow",
    "sapling",
    "piper",
    "citc",
    "github_actions",
    "buck2",
    "kubernetes",
];

const REQUIRED_ADOPTED_PATTERNS: &[&str] = &[
    "disjoint_merge_pools",
    "stacked_changes",
    "cloud_workspaces",
    "required_status_rollup",
    "affected_builds",
    "kubernetes_native_job_execution",
    "source_based_coverage",
];

const REQUIRED_ALTERNATIVES: &[&str] = &[
    "github_only_permanent",
    "upstream_prow_as_is",
    "sapling_fork_without_rust_boundary",
    "build_from_scratch_ignoring_proven_patterns",
];

const REQUIRED_FORBIDDEN_CLAIMS: &[&str] = &[
    "GitHub is the permanent SCM",
    "GitHub Actions is the permanent CI authority",
    "GitHub Actions is the permanent CD authority",
    "retired external SCM/CI/CD substrates are interim authorities",
    "P0.0 green",
    "Phase 0 complete",
    "cloud-ci/oya-ci live authority proven",
];

const REQUIRED_OFFICIAL_SOURCE_URLS: &[&str] = &[
    "https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax",
    "https://docs.github.com/en/actions/reference/github-hosted-runners-reference",
    "https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/run-job-variations",
    "https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency",
    "https://docs.github.com/en/actions/reference/workflows-and-actions/reusing-workflow-configurations",
    "https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions",
    "https://docs.github.com/en/actions/security-for-github-actions/security-guides/automatic-token-authentication",
    "https://github.com/actions/checkout",
    "https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches",
    "https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/merging-a-pull-request-with-a-merge-queue",
    "https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/automatically-merging-a-pull-request",
    "https://docs.prow.k8s.io/docs/overview/",
    "https://buck2.build/docs/users/commands/build/",
    "https://doc.rust-lang.org/rustc/instrument-coverage.html",
    "https://sapling-scm.com/docs/introduction/",
    "https://sapling-scm.com/docs/scale/overview/",
    "https://cacm.acm.org/research/why-google-stores-billions-of-lines-of-code-in-a-single-repository/",
    "https://kubernetes.io/docs/tasks/run-application/scale-deployment/",
    "https://github.com/cncf/toc/blob/main/DEFINITION.md",
    "https://architecture.cncf.io/",
    "https://github.com/actions/checkout/releases/tag/v6.0.3",
    "https://nodejs.org/en/about/previous-releases",
];

const REQUIRED_WORKFLOW_NEEDLES: &[&str] = &[
    "name: github-lane-unlocker-ci-cd",
    "pull_request:",
    "branches: [dev]",
    "push:",
    "permissions:",
    "contents: read",
    "pull-requests: read",
    "BUCK2_RELEASE: \"2026-06-01\"",
    "runs-on: ubuntu-24.04-arm",
    "Bootstrap Rust and Buck2 toolchains",
    "scripts/ci/github-actions-lane-unlocker-bootstrap.sh",
    "concurrency:",
    "github.event.pull_request.number || github.head_ref || github.run_id",
    "cancel-in-progress: true",
    "strategy:",
    "fail-fast: false",
    "max-parallel: 4",
    "matrix:",
    "lane: [governance, buck2-authority, rust-llvm-coverage, affected-build]",
    "name: github-lane-unlocker-required",
    "FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: \"true\"",
    "uses: actions/checkout@v6",
    "fetch-depth: 0",
    "persist-credentials: false",
    THIRD_PARTY_HAND_EDITS_BUCK2_COMMAND,
    "buck2 build //:github-lane-unlocker-bridge-check //:buck2-authority-policy-check",
    "buck2 build //:rust-llvm-coverage-runner-contract-check //:rust-llvm-coverage-smoke-check",
    "infra/ci/buck2-affected-gate.sh origin/dev HEAD",
    "name: github-lane-unlocker-cd-dry-run",
    BRIDGE_BUCK2_COMMAND,
];

const REQUIRED_BOOTSTRAP_NEEDLES: &[&str] = &[
    "Deterministic GitHub Actions bootstrap",
    "RUSTUP_CONCURRENT_DOWNLOADS",
    "rustup toolchain install",
    "llvm-tools-preview",
    "x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu",
    "rustup component add",
    "rustup target add",
    "llvm-profdata",
    "llvm-cov",
    "rustc --print=cfg --target=aarch64-unknown-linux-gnu",
    "https://github.com/facebook/buck2/releases/download/${BUCK2_RELEASE}/buck2-${buck2_arch}.zst",
    "sudo install -m 0755 /tmp/buck2 /usr/local/bin/buck2",
    "buck2 --version",
];

const REQUIRED_RUST_TOOLCHAIN_NEEDLES: &[&str] = &[
    "channel = \"1.96.0\"",
    "components = [\"rustfmt\", \"clippy\", \"llvm-tools-preview\"]",
    "targets = [\"x86_64-unknown-linux-gnu\", \"aarch64-unknown-linux-gnu\"]",
    "profile = \"minimal\"",
];

const REQUIRED_DOC_NEEDLES: &[&str] = &[
    "GitHub/GitHub Actions",
    "temporary lane-unlocker",
    "no retired external SCM/CI/CD substrates",
    "retired-external-substrate-registry.json",
    "pure-Rust Sapling-compatible native SCM",
    "best-of-existing hyperscaler patterns",
    "not a wholesale reimplementation",
    "cloud native",
    "Kubernetes-native",
    "hyperscaler native",
    "loosely coupled microservices",
    "secure, resilient, manageable, sustainable, and observable",
    "distributable, observable, portable, interoperable, and available",
    "Cloud auth/shared substrate and Oyatie product auth/shared substrate are decoupled now",
    "no shared contract or shared surface",
    "rewrite and rewire Oyatie products to consume the Cloud IdP",
    "Buck2",
    "LLVM source-based coverage",
    "not P0.0 green",
];

const BUCK2_POLICY_REQUIRED_FILES: &[&str] = &[
    ".github/workflows/github-lane-unlocker-ci-cd.yml",
    "scripts/ci/assert-github-lane-unlocker-bridge.rs",
    "scripts/tests/github_lane_unlocker_bridge_check.rs",
    "scripts/ci/assert-third-party-durable-handedits.rs",
    "scripts/tests/third_party_durable_handedits_check.rs",
    "scripts/ci/github-actions-lane-unlocker-bootstrap.sh",
    "rust-toolchain.toml",
    "specs/github-lane-unlocker-bridge.json",
    "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
    "docs/ci/github-actions-lane-unlocker.md",
    "infra/branch-protection/dev.json",
    ".github/branch-protection.yaml",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub failures: Vec<String>,
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

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&format!(
        "\"{}\":{}",
        key,
        if value { "true" } else { "false" }
    ))
}

fn has_string_value(text: &str, key: &str, value: &str) -> bool {
    compact_json_text(text).contains(&format!("\"{}\":\"{}\"", key, json_escape(value)))
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

pub fn spec_failures(spec: &str) -> Vec<String> {
    let mut failures = Vec::new();

    require(
        has_string_value(
            spec,
            "bridge_status",
            "github_actions_shadow_bridge_not_native_authority",
        ),
        &mut failures,
        "spec.bridge_status must be github_actions_shadow_bridge_not_native_authority",
    );

    for (key, value, message) in [
        ("temporary", true, "github_bridge.temporary must be true"),
        (
            "permanent_first_class",
            false,
            "github_bridge.permanent_first_class must be false",
        ),
        (
            "is_destination_scm",
            false,
            "github_bridge.is_destination_scm must be false",
        ),
        (
            "is_destination_ci",
            false,
            "github_bridge.is_destination_ci must be false",
        ),
        (
            "is_destination_cd",
            false,
            "github_bridge.is_destination_cd must be false",
        ),
        (
            "manual_oya_ci_required_bridge_allowed",
            false,
            "manual oya-ci-required bridge must be disabled during the GitHub shadow bridge",
        ),
        (
            "force_node24_runtime",
            true,
            "github_bridge.javascript_action_runtime.force_node24_runtime must be true",
        ),
        (
            "node26_action_runtime_used",
            false,
            "github_bridge.javascript_action_runtime.node26_action_runtime_used must remain false for JavaScript action runtime",
        ),
        (
            "unsecure_node20_opt_out_allowed",
            false,
            "github_bridge.javascript_action_runtime.unsecure_node20_opt_out_allowed must be false",
        ),
        (
            "write_permissions_allowed",
            false,
            "github_bridge.workflow_security.token_permissions.write_permissions_allowed must be false",
        ),
        (
            "broad_permissions_allowed",
            false,
            "github_bridge.workflow_security.token_permissions.broad_permissions_allowed must be false",
        ),
        (
            "id_token_write_allowed_before_oidc_lane",
            false,
            "github_bridge.workflow_security.token_permissions.id_token_write_allowed_before_oidc_lane must be false",
        ),
        (
            "checkout_persist_credentials",
            false,
            "github_bridge.workflow_security.checkout_persist_credentials must be false",
        ),
        (
            "long_lived_github_secrets_allowed",
            false,
            "github_bridge.workflow_security.long_lived_github_secrets_allowed must be false",
        ),
        (
            "live_deployments_enabled",
            false,
            "github_actions_cd_bridge.live_deployments_enabled must be false",
        ),
        (
            "python_cpp_in_durable_path",
            false,
            "native_destination_seams.oyatie_scm.python_cpp_in_durable_path must be false",
        ),
        (
            "wholesale_clone_or_reimplementation",
            false,
            "pattern_adoption_strategy.wholesale_clone_or_reimplementation must be false",
        ),
        (
            "cloud_native",
            true,
            "cloud_native_hyperscaler_fit.cloud_native must be true",
        ),
        (
            "kubernetes_native",
            true,
            "cloud_native_hyperscaler_fit.kubernetes_native must be true",
        ),
        (
            "hyperscaler_native",
            true,
            "cloud_native_hyperscaler_fit.hyperscaler_native must be true",
        ),
        (
            "parallel_lane_ready",
            true,
            "cloud_native_hyperscaler_fit.parallel_lane_ready must be true",
        ),
        (
            "loosely_coupled_microservices",
            true,
            "cloud_native_hyperscaler_fit.cncf_cloud_native.loose microservice coupling must be true",
        ),
        (
            "interoperate_secure_resilient_manageable_observable",
            true,
            "cloud_native_hyperscaler_fit.cncf_cloud_native must preserve CNCF interoperability properties",
        ),
        (
            "separation_of_concerns",
            true,
            "cloud_native_hyperscaler_fit.cncf_cloud_native.separation_of_concerns must be true",
        ),
        (
            "no_shared_contract_or_surface_now",
            true,
            "auth_shared_substrate_decoupling.no_shared_contract_or_surface_now must be true",
        ),
        (
            "higher_concurrency_expected",
            true,
            "auth_shared_substrate_decoupling.higher_concurrency_expected must be true",
        ),
        (
            "p0_0_green",
            false,
            "claim_boundary.p0_0_green must be false",
        ),
        (
            "phase0_complete",
            false,
            "claim_boundary.phase0_complete must be false",
        ),
        (
            "github_is_permanent",
            false,
            "claim_boundary.github_is_permanent must be false",
        ),
        (
            "native_ci_authority_proven",
            false,
            "claim_boundary.native_ci_authority_proven must remain false until live parity evidence is attached",
        ),
    ] {
        require(has_bool(spec, key, value), &mut failures, message);
    }

    for property_name in [
        "distributable",
        "observable",
        "portable",
        "interoperable",
        "available",
    ] {
        require(
            has_bool(spec, property_name, true),
            &mut failures,
            format!("cloud_native_hyperscaler_fit.cncf_cloud_native.{property_name} must be true"),
        );
    }

    for (key, value, message) in [
        (
            "interim_scm",
            "github_adapter",
            "github_bridge.interim_scm must be github_adapter",
        ),
        (
            "interim_ci",
            "github_actions_shadow",
            "github_bridge.interim_ci must be github_actions_shadow",
        ),
        (
            "interim_cd",
            "github_actions_dry_run_shadow",
            "github_bridge.interim_cd must be github_actions_dry_run_shadow",
        ),
        (
            "legacy_shadow_context",
            LEGACY_SHADOW_CONTEXT,
            "github_bridge.legacy_shadow_context must be github-lane-unlocker-required",
        ),
        (
            "required_native_context",
            REQUIRED_NATIVE_CONTEXT,
            "github_bridge.required_native_context must be oya-ci-required",
        ),
        (
            "workflow_path",
            WORKFLOW_PATH,
            "github_bridge.workflow_path must point to .github/workflows/github-lane-unlocker-ci-cd.yml",
        ),
        (
            "branch_protection_application",
            "target_dev_required_context_is_oya_ci_required_github_actions_shadow_only",
            "github_bridge.branch_protection_application must mark oya-ci-required as the target dev context",
        ),
        (
            "native_cutover_target_context",
            REQUIRED_NATIVE_CONTEXT,
            "github_bridge.native_cutover_target_context must remain oya-ci-required",
        ),
        (
            "checkout_action_ref",
            "actions/checkout@v6",
            "github_bridge.javascript_action_runtime.checkout_action_ref must use actions/checkout@v6",
        ),
        (
            "latest_checkout_release_verified",
            "v6.0.3",
            "github_bridge.javascript_action_runtime.latest_checkout_release_verified must record the verified v6 latest release",
        ),
        (
            "force_node24_env",
            "FORCE_JAVASCRIPT_ACTIONS_TO_NODE24",
            "github_bridge.javascript_action_runtime.force_node24_env must name the GitHub Actions Node24 opt-in",
        ),
        (
            "contents",
            "read",
            "github_bridge.workflow_security.token_permissions.contents must be read",
        ),
        (
            "pull_requests",
            "read",
            "github_bridge.workflow_security.token_permissions.pull_requests must be read",
        ),
        (
            "credential_persistence_policy",
            "checkout_persist_credentials_false_oidc_lane_required_for_future_cloud_credentials",
            "github_bridge.workflow_security.credential_persistence_policy must require checkout credential persistence to stay disabled",
        ),
        (
            "mode",
            "github_actions_cd_shadow_until_release_conveyor_cutover",
            "github_actions_cd_bridge.mode must be GitHub Actions CD shadow",
        ),
        (
            "cutover_destination",
            "release_conveyor_cd",
            "github_actions_cd_bridge.cutover_destination must be release_conveyor_cd",
        ),
        (
            "decision",
            "pure_rust_sapling_compatible_native_scm",
            "native_destination_seams.oyatie_scm.decision must be pure_rust_sapling_compatible_native_scm",
        ),
        (
            "implementation_strategy",
            "adopt_existing_hyperscaler_patterns_not_wholesale_reimplementation",
            "native_destination_seams.oyatie_scm must adopt existing patterns instead of reinventing the wheel",
        ),
        (
            "durable_language",
            "rust",
            "native_destination_seams.oyatie_scm.durable_language must be rust",
        ),
        (
            "upstream_sapling_role",
            "behavioral_reference_not_permanent_fork_authority",
            "native_destination_seams.oyatie_scm.upstream_sapling_role must be behavioral reference, not fork authority",
        ),
        (
            "pod_shutdown_policy",
            "scale_controllers_to_zero_not_delete_pods_blindly",
            "cloud_native_hyperscaler_fit must prefer controller scale-to-zero over blind pod deletion",
        ),
        (
            "status",
            "cloud_and_oyatie_auth_shared_substrates_decoupled_now",
            "auth_shared_substrate_decoupling.status must decouple cloud and Oyatie auth/shared substrates now",
        ),
        (
            "conflict_avoidance",
            "separate_contract_files_schemas_and_runtime_surfaces",
            "auth_shared_substrate_decoupling.conflict_avoidance must separate contract files, schemas, and runtime surfaces",
        ),
        (
            "future_integration",
            "rewrite_and_rewire_oyatie_products_to_consume_cloud_idp_after_cloud_substrate_stabilizes",
            "auth_shared_substrate_decoupling.future_integration must record later rewrite/rewire through Cloud IdP",
        ),
    ] {
        require(has_string_value(spec, key, value), &mut failures, message);
    }

    require(
        !has_string_value(spec, "legacy_shadow_context", REQUIRED_NATIVE_CONTEXT),
        &mut failures,
        "GitHub shadow bridge must not reuse destination oya-ci-required as its legacy_shadow_context",
    );

    require(
        has_bool(spec, "retired_external_scm_ci_cd_substrates", false),
        &mut failures,
        "not_interim_authorities.retired_external_scm_ci_cd_substrates must be false",
    );

    for seam in REQUIRED_NATIVE_SEAMS {
        require(
            contains_json_string(spec, seam),
            &mut failures,
            format!("native_destination_seams missing {seam}"),
        );
    }
    require(
        has_string_value(
            spec,
            "mode",
            "best_of_existing_systems_not_wholesale_reimplementation",
        ),
        &mut failures,
        "pattern_adoption_strategy.mode must reject wholesale reinvention",
    );

    for source in REQUIRED_PATTERN_SOURCES {
        require(
            contains_json_string(spec, source),
            &mut failures,
            format!("pattern_adoption_strategy.source_systems missing {source}"),
        );
    }
    for pattern in REQUIRED_ADOPTED_PATTERNS {
        require(
            contains_json_string(spec, pattern),
            &mut failures,
            format!("pattern_adoption_strategy.adopted_patterns missing {pattern}"),
        );
    }
    for alternative in REQUIRED_ALTERNATIVES {
        require(
            contains_json_string(spec, alternative),
            &mut failures,
            format!("alternatives_and_counterarguments missing {alternative}"),
        );
    }
    for claim in REQUIRED_FORBIDDEN_CLAIMS {
        require(
            contains_json_string(spec, claim),
            &mut failures,
            format!("claim_boundary.forbidden_claims missing {claim:?}"),
        );
    }
    for source in REQUIRED_OFFICIAL_SOURCE_URLS {
        require(
            contains_json_string(spec, source),
            &mut failures,
            format!("official_sources missing {source}"),
        );
    }

    for family in ["product", "cloud_platform", "scm_ci_cd", "governance"] {
        require(
            contains_json_string(spec, family),
            &mut failures,
            format!("lane_graph.families missing {family}"),
        );
    }
    for non_dependency in [
        "github temporary bridge internals",
        "retired external SCM bridge internals",
        "cloud-ci internals",
        "workspace substrate internals",
    ] {
        require(
            contains_json_string(spec, non_dependency),
            &mut failures,
            "lane_graph.product_lanes_must_not_depend_on must keep product lanes platform-agnostic",
        );
    }

    failures
}

pub fn workflow_failures(workflow: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for needle in REQUIRED_WORKFLOW_NEEDLES {
        require_contains(workflow, needle, &mut failures, "lane unlocker workflow");
    }
    let checkout_count = workflow.matches("uses: actions/checkout@v6").count();
    let persisted_credential_disabled_count =
        workflow.matches("persist-credentials: false").count();
    require(
        checkout_count > 0 && checkout_count == persisted_credential_disabled_count,
        &mut failures,
        "lane unlocker workflow must disable persisted checkout credentials for every checkout step",
    );
    require(
        !workflow_requests_broad_or_write_permissions(workflow),
        &mut failures,
        "lane unlocker workflow must not request broad or write token permissions",
    );
    require(
        !workflow.contains("secrets."),
        &mut failures,
        "lane unlocker workflow must not consume long-lived GitHub secrets",
    );
    require(
        !workflow.contains(RETIRED_PYTHON_BRIDGE_COMMAND),
        &mut failures,
        "lane unlocker workflow must not invoke retired Python bridge checker directly",
    );
    require(
        !workflow.contains("actions/checkout@v4"),
        &mut failures,
        "lane unlocker workflow must not use the legacy checkout v4 action",
    );
    require(
        !workflow.contains("ACTIONS_ALLOW_USE_UNSECURE_NODE_VERSION"),
        &mut failures,
        "lane unlocker workflow must not opt out to an unsecure JavaScript action runtime",
    );
    require(
        !workflow.to_ascii_lowercase().contains("cargo tarpaulin"),
        &mut failures,
        "lane unlocker workflow must not use Tarpaulin",
    );
    require(
        workflow.contains(LEGACY_SHADOW_CONTEXT),
        &mut failures,
        "lane unlocker workflow must expose the legacy shadow context",
    );
    require(
        !workflow.contains(REQUIRED_NATIVE_CONTEXT),
        &mut failures,
        "lane unlocker workflow must not impersonate oya-ci-required",
    );
    for forbidden in ["jenkins", "forgejo", "argocd"] {
        require(
            !workflow.to_ascii_lowercase().contains(forbidden),
            &mut failures,
            format!(
                "lane unlocker workflow must not invoke or describe {forbidden} as interim authority"
            ),
        );
    }
    require(
        workflow.matches(BOOTSTRAP_PATH).count() == 3,
        &mut failures,
        "lane unlocker workflow must bootstrap Rust and Buck2 in fanout, aggregator, and dry-run jobs",
    );
    require(
        workflow.matches("runs-on: ubuntu-24.04-arm").count() == 3,
        &mut failures,
        "lane unlocker workflow must use arm64 Ubuntu runners for the repo default aarch64 Buck2 Rust toolchain",
    );
    failures
}

fn workflow_requests_broad_or_write_permissions(workflow: &str) -> bool {
    workflow.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            return false;
        }
        matches!(trimmed, "permissions: write-all" | "permissions: read-all")
            || trimmed.ends_with(": write")
    })
}

pub fn evaluate(root: &Path) -> Evaluation {
    let mut failures = Vec::new();

    let spec = read(root, SPEC_PATH, &mut failures);
    let workflow = read(root, WORKFLOW_PATH, &mut failures);
    let bootstrap = read(root, BOOTSTRAP_PATH, &mut failures);
    let rust_toolchain = read(root, RUST_TOOLCHAIN_PATH, &mut failures);
    let branch_json = read(root, BRANCH_PROTECTION_JSON, &mut failures);
    let branch_yaml = read(root, BRANCH_PROTECTION_YAML, &mut failures);
    let root_hub = read(root, ROOT_HUB_PATH, &mut failures);
    let buck2_policy = read(root, BUCK2_POLICY_PATH, &mut failures);
    let adr = read(root, ADR_PATH, &mut failures);
    let procedure = read(root, PROCEDURE_PATH, &mut failures);
    let buck = read(root, BUCK_PATH, &mut failures);
    let repo_hygiene_checker = read(root, REPO_HYGIENE_CHECKER_PATH, &mut failures);
    let repo_hygiene_spec = read(root, REPO_HYGIENE_SPEC_PATH, &mut failures);
    let masterplan = read(root, MASTERPLAN_PATH, &mut failures);

    failures.extend(spec_failures(&spec));
    failures.extend(workflow_failures(&workflow));

    for needle in REQUIRED_BOOTSTRAP_NEEDLES {
        require_contains(&bootstrap, needle, &mut failures, "lane unlocker bootstrap");
    }
    for forbidden in ["jenkins", "forgejo", "argocd"] {
        require(
            !bootstrap.to_ascii_lowercase().contains(forbidden),
            &mut failures,
            format!(
                "lane unlocker bootstrap must not invoke or describe {forbidden} as interim authority"
            ),
        );
    }

    for needle in REQUIRED_RUST_TOOLCHAIN_NEEDLES {
        require_contains(&rust_toolchain, needle, &mut failures, RUST_TOOLCHAIN_PATH);
    }

    for (key, value, message) in [
        (
            "status",
            "github_actions_shadow_not_destination_authority",
            "infra/branch-protection/dev.json must declare GitHub Actions shadow status",
        ),
        (
            "required_context",
            REQUIRED_NATIVE_CONTEXT,
            "infra/branch-protection/dev.json target required context must be oya-ci-required",
        ),
        (
            "legacy_shadow_context",
            LEGACY_SHADOW_CONTEXT,
            "infra/branch-protection/dev.json legacy shadow context must be github-lane-unlocker-required",
        ),
        (
            "native_cutover_target_context",
            REQUIRED_NATIVE_CONTEXT,
            "infra/branch-protection/dev.json must preserve native cutover context",
        ),
    ] {
        require(
            has_string_value(&branch_json, key, value),
            &mut failures,
            message,
        );
    }
    require(
        compact_json_text(&branch_json).contains("\"contexts\":[\"oya-ci-required\"]"),
        &mut failures,
        "infra/branch-protection/dev.json must target oya-ci-required as the required check",
    );
    require(
        has_bool(&branch_json, "live_mutation_performed_by_this_file", false),
        &mut failures,
        "infra/branch-protection/dev.json must not claim live mutation",
    );
    require(
        has_bool(
            &branch_json,
            "retired_external_scm_ci_cd_substrates_interim",
            false,
        ),
        &mut failures,
        "infra/branch-protection/dev.json must reject retired external SCM/CI/CD substrates as interim",
    );

    for needle in [
        REQUIRED_NATIVE_CONTEXT,
        "Prow/Kubernetes-native oya-ci is CI authority",
        "github-lane-unlocker-required as shadow",
        "GitHub/GitHub Actions remains",
    ] {
        require_contains(&branch_yaml, needle, &mut failures, BRANCH_PROTECTION_YAML);
    }

    for needle in [
        "\"github_lane_unlocker_bridge\"",
        "\"current_path\": \"/specs/github-lane-unlocker-bridge.json\"",
    ] {
        require_contains(&root_hub, needle, &mut failures, ROOT_HUB_PATH);
    }

    for required in BUCK2_POLICY_REQUIRED_FILES {
        require(
            contains_json_string(&buck2_policy, required),
            &mut failures,
            format!("buck2 policy command_scan_files missing {required}"),
        );
    }
    require(
        !contains_json_string(&buck2_policy, RETIRED_PYTHON_BRIDGE_PATH),
        &mut failures,
        "buck2 policy must not scan retired Python bridge checker",
    );
    require(
        !buck2_policy.contains(RETIRED_PYTHON_BRIDGE_COMMAND),
        &mut failures,
        "buck2 policy must not require retired Python bridge command",
    );

    for (label, text) in [
        ("ADR-0516", adr.as_str()),
        ("GitHub lane unlocker procedure", procedure.as_str()),
    ] {
        for needle in REQUIRED_DOC_NEEDLES {
            require_contains(text, needle, &mut failures, label);
        }
        require_contains(text, BRIDGE_BUCK2_COMMAND, &mut failures, label);
        require(
            !text.contains(RETIRED_PYTHON_BRIDGE_COMMAND),
            &mut failures,
            format!("{label}: must not recommend retired Python bridge checker directly"),
        );
    }

    for needle in [
        "github-lane-unlocker-bridge-check",
        "assert-github-lane-unlocker-bridge.rs",
        "github_lane_unlocker_bridge_check.rs",
        "github-lane-unlocker-bridge.json",
    ] {
        require_contains(&buck, needle, &mut failures, BUCK_PATH);
    }
    require(
        !buck.contains(RETIRED_PYTHON_BRIDGE_PATH),
        &mut failures,
        "BUCK must not depend on retired Python bridge checker",
    );

    require(
        repo_hygiene_checker.contains(BRIDGE_BUCK2_COMMAND),
        &mut failures,
        "repo hygiene checker must require Buck2-owned GitHub bridge check",
    );
    require(
        !repo_hygiene_checker.contains(RETIRED_PYTHON_BRIDGE_COMMAND),
        &mut failures,
        "repo hygiene checker must not require retired Python bridge command",
    );
    require(
        contains_json_string(&repo_hygiene_spec, BRIDGE_BUCK2_COMMAND),
        &mut failures,
        "repo hygiene spec automation commands must include Buck2-owned GitHub bridge check",
    );
    require(
        !repo_hygiene_spec.contains(RETIRED_PYTHON_BRIDGE_COMMAND),
        &mut failures,
        "repo hygiene spec automation commands must not include retired Python bridge command",
    );
    require_contains(
        &masterplan,
        BRIDGE_BUCK2_COMMAND,
        &mut failures,
        MASTERPLAN_PATH,
    );
    require(
        !masterplan.contains(RETIRED_PYTHON_BRIDGE_COMMAND),
        &mut failures,
        "masterplan must not recommend retired Python bridge checker directly",
    );

    Evaluation {
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned(),
        failures,
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
        "{{\"verdict\":\"{}\",\"spec\":\"{}\",\"workflow\":\"{}\",\"legacy_shadow_context\":\"{}\",\"required_native_context\":\"{}\",\"p0_0_green\":false,\"phase0_complete\":false,\"local_static_only\":true,\"live_mutation_performed\":false,\"checker_language\":\"rust\",\"failures\":[{}]}}",
        evaluation.verdict,
        SPEC_PATH,
        WORKFLOW_PATH,
        LEGACY_SHADOW_CONTEXT,
        REQUIRED_NATIVE_CONTEXT,
        failures
    )
}

fn config() -> (PathBuf, bool) {
    let mut json = false;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--json" => json = true,
            unknown => {
                eprintln!("assert-github-lane-unlocker-bridge: unknown argument {unknown}");
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
            eprintln!("github-lane-unlocker-bridge: RED");
            for failure in &evaluation.failures {
                eprintln!("- {failure}");
            }
        }
        std::process::exit(1);
    }
}
