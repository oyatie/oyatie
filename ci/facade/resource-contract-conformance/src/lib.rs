//! # cloud-ci-cloud-resource-contracts
//!
//! Configurable Rust/Buck2 replacement for the first P0 Python cloud-resource validator slice:
//! `scripts/tests/cloud_resource_contract_parity_catalog_check.py`,
//! `scripts/tests/cloud_control_plane_operation_contract_check.py`, and
//! `scripts/tests/cloud_enforceability_facets_check.py`.
//!
//! The primary surface is API/config shaped: callers pass policy data plus typed JSON artifacts to
//! [`evaluate_configured`]. The gate does not shell out to Python, mutate files, or read ambient
//! repository state. Repository-specific paths and selected legacy sources live in
//! `cloud-resource-contracts-policy.json`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

pub const GATE_ID: &str = "cloud-ci-cloud-resource-contracts";
const REPLACEMENT_GATE_TARGET: &str =
    "//ci/facade/resource-contract-conformance:ci-resource-contract-conformance-gate";

const REQUIRED_RESOURCE_FACETS: &[&str] = &[
    "orn",
    "lifecycle_state",
    "quota_cost",
    "billing_meters",
    "audit_events",
    "tenant_account_project",
    "region_cell",
    "owner",
    "deletion_retention",
    "slo_tier",
];
const REQUIRED_CATEGORY_IDS: &[&str] = &[
    "identity_access_policy",
    "compute_instances",
    "containers_kubernetes",
    "serverless_functions",
    "storage_object_block_file",
    "networking_dns_edge",
    "databases_data_analytics",
    "kms_secrets_confidentiality",
    "observability_operations",
    "billing_finops_quotas",
    "marketplace_isv_ecosystem",
    "security_posture_guardrails",
    "cloud_native_platform_contract",
];
const REQUIRED_RESOURCE_NONCLAIMS: &[&str] = &[
    "no_live_provider_provisioning",
    "no_provider_feature_parity_claim",
    "no_production_readiness_claim",
    "no_tenant_workload_claim",
    "no_public_sla_slo_claim",
];
const REQUIRED_OPERATION_STAGES: &[&str] = &[
    "api_gateway",
    "resource_registry",
    "operation_ledger",
    "workflow_reconciler",
    "backend_actuation_boundary",
];
const REQUIRED_OPERATION_FIELDS: &[&str] = &[
    "operation_id",
    "idempotency_key",
    "request_hash",
    "resource_orn",
    "desired_generation",
    "observed_generation",
    "state",
    "phase",
    "tenant_account_project",
    "region_cell",
    "principal",
    "audit_chain_id",
    "retry_policy",
    "cancellation",
    "compensation",
];
const REQUIRED_OPERATION_STATES: &[&str] = &[
    "accepted",
    "validating",
    "queued",
    "running",
    "waiting_for_reconciler",
    "succeeded",
    "failed",
    "cancel_requested",
    "cancelled",
    "compensating",
    "rolled_back",
];
const REQUIRED_OPERATION_SEMANTICS: &[&str] = &[
    "idempotent_retry",
    "resumable_after_restart",
    "cancel_safe",
    "compensating_action",
    "no_partial_apply_without_ledger",
];
const REQUIRED_CANNOT_CLAIM: &[&str] = &[
    "reconciler worker availability",
    "adapter-side external execution",
    "tenant workload migration",
    "launch or availability guarantees",
];
const REQUIRED_RESOURCE_TRANSITIONS: &[&str] =
    &["create", "update", "delete", "suspend", "resume", "purge"];
const REQUIRED_ENFORCEABILITY_FACETS: &[&str] = &[
    "cedar_policy",
    "tenant_scope",
    "audit",
    "quota_cost",
    "metering",
    "billing",
];
const REQUIRED_CEDAR_FIELDS: &[&str] = &[
    "principal",
    "action",
    "resource_orn",
    "tenant_account_project",
    "region_cell",
    "policy_snapshot",
    "decision",
    "reason_code",
];
const REQUIRED_AUDIT_FIELDS: &[&str] = &[
    "audit_event_type",
    "audit_chain_id",
    "operation_id",
    "resource_orn",
    "principal",
    "tenant_account_project",
    "region_cell",
    "previous_state",
    "next_state",
    "reason_code",
];
const REQUIRED_BILLING_FIELDS: &[&str] = &[
    "meter_name",
    "unit",
    "aggregation",
    "billing_account",
    "cost_center",
    "currency",
    "rated_usage_ref",
];
const REQUIRED_ENFORCEABILITY_NONCLAIMS: &[&str] = &[
    "no_runtime_policy_engine",
    "no_billing_runtime",
    "no_audit_runtime",
    "no_tenant_workload_readiness",
];
const REQUIRED_SELECTED_SOURCES: &[&str] = &[
    "scripts/tests/cloud_resource_contract_parity_catalog_check.py",
    "scripts/tests/cloud_control_plane_operation_contract_check.py",
    "scripts/tests/cloud_enforceability_facets_check.py",
];
const REQUIRED_CORPUS_KEYS: &[&str] = &[
    "cloud_resource_contract_parity_catalog",
    "cloud_control_plane_operation_contract",
    "cloud_enforceability_facets",
    "cloud_hyperscaler_parity_taxonomy",
    "cloud_resource_catalog_target",
    "cloud_control_plane_canonical",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub key: String,
    pub remediation: String,
}

impl Finding {
    fn new(code: &str, key: impl Into<String>, remediation: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            key: key.into(),
            remediation: remediation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub verdict: Verdict,
    pub violations: BTreeSet<String>,
    pub findings: BTreeSet<Finding>,
}

impl Report {
    fn from_findings(findings: BTreeSet<Finding>) -> Self {
        let violations = findings
            .iter()
            .map(|finding| finding.code.clone())
            .collect();
        let verdict = if findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Self {
            verdict,
            violations,
            findings,
        }
    }
}

pub fn evaluate_configured(policy: &Value, corpus: &Value) -> Report {
    Report::from_findings(evaluate_keyed(policy, corpus))
}

pub fn evaluate_keyed(policy: &Value, corpus: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    validate_policy(policy, &mut findings);
    for key in REQUIRED_CORPUS_KEYS {
        if corpus.get(*key).is_none() {
            findings.insert(Finding::new(
                "cloud_resource_contract_corpus_missing_input",
                *key,
                "provide the configured JSON artifact in the evaluation corpus",
            ));
        }
    }
    if !findings.is_empty() {
        return findings;
    }

    let marker_policy = MarkerPolicy::from_policy(policy);
    validate_resource_catalog(
        &corpus["cloud_resource_contract_parity_catalog"],
        &corpus["cloud_hyperscaler_parity_taxonomy"],
        &corpus["cloud_resource_catalog_target"],
        policy_path(policy, "cloud_hyperscaler_parity_taxonomy"),
        policy_path(policy, "cloud_resource_catalog_target"),
        &marker_policy,
        &mut findings,
    );
    validate_operation_contract(
        &corpus["cloud_control_plane_operation_contract"],
        &corpus["cloud_control_plane_canonical"],
        &corpus["cloud_resource_contract_parity_catalog"],
        policy_path(policy, "cloud_control_plane_canonical"),
        policy_path(policy, "cloud_resource_contract_parity_catalog"),
        &marker_policy,
        &mut findings,
    );
    validate_enforceability(
        &corpus["cloud_enforceability_facets"],
        &corpus["cloud_resource_contract_parity_catalog"],
        &corpus["cloud_control_plane_operation_contract"],
        policy_path(policy, "cloud_resource_contract_parity_catalog"),
        policy_path(policy, "cloud_control_plane_operation_contract"),
        &marker_policy,
        &mut findings,
    );
    findings
}

struct MarkerPolicy {
    forbidden_positive: Vec<String>,
    forbidden_actuation: Vec<String>,
}

impl MarkerPolicy {
    fn from_policy(policy: &Value) -> Self {
        let claim_policy = policy.get("claim_policy").unwrap_or(&Value::Null);
        Self {
            forbidden_positive: string_array_field(
                claim_policy,
                "forbidden_positive_claim_markers",
            ),
            forbidden_actuation: string_array_field(claim_policy, "forbidden_actuation_markers"),
        }
    }
}

fn validate_policy(policy: &Value, findings: &mut BTreeSet<Finding>) {
    require_eq_str(
        policy,
        "gate_id",
        GATE_ID,
        "cloud_resource_contract_policy_gate_id_mismatch",
        "policy.gate_id must match the cloud-ci gate id",
        findings,
    );
    require_eq_str(
        policy,
        "primary_execution_path",
        "rust_buck2_cloud_ci_gate",
        "cloud_resource_contract_policy_primary_path_not_rust",
        "policy must declare the Rust/Buck2 cloud-ci gate as the primary execution path",
        findings,
    );
    let mut legacy_paths = BTreeSet::new();
    for row in array_field(policy, "source_migration_slice") {
        let legacy = string_field(row, "legacy_path").unwrap_or_default();
        let disposition = string_field(row, "disposition").unwrap_or_default();
        let replacement = string_field(row, "replacement_target").unwrap_or_default();
        if legacy.is_empty()
            || disposition != "retired_primary_path"
            || replacement != REPLACEMENT_GATE_TARGET
        {
            findings.insert(Finding::new(
                "cloud_resource_contract_policy_source_migration_row_invalid",
                legacy.to_owned(),
                "each selected legacy source must point at this Rust gate and be marked retired_primary_path",
            ));
        }
        legacy_paths.insert(legacy.to_owned());
    }
    for required in REQUIRED_SELECTED_SOURCES {
        if !legacy_paths.contains(*required) {
            findings.insert(Finding::new(
                "cloud_resource_contract_policy_missing_selected_source",
                *required,
                "policy.source_migration_slice must include the selected P0 Python validator source",
            ));
        }
    }
    for key in REQUIRED_CORPUS_KEYS {
        if policy_path(policy, key).is_none() {
            findings.insert(Finding::new(
                "cloud_resource_contract_policy_missing_spec_input",
                *key,
                "policy.spec_inputs must name every JSON artifact consumed by the Rust gate",
            ));
        }
    }
    let claim_policy = policy.get("claim_policy").unwrap_or(&Value::Null);
    if claim_policy
        .get("metadata_only_required")
        .and_then(Value::as_bool)
        != Some(true)
    {
        findings.insert(Finding::new(
            "cloud_resource_contract_policy_metadata_only_not_required",
            "claim_policy.metadata_only_required",
            "policy must require metadata-only claim controls for target cloud-resource artifacts",
        ));
    }
}

fn validate_resource_catalog(
    spec: &Value,
    taxonomy: &Value,
    base_catalog: &Value,
    taxonomy_path: Option<&str>,
    base_catalog_path: Option<&str>,
    marker_policy: &MarkerPolicy,
    findings: &mut BTreeSet<Finding>,
) {
    for field in [
        "spec_id",
        "title",
        "status",
        "source_taxonomy",
        "base_catalog",
        "claim_controls",
        "required_facets",
        "resource_contracts",
        "category_coverage",
        "nonclaims",
        "next_goal_links",
    ] {
        require_has_field(
            spec,
            field,
            "cloud_resource_contract_missing_top_level_field",
            findings,
        );
    }
    require_eq_str(
        spec,
        "status",
        "Proposed-target",
        "cloud_resource_contract_status_not_proposed_target",
        "resource contract catalog must remain Proposed-target until runtime evidence exists",
        findings,
    );
    require_eq_optional_path(
        spec,
        "source_taxonomy",
        taxonomy_path,
        "cloud_resource_contract_source_taxonomy_mismatch",
        findings,
    );
    require_eq_optional_path(
        spec,
        "base_catalog",
        base_catalog_path,
        "cloud_resource_contract_base_catalog_mismatch",
        findings,
    );
    require_array_superset(
        spec,
        "required_facets",
        REQUIRED_RESOURCE_FACETS,
        "cloud_resource_contract_missing_required_facet",
        findings,
    );
    let controls = spec.get("claim_controls").unwrap_or(&Value::Null);
    require_bool_field(
        controls,
        "no_live_provider_apply",
        true,
        "cloud_resource_contract_allows_live_provider_apply",
        findings,
    );
    require_bool_field(
        controls,
        "strict_separation",
        true,
        "cloud_resource_contract_strict_separation_disabled",
        findings,
    );
    require_bool_field(
        controls,
        "pure_dogfood",
        true,
        "cloud_resource_contract_pure_dogfood_disabled",
        findings,
    );
    require_bool_field(
        controls,
        "metadata_only",
        true,
        "cloud_resource_contract_not_metadata_only",
        findings,
    );
    forbid_text_markers(
        controls.get("can_claim_now"),
        &marker_policy.forbidden_positive,
        "cloud_resource_contract_forbidden_positive_claim",
        "claim_controls.can_claim_now",
        findings,
    );

    let taxonomy_ids = set_from_array_path(taxonomy, &["category_taxonomy"], "id");
    for category in REQUIRED_CATEGORY_IDS {
        if !taxonomy_ids.contains(*category) {
            findings.insert(Finding::new(
                "cloud_resource_contract_taxonomy_missing_required_category",
                *category,
                "source taxonomy must contain every required hyperscaler category",
            ));
        }
    }
    let existing_services = set_from_array_path(base_catalog, &["services"], "service");
    let contracts = array_field(spec, "resource_contracts");
    if contracts.is_empty() {
        findings.insert(Finding::new(
            "cloud_resource_contract_empty_contracts",
            "resource_contracts",
            "resource_contracts must be a non-empty list",
        ));
    }
    let mut contract_ids = BTreeSet::new();
    let mut categories_by_contract = BTreeSet::new();
    for contract in contracts {
        let cid = string_field(contract, "id").unwrap_or_default();
        if cid.is_empty() || !contract_ids.insert(cid.to_owned()) {
            findings.insert(Finding::new(
                "cloud_resource_contract_duplicate_or_missing_id",
                cid.to_owned(),
                "each resource contract must have a unique non-empty id",
            ));
        }
        let category = string_field(contract, "category_id").unwrap_or_default();
        if !taxonomy_ids.contains(category) {
            findings.insert(Finding::new(
                "cloud_resource_contract_invalid_category",
                format!("{cid}:{category}"),
                "resource contract category_id must exist in the source taxonomy",
            ));
        }
        if !category.is_empty() {
            categories_by_contract.insert(category.to_owned());
        }
        let service = string_field(contract, "service").unwrap_or_default();
        if !existing_services.contains(service) && service != "cloud-marketplace" {
            findings.insert(Finding::new(
                "cloud_resource_contract_unknown_service",
                format!("{cid}:{service}"),
                "resource contract service must reference the base catalog or scoped cloud-marketplace target",
            ));
        }
        require_array_superset_on(
            contract,
            "facets",
            REQUIRED_RESOURCE_FACETS,
            "cloud_resource_contract_row_missing_required_facet",
            cid,
            findings,
        );
        if !string_field(contract, "orn_pattern")
            .unwrap_or_default()
            .starts_with("orn:oyatie:cloud:")
        {
            findings.insert(Finding::new(
                "cloud_resource_contract_invalid_orn_pattern",
                cid.to_owned(),
                "ORN pattern must use orn:oyatie:cloud prefix",
            ));
        }
        for required in [
            "lifecycle_states",
            "quota_cost",
            "billing_meters",
            "audit_events",
            "slo_tier",
            "deletion_retention",
            "owner",
            "tenant_account_project",
            "region_cell",
        ] {
            if value_is_empty(contract.get(required)) {
                findings.insert(Finding::new(
                    "cloud_resource_contract_row_missing_field",
                    format!("{cid}.{required}"),
                    "resource contract row must carry every required cloud-resource facet",
                ));
            }
        }
        if !matches!(
            string_field(contract, "actuation_status"),
            Some("metadata_only" | "adapter_boundary_only" | "evidence_required")
        ) {
            findings.insert(Finding::new(
                "cloud_resource_contract_invalid_actuation_status",
                cid.to_owned(),
                "actuation_status must be metadata_only, adapter_boundary_only, or evidence_required",
            ));
        }
        forbid_text_markers(
            Some(contract),
            &marker_policy.forbidden_actuation,
            "cloud_resource_contract_forbidden_actuation_marker",
            cid,
            findings,
        );
        forbid_text_markers(
            contract.get("honest_claim"),
            &marker_policy.forbidden_positive,
            "cloud_resource_contract_forbidden_positive_claim",
            cid,
            findings,
        );
        require_set_superset(
            &string_set_field(contract, "blocked_claim_families"),
            REQUIRED_RESOURCE_NONCLAIMS,
            "cloud_resource_contract_missing_blocked_claim_family",
            cid,
            findings,
        );
    }
    for category in &taxonomy_ids {
        if !categories_by_contract.contains(category.as_str()) {
            findings.insert(Finding::new(
                "cloud_resource_contract_missing_category_contract",
                category.clone(),
                "every taxonomy category must have at least one resource contract",
            ));
        }
    }
    let coverage = object_field(spec, "category_coverage");
    for category in &taxonomy_ids {
        let Some(row) = coverage.get(category) else {
            findings.insert(Finding::new(
                "cloud_resource_contract_missing_category_coverage",
                category.clone(),
                "category_coverage must cover every taxonomy category",
            ));
            continue;
        };
        if array_field(row, "resource_contract_ids").is_empty() {
            findings.insert(Finding::new(
                "cloud_resource_contract_category_coverage_empty_contracts",
                category.clone(),
                "category coverage must list covered resource_contract_ids",
            ));
        }
        for contract_id in string_array_field(row, "resource_contract_ids") {
            if !contract_ids.contains(contract_id.as_str()) {
                findings.insert(Finding::new(
                    "cloud_resource_contract_category_coverage_unknown_contract",
                    format!("{category}:{contract_id}"),
                    "category coverage must reference known contract ids",
                ));
            }
        }
        if !matches!(
            string_field(row, "claim_status"),
            Some("metadata_only" | "target_spec_only" | "evidence_required")
        ) {
            findings.insert(Finding::new(
                "cloud_resource_contract_category_coverage_invalid_claim_status",
                category.clone(),
                "category coverage claim_status must be metadata_only, target_spec_only, or evidence_required",
            ));
        }
        require_set_superset(
            &string_set_field(row, "blocked_claim_families"),
            REQUIRED_RESOURCE_NONCLAIMS,
            "cloud_resource_contract_missing_blocked_claim_family",
            category,
            findings,
        );
        let mut positive_scan = (*row).clone();
        if let Some(obj) = positive_scan.as_object_mut() {
            obj.remove("blocked_claim_families");
        }
        forbid_text_markers(
            Some(&positive_scan),
            &marker_policy.forbidden_positive,
            "cloud_resource_contract_forbidden_positive_claim",
            category,
            findings,
        );
    }
    let nonclaims = set_from_array_path(spec, &["nonclaims"], "id");
    require_set_superset(
        &nonclaims,
        REQUIRED_RESOURCE_NONCLAIMS,
        "cloud_resource_contract_missing_nonclaim",
        "nonclaims",
        findings,
    );
    for item in array_field(spec, "nonclaims") {
        let id = string_field(item, "id").unwrap_or("nonclaim");
        forbid_text_markers(
            item.get("statement"),
            &marker_policy.forbidden_positive,
            "cloud_resource_contract_forbidden_positive_claim",
            id,
            findings,
        );
    }
    let next_values = object_field(spec, "next_goal_links")
        .values()
        .filter_map(|value| value.as_str())
        .collect::<BTreeSet<_>>();
    require_set_superset_refs(
        &next_values,
        &["G003", "G004", "G005", "G006", "G007"],
        "cloud_resource_contract_missing_next_goal_link",
        "next_goal_links",
        findings,
    );
}

fn validate_operation_contract(
    spec: &Value,
    control_plane: &Value,
    resource_catalog: &Value,
    control_plane_path: Option<&str>,
    resource_catalog_path: Option<&str>,
    marker_policy: &MarkerPolicy,
    findings: &mut BTreeSet<Finding>,
) {
    for field in [
        "spec_id",
        "title",
        "status",
        "source_control_plane",
        "source_resource_catalog",
        "claim_controls",
        "pipeline",
        "resource_registry_entry",
        "operation_ledger_entry",
        "operation_state_machine",
        "idempotency_retry_cancel_contract",
        "resource_state_transition_contract",
        "nonclaims",
        "next_goal_links",
    ] {
        require_has_field(
            spec,
            field,
            "cloud_operation_missing_top_level_field",
            findings,
        );
    }
    require_eq_str(
        spec,
        "status",
        "Proposed-target",
        "cloud_operation_status_not_proposed_target",
        "operation contract must remain Proposed-target",
        findings,
    );
    require_eq_optional_path(
        spec,
        "source_control_plane",
        control_plane_path,
        "cloud_operation_source_control_plane_mismatch",
        findings,
    );
    require_eq_optional_path(
        spec,
        "source_resource_catalog",
        resource_catalog_path,
        "cloud_operation_source_resource_catalog_mismatch",
        findings,
    );
    let controls = spec.get("claim_controls").unwrap_or(&Value::Null);
    for (field, code) in [
        ("metadata_only", "cloud_operation_not_metadata_only"),
        ("no_provider_apply", "cloud_operation_allows_provider_apply"),
        (
            "no_runtime_reconciler_claim",
            "cloud_operation_allows_runtime_reconciler_claim",
        ),
        (
            "strict_separation",
            "cloud_operation_strict_separation_disabled",
        ),
        ("pure_dogfood", "cloud_operation_pure_dogfood_disabled"),
    ] {
        require_bool_field(controls, field, true, code, findings);
    }
    forbid_text_markers(
        controls.get("can_claim_now"),
        &marker_policy.forbidden_positive,
        "cloud_operation_forbidden_positive_claim",
        "claim_controls.can_claim_now",
        findings,
    );
    require_set_superset(
        &string_set_field(controls, "cannot_claim_yet"),
        REQUIRED_CANNOT_CLAIM,
        "cloud_operation_missing_cannot_claim",
        "claim_controls.cannot_claim_yet",
        findings,
    );
    forbid_text_markers(
        Some(spec),
        &marker_policy.forbidden_actuation,
        "cloud_operation_forbidden_actuation_marker",
        "operation_contract",
        findings,
    );
    let stage_ids = array_field(spec.get("pipeline").unwrap_or(&Value::Null), "stages")
        .into_iter()
        .filter_map(|stage| string_field(stage, "id"))
        .collect::<Vec<_>>();
    if stage_ids != REQUIRED_OPERATION_STAGES {
        findings.insert(Finding::new(
            "cloud_operation_pipeline_stage_order_invalid",
            "pipeline.stages",
            "pipeline stages/order must match the control-plane sequence",
        ));
    }
    let registry = spec.get("resource_registry_entry").unwrap_or(&Value::Null);
    for field in [
        "resource_orn",
        "resource_type",
        "desired_spec",
        "desired_generation",
        "observed_generation",
        "tenant_account_project",
        "region_cell",
        "owner",
        "deletion_retention",
        "slo_tier",
        "policy_snapshot",
        "quota_snapshot",
        "billing_meter_bindings",
        "audit_event_bindings",
    ] {
        if !string_set_field(registry, "required_fields").contains(field) {
            findings.insert(Finding::new(
                "cloud_operation_resource_registry_missing_field",
                field,
                "resource registry must require the cloud control-plane field",
            ));
        }
    }
    for block in ["generation_rules", "identity_rules"] {
        if value_is_empty(registry.get(block)) {
            findings.insert(Finding::new(
                "cloud_operation_resource_registry_missing_rule_block",
                block,
                "resource registry must define generation and identity semantics",
            ));
        }
    }
    let ledger = spec.get("operation_ledger_entry").unwrap_or(&Value::Null);
    require_set_superset(
        &string_set_field(ledger, "required_fields"),
        REQUIRED_OPERATION_FIELDS,
        "cloud_operation_ledger_missing_field",
        "operation_ledger_entry.required_fields",
        findings,
    );
    if value_is_empty(ledger.get("identity")) {
        findings.insert(Finding::new(
            "cloud_operation_ledger_missing_identity",
            "operation_ledger_entry.identity",
            "operation ledger must define identity semantics",
        ));
    }
    require_bool_path(
        ledger,
        &["durability", "write_before_ack"],
        true,
        "cloud_operation_ledger_not_write_before_ack",
        findings,
    );
    require_bool_path(
        ledger,
        &["durability", "audit_chain_required"],
        true,
        "cloud_operation_ledger_audit_chain_not_required",
        findings,
    );
    let machine = spec.get("operation_state_machine").unwrap_or(&Value::Null);
    require_set_superset(
        &string_set_field(machine, "states"),
        REQUIRED_OPERATION_STATES,
        "cloud_operation_state_machine_missing_state",
        "operation_state_machine.states",
        findings,
    );
    let transitions = transition_pairs(machine.get("allowed_transitions"));
    if transitions.is_empty() {
        findings.insert(Finding::new(
            "cloud_operation_state_machine_missing_transitions",
            "operation_state_machine.allowed_transitions",
            "operation state machine must define allowed transitions",
        ));
    }
    if value_is_empty(machine.get("transition_rules")) {
        findings.insert(Finding::new(
            "cloud_operation_state_machine_missing_transition_rules",
            "operation_state_machine.transition_rules",
            "operation state machine must define transition rules",
        ));
    }
    let terminal_states = string_set_field(machine, "terminal_states");
    if terminal_states.is_empty() {
        findings.insert(Finding::new(
            "cloud_operation_state_machine_missing_terminal_states",
            "operation_state_machine.terminal_states",
            "operation state machine must define terminal states",
        ));
    }
    for (source, _) in transitions
        .iter()
        .filter(|(source, _)| terminal_states.contains(source))
    {
        findings.insert(Finding::new(
            "cloud_operation_terminal_state_has_outgoing_transition",
            source.clone(),
            "terminal operation states must not have outgoing transitions",
        ));
    }
    let reachable = reachable_states(&transitions, "accepted");
    for state in REQUIRED_OPERATION_STATES {
        if !reachable.contains(*state) {
            findings.insert(Finding::new(
                "cloud_operation_state_unreachable_from_accepted",
                *state,
                "operation states must be reachable from accepted",
            ));
        }
    }
    require_transition(
        &transitions,
        "running",
        "compensating",
        "cloud_operation_missing_compensation_transition",
        findings,
    );
    require_transition(
        &transitions,
        "compensating",
        "rolled_back",
        "cloud_operation_missing_rollback_transition",
        findings,
    );
    let retry_cancel = spec
        .get("idempotency_retry_cancel_contract")
        .unwrap_or(&Value::Null);
    require_set_superset(
        &string_set_field(retry_cancel, "required_semantics"),
        REQUIRED_OPERATION_SEMANTICS,
        "cloud_operation_missing_required_semantic",
        "idempotency_retry_cancel_contract.required_semantics",
        findings,
    );
    for block in [
        "idempotency",
        "retry_policy",
        "cancellation",
        "compensation",
    ] {
        if value_is_empty(retry_cancel.get(block)) {
            findings.insert(Finding::new(
                "cloud_operation_missing_retry_cancel_block",
                block,
                "idempotency/retry/cancel contract must define every required block",
            ));
        }
    }
    let cancellation = retry_cancel.get("cancellation").unwrap_or(&Value::Null);
    for state in string_set_field(cancellation, "accepted_states") {
        if !transitions.contains(&(state.clone(), "cancel_requested".to_owned())) {
            findings.insert(Finding::new(
                "cloud_operation_cancellable_state_missing_cancel_requested_transition",
                state,
                "every cancellable state must transition to cancel_requested",
            ));
        }
    }
    for state in string_set_field(cancellation, "terminal_result") {
        if !transitions.contains(&("cancel_requested".to_owned(), state.clone())) {
            findings.insert(Finding::new(
                "cloud_operation_cancel_requested_missing_terminal_transition",
                state,
                "cancel_requested must transition to every configured terminal result",
            ));
        }
    }
    let transition_contract = spec
        .get("resource_state_transition_contract")
        .unwrap_or(&Value::Null);
    require_set_superset(
        &string_set_field(transition_contract, "verbs"),
        REQUIRED_RESOURCE_TRANSITIONS,
        "cloud_operation_missing_resource_transition_verb",
        "resource_state_transition_contract.verbs",
        findings,
    );
    let catalog_ids = set_from_array_path(resource_catalog, &["resource_contracts"], "id");
    require_set_superset_owned(
        &string_set_field(transition_contract, "applies_to_resource_contract_ids"),
        &catalog_ids,
        "cloud_operation_transition_contract_missing_resource_contract",
        "resource_state_transition_contract.applies_to_resource_contract_ids",
        findings,
    );
    let nonclaim_ids = set_from_array_path(spec, &["nonclaims"], "id");
    require_set_superset_refs(
        &nonclaim_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        &[
            "no_provider_apply",
            "no_runtime_reconciler",
            "no_production_readiness",
            "no_tenant_workload_readiness",
        ],
        "cloud_operation_missing_nonclaim",
        "nonclaims",
        findings,
    );
    require_eq_str(
        control_plane,
        "spec_id",
        "EXE-CLOUD-CONTROL-PLANE-CANONICAL",
        "cloud_operation_unexpected_control_plane_source",
        "source control-plane artifact must be the canonical control-plane spec",
        findings,
    );
    let next = object_field(spec, "next_goal_links");
    for (key, expected) in [
        ("authz_tenancy_audit_metering_billing", "G004"),
        ("observability_slo_evidence", "G005"),
        ("production_quality_kits", "G006"),
    ] {
        if next.get(key).and_then(|value| value.as_str()) != Some(expected) {
            findings.insert(Finding::new(
                "cloud_operation_missing_next_goal_link",
                key,
                "operation contract must route follow-on goals to the required goal ids",
            ));
        }
    }
}

fn validate_enforceability(
    spec: &Value,
    resource_catalog: &Value,
    operation: &Value,
    resource_catalog_path: Option<&str>,
    operation_path: Option<&str>,
    marker_policy: &MarkerPolicy,
    findings: &mut BTreeSet<Finding>,
) {
    for field in [
        "spec_id",
        "title",
        "status",
        "source_resource_catalog",
        "source_operation_contract",
        "claim_controls",
        "facet_vocabulary",
        "resource_enforceability",
        "integration_points",
        "nonclaims",
        "next_goal_links",
    ] {
        require_has_field(
            spec,
            field,
            "cloud_enforceability_missing_top_level_field",
            findings,
        );
    }
    require_eq_str(
        spec,
        "status",
        "Proposed-target",
        "cloud_enforceability_status_not_proposed_target",
        "enforceability facets must remain Proposed-target",
        findings,
    );
    require_eq_optional_path(
        spec,
        "source_resource_catalog",
        resource_catalog_path,
        "cloud_enforceability_source_resource_catalog_mismatch",
        findings,
    );
    require_eq_optional_path(
        spec,
        "source_operation_contract",
        operation_path,
        "cloud_enforceability_source_operation_contract_mismatch",
        findings,
    );
    let controls = spec.get("claim_controls").unwrap_or(&Value::Null);
    for (field, code) in [
        ("metadata_only", "cloud_enforceability_not_metadata_only"),
        (
            "no_runtime_policy_engine",
            "cloud_enforceability_allows_runtime_policy_engine",
        ),
        (
            "no_billing_runtime",
            "cloud_enforceability_allows_billing_runtime",
        ),
        (
            "no_audit_runtime",
            "cloud_enforceability_allows_audit_runtime",
        ),
        (
            "strict_separation",
            "cloud_enforceability_strict_separation_disabled",
        ),
        ("pure_dogfood", "cloud_enforceability_pure_dogfood_disabled"),
        (
            "no_tenant_workload_readiness",
            "cloud_enforceability_allows_tenant_workload_readiness",
        ),
    ] {
        require_bool_field(controls, field, true, code, findings);
    }
    forbid_text_markers(
        controls.get("can_claim_now"),
        &marker_policy.forbidden_positive,
        "cloud_enforceability_forbidden_positive_claim",
        "claim_controls.can_claim_now",
        findings,
    );
    let vocab = spec.get("facet_vocabulary").unwrap_or(&Value::Null);
    require_set_superset(
        &string_set_field(vocab, "required_facets"),
        REQUIRED_ENFORCEABILITY_FACETS,
        "cloud_enforceability_vocab_missing_required_facet",
        "facet_vocabulary.required_facets",
        findings,
    );
    require_set_superset(
        &string_set_field(vocab, "cedar_decision_fields"),
        REQUIRED_CEDAR_FIELDS,
        "cloud_enforceability_vocab_missing_cedar_field",
        "facet_vocabulary.cedar_decision_fields",
        findings,
    );
    require_set_superset(
        &string_set_field(vocab, "audit_event_fields"),
        REQUIRED_AUDIT_FIELDS,
        "cloud_enforceability_vocab_missing_audit_field",
        "facet_vocabulary.audit_event_fields",
        findings,
    );
    require_set_superset(
        &string_set_field(vocab, "billing_meter_fields"),
        REQUIRED_BILLING_FIELDS,
        "cloud_enforceability_vocab_missing_billing_field",
        "facet_vocabulary.billing_meter_fields",
        findings,
    );
    let contract_ids = set_from_array_path(resource_catalog, &["resource_contracts"], "id");
    let rows = array_field(spec, "resource_enforceability");
    if rows.is_empty() {
        findings.insert(Finding::new(
            "cloud_enforceability_empty_rows",
            "resource_enforceability",
            "resource_enforceability must be non-empty",
        ));
    }
    let row_ids = rows
        .iter()
        .filter_map(|row| string_field(row, "resource_contract_id"))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    require_set_superset_owned(
        &row_ids,
        &contract_ids,
        "cloud_enforceability_missing_contract_row",
        "resource_enforceability",
        findings,
    );
    for row in rows {
        let cid = string_field(row, "resource_contract_id").unwrap_or_default();
        if !contract_ids.contains(cid) {
            findings.insert(Finding::new(
                "cloud_enforceability_unknown_contract_id",
                cid.to_owned(),
                "resource_enforceability row must reference a known resource contract",
            ));
        }
        require_array_superset_on(
            row,
            "facets",
            REQUIRED_ENFORCEABILITY_FACETS,
            "cloud_enforceability_row_missing_required_facet",
            cid,
            findings,
        );
        let cedar = row.get("cedar_policy").unwrap_or(&Value::Null);
        require_set_superset(
            &string_set_field(cedar, "decision_fields"),
            REQUIRED_CEDAR_FIELDS,
            "cloud_enforceability_cedar_missing_decision_field",
            cid,
            findings,
        );
        require_eq_str(
            cedar,
            "default",
            "deny",
            "cloud_enforceability_cedar_not_default_deny",
            "Cedar policy default must be deny",
            findings,
        );
        let tenant = row.get("tenant_scope").unwrap_or(&Value::Null);
        require_bool_field(
            tenant,
            "required",
            true,
            "cloud_enforceability_tenant_scope_not_required",
            findings,
        );
        if string_array_field(tenant, "fields") != ["tenant", "account", "project"] {
            findings.insert(Finding::new(
                "cloud_enforceability_tenant_scope_fields_invalid",
                cid.to_owned(),
                "tenant scope must require tenant/account/project fields",
            ));
        }
        let audit = row.get("audit").unwrap_or(&Value::Null);
        require_set_superset(
            &string_set_field(audit, "event_fields"),
            REQUIRED_AUDIT_FIELDS,
            "cloud_enforceability_audit_missing_event_field",
            cid,
            findings,
        );
        require_eq_str(
            row.get("quota_cost").unwrap_or(&Value::Null),
            "admission",
            "fail_closed",
            "cloud_enforceability_quota_not_fail_closed",
            "quota/cost admission must fail closed",
            findings,
        );
        for (block, integration) in [("metering", "oya-meter"), ("billing", "oya-billing")] {
            let obj = row.get(block).unwrap_or(&Value::Null);
            require_set_superset(
                &string_set_field(obj, "fields"),
                REQUIRED_BILLING_FIELDS,
                "cloud_enforceability_billing_meter_missing_field",
                format!("{cid}.{block}"),
                findings,
            );
            require_eq_str(
                obj,
                "integration_point",
                integration,
                "cloud_enforceability_integration_point_invalid",
                format!("{block} integration point must be {integration}"),
                findings,
            );
            require_eq_str(
                obj,
                "runtime_status",
                "integration_point_only",
                "cloud_enforceability_runtime_status_not_integration_point_only",
                format!("{block} runtime_status must remain integration_point_only"),
                findings,
            );
        }
        require_set_superset(
            &string_set_field(row, "blocked_claim_families"),
            REQUIRED_ENFORCEABILITY_NONCLAIMS,
            "cloud_enforceability_missing_blocked_claim_family",
            cid,
            findings,
        );
        forbid_text_markers(
            row.get("honest_claim"),
            &marker_policy.forbidden_positive,
            "cloud_enforceability_forbidden_positive_claim",
            cid,
            findings,
        );
    }
    let integrations = spec.get("integration_points").unwrap_or(&Value::Null);
    for (name, expected_runtime) in [
        ("cedar", "metadata_contract_only"),
        ("tenancy", "integration_point_only"),
        ("audit", "integration_point_only"),
        ("metering", "integration_point_only"),
        ("billing", "integration_point_only"),
    ] {
        if integrations.get(name).is_none() {
            findings.insert(Finding::new(
                "cloud_enforceability_missing_integration_point",
                name,
                "all enforceability integration points must be declared",
            ));
            continue;
        }
        require_eq_str(
            integrations.get(name).unwrap_or(&Value::Null),
            "runtime_status",
            expected_runtime,
            "cloud_enforceability_integration_runtime_status_invalid",
            format!("{name} runtime_status must stay {expected_runtime}"),
            findings,
        );
    }
    let nonclaim_ids = set_from_array_path(spec, &["nonclaims"], "id");
    require_set_superset(
        &nonclaim_ids,
        REQUIRED_ENFORCEABILITY_NONCLAIMS,
        "cloud_enforceability_missing_nonclaim",
        "nonclaims",
        findings,
    );
    for item in array_field(spec, "nonclaims") {
        let id = string_field(item, "id").unwrap_or("nonclaim");
        forbid_text_markers(
            item.get("statement"),
            &marker_policy.forbidden_positive,
            "cloud_enforceability_forbidden_positive_claim",
            id,
            findings,
        );
    }
    require_eq_str(
        operation,
        "spec_id",
        "EXE-CLOUD-CONTROL-PLANE-OPERATION-CONTRACT",
        "cloud_enforceability_unexpected_operation_source",
        "source operation artifact must be the cloud control-plane operation contract",
        findings,
    );
    let next = object_field(spec, "next_goal_links");
    for (key, expected) in [
        ("observability_slo_evidence", "G005"),
        ("production_quality_kits", "G006"),
        ("dogfood_ci_lane", "G007"),
    ] {
        if next.get(key).and_then(|value| value.as_str()) != Some(expected) {
            findings.insert(Finding::new(
                "cloud_enforceability_missing_next_goal_link",
                key,
                "enforceability facets must route follow-on goals to the required ids",
            ));
        }
    }
}

fn require_has_field(value: &Value, field: &str, code: &str, findings: &mut BTreeSet<Finding>) {
    if value.get(field).is_none() {
        findings.insert(Finding::new(
            code,
            field,
            "required top-level field is absent",
        ));
    }
}

fn require_eq_str(
    value: &Value,
    field: &str,
    expected: &str,
    code: &str,
    remediation: impl Into<String>,
    findings: &mut BTreeSet<Finding>,
) {
    if string_field(value, field) != Some(expected) {
        findings.insert(Finding::new(code, field, remediation));
    }
}

fn require_eq_optional_path(
    value: &Value,
    field: &str,
    expected: Option<&str>,
    code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if let Some(expected) = expected
        && string_field(value, field) != Some(expected)
    {
        findings.insert(Finding::new(
            code,
            field,
            format!("field must point at configured artifact path {expected}"),
        ));
    }
}

fn require_bool_field(
    value: &Value,
    field: &str,
    expected: bool,
    code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if value.get(field).and_then(Value::as_bool) != Some(expected) {
        findings.insert(Finding::new(
            code,
            field,
            format!("boolean field {field} must be {expected}"),
        ));
    }
}

fn require_bool_path(
    value: &Value,
    path: &[&str],
    expected: bool,
    code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    let mut current = value;
    for segment in path {
        current = current.get(*segment).unwrap_or(&Value::Null);
    }
    if current.as_bool() != Some(expected) {
        findings.insert(Finding::new(
            code,
            path.join("."),
            format!("boolean path must be {expected}"),
        ));
    }
}

fn require_array_superset(
    value: &Value,
    field: &str,
    required: &[&str],
    code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    require_set_superset(
        &string_set_field(value, field),
        required,
        code,
        field,
        findings,
    );
}

fn require_array_superset_on(
    value: &Value,
    field: &str,
    required: &[&str],
    code: &str,
    prefix: &str,
    findings: &mut BTreeSet<Finding>,
) {
    require_set_superset(
        &string_set_field(value, field),
        required,
        code,
        prefix,
        findings,
    );
}

fn require_set_superset(
    actual: &BTreeSet<String>,
    required: &[&str],
    code: &str,
    key_prefix: impl AsRef<str>,
    findings: &mut BTreeSet<Finding>,
) {
    let prefix = key_prefix.as_ref();
    for item in required {
        if !actual.contains(*item) {
            findings.insert(Finding::new(
                code,
                format!("{prefix}:{item}"),
                "required value missing from configured set",
            ));
        }
    }
}

fn require_set_superset_refs(
    actual: &BTreeSet<&str>,
    required: &[&str],
    code: &str,
    key_prefix: impl AsRef<str>,
    findings: &mut BTreeSet<Finding>,
) {
    let prefix = key_prefix.as_ref();
    for item in required {
        if !actual.contains(item) {
            findings.insert(Finding::new(
                code,
                format!("{prefix}:{item}"),
                "required value missing from configured set",
            ));
        }
    }
}

fn require_set_superset_owned(
    actual: &BTreeSet<String>,
    required: &BTreeSet<String>,
    code: &str,
    key_prefix: impl AsRef<str>,
    findings: &mut BTreeSet<Finding>,
) {
    let prefix = key_prefix.as_ref();
    for item in required {
        if !actual.contains(item) {
            findings.insert(Finding::new(
                code,
                format!("{prefix}:{item}"),
                "required value missing from configured set",
            ));
        }
    }
}

fn require_transition(
    transitions: &[(String, String)],
    source: &str,
    target: &str,
    code: &str,
    findings: &mut BTreeSet<Finding>,
) {
    if !transitions
        .iter()
        .any(|(actual_source, actual_target)| actual_source == source && actual_target == target)
    {
        findings.insert(Finding::new(
            code,
            format!("{source}->{target}"),
            "required operation state transition is absent",
        ));
    }
}

fn forbid_text_markers(
    value: Option<&Value>,
    markers: &[String],
    code: &str,
    key: impl Into<String>,
    findings: &mut BTreeSet<Finding>,
) {
    if let Some(value) = value
        && contains_marker(value, markers)
    {
        findings.insert(Finding::new(
            code,
            key,
            "remove forbidden readiness/parity/actuation wording or put it only in explicit blocked-claim fields",
        ));
    }
}

fn contains_marker(value: &Value, markers: &[String]) -> bool {
    let haystack = format!(" {} ", normalized_value(value));
    markers.iter().any(|marker| {
        let needle = format!(" {} ", normalize(marker));
        haystack.contains(&needle)
    })
}

fn normalized_value(value: &Value) -> String {
    normalize(&flatten_text(value))
}

fn flatten_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.to_lowercase(),
        Value::Array(values) => values
            .iter()
            .map(flatten_text)
            .collect::<Vec<_>>()
            .join(" "),
        Value::Object(map) => map.values().map(flatten_text).collect::<Vec<_>>().join(" "),
    }
}

fn normalize(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut last_space = true;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    out.trim().to_owned()
}

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str)
}

fn string_array_field(value: &Value, field: &str) -> Vec<String> {
    array_field(value, field)
        .into_iter()
        .filter_map(|value| value.as_str())
        .map(str::to_owned)
        .collect()
}

fn string_set_field(value: &Value, field: &str) -> BTreeSet<String> {
    string_array_field(value, field).into_iter().collect()
}

fn array_field<'a>(value: &'a Value, field: &str) -> Vec<&'a Value> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}

fn object_field<'a>(value: &'a Value, field: &str) -> BTreeMap<String, &'a Value> {
    value
        .get(field)
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .map(|(key, value)| (key.clone(), value))
                .collect()
        })
        .unwrap_or_default()
}

fn policy_path<'a>(policy: &'a Value, key: &str) -> Option<&'a str> {
    policy.get("spec_inputs")?.get(key)?.as_str()
}

fn set_from_array_path(value: &Value, path: &[&str], field: &str) -> BTreeSet<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment).unwrap_or(&Value::Null);
    }
    current
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|row| string_field(row, field))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn transition_pairs(value: Option<&Value>) -> Vec<(String, String)> {
    value
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    let items = row.as_array()?;
                    if items.len() != 2 {
                        return None;
                    }
                    Some((items[0].as_str()?.to_owned(), items[1].as_str()?.to_owned()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn reachable_states(transitions: &[(String, String)], start: &str) -> BTreeSet<String> {
    let mut reachable = BTreeSet::from([start.to_owned()]);
    let mut changed = true;
    while changed {
        changed = false;
        for (source, target) in transitions {
            if reachable.contains(source) && !reachable.contains(target) {
                reachable.insert(target.clone());
                changed = true;
            }
        }
    }
    reachable
}

fn value_is_empty(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => true,
        Some(Value::String(v)) => v.is_empty(),
        Some(Value::Array(v)) => v.is_empty(),
        Some(Value::Object(v)) => v.is_empty(),
        Some(Value::Bool(_)) | Some(Value::Number(_)) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn configured_policy_must_select_rust_primary_path() {
        let policy = json!({
            "gate_id": GATE_ID,
            "primary_execution_path": "python_script",
            "source_migration_slice": [],
            "spec_inputs": {},
            "claim_policy": {"metadata_only_required": true}
        });
        let findings = evaluate_keyed(&policy, &json!({}));
        assert!(
            findings
                .iter()
                .any(|finding| finding.code
                    == "cloud_resource_contract_policy_primary_path_not_rust")
        );
    }

    #[test]
    fn source_migration_replacement_target_must_name_buck_gate() {
        let policy = json!({
            "gate_id": GATE_ID,
            "primary_execution_path": "rust_buck2_cloud_ci_gate",
            "source_migration_slice": [{
                "legacy_path": "scripts/tests/cloud_resource_contract_parity_catalog_check.py",
                "disposition": "retired_primary_path",
                "replacement_target": REPLACEMENT_GATE_TARGET
            }],
            "spec_inputs": {
                "cloud_resource_contract_parity_catalog": "specs/cloud-resource-contract-parity-catalog.json",
                "cloud_control_plane_operation_contract": "specs/cloud-control-plane-operation-contract.json",
                "cloud_enforceability_facets": "specs/cloud-enforceability-facets.json",
                "cloud_hyperscaler_parity_taxonomy": "specs/cloud-hyperscaler-parity-taxonomy.json",
                "cloud_resource_catalog_target": "specs/cloud-resource-catalog-target.json",
                "cloud_control_plane_canonical": "specs/cloud-control-plane-canonical.json"
            },
            "claim_policy": {"metadata_only_required": true}
        });
        let findings = evaluate_keyed(&policy, &json!({}));
        assert!(
            !findings.iter().any(|finding| {
                finding.code == "cloud_resource_contract_policy_source_migration_row_invalid"
            }),
            "expected valid replacement_target, got {findings:?}"
        );
    }

    #[test]
    fn source_migration_replacement_target_rejects_stale_gate_label() {
        let policy = json!({
            "gate_id": GATE_ID,
            "primary_execution_path": "rust_buck2_cloud_ci_gate",
            "source_migration_slice": [{
                "legacy_path": "scripts/tests/cloud_resource_contract_parity_catalog_check.py",
                "disposition": "retired_primary_path",
                "replacement_target": "//ci/facade/resource-contract-conformance:oya-cloud-ci-cloud-resource-contracts-app-gate"
            }],
            "spec_inputs": {},
            "claim_policy": {"metadata_only_required": true}
        });
        let findings = evaluate_keyed(&policy, &json!({}));
        assert!(findings.iter().any(|finding| {
            finding.code == "cloud_resource_contract_policy_source_migration_row_invalid"
        }));
    }

    #[test]
    fn marker_detection_is_word_boundary_normalized() {
        let markers = vec!["feature parity".to_owned()];
        assert!(contains_marker(
            &json!({"claim": "Feature-parity with AWS"}),
            &markers
        ));
        assert!(!contains_marker(
            &json!({"claim": "features are compared in a parity matrix"}),
            &markers
        ));
    }
}
