//! FD-001 tenant workload runtime evidence contract for Oyatie Cloud.
//!
//! This review-only crate records the per-workload runtime evidence that must
//! exist before FD-001 Tenant RBAC, HR, Payroll, and Accounting services
//! can claim they are running as real tenant workloads on the future Oyatie
//! Cloud substrate. It validates Kubernetes/Gateway/OpenTelemetry evidence
//! requirements against the existing tenant-workload manifest. It does not
//! create namespaces, apply quotas or network policies, attach a Gateway
//! controller, deploy workloads, attach a cloud substrate runtime, emit runtime
//! audit-chain events, or claim production workload evidence.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use iam_tenant_rbac_tenant_workload_manifest::{
    Fd001TenantWorkloadManifestError, fd001_tenant_workload_manifest,
    validate_fd001_tenant_workload_manifest,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_REQUIREMENT_COUNT: usize = 14;
const PLAN_NAME: &str = "fd001-tenant-workload-runtime-evidence-plan";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const TENANT_CELL_ID: &str = "cell-us-east-001";
const RESIDENCY_REGION: &str = "us-east-1";
const OTEL_SERVICE_NAMESPACE: &str = "fd001-tenant-rbac";
const TENANT_CLAIM: &str = "tenant_id";
const SOURCE_MANIFEST_REF: &str =
    "crates/tenant-rbac-tenant-workload-manifest/src/lib.rs::fd001_tenant_workload_manifest";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TenantWorkloadRuntimeEvidenceRequirementKind {
    NamespaceObserved,
    ResourceQuotaUsageObserved,
    NetworkPolicyDefaultDenyObserved,
    ServiceAccountBoundaryObserved,
    PodSecurityContextObserved,
    WorkloadScheduled,
    ResourceRequestsLimitsObserved,
    ReadinessProbeObserved,
    LivenessProbeObserved,
    GatewayRouteAccepted,
    TenantClaimPropagated,
    OTelResourceIdentityObserved,
    RolloutRecoveryObserved,
    WorkloadAuditEventRecorded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantWorkloadRuntimeEvidenceRequirement {
    pub requirement_id: &'static str, // data_class: PUBLIC
    pub requirement_kind: TenantWorkloadRuntimeEvidenceRequirementKind, // data_class: PUBLIC
    pub workload_scope: &'static str, // data_class: PUBLIC
    pub official_doc_url: &'static str, // data_class: PUBLIC
    pub expected_evidence_ref: &'static str, // data_class: INTERNAL_ONLY
    pub source_manifest_ref: &'static str, // data_class: INTERNAL_ONLY
    pub tenant_namespace: &'static str, // data_class: INTERNAL_ONLY
    pub tenant_cell_id: &'static str, // data_class: INTERNAL_ONLY
    pub residency_region: &'static str, // data_class: INTERNAL_ONLY
    pub tenant_claim: &'static str,   // data_class: INTERNAL_ONLY
    pub requires_namespace_observation: bool, // data_class: PUBLIC
    pub requires_resource_quota_usage: bool, // data_class: PUBLIC
    pub requires_network_policy_default_deny: bool, // data_class: PUBLIC
    pub requires_service_account_boundary: bool, // data_class: PUBLIC
    pub requires_pod_security_context: bool, // data_class: PUBLIC
    pub requires_workload_scheduled: bool, // data_class: PUBLIC
    pub requires_resource_requests_limits: bool, // data_class: PUBLIC
    pub requires_readiness_probe: bool, // data_class: PUBLIC
    pub requires_liveness_probe: bool, // data_class: PUBLIC
    pub requires_gateway_route_acceptance: bool, // data_class: PUBLIC
    pub requires_tenant_claim_propagation: bool, // data_class: PUBLIC
    pub requires_otel_resource_identity: bool, // data_class: PUBLIC
    pub requires_rollout_recovery: bool, // data_class: PUBLIC
    pub requires_workload_audit_event: bool, // data_class: PUBLIC
    pub runtime_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacTenantWorkloadRuntimeEvidencePlan {
    pub plan_name: &'static str,                     // data_class: PUBLIC
    pub program_name: &'static str,                  // data_class: PUBLIC
    pub substrate_name: &'static str,                // data_class: PUBLIC
    pub tenant_namespace: &'static str,              // data_class: INTERNAL_ONLY
    pub tenant_cell_id: &'static str,                // data_class: INTERNAL_ONLY
    pub residency_region: &'static str,              // data_class: INTERNAL_ONLY
    pub otel_service_namespace: &'static str,        // data_class: INTERNAL_ONLY
    pub tenant_claim: &'static str,                  // data_class: INTERNAL_ONLY
    pub tenant_workload_manifest_name: &'static str, // data_class: INTERNAL_ONLY
    pub manifest_workload_count: usize,              // data_class: PUBLIC
    pub requirements: Vec<TenantWorkloadRuntimeEvidenceRequirement>, // data_class: INTERNAL_ONLY
    pub fd001_product_delivery_master_goal_preserved: bool, // data_class: PUBLIC
    pub oyatie_cloud_substrate_proof_required: bool, // data_class: PUBLIC
    pub official_docs_required: bool,                // data_class: PUBLIC
    pub tenant_namespace_runtime_evidence_required: bool, // data_class: PUBLIC
    pub per_workload_runtime_evidence_required: bool, // data_class: PUBLIC
    pub namespace_observation_required: bool,        // data_class: PUBLIC
    pub resource_quota_usage_required: bool,         // data_class: PUBLIC
    pub network_policy_default_deny_required: bool,  // data_class: PUBLIC
    pub service_account_boundary_required: bool,     // data_class: PUBLIC
    pub pod_security_context_required: bool,         // data_class: PUBLIC
    pub workload_scheduled_required: bool,           // data_class: PUBLIC
    pub resource_requests_limits_required: bool,     // data_class: PUBLIC
    pub readiness_probe_required: bool,              // data_class: PUBLIC
    pub liveness_probe_required: bool,               // data_class: PUBLIC
    pub gateway_route_acceptance_required: bool,     // data_class: PUBLIC
    pub tenant_claim_propagation_required: bool,     // data_class: PUBLIC
    pub otel_resource_identity_required: bool,       // data_class: PUBLIC
    pub rollout_recovery_required: bool,             // data_class: PUBLIC
    pub workload_audit_event_required: bool,         // data_class: PUBLIC
    pub review_only_contract: bool,                  // data_class: PUBLIC
    pub production_tenant_attached: bool,            // data_class: INTERNAL_ONLY
    pub kubernetes_runtime_attached: bool,           // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed_attached: bool,    // data_class: INTERNAL_ONLY
    pub gateway_controller_attached: bool,           // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool,      // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub production_workload_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                         // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacTenantWorkloadRuntimeEvidenceError {
    TenantWorkloadManifest(Fd001TenantWorkloadManifestError),
    InvalidPlanName,
    InvalidProgramName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidTenantCellId,
    InvalidResidencyRegion,
    InvalidOtelServiceNamespace,
    InvalidTenantClaim,
    InvalidTenantWorkloadManifestName,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingRequirementKind(TenantWorkloadRuntimeEvidenceRequirementKind),
    InvalidRequirementId,
    InvalidWorkloadScope,
    InvalidOfficialDocUrl,
    InvalidExpectedEvidenceRef,
    InvalidSourceManifestRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_tenant_workload_runtime_evidence_plan()
-> Result<TenantRbacTenantWorkloadRuntimeEvidencePlan, TenantRbacTenantWorkloadRuntimeEvidenceError>
{
    let manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&manifest)
        .map_err(TenantRbacTenantWorkloadRuntimeEvidenceError::TenantWorkloadManifest)?;

    Ok(TenantRbacTenantWorkloadRuntimeEvidencePlan {
        plan_name: PLAN_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: manifest.tenant_namespace,
        tenant_cell_id: manifest.tenant_cell_id,
        residency_region: manifest.residency_region,
        otel_service_namespace: OTEL_SERVICE_NAMESPACE,
        tenant_claim: TENANT_CLAIM,
        tenant_workload_manifest_name: manifest.manifest_name,
        manifest_workload_count: manifest.workloads.len(),
        requirements: runtime_requirements(),
        fd001_product_delivery_master_goal_preserved: true,
        oyatie_cloud_substrate_proof_required: true,
        official_docs_required: true,
        tenant_namespace_runtime_evidence_required: true,
        per_workload_runtime_evidence_required: true,
        namespace_observation_required: true,
        resource_quota_usage_required: true,
        network_policy_default_deny_required: true,
        service_account_boundary_required: true,
        pod_security_context_required: true,
        workload_scheduled_required: true,
        resource_requests_limits_required: true,
        readiness_probe_required: true,
        liveness_probe_required: true,
        gateway_route_acceptance_required: true,
        tenant_claim_propagation_required: true,
        otel_resource_identity_required: true,
        rollout_recovery_required: true,
        workload_audit_event_required: true,
        review_only_contract: true,
        production_tenant_attached: false,
        kubernetes_runtime_attached: false,
        workload_runtime_deployed_attached: false,
        gateway_controller_attached: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        production_workload_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_tenant_rbac_tenant_workload_runtime_evidence_plan(
    plan: &TenantRbacTenantWorkloadRuntimeEvidencePlan,
) -> Result<(), TenantRbacTenantWorkloadRuntimeEvidenceError> {
    validate_slug(
        plan.plan_name,
        TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidPlanName,
    )?;
    if plan.program_name != PROGRAM_NAME {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidProgramName);
    }
    if plan.substrate_name != SUBSTRATE_NAME {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidSubstrateName);
    }
    if plan.tenant_namespace != TENANT_NAMESPACE {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidTenantNamespace);
    }
    if plan.tenant_cell_id != TENANT_CELL_ID {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidTenantCellId);
    }
    if plan.residency_region != RESIDENCY_REGION {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidResidencyRegion);
    }
    if plan.otel_service_namespace != OTEL_SERVICE_NAMESPACE {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidOtelServiceNamespace);
    }
    if plan.tenant_claim != TENANT_CLAIM {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidTenantClaim);
    }
    if plan.tenant_workload_manifest_name != "fd001-tenant-rbac-tenant-workload-manifest" {
        return Err(
            TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidTenantWorkloadManifestName,
        );
    }
    if plan.manifest_workload_count < 4
        || plan.requirements.len() < MIN_REQUIREMENT_COUNT
        || plan.schema_version != SCHEMA_VERSION
    {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::MissingRequirements);
    }
    validate_required_controls(plan)?;
    validate_nonclaims(plan)?;
    validate_runtime_requirements(plan)?;
    Ok(())
}

pub fn tenant_workload_runtime_evidence_doc_urls(
    plan: &TenantRbacTenantWorkloadRuntimeEvidencePlan,
) -> Vec<&'static str> {
    plan.requirements
        .iter()
        .map(|requirement| requirement.official_doc_url)
        .collect()
}

fn runtime_requirements() -> Vec<TenantWorkloadRuntimeEvidenceRequirement> {
    vec![
        requirement(
            "namespace-observed",
            TenantWorkloadRuntimeEvidenceRequirementKind::NamespaceObserved,
            "https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/namespace-observed.json",
        ),
        requirement(
            "resource-quota-usage-observed",
            TenantWorkloadRuntimeEvidenceRequirementKind::ResourceQuotaUsageObserved,
            "https://kubernetes.io/docs/concepts/policy/resource-quotas/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/resource-quota-usage.json",
        ),
        requirement(
            "network-policy-default-deny-observed",
            TenantWorkloadRuntimeEvidenceRequirementKind::NetworkPolicyDefaultDenyObserved,
            "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/network-policy-default-deny.json",
        ),
        requirement(
            "service-account-boundary-observed",
            TenantWorkloadRuntimeEvidenceRequirementKind::ServiceAccountBoundaryObserved,
            "https://kubernetes.io/docs/concepts/security/service-accounts/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/service-account-boundary.json",
        ),
        requirement(
            "pod-security-context-observed",
            TenantWorkloadRuntimeEvidenceRequirementKind::PodSecurityContextObserved,
            "https://kubernetes.io/docs/concepts/security/pod-security-standards/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/pod-security-context.json",
        ),
        requirement(
            "workload-scheduled",
            TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadScheduled,
            "https://kubernetes.io/docs/concepts/workloads/controllers/deployment/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/workload-scheduled.json",
        ),
        requirement(
            "resource-requests-limits-observed",
            TenantWorkloadRuntimeEvidenceRequirementKind::ResourceRequestsLimitsObserved,
            "https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/resource-requests-limits.json",
        ),
        requirement(
            "readiness-probe-observed",
            TenantWorkloadRuntimeEvidenceRequirementKind::ReadinessProbeObserved,
            "https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#container-probes",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/readiness-probe.json",
        ),
        requirement(
            "liveness-probe-observed",
            TenantWorkloadRuntimeEvidenceRequirementKind::LivenessProbeObserved,
            "https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#container-probes",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/liveness-probe.json",
        ),
        requirement(
            "gateway-route-accepted",
            TenantWorkloadRuntimeEvidenceRequirementKind::GatewayRouteAccepted,
            "https://gateway-api.sigs.k8s.io/docs/introduction/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/gateway-route-accepted.json",
        ),
        requirement(
            "tenant-claim-propagated",
            TenantWorkloadRuntimeEvidenceRequirementKind::TenantClaimPropagated,
            "https://kubernetes.io/docs/concepts/security/service-accounts/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/tenant-claim-propagated.json",
        ),
        requirement(
            "otel-resource-identity-observed",
            TenantWorkloadRuntimeEvidenceRequirementKind::OTelResourceIdentityObserved,
            "https://opentelemetry.io/docs/specs/semconv/resource/service/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/otel-resource-identity.json",
        ),
        requirement(
            "rollout-recovery-observed",
            TenantWorkloadRuntimeEvidenceRequirementKind::RolloutRecoveryObserved,
            "https://kubernetes.io/docs/concepts/workloads/controllers/deployment/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/rollout-recovery.json",
        ),
        requirement(
            "workload-audit-event-recorded",
            TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadAuditEventRecorded,
            "https://opentelemetry.io/docs/specs/semconv/resource/service/",
            "evidence/tenant-workload-runtime/fd001-tenant-rbac/workload-audit-event.json",
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    requirement_kind: TenantWorkloadRuntimeEvidenceRequirementKind,
    official_doc_url: &'static str,
    expected_evidence_ref: &'static str,
) -> TenantWorkloadRuntimeEvidenceRequirement {
    let requires_namespace_observation = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::NamespaceObserved
            | TenantWorkloadRuntimeEvidenceRequirementKind::ResourceQuotaUsageObserved
            | TenantWorkloadRuntimeEvidenceRequirementKind::NetworkPolicyDefaultDenyObserved
            | TenantWorkloadRuntimeEvidenceRequirementKind::ServiceAccountBoundaryObserved
    );
    let requires_resource_quota_usage = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::ResourceQuotaUsageObserved
            | TenantWorkloadRuntimeEvidenceRequirementKind::ResourceRequestsLimitsObserved
    );
    let requires_network_policy_default_deny = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::NetworkPolicyDefaultDenyObserved
            | TenantWorkloadRuntimeEvidenceRequirementKind::GatewayRouteAccepted
    );
    let requires_service_account_boundary = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::ServiceAccountBoundaryObserved
            | TenantWorkloadRuntimeEvidenceRequirementKind::TenantClaimPropagated
    );
    let requires_pod_security_context = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::PodSecurityContextObserved
    );
    let requires_workload_scheduled = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadScheduled
            | TenantWorkloadRuntimeEvidenceRequirementKind::RolloutRecoveryObserved
    );
    let requires_resource_requests_limits = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::ResourceRequestsLimitsObserved
    );
    let requires_readiness_probe = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::ReadinessProbeObserved
            | TenantWorkloadRuntimeEvidenceRequirementKind::GatewayRouteAccepted
            | TenantWorkloadRuntimeEvidenceRequirementKind::RolloutRecoveryObserved
    );
    let requires_liveness_probe = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::LivenessProbeObserved
            | TenantWorkloadRuntimeEvidenceRequirementKind::RolloutRecoveryObserved
    );
    let requires_gateway_route_acceptance = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::GatewayRouteAccepted
    );
    let requires_tenant_claim_propagation = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::TenantClaimPropagated
            | TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadAuditEventRecorded
    );
    let requires_otel_resource_identity = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::OTelResourceIdentityObserved
            | TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadAuditEventRecorded
    );
    let requires_rollout_recovery = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::RolloutRecoveryObserved
    );
    let requires_workload_audit_event = matches!(
        requirement_kind,
        TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadAuditEventRecorded
    );

    TenantWorkloadRuntimeEvidenceRequirement {
        requirement_id,
        requirement_kind,
        workload_scope: "all-fd001-workloads",
        official_doc_url,
        expected_evidence_ref,
        source_manifest_ref: SOURCE_MANIFEST_REF,
        tenant_namespace: TENANT_NAMESPACE,
        tenant_cell_id: TENANT_CELL_ID,
        residency_region: RESIDENCY_REGION,
        tenant_claim: TENANT_CLAIM,
        requires_namespace_observation,
        requires_resource_quota_usage,
        requires_network_policy_default_deny,
        requires_service_account_boundary,
        requires_pod_security_context,
        requires_workload_scheduled,
        requires_resource_requests_limits,
        requires_readiness_probe,
        requires_liveness_probe,
        requires_gateway_route_acceptance,
        requires_tenant_claim_propagation,
        requires_otel_resource_identity,
        requires_rollout_recovery,
        requires_workload_audit_event,
        runtime_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_required_controls(
    plan: &TenantRbacTenantWorkloadRuntimeEvidencePlan,
) -> Result<(), TenantRbacTenantWorkloadRuntimeEvidenceError> {
    for control in [
        (
            plan.fd001_product_delivery_master_goal_preserved,
            "fd001_product_delivery_master_goal_preserved",
        ),
        (
            plan.oyatie_cloud_substrate_proof_required,
            "oyatie_cloud_substrate_proof_required",
        ),
        (plan.official_docs_required, "official_docs_required"),
        (
            plan.tenant_namespace_runtime_evidence_required,
            "tenant_namespace_runtime_evidence_required",
        ),
        (
            plan.per_workload_runtime_evidence_required,
            "per_workload_runtime_evidence_required",
        ),
        (
            plan.namespace_observation_required,
            "namespace_observation_required",
        ),
        (
            plan.resource_quota_usage_required,
            "resource_quota_usage_required",
        ),
        (
            plan.network_policy_default_deny_required,
            "network_policy_default_deny_required",
        ),
        (
            plan.service_account_boundary_required,
            "service_account_boundary_required",
        ),
        (
            plan.pod_security_context_required,
            "pod_security_context_required",
        ),
        (
            plan.workload_scheduled_required,
            "workload_scheduled_required",
        ),
        (
            plan.resource_requests_limits_required,
            "resource_requests_limits_required",
        ),
        (plan.readiness_probe_required, "readiness_probe_required"),
        (plan.liveness_probe_required, "liveness_probe_required"),
        (
            plan.gateway_route_acceptance_required,
            "gateway_route_acceptance_required",
        ),
        (
            plan.tenant_claim_propagation_required,
            "tenant_claim_propagation_required",
        ),
        (
            plan.otel_resource_identity_required,
            "otel_resource_identity_required",
        ),
        (plan.rollout_recovery_required, "rollout_recovery_required"),
        (
            plan.workload_audit_event_required,
            "workload_audit_event_required",
        ),
        (plan.review_only_contract, "review_only_contract"),
    ] {
        require_control(control.0, control.1)?;
    }
    Ok(())
}

fn validate_nonclaims(
    plan: &TenantRbacTenantWorkloadRuntimeEvidencePlan,
) -> Result<(), TenantRbacTenantWorkloadRuntimeEvidenceError> {
    if plan.production_tenant_attached
        || plan.kubernetes_runtime_attached
        || plan.workload_runtime_deployed_attached
        || plan.gateway_controller_attached
        || plan.cloud_substrate_runtime_attached
        || plan.runtime_audit_chain_emission_attached
        || plan.production_workload_evidence_attached
    {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_runtime_requirements(
    plan: &TenantRbacTenantWorkloadRuntimeEvidencePlan,
) -> Result<(), TenantRbacTenantWorkloadRuntimeEvidenceError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_kinds = BTreeSet::new();
    for requirement in &plan.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(
                TenantRbacTenantWorkloadRuntimeEvidenceError::DuplicateRequirement(
                    requirement.requirement_id.to_owned(),
                ),
            );
        }
        seen_kinds.insert(requirement.requirement_kind);
    }
    for kind in required_requirement_kinds() {
        if !seen_kinds.contains(&kind) {
            return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::MissingRequirementKind(kind));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &TenantWorkloadRuntimeEvidenceRequirement,
) -> Result<(), TenantRbacTenantWorkloadRuntimeEvidenceError> {
    validate_slug(
        requirement.requirement_id,
        TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidRequirementId,
    )?;
    validate_workload_scope(requirement.workload_scope)?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/tenant-workload-runtime/fd001-tenant-rbac/",
        TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidExpectedEvidenceRef,
    )?;
    validate_prefixed_ref(
        requirement.source_manifest_ref,
        "crates/tenant-rbac-tenant-workload-manifest/",
        TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidSourceManifestRef,
    )?;
    if requirement.tenant_namespace != TENANT_NAMESPACE {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidTenantNamespace);
    }
    if requirement.tenant_cell_id != TENANT_CELL_ID {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidTenantCellId);
    }
    if requirement.residency_region != RESIDENCY_REGION {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidResidencyRegion);
    }
    if requirement.tenant_claim != TENANT_CLAIM {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidTenantClaim);
    }
    if requirement.runtime_evidence_attached || requirement.schema_version != SCHEMA_VERSION {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn required_requirement_kinds() -> [TenantWorkloadRuntimeEvidenceRequirementKind; 14] {
    [
        TenantWorkloadRuntimeEvidenceRequirementKind::NamespaceObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::ResourceQuotaUsageObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::NetworkPolicyDefaultDenyObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::ServiceAccountBoundaryObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::PodSecurityContextObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadScheduled,
        TenantWorkloadRuntimeEvidenceRequirementKind::ResourceRequestsLimitsObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::ReadinessProbeObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::LivenessProbeObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::GatewayRouteAccepted,
        TenantWorkloadRuntimeEvidenceRequirementKind::TenantClaimPropagated,
        TenantWorkloadRuntimeEvidenceRequirementKind::OTelResourceIdentityObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::RolloutRecoveryObserved,
        TenantWorkloadRuntimeEvidenceRequirementKind::WorkloadAuditEventRecorded,
    ]
}

fn validate_workload_scope(
    value: &str,
) -> Result<(), TenantRbacTenantWorkloadRuntimeEvidenceError> {
    if value != "all-fd001-workloads" || has_unsafe_text(value) || has_path_traversal(value) {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidWorkloadScope);
    }
    Ok(())
}

fn validate_doc_url(value: &str) -> Result<(), TenantRbacTenantWorkloadRuntimeEvidenceError> {
    if !matches!(
        value,
        "https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/"
            | "https://kubernetes.io/docs/concepts/policy/resource-quotas/"
            | "https://kubernetes.io/docs/concepts/services-networking/network-policies/"
            | "https://kubernetes.io/docs/concepts/security/service-accounts/"
            | "https://kubernetes.io/docs/concepts/security/pod-security-standards/"
            | "https://kubernetes.io/docs/concepts/workloads/controllers/deployment/"
            | "https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/"
            | "https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#container-probes"
            | "https://gateway-api.sigs.k8s.io/docs/introduction/"
            | "https://opentelemetry.io/docs/specs/semconv/resource/service/"
    ) {
        return Err(TenantRbacTenantWorkloadRuntimeEvidenceError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: TenantRbacTenantWorkloadRuntimeEvidenceError,
) -> Result<(), TenantRbacTenantWorkloadRuntimeEvidenceError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(error);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: TenantRbacTenantWorkloadRuntimeEvidenceError,
) -> Result<(), TenantRbacTenantWorkloadRuntimeEvidenceError> {
    if !value.starts_with(prefix)
        || value.len() <= prefix.len()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
    {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    name: &'static str,
) -> Result<(), TenantRbacTenantWorkloadRuntimeEvidenceError> {
    if value {
        Ok(())
    } else {
        Err(TenantRbacTenantWorkloadRuntimeEvidenceError::MissingRequiredControl(name))
    }
}

fn has_unsafe_text(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    ["pending", "todo", "fixme", "placeholder", "mock", "stub"]
        .iter()
        .any(|needle| lowered.contains(needle))
}

fn has_path_traversal(value: &str) -> bool {
    value.contains("..") || value.starts_with('/') || value.contains('\\')
}

fn has_credential_shape(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    [
        "secret",
        "token",
        "password",
        "api-key",
        "apikey",
        "credential",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}
