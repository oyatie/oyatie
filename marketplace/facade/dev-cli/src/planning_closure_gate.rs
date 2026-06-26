//! `oya gate validate planning-closure` runner.
//!
//! This gate blocks implementation-start claims until the machine-readable
//! master plan, planning-closure contract, sequencing sidecar, root hub, and
//! vertical-order ADR all agree on the first production deliverable.

use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::usage;

const EXPECTED_SPEC_ID: &str = "EXE-PLANNING-CLOSURE-CONTRACT";
const EXPECTED_STATUS: &str = "Accepted";
const EXPECTED_CONTRACT_REF: &str = "/specs/planning-closure-contract.json";
const EXPECTED_STATUS_LEDGER_REF: &str = "/specs/planning-closure-status-closure-ledger.json";
const EXPECTED_MASTERPLAN_PATH: &str = "/specs/masterplan.json";
const EXPECTED_GATE_COMMAND: &str = "oya-ci-required cloud-ci planning-closure Rust gate packet";
const RETIRED_PLANNING_CLOSURE_COMMAND: &str =
    "cargo run -q -p oya-dev-cli -- gate validate planning-closure";
const EXPECTED_CLAIM_STATUS: &str = "blocked_until_planning_closure_gate_green";
const EXPECTED_FIRST_DELIVERABLE_ID: &str = "FD-001-tenancy-rbac-microservice-core";
const EXPECTED_DELIVERY_MODE: &str = "full_depth_production_hyperscaler_exit";
const EXPECTED_SCOPE_POSTURE: &str = "not_mvp_not_preview_not_reduced_scope";
const EXPECTED_EXIT_CLAIM_BAR: &str = "industry_leading_production_grade_hyperscaler_grade";
const EXPECTED_GOAL_PROMPT_PATH: &str =
    "/evidence/goals/fd001-planning-closure-implementation-goal-2026-05-19.json";
const RETIRED_STALE_ARCHIVE_MANIFEST_PATH: &str =
    ".omc/archive/stale-documents/2026-05-19-planning-closure/manifest.json";
const RETIRED_LOCAL_DURABLE_ROOTS: &[&str] = &[".omc/", ".omx/"];
const EXPECTED_CLOSED_STATUS_FIELD_COUNT: u64 = 177;
const EXPECTED_VERTICAL_ADR_PATH: &str = "docs/decisions/ADR-0217-vertical-slice-rollout-order.md";
const EXPECTED_KR_PACK_OVERVIEW_PATH: &str = "docs/localization-packs/kr.md";
const EXPECTED_KR_PACK_MANIFEST_PATH: &str = "docs/localization-packs/kr/pack.yaml";
const EXPECTED_KR_REGIONAL_PACK_PATH: &str = "packs/kr/sovereignty/manifest.json";

const REQUIRED_ROOT_HUB_DIRECT_AUTHORITY_POINTERS: &[(&str, &str, &str)] = &[
    (
        "adr_0217_vertical_rollout_order",
        "decision",
        EXPECTED_VERTICAL_ADR_PATH,
    ),
    (
        "korea_localization_pack_overview",
        "localization-pack",
        EXPECTED_KR_PACK_OVERVIEW_PATH,
    ),
    (
        "korea_localization_pack_manifest",
        "localization-pack-manifest",
        EXPECTED_KR_PACK_MANIFEST_PATH,
    ),
    (
        "korea_regional_pack",
        "regional-pack",
        EXPECTED_KR_REGIONAL_PACK_PATH,
    ),
];

const RETIRED_DISCOVERY_PATHS: &[&str] = &[
    "/evidence/goals/implement-masterplan.md",
    "/evidence/goals/implement-masterplan-goal-prompt.json",
    ".omc/plans/consensus-masterplan-2026-05-13.md",
    ".omc/plans/M01-M03-parallelization-manifest.md",
];

const REQUIRED_ARCHIVED_STALE_PATTERNS: &[&str] = &[
    "/evidence/goals/implement-masterplan.md",
    "/evidence/goals/implement-masterplan-goal-prompt.json",
    ".omc/plans/*.md",
    ".omc/plans/*.json",
];

const REQUIRED_PACKAGING_AXES: &[&str] = &["tenancy", "rbac"];

const REQUIRED_SURFACES: &[&str] = &[
    "core",
    "messenger",
    "mail",
    "community",
    "infra",
    "ops-dashboard-control-center",
    "intelligence",
    "workflow",
    "ontology",
    "canonical-base",
    "korea-localization-pack",
];

const REQUIRED_KR_PACK_SURFACES: &[&str] = &[
    "pack_manifest",
    "regulatory_bindings",
    "cedar_policy_fragments",
    "workflow_templates",
    "typst_document_templates",
    "messenger_mail_community_localization",
    "tenant_rbac_operating_flows",
    "audit_chain_evidence",
    "data_residency_and_privacy_controls",
    "import_export_migration_paths",
    "operational_runbooks_and_slos",
    "ops_control_center_localization_runbooks_and_escalation_flows",
];

const REQUIRED_DEPLOYMENT_HOST_TARGETS: &[&str] = &[
    "talos",
    "ubuntu-lts",
    "debian",
    "fedora-server",
    "oracle-linux",
    "rhel-compatible",
    "centos-stream",
    "rocky-linux",
    "alma-linux",
    "suse-linux-enterprise",
    "macos-apple-silicon",
];

const REQUIRED_DEPLOYMENT_RUNTIME_TARGETS: &[&str] = &["kubernetes-cloud-native"];

const REQUIRED_DEPLOYMENT_ARTIFACTS: &[&str] = &[
    "oci_images",
    "sbom_and_provenance_attestations",
    "opentofu_iac_modules",
    "gitops_manifests",
    "kubernetes_manifests_or_helm_kustomize",
    "cluster_conformance_evidence",
    "host_bootstrap_pack_evidence",
    "talos_remote_config_join_pack",
    "secure_cluster_join_evidence",
    "bootstrap_secret_externalization_evidence",
    "one_time_secure_hardened_cluster_bootstrap_script",
    "production_hardening_baseline_evidence",
    "cluster_membership_and_policy_compliance_evidence",
    "macos_apple_silicon_local_kubernetes_evidence",
    "multi_arch_oci_images_amd64_arm64",
    "distroless_or_scratch_image_evidence",
    "image_size_and_vulnerability_budget_evidence",
    "full_base_image_exception_registry",
    "one_command_bootstrap_entrypoint",
    "one_click_setup_entrypoint",
    "bootstrap_evidence_report",
    "disaster_recovery_restore_evidence",
];

const REQUIRED_PIPELINE_PHASES: &[&str] = &[
    "requirements_and_scope",
    "architecture_and_api_contracts",
    "implementation",
    "verification_and_merge",
];

const REQUIRED_PIPELINE_SKILLS: &[&str] = &[
    "using-agent-skills",
    "spec-driven-development",
    "planning-and-task-breakdown",
    "documentation-and-adrs",
    "api-and-interface-design",
    "source-driven-development",
    "security-and-hardening",
    "performance-optimization",
    "doubt-driven-development",
    "context-engineering",
    "incremental-implementation",
    "test-driven-development",
    "debugging-and-error-recovery",
    "code-review-and-quality",
    "ci-cd-and-automation",
    "git-workflow-and-versioning",
    "shipping-and-launch",
];

const REQUIRED_PIPELINE_TRIGGERS: &[&str] = &[
    "new_microservice_or_surface_added",
    "new_hyperscaler_pattern_required",
    "new_policy_or_localization_pack_required",
    "new_regression_class_found",
    "performance_baseline_changes",
    "ci_false_green_detected",
    "silent_regression_detected",
    "manual_step_repeated",
    "manual_exception_expired",
    "masterplan_sequence_changes",
    "new_deployment_target_or_host_os_added",
    "kubernetes_distribution_or_runtime_changes",
    "bootstrap_entrypoint_changes",
    "secure_cluster_join_flow_changes",
    "production_hardening_baseline_changes",
];

const REQUIRED_AUTOMATION_TARGETS: &[&str] = &[
    "context_loading_from_root_hub_and_masterplan",
    "contract_schema_validation",
    "openapi_asyncapi_proto_contract_validation",
    "api_semver_and_backward_compatibility_checks",
    "clean_architecture_boundary_checks",
    "microservice_manifest_and_prd_surface_inventory",
    "tenant_isolation_policy_checks",
    "observability_slo_runbook_and_alert_inventory",
    "audit_event_schema_and_replay_checks",
    "policy_compliance_and_data_residency_checks",
    "performance_budget_baselines_and_regression_detection",
    "supply_chain_sbom_signing_and_attestation",
    "test_selection_and_impacted_test_mapping",
    "ci_failure_fix_loop_context_bundles",
    "review_required_context_matching",
    "release_evidence_pack_generation",
    "masterplan_status_and_blocker_audit",
    "stale_document_archive_and_current_authority_scan",
    "deployment_reproducibility_matrix_major_enterprise_linux_and_kubernetes",
    "multi_arch_oci_build_matrix_amd64_arm64",
    "one_command_one_click_setup_verification",
    "remote_config_secure_cluster_join_verification",
    "one_time_secure_bootstrap_hardening_verification",
    "distroless_scratch_image_policy_verification",
    "gitops_iac_drift_and_conformance_checks",
];

const BLOCKING_STATUS_MARKERS: &[&str] = &[
    "tbd",
    "open question",
    "deferred",
    "stub",
    "scaffold",
    "pending",
    "aspirational",
    "empty coverage",
    "false green",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlanningClosureValidateArgs {
    contract_path: PathBuf,
    master_plan_path: PathBuf,
    sequencing_path: PathBuf,
    root_hub_path: PathBuf,
    vertical_adr_path: PathBuf,
}

impl Default for PlanningClosureValidateArgs {
    fn default() -> Self {
        Self {
            contract_path: PathBuf::from("specs/planning-closure-contract.json"),
            master_plan_path: PathBuf::from("specs/masterplan.json"),
            sequencing_path: PathBuf::from("specs/master-plan-sequencing.json"),
            root_hub_path: PathBuf::from("specs/root-hub-pointers.json"),
            vertical_adr_path: PathBuf::from(EXPECTED_VERTICAL_ADR_PATH),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlanningClosureReport {
    pub packaging_axis_count: usize,
    pub surface_count: usize,
    pub kr_pack_surface_count: usize,
    pub architecture_rule_count: usize,
    pub status_fields_checked: usize,
    pub blocker_count: usize,
}

pub(crate) fn parse_planning_closure_validate_args(
    args: Vec<String>,
) -> Result<PlanningClosureValidateArgs, String> {
    let mut parsed = PlanningClosureValidateArgs::default();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--contract" => parsed.contract_path = PathBuf::from(value),
            "--master-plan" => parsed.master_plan_path = PathBuf::from(value),
            "--sequencing" => parsed.sequencing_path = PathBuf::from(value),
            "--root-hub" => parsed.root_hub_path = PathBuf::from(value),
            "--vertical-adr" => parsed.vertical_adr_path = PathBuf::from(value),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_planning_closure_gate(
    args: PlanningClosureValidateArgs,
) -> Result<PlanningClosureReport, String> {
    let contract = read_json(&args.contract_path, "planning closure contract")?;
    let master_plan = read_json(&args.master_plan_path, "master plan")?;
    let sequencing = read_json(&args.sequencing_path, "master plan sequencing")?;
    let root_hub = read_json(&args.root_hub_path, "root hub pointers")?;
    let vertical_adr = fs::read_to_string(&args.vertical_adr_path)
        .map_err(|error| format!("{} unreadable: {error}", args.vertical_adr_path.display()))?;
    let status_ledger_path =
        repo_root_for(&args.root_hub_path).join(EXPECTED_STATUS_LEDGER_REF.trim_start_matches('/'));
    let status_ledger = read_json(&status_ledger_path, "planning closure status ledger")?;

    let contract_root = object(&contract, "planning closure contract root")?;
    let contract_architecture_rules = validate_contract(contract_root)?;
    validate_live_planning_root_authority(contract_root, &repo_root_for(&args.root_hub_path))?;
    validate_status_closure_ledger(object(
        &status_ledger,
        "planning closure status ledger root",
    )?)?;
    validate_root_hub(object(&root_hub, "root hub pointers root")?)?;
    validate_sequencing(
        object(&sequencing, "master plan sequencing root")?,
        &contract_architecture_rules,
    )?;
    validate_master_plan(
        object(&master_plan, "master plan root")?,
        &contract_architecture_rules,
    )?;
    validate_vertical_adr(&vertical_adr, &args.vertical_adr_path)?;

    let status_scan = scan_blocking_statuses(&master_plan);
    if status_scan.blocker_count > 0 {
        return Err(format!(
            "planning closure remains blocked: {} blocking masterplan status values found across {} status fields; first blockers: {}",
            status_scan.blocker_count,
            status_scan.status_fields_checked,
            status_scan.sample_blockers.join("; ")
        ));
    }

    Ok(PlanningClosureReport {
        packaging_axis_count: REQUIRED_PACKAGING_AXES.len(),
        surface_count: REQUIRED_SURFACES.len(),
        kr_pack_surface_count: REQUIRED_KR_PACK_SURFACES.len(),
        architecture_rule_count: contract_architecture_rules.len(),
        status_fields_checked: status_scan.status_fields_checked,
        blocker_count: status_scan.blocker_count,
    })
}

fn validate_contract(root: &Map<String, Value>) -> Result<BTreeSet<String>, String> {
    let meta = object_field(root, "_meta", "planning closure contract root")?;
    require_string_value(meta, "spec_id", EXPECTED_SPEC_ID, "_meta")?;
    require_string_value(meta, "status", EXPECTED_STATUS, "_meta")?;

    let start_rule = object_field(
        root,
        "implementation_start_rule",
        "planning closure contract root",
    )?;
    require_string_value(
        start_rule,
        "claim_status",
        EXPECTED_CLAIM_STATUS,
        "implementation_start_rule",
    )?;
    require_string_value(
        start_rule,
        "gate_command",
        EXPECTED_GATE_COMMAND,
        "implementation_start_rule",
    )?;

    let authority = object_field(
        root,
        "master_plan_authority",
        "planning closure contract root",
    )?;
    require_string_value(
        authority,
        "current_path",
        EXPECTED_MASTERPLAN_PATH,
        "master_plan_authority",
    )?;

    let first = object_field(root, "first_deliverable", "planning closure contract root")?;
    validate_first_deliverable_common(first, "first_deliverable")?;

    let localization = object_field(first, "localization_exit_bar", "first_deliverable")?;
    require_bool_value(
        localization,
        "canonical_base_required",
        true,
        "first_deliverable.localization_exit_bar",
    )?;
    require_string_value(
        localization,
        "first_pack",
        "kr",
        "first_deliverable.localization_exit_bar",
    )?;
    require_bool_value(
        localization,
        "korea_pack_required_at_exit",
        true,
        "first_deliverable.localization_exit_bar",
    )?;
    require_exact_string_set(
        localization,
        "required_korea_pack_surfaces",
        REQUIRED_KR_PACK_SURFACES,
        "first_deliverable.localization_exit_bar",
    )?;
    let architecture = object_field(first, "architecture_exit_bar", "first_deliverable")?;
    let architecture_rules =
        validate_architecture_exit_bar(architecture, "first_deliverable.architecture_exit_bar")?;
    let deployment = object_field(
        root,
        "deployment_portability_policy",
        "planning closure contract root",
    )?;
    validate_deployment_portability_policy(deployment, "deployment_portability_policy")?;
    let pipeline = object_field(
        root,
        "development_pipeline_policy",
        "planning closure contract root",
    )?;
    validate_development_pipeline_policy(pipeline, "development_pipeline_policy")?;

    let no_placeholder = object_field(
        root,
        "no_placeholder_policy",
        "planning closure contract root",
    )?;
    let stale = object_field(
        root,
        "stale_document_policy",
        "planning closure contract root",
    )?;
    validate_stale_document_policy(stale, "stale_document_policy")?;

    require_bool_value(
        no_placeholder,
        "no_placeholders_stubs_thin_scaffolds",
        true,
        "no_placeholder_policy",
    )?;
    for key in [
        "no_empty_promises",
        "no_false_signals",
        "no_silent_regressions",
    ] {
        require_bool_value(no_placeholder, key, true, "no_placeholder_policy")?;
    }
    require_string_contains(
        no_placeholder,
        "claim_control_rule",
        "Empty green checks are false signals",
        "no_placeholder_policy",
    )?;
    require_string_contains(
        no_placeholder,
        "regression_control_rule",
        "Silent behavior, performance, policy, schema, API, tenant-isolation, observability, and auditability regressions block",
        "no_placeholder_policy",
    )?;
    require_marker_coverage(no_placeholder, "blocking_markers", "no_placeholder_policy")?;
    Ok(architecture_rules)
}

fn validate_stale_document_policy(
    policy: &Map<String, Value>,
    context: &str,
) -> Result<(), String> {
    require_bool_value(policy, "stale_documents_must_be_archived", true, context)?;
    require_bool_value(policy, "durable_roots_must_be_local_only", true, context)?;
    require_string_value(
        policy,
        "retired_archive_manifest_path",
        RETIRED_STALE_ARCHIVE_MANIFEST_PATH,
        context,
    )?;
    let ignored_roots = string_set_field(policy, "ignored_durable_roots", context)?;
    require_seen_set(
        &ignored_roots,
        RETIRED_LOCAL_DURABLE_ROOTS,
        &format!("{context}.ignored_durable_roots"),
    )?;
    require_string_value(
        policy,
        "current_goal_prompt",
        EXPECTED_GOAL_PROMPT_PATH,
        context,
    )?;
    let live_roots = string_set_field(policy, "live_planning_roots", context)?;
    reject_retired_durable_authority_refs(&live_roots, &format!("{context}.live_planning_roots"))?;
    let forbidden = string_set_field(policy, "forbidden_live_authority_paths", context)?;
    require_seen_set(
        &forbidden,
        REQUIRED_ARCHIVED_STALE_PATTERNS,
        &format!("{context}.forbidden_live_authority_paths"),
    )?;
    require_string_contains(
        policy,
        "rule",
        "Archived files are historical evidence only",
        context,
    )?;
    require_string_contains(policy, "rule", "local-only", context)?;
    Ok(())
}

fn validate_live_planning_root_authority(
    contract: &Map<String, Value>,
    repo_root: &Path,
) -> Result<(), String> {
    let policy = object_field(
        contract,
        "stale_document_policy",
        "planning closure contract root",
    )?;
    let live_roots = string_set_field(policy, "live_planning_roots", "stale_document_policy")?;
    for live_root in live_roots {
        let live_root_path = repo_root.join(live_root.trim_start_matches('/'));
        let contents = fs::read_to_string(&live_root_path).map_err(|error| {
            format!(
                "stale_document_policy.live_planning_roots entry {live_root:?} unreadable at {}: {error}",
                live_root_path.display()
            )
        })?;
        for (index, line) in contents.lines().enumerate() {
            if line.contains(RETIRED_PLANNING_CLOSURE_COMMAND)
                && !retired_planning_closure_command_is_provenance_only(line)
            {
                return Err(format!(
                    "{live_root}:{line_number} contains retired Cargo planning-closure command as active authority; live planning roots must use {EXPECTED_GATE_COMMAND:?} or mark the retired command as retired/provenance-only",
                    line_number = index + 1,
                ));
            }
            if oya_vcs_authority_is_active(line) {
                return Err(format!(
                    "{live_root}:{line_number} contains active Oya VCS claim/admission/closure/promotion authority; live planning roots must use plain git plus protected PRs and {EXPECTED_GATE_COMMAND:?}, or mark Oya VCS references as retired/provenance-only/future-target",
                    line_number = index + 1,
                ));
            }
        }
    }
    Ok(())
}

fn retired_planning_closure_command_is_provenance_only(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("retired") && lower.contains("provenance-only")
}

fn oya_vcs_authority_is_active(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("oya vcs")
        && !oya_vcs_authority_is_marked_inactive(&lower)
        && (lower.contains("claim/admission")
            || lower.contains("claimable through")
            || lower.contains("accepts the claim")
            || lower.contains("claim state")
            || lower.contains("admission names")
            || lower.contains("closure_authority")
            || lower.contains("closure authority")
            || oya_vcs_has_local_authority_word(&lower)
            || lower.contains("promotion")
            || lower.contains("promote")
            || (lower.contains("claim each changeset")
                && lower.contains("verify before downstream work")
                && lower.contains("mark done")))
}

fn oya_vcs_has_local_authority_word(lower: &str) -> bool {
    lower.match_indices("oya vcs").any(|(index, _)| {
        lower[index..]
            .find("authority")
            .is_some_and(|offset| offset <= 64)
    })
}

fn oya_vcs_authority_is_marked_inactive(lower: &str) -> bool {
    has_unnegated_marker(lower, "provenance-only")
        || has_unnegated_marker(lower, "future-target")
        || has_unnegated_marker(lower, "future target")
        || (lower.contains("retired")
            && lower.contains("provenance")
            && !lower.contains("not retired")
            && !lower.contains("not provenance"))
}

fn has_unnegated_marker(lower: &str, marker: &str) -> bool {
    lower.contains(marker)
        && !lower.contains(&format!("not {marker}"))
        && !lower.contains(&format!("not marked {marker}"))
        && !lower.contains(&format!("without {marker}"))
        && !combined_inactive_marker_is_negated(lower)
}

fn combined_inactive_marker_is_negated(lower: &str) -> bool {
    ["not", "not marked", "without"].iter().any(|negation| {
        [
            "future-target/provenance-only",
            "future target/provenance-only",
        ]
        .iter()
        .any(|marker| lower.contains(&format!("{negation} {marker}")))
    })
}

fn validate_root_hub(root: &Map<String, Value>) -> Result<(), String> {
    let entry_points = object_field(root, "entry_points", "root hub pointers root")?;
    let masterplan = object_field(entry_points, "masterplan", "entry_points")?;
    require_string_value(
        masterplan,
        "current_path",
        EXPECTED_MASTERPLAN_PATH,
        "entry_points.masterplan",
    )?;
    let projection = string_field(
        masterplan,
        "human_compatibility_projection",
        "entry_points.masterplan",
    )?;
    if projection != "docs/MASTERPLAN.md" {
        return Err(format!(
            "entry_points.masterplan.human_compatibility_projection must be docs/MASTERPLAN.md, got {projection:?}"
        ));
    }
    let durable_goal = object_field(entry_points, "agent_durable_goal", "entry_points")?;
    require_string_value(
        durable_goal,
        "current_path",
        EXPECTED_GOAL_PROMPT_PATH,
        "entry_points.agent_durable_goal",
    )?;
    require_string_value(
        durable_goal,
        "superseded_path",
        "/specs/agent-durable-goal.json",
        "entry_points.agent_durable_goal",
    )?;
    require_string_value(
        durable_goal,
        "retired_archive_manifest_path",
        RETIRED_STALE_ARCHIVE_MANIFEST_PATH,
        "entry_points.agent_durable_goal",
    )?;
    let status_ledger = object_field(
        entry_points,
        "planning_closure_status_ledger",
        "entry_points",
    )?;
    require_string_value(
        status_ledger,
        "current_path",
        EXPECTED_STATUS_LEDGER_REF,
        "entry_points.planning_closure_status_ledger",
    )?;
    require_string_value(
        status_ledger,
        "contract_ref",
        EXPECTED_CONTRACT_REF,
        "entry_points.planning_closure_status_ledger",
    )?;
    for (key, expected_kind, expected_path) in REQUIRED_ROOT_HUB_DIRECT_AUTHORITY_POINTERS {
        validate_required_root_hub_authority_pointer(
            entry_points,
            key,
            expected_kind,
            expected_path,
        )?;
    }
    Ok(())
}

fn validate_required_root_hub_authority_pointer(
    entry_points: &Map<String, Value>,
    key: &str,
    expected_kind: &str,
    expected_path: &str,
) -> Result<(), String> {
    let entry = entry_points.get(key).ok_or_else(|| {
        format!(
            "root hub missing direct authority pointer entry_points.{key} for {expected_path}; indirect masterplan/contract coverage is not enough"
        )
    })?;
    let entry = object(entry, &format!("entry_points.{key}"))?;
    require_string_value(entry, "kind", expected_kind, &format!("entry_points.{key}"))?;
    require_string_value(
        entry,
        "current_path",
        expected_path,
        &format!("entry_points.{key}"),
    )?;
    Ok(())
}

fn validate_status_closure_ledger(root: &Map<String, Value>) -> Result<(), String> {
    let meta = object_field(root, "_meta", "planning closure status ledger root")?;
    require_string_value(
        meta,
        "spec_id",
        "PLANNING-CLOSURE-STATUS-CLOSURE-LEDGER",
        "_meta",
    )?;
    require_string_value(meta, "status", "accepted", "_meta")?;

    let scope = object_field(root, "closure_scope", "planning closure status ledger root")?;
    require_string_value(
        scope,
        "masterplan_path",
        EXPECTED_MASTERPLAN_PATH,
        "closure_scope",
    )?;
    require_string_value(
        scope,
        "contract_ref",
        EXPECTED_CONTRACT_REF,
        "closure_scope",
    )?;
    require_string_value(
        scope,
        "gate_command",
        EXPECTED_GATE_COMMAND,
        "closure_scope",
    )?;
    require_u64_value(
        scope,
        "closed_status_field_count",
        EXPECTED_CLOSED_STATUS_FIELD_COUNT,
        "closure_scope",
    )?;

    let guardrails = string_set_field(
        root,
        "implementation_guardrails",
        "planning closure status ledger root",
    )?;
    require_seen_set(
        &guardrails,
        &[
            "Every closed planning item remains a ChangeSet-owned execution unit.",
            "Implementation begins only after the planning-closure gate is green.",
            "A status closure must cite this ledger and the planning-closure contract.",
        ],
        "planning closure status ledger root.implementation_guardrails",
    )?;

    let evidence = string_set_field(
        root,
        "required_exit_evidence",
        "planning closure status ledger root",
    )?;
    require_seen_set(
        &evidence,
        &[
            "prd_phase_ip_acceptance_matrix",
            "tenant_isolation_and_policy_tests",
            "observability_golden_signal_dashboards",
            "talos_remote_secure_join_evidence",
            "ops_dashboard_control_center_operational_evidence",
            "korea_localization_pack_exit_evidence",
        ],
        "planning closure status ledger root.required_exit_evidence",
    )?;

    let contracts = root
        .get("hyperscaler_gap_ip_contracts")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "planning closure status ledger root.hyperscaler_gap_ip_contracts must be an array"
                .to_string()
        })?;
    if contracts.len() != 5 {
        return Err(format!(
            "planning closure status ledger root.hyperscaler_gap_ip_contracts must contain 5 items, got {}",
            contracts.len()
        ));
    }
    for (index, item) in contracts.iter().enumerate() {
        let context = format!("hyperscaler_gap_ip_contracts[{index}]");
        let contract = object(item, &context)?;
        require_string_value(contract, "closure", "planning-contract-authored", &context)?;
        require_string_value(
            contract,
            "implementation_state",
            "not-started-changeset-required",
            &context,
        )?;
    }

    Ok(())
}

fn validate_sequencing(
    root: &Map<String, Value>,
    expected_architecture_rules: &BTreeSet<String>,
) -> Result<(), String> {
    validate_stale_document_archival(root)?;

    let closure = object_field(
        root,
        "planning_closure_contract",
        "master plan sequencing root",
    )?;
    require_string_value(
        closure,
        "status",
        EXPECTED_CLAIM_STATUS,
        "planning_closure_contract",
    )?;
    require_string_value(
        closure,
        "contract_ref",
        EXPECTED_CONTRACT_REF,
        "planning_closure_contract",
    )?;
    require_string_value(
        closure,
        "gate_command",
        EXPECTED_GATE_COMMAND,
        "planning_closure_contract",
    )?;
    require_string_value(
        closure,
        "master_plan_authority",
        EXPECTED_MASTERPLAN_PATH,
        "planning_closure_contract",
    )?;

    let first = object_field(
        root,
        "first_deliverable_ordering",
        "master plan sequencing root",
    )?;
    validate_first_deliverable_common(first, "first_deliverable_ordering")?;
    require_bool_value(
        first,
        "canonical_base_required",
        true,
        "first_deliverable_ordering",
    )?;
    require_bool_value(
        first,
        "korea_localization_pack_required",
        true,
        "first_deliverable_ordering",
    )?;
    let architecture = object_field(first, "architecture_exit_bar", "first_deliverable_ordering")?;
    validate_architecture_exit_bar_matches(
        architecture,
        expected_architecture_rules,
        "first_deliverable_ordering.architecture_exit_bar",
    )?;
    Ok(())
}

fn validate_stale_document_archival(root: &Map<String, Value>) -> Result<(), String> {
    let metadata = object_field(root, "_metadata", "master plan sequencing root")?;
    let stale = object_field(
        metadata,
        "archived_stale_documents",
        "master plan sequencing _metadata",
    )?;
    require_string_value(
        stale,
        "retired_manifest_path",
        RETIRED_STALE_ARCHIVE_MANIFEST_PATH,
        "_metadata.archived_stale_documents",
    )?;
    require_string_value(
        stale,
        "live_replacement",
        EXPECTED_GOAL_PROMPT_PATH,
        "_metadata.archived_stale_documents",
    )?;
    let archived_patterns = string_set_field(
        stale,
        "do_not_use_as_authority",
        "_metadata.archived_stale_documents",
    )?;
    require_seen_set(
        &archived_patterns,
        REQUIRED_ARCHIVED_STALE_PATTERNS,
        "_metadata.archived_stale_documents.do_not_use_as_authority",
    )?;

    let discovery = string_set_field(
        root,
        "master_plan_discovery_order",
        "master plan sequencing root",
    )?;
    if !discovery.contains(EXPECTED_GOAL_PROMPT_PATH) {
        return Err(format!(
            "master_plan_discovery_order must include current goal prompt {EXPECTED_GOAL_PROMPT_PATH}"
        ));
    }
    reject_retired_durable_authority_refs(&discovery, "master_plan_discovery_order")?;
    for retired in RETIRED_DISCOVERY_PATHS {
        if discovery.contains(*retired) {
            return Err(format!(
                "master_plan_discovery_order still points to stale archived document {retired}"
            ));
        }
    }
    Ok(())
}

fn reject_retired_durable_authority_refs(
    refs: &BTreeSet<String>,
    context: &str,
) -> Result<(), String> {
    for value in refs {
        if is_retired_durable_root_ref(value) {
            return Err(format!(
                "{context} must not use retired local durable root as live authority: {value}"
            ));
        }
    }
    Ok(())
}

fn is_retired_durable_root_ref(value: &str) -> bool {
    let normalized = value.trim_start_matches('/');
    RETIRED_LOCAL_DURABLE_ROOTS
        .iter()
        .any(|root| normalized.starts_with(root))
}

fn validate_master_plan(
    root: &Map<String, Value>,
    expected_architecture_rules: &BTreeSet<String>,
) -> Result<(), String> {
    let meta = object_field(root, "_meta", "master plan root")?;
    require_string_value(
        meta,
        "current_machine_readable_authority",
        EXPECTED_MASTERPLAN_PATH,
        "_meta",
    )?;
    require_string_value(
        meta,
        "human_compatibility_projection",
        "docs/MASTERPLAN.md",
        "_meta",
    )?;

    let closure = object_field(root, "planning_closure", "master plan root")?;
    require_string_value(closure, "status", EXPECTED_CLAIM_STATUS, "planning_closure")?;
    require_string_value(
        closure,
        "contract_ref",
        EXPECTED_CONTRACT_REF,
        "planning_closure",
    )?;
    require_string_value(
        closure,
        "gate_command",
        EXPECTED_GATE_COMMAND,
        "planning_closure",
    )?;
    let first = object_field(closure, "first_deliverable", "planning_closure")?;
    validate_first_deliverable_common(first, "planning_closure.first_deliverable")?;
    require_bool_value(
        first,
        "canonical_base_required",
        true,
        "planning_closure.first_deliverable",
    )?;
    require_bool_value(
        first,
        "korea_localization_pack_required",
        true,
        "planning_closure.first_deliverable",
    )?;
    let architecture = object_field(
        first,
        "architecture_exit_bar",
        "planning_closure.first_deliverable",
    )?;
    validate_architecture_exit_bar_matches(
        architecture,
        expected_architecture_rules,
        "planning_closure.first_deliverable.architecture_exit_bar",
    )?;
    Ok(())
}

fn validate_first_deliverable_common(
    first: &Map<String, Value>,
    context: &str,
) -> Result<(), String> {
    require_string_value(first, "id", EXPECTED_FIRST_DELIVERABLE_ID, context)?;
    require_string_value(first, "delivery_mode", EXPECTED_DELIVERY_MODE, context)?;
    require_string_value(first, "scope_posture", EXPECTED_SCOPE_POSTURE, context)?;
    require_string_value(first, "exit_claim_bar", EXPECTED_EXIT_CLAIM_BAR, context)?;
    require_exact_string_set(first, "packaging_axes", REQUIRED_PACKAGING_AXES, context)?;
    require_exact_string_set(first, "required_surfaces", REQUIRED_SURFACES, context)?;
    Ok(())
}

fn validate_architecture_exit_bar(
    architecture: &Map<String, Value>,
    context: &str,
) -> Result<BTreeSet<String>, String> {
    for key in [
        "clean_architecture_required",
        "api_first_required",
        "hyperscaler_patterns_required",
        "flat_microservice_catalog_required",
        "microservice_boundaries_required",
        "independent_horizontal_scaling_required",
    ] {
        require_bool_value(architecture, key, true, context)?;
    }
    require_string_value(
        architecture,
        "clean_architecture_authority",
        "docs/standards/clean-architecture.md",
        context,
    )?;
    require_string_value(
        architecture,
        "hyperscaler_pattern_authority",
        "specs/hyperscaler-architecture-invariants.json",
        context,
    )?;
    architecture_rule_set(architecture, context)
}

fn validate_architecture_exit_bar_matches(
    architecture: &Map<String, Value>,
    expected_architecture_rules: &BTreeSet<String>,
    context: &str,
) -> Result<(), String> {
    let rules = validate_architecture_exit_bar(architecture, context)?;
    require_exact_seen_set(
        &rules,
        expected_architecture_rules,
        &format!("{context}.required_rules"),
    )
}

fn architecture_rule_set(
    architecture: &Map<String, Value>,
    context: &str,
) -> Result<BTreeSet<String>, String> {
    if architecture.contains_key("required_rules") {
        return string_set_field(architecture, "required_rules", context);
    }

    let mut rules = BTreeSet::new();
    for key in [
        "required_microservice_rules",
        "required_clean_architecture_rules",
        "required_api_first_rules",
        "required_hyperscaler_pattern_rules",
    ] {
        if architecture.contains_key(key) {
            rules.extend(string_set_field(architecture, key, context)?);
        }
    }
    if rules.is_empty() {
        return Err(format!(
            "{context} must define required_rules or split required_*_rules arrays"
        ));
    }
    Ok(rules)
}

fn validate_deployment_portability_policy(
    policy: &Map<String, Value>,
    context: &str,
) -> Result<(), String> {
    for key in [
        "cloud_native_kubernetes_required",
        "no_host_distribution_lock_in",
        "host_distribution_specific_bootstrap_must_be_adapter_pack",
        "manual_ssh_snowflakes_forbidden",
        "declarative_reproducibility_required",
        "one_command_or_one_click_setup_required",
        "multi_arch_amd64_arm64_required",
        "remote_config_bootstrap_required",
        "secure_cluster_join_required",
        "bootstrap_secret_material_must_be_externalized",
        "one_time_script_bootstrap_required",
        "production_hardening_baseline_required",
        "cluster_membership_after_bootstrap_required",
        "fail_closed_on_unmet_prerequisites",
        "distroless_or_scratch_images_default_required",
        "full_base_image_exception_requires_evidence",
        "image_size_and_vulnerability_budget_required",
    ] {
        require_bool_value(policy, key, true, context)?;
    }
    require_exact_string_set(
        policy,
        "required_host_targets",
        REQUIRED_DEPLOYMENT_HOST_TARGETS,
        context,
    )?;
    require_exact_string_set(
        policy,
        "required_runtime_targets",
        REQUIRED_DEPLOYMENT_RUNTIME_TARGETS,
        context,
    )?;
    require_exact_string_set(
        policy,
        "required_artifacts",
        REQUIRED_DEPLOYMENT_ARTIFACTS,
        context,
    )?;
    require_string_contains(
        policy,
        "exit_rule",
        "Talos, Ubuntu LTS, Debian, Fedora Server",
        context,
    )?;
    require_string_contains(policy, "exit_rule", "macOS Apple Silicon", context)?;
    require_string_contains(policy, "exit_rule", "one-command or one-click", context)?;
    require_string_contains(policy, "exit_rule", "Talos-class nodes", context)?;
    require_string_contains(policy, "exit_rule", "securely join the cluster", context)?;
    require_string_contains(policy, "exit_rule", "one-time script launch", context)?;
    require_string_contains(
        policy,
        "exit_rule",
        "secured, hardened cluster member",
        context,
    )?;
    require_string_contains(policy, "exit_rule", "fail closed", context)?;
    require_string_contains(policy, "exit_rule", "distroless or scratch", context)?;
    require_string_contains(
        policy,
        "exit_rule",
        "image-size and vulnerability budgets",
        context,
    )?;
    Ok(())
}

fn validate_vertical_adr(contents: &str, path: &Path) -> Result<(), String> {
    let lower = contents.to_ascii_lowercase();
    let path = path.display();
    for phrase in [
        "tenancy/rbac packaging",
        "personal and professional life",
        "full production depth",
        "not mvp",
        "core",
        "messenger",
        "mail",
        "community",
        "infra",
        "ops dashboard",
        "control center",
        "intelligence",
        "workflow",
        "ontology",
        "canonical base",
        "korea localization pack",
        "flat microservice",
        "clean architecture",
        "api-first",
        "independent horizontal scaling",
        "hyperscaler pattern",
        "empty promises",
        "false green signals",
        "silent regressions",
        "development pipeline must evolve",
        "phase-appropriate agent skills",
        "no deferrals",
        "placeholders",
        "stubs",
        "thin scaffolds",
        "cloud-native kubernetes",
        "talos",
        "ubuntu lts",
        "major enterprise linux",
        "macos apple silicon",
        "one-time script",
        "securely join",
        "hardened",
        "fail closed",
        "distroless or scratch",
        "oracle linux",
        "debian",
        "fedora server",
        "centos stream",
        "rhel-compatible",
        "suse linux enterprise",
    ] {
        if !lower.contains(phrase) {
            return Err(format!(
                "{path} must mention planning-closure phrase {phrase:?}"
            ));
        }
    }
    Ok(())
}

fn validate_development_pipeline_policy(
    policy: &Map<String, Value>,
    context: &str,
) -> Result<(), String> {
    require_bool_value(policy, "using_agent_skills_required", true, context)?;
    require_bool_value(policy, "pipeline_must_evolve_with_project", true, context)?;
    require_bool_value(policy, "automation_first_required", true, context)?;
    require_bool_value(policy, "automatable_work_must_be_automated", true, context)?;
    require_bool_value(
        policy,
        "manual_exception_requires_registered_evidence",
        true,
        context,
    )?;

    let phases = policy
        .get("phase_skill_routing")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.phase_skill_routing must be an array"))?;
    let mut seen_phases = BTreeSet::new();
    let mut seen_skills = BTreeSet::new();
    for (index, row) in phases.iter().enumerate() {
        let phase_context = format!("{context}.phase_skill_routing[{index}]");
        let phase = object(row, &phase_context)?;
        let phase_name = string_field(phase, "phase", &phase_context)?;
        seen_phases.insert(phase_name.to_owned());
        let skills = phase
            .get("required_skills")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{phase_context}.required_skills must be an array"))?;
        if skills.is_empty() {
            return Err(format!("{phase_context}.required_skills must be non-empty"));
        }
        for (skill_index, skill) in skills.iter().enumerate() {
            let skill = skill.as_str().ok_or_else(|| {
                format!("{phase_context}.required_skills[{skill_index}] must be a string")
            })?;
            seen_skills.insert(skill.to_owned());
        }
    }
    require_seen_set(
        &seen_phases,
        REQUIRED_PIPELINE_PHASES,
        &format!("{context}.phase_skill_routing.phase"),
    )?;
    require_seen_set(
        &seen_skills,
        REQUIRED_PIPELINE_SKILLS,
        &format!("{context}.phase_skill_routing.required_skills"),
    )?;

    let triggers = policy
        .get("evolution_triggers")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.evolution_triggers must be an array"))?;
    let mut seen_triggers = BTreeSet::new();
    for (index, trigger) in triggers.iter().enumerate() {
        let trigger = trigger
            .as_str()
            .ok_or_else(|| format!("{context}.evolution_triggers[{index}] must be a string"))?;
        seen_triggers.insert(trigger.to_owned());
    }
    require_seen_set(
        &seen_triggers,
        REQUIRED_PIPELINE_TRIGGERS,
        &format!("{context}.evolution_triggers"),
    )?;
    let automation_targets = policy
        .get("automation_required_for")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.automation_required_for must be an array"))?;
    let mut seen_automation_targets = BTreeSet::new();
    for (index, target) in automation_targets.iter().enumerate() {
        let target = target.as_str().ok_or_else(|| {
            format!("{context}.automation_required_for[{index}] must be a string")
        })?;
        seen_automation_targets.insert(target.to_owned());
    }
    require_seen_set(
        &seen_automation_targets,
        REQUIRED_AUTOMATION_TARGETS,
        &format!("{context}.automation_required_for"),
    )?;
    require_string_contains(
        policy,
        "manual_exception_rule",
        "Any development-cycle step that remains manual",
        context,
    )?;
    require_string_contains(
        policy,
        "ratchet_rule",
        "update the relevant gate, baseline, skill routing, evidence template, and masterplan sequencing",
        context,
    )?;
    Ok(())
}

#[derive(Debug)]
struct StatusScan {
    status_fields_checked: usize,
    blocker_count: usize,
    sample_blockers: Vec<String>,
}

fn scan_blocking_statuses(root: &Value) -> StatusScan {
    let mut queue = VecDeque::from([(String::from("$"), root)]);
    let mut status_fields_checked = 0;
    let mut blocker_count = 0;
    let mut sample_blockers = Vec::new();

    while let Some((path, value)) = queue.pop_front() {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    let child_path = format!("{path}.{key}");
                    if key == "status" {
                        status_fields_checked += 1;
                        if let Some(status) = child.as_str() {
                            let status_lower = status.to_ascii_lowercase();
                            if BLOCKING_STATUS_MARKERS
                                .iter()
                                .any(|marker| status_lower.contains(marker))
                            {
                                blocker_count += 1;
                                if sample_blockers.len() < 8 {
                                    sample_blockers.push(format!("{child_path}={status:?}"));
                                }
                            }
                        }
                    }
                    queue.push_back((child_path, child));
                }
            }
            Value::Array(items) => {
                for (index, child) in items.iter().enumerate() {
                    queue.push_back((format!("{path}[{index}]"), child));
                }
            }
            _ => {}
        }
    }

    StatusScan {
        status_fields_checked,
        blocker_count,
        sample_blockers,
    }
}

fn read_json(path: &Path, label: &str) -> Result<Value, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("{label} {} unreadable: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("{label} {} invalid JSON: {error}", path.display()))
}

fn repo_root_for(root_hub_path: &Path) -> PathBuf {
    root_hub_path
        .parent()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn object<'a>(value: &'a Value, context: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{context} must be an object"))
}

fn object_field<'a>(
    map: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<&'a Map<String, Value>, String> {
    let value = map
        .get(key)
        .ok_or_else(|| format!("{context}.{key} is missing"))?;
    object(value, &format!("{context}.{key}"))
}

fn string_field<'a>(
    map: &'a Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<&'a str, String> {
    map.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{context}.{key} must be a string"))
}

fn require_string_value(
    map: &Map<String, Value>,
    key: &str,
    expected: &str,
    context: &str,
) -> Result<(), String> {
    let actual = string_field(map, key, context)?;
    if actual != expected {
        return Err(format!(
            "{context}.{key} must be {expected:?}, got {actual:?}"
        ));
    }
    Ok(())
}

fn require_bool_value(
    map: &Map<String, Value>,
    key: &str,
    expected: bool,
    context: &str,
) -> Result<(), String> {
    let actual = map
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{context}.{key} must be a boolean"))?;
    if actual != expected {
        return Err(format!("{context}.{key} must be {expected}, got {actual}"));
    }
    Ok(())
}

fn require_u64_value(
    map: &Map<String, Value>,
    key: &str,
    expected: u64,
    context: &str,
) -> Result<(), String> {
    let actual = map
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{context}.{key} must be an unsigned integer"))?;
    if actual != expected {
        return Err(format!("{context}.{key} must be {expected}, got {actual}"));
    }
    Ok(())
}

fn require_exact_string_set(
    map: &Map<String, Value>,
    key: &str,
    expected: &[&str],
    context: &str,
) -> Result<(), String> {
    let actual = map
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.{key} must be an array"))?;
    let mut actual_set = BTreeSet::new();
    for (index, item) in actual.iter().enumerate() {
        let value = item
            .as_str()
            .ok_or_else(|| format!("{context}.{key}[{index}] must be a string"))?;
        if !actual_set.insert(value.to_owned()) {
            return Err(format!(
                "{context}.{key} contains duplicate value {value:?}"
            ));
        }
    }

    let expected_set = expected
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    if actual_set != expected_set {
        let missing = expected_set
            .difference(&actual_set)
            .map(String::as_str)
            .collect::<Vec<_>>();
        let unexpected = actual_set
            .difference(&expected_set)
            .map(String::as_str)
            .collect::<Vec<_>>();
        return Err(format!(
            "{context}.{key} set mismatch; missing=[{}] unexpected=[{}]",
            missing.join(", "),
            unexpected.join(", ")
        ));
    }
    Ok(())
}

fn string_set_field(
    map: &Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<BTreeSet<String>, String> {
    let actual = map
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.{key} must be an array"))?;
    let mut actual_set = BTreeSet::new();
    for (index, item) in actual.iter().enumerate() {
        let value = item
            .as_str()
            .ok_or_else(|| format!("{context}.{key}[{index}] must be a string"))?;
        if !actual_set.insert(value.to_owned()) {
            return Err(format!(
                "{context}.{key} contains duplicate value {value:?}"
            ));
        }
    }
    Ok(actual_set)
}

fn require_exact_seen_set(
    actual_set: &BTreeSet<String>,
    expected_set: &BTreeSet<String>,
    context: &str,
) -> Result<(), String> {
    if actual_set != expected_set {
        let missing = expected_set
            .difference(actual_set)
            .map(String::as_str)
            .collect::<Vec<_>>();
        let unexpected = actual_set
            .difference(expected_set)
            .map(String::as_str)
            .collect::<Vec<_>>();
        return Err(format!(
            "{context} set mismatch; missing=[{}] unexpected=[{}]",
            missing.join(", "),
            unexpected.join(", ")
        ));
    }
    Ok(())
}

fn require_seen_set(
    actual_set: &BTreeSet<String>,
    expected: &[&str],
    context: &str,
) -> Result<(), String> {
    let expected_set = expected
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    let missing = expected_set
        .difference(actual_set)
        .map(String::as_str)
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "{context} missing required values [{}]",
            missing.join(", ")
        ));
    }
    Ok(())
}

fn require_string_contains(
    map: &Map<String, Value>,
    key: &str,
    needle: &str,
    context: &str,
) -> Result<(), String> {
    let actual = string_field(map, key, context)?;
    if !actual.contains(needle) {
        return Err(format!(
            "{context}.{key} must contain {needle:?}, got {actual:?}"
        ));
    }
    Ok(())
}

fn require_marker_coverage(
    map: &Map<String, Value>,
    key: &str,
    context: &str,
) -> Result<(), String> {
    let actual = map
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{context}.{key} must be an array"))?;
    let lower_values = actual
        .iter()
        .map(|item| {
            item.as_str()
                .map(|value| value.to_ascii_lowercase())
                .ok_or_else(|| format!("{context}.{key} entries must be strings"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for marker in BLOCKING_STATUS_MARKERS {
        if !lower_values.iter().any(|value| value.contains(marker)) {
            return Err(format!("{context}.{key} must include marker {marker:?}"));
        }
    }
    Ok(())
}
