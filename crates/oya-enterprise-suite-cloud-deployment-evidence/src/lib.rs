//! Enterprise Suite cloud deployment evidence contract.
//!
//! This review-only crate records the evidence that must exist before FD-001
//! tenant workloads can claim deployment on the future Oyatie Cloud substrate.
//! It validates evidence refs and control requirements, but it does not contact
//! Kubernetes, Argo CD, Sigstore, Gateway controllers, telemetry backends, or
//! audit sinks and does not claim live cloud deployment evidence.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use oya_enterprise_suite_cloud_deployment_manifest::{
    CloudDeploymentManifestError, enterprise_suite_cloud_deployment_manifest,
    validate_cloud_deployment_manifest,
};
use oya_enterprise_suite_tenant_workload_manifest::{
    Fd001TenantWorkloadManifestError, fd001_tenant_workload_manifest,
    validate_fd001_tenant_workload_manifest,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_REQUIREMENT_COUNT: usize = 14;
const PLAN_NAME: &str = "enterprise-suite-cloud-deployment-evidence-plan";
const SERVICE_NAME: &str = "enterprise-suite";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-enterprise-dev";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CloudDeploymentEvidenceRequirementKind {
    ArgoCdApplicationSynced,
    ArgoCdApplicationHealthy,
    GitRevisionPinned,
    CosignImageVerified,
    NamespaceObserved,
    ResourceQuotaObserved,
    NetworkPolicyObserved,
    ServiceAccountObserved,
    DeploymentAvailable,
    ReadinessProbeGreen,
    GatewayHttpRouteAccepted,
    OtelResourceIdentityObserved,
    DeploymentAuditEventRecorded,
    RollbackPlanAttached,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudDeploymentEvidenceRequirement {
    pub requirement_id: &'static str, // data_class: PUBLIC
    pub requirement_kind: CloudDeploymentEvidenceRequirementKind, // data_class: PUBLIC
    pub workload_scope: &'static str, // data_class: PUBLIC
    pub official_doc_url: &'static str, // data_class: PUBLIC
    pub expected_evidence_ref: &'static str, // data_class: INTERNAL_ONLY
    pub source_manifest_ref: &'static str, // data_class: INTERNAL_ONLY
    pub tenant_namespace: &'static str, // data_class: INTERNAL_ONLY
    pub requires_controller_observation: bool, // data_class: PUBLIC
    pub requires_digest_pinned_revision: bool, // data_class: PUBLIC
    pub requires_tenant_namespace: bool, // data_class: PUBLIC
    pub requires_runtime_health_evidence: bool, // data_class: PUBLIC
    pub requires_security_verification: bool, // data_class: PUBLIC
    pub runtime_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterpriseCloudDeploymentEvidencePlan {
    pub plan_name: &'static str,                     // data_class: PUBLIC
    pub service_name: &'static str,                  // data_class: PUBLIC
    pub substrate_name: &'static str,                // data_class: PUBLIC
    pub tenant_namespace: &'static str,              // data_class: INTERNAL_ONLY
    pub deployment_manifest_name: &'static str,      // data_class: INTERNAL_ONLY
    pub tenant_workload_manifest_name: &'static str, // data_class: INTERNAL_ONLY
    pub requirements: Vec<CloudDeploymentEvidenceRequirement>, // data_class: INTERNAL_ONLY
    pub fd001_product_delivery_master_goal_preserved: bool, // data_class: PUBLIC
    pub oyatie_cloud_substrate_proof_required: bool, // data_class: PUBLIC
    pub official_docs_required: bool,                // data_class: PUBLIC
    pub argocd_sync_evidence_required: bool,         // data_class: PUBLIC
    pub argocd_health_evidence_required: bool,       // data_class: PUBLIC
    pub git_revision_pin_required: bool,             // data_class: PUBLIC
    pub cosign_verification_required: bool,          // data_class: PUBLIC
    pub namespace_observation_required: bool,        // data_class: PUBLIC
    pub quota_observation_required: bool,            // data_class: PUBLIC
    pub network_policy_observation_required: bool,   // data_class: PUBLIC
    pub service_account_observation_required: bool,  // data_class: PUBLIC
    pub deployment_available_required: bool,         // data_class: PUBLIC
    pub readiness_probe_required: bool,              // data_class: PUBLIC
    pub gateway_route_acceptance_required: bool,     // data_class: PUBLIC
    pub otel_resource_identity_required: bool,       // data_class: PUBLIC
    pub deployment_audit_event_required: bool,       // data_class: PUBLIC
    pub rollback_plan_required: bool,                // data_class: PUBLIC
    pub review_only_contract: bool,                  // data_class: PUBLIC
    pub argocd_controller_attached: bool,            // data_class: INTERNAL_ONLY
    pub kubernetes_cluster_attached: bool,           // data_class: INTERNAL_ONLY
    pub namespace_created_attached: bool,            // data_class: INTERNAL_ONLY
    pub quota_applied_attached: bool,                // data_class: INTERNAL_ONLY
    pub network_policy_applied_attached: bool,       // data_class: INTERNAL_ONLY
    pub gateway_route_attached: bool,                // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed_attached: bool,    // data_class: INTERNAL_ONLY
    pub runtime_otel_export_attached: bool,          // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub production_cloud_deployment_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                         // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnterpriseCloudDeploymentEvidenceError {
    DeploymentManifest(CloudDeploymentManifestError),
    TenantWorkloadManifest(Fd001TenantWorkloadManifestError),
    InvalidPlanName,
    InvalidServiceName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidDeploymentManifestName,
    InvalidTenantWorkloadManifestName,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingRequirementKind(CloudDeploymentEvidenceRequirementKind),
    InvalidRequirementId,
    InvalidWorkloadScope,
    InvalidOfficialDocUrl,
    InvalidExpectedEvidenceRef,
    InvalidSourceManifestRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn enterprise_cloud_deployment_evidence_plan()
-> Result<EnterpriseCloudDeploymentEvidencePlan, EnterpriseCloudDeploymentEvidenceError> {
    let deployment_manifest = enterprise_suite_cloud_deployment_manifest();
    validate_cloud_deployment_manifest(&deployment_manifest)
        .map_err(EnterpriseCloudDeploymentEvidenceError::DeploymentManifest)?;
    let tenant_workload_manifest = fd001_tenant_workload_manifest();
    validate_fd001_tenant_workload_manifest(&tenant_workload_manifest)
        .map_err(EnterpriseCloudDeploymentEvidenceError::TenantWorkloadManifest)?;

    Ok(EnterpriseCloudDeploymentEvidencePlan {
        plan_name: PLAN_NAME,
        service_name: SERVICE_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: TENANT_NAMESPACE,
        deployment_manifest_name: deployment_manifest.manifest_name,
        tenant_workload_manifest_name: tenant_workload_manifest.manifest_name,
        requirements: vec![
            requirement(
                "argocd-application-synced",
                CloudDeploymentEvidenceRequirementKind::ArgoCdApplicationSynced,
                "fd001-enterprise-workloads",
                "https://argo-cd.readthedocs.io/en/stable/",
                "evidence/cloud-deployment/enterprise-suite/argocd-sync.json",
            ),
            requirement(
                "argocd-application-healthy",
                CloudDeploymentEvidenceRequirementKind::ArgoCdApplicationHealthy,
                "fd001-enterprise-workloads",
                "https://argo-cd.readthedocs.io/en/stable/",
                "evidence/cloud-deployment/enterprise-suite/argocd-health.json",
            ),
            requirement(
                "git-revision-pinned",
                CloudDeploymentEvidenceRequirementKind::GitRevisionPinned,
                "fd001-enterprise-workloads",
                "https://argo-cd.readthedocs.io/en/stable/user-guide/auto_sync/",
                "evidence/cloud-deployment/enterprise-suite/git-revision.json",
            ),
            requirement(
                "cosign-image-verified",
                CloudDeploymentEvidenceRequirementKind::CosignImageVerified,
                "enterprise-suite-runtime",
                "https://docs.sigstore.dev/cosign/verifying/verify/",
                "evidence/cloud-deployment/enterprise-suite/cosign-verify.json",
            ),
            requirement(
                "namespace-observed",
                CloudDeploymentEvidenceRequirementKind::NamespaceObserved,
                "oyatie-fd001-enterprise-dev",
                "https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/",
                "evidence/cloud-deployment/enterprise-suite/namespace.json",
            ),
            requirement(
                "resource-quota-observed",
                CloudDeploymentEvidenceRequirementKind::ResourceQuotaObserved,
                "oyatie-fd001-enterprise-dev",
                "https://kubernetes.io/docs/concepts/policy/resource-quotas/",
                "evidence/cloud-deployment/enterprise-suite/resource-quota.json",
            ),
            requirement(
                "network-policy-observed",
                CloudDeploymentEvidenceRequirementKind::NetworkPolicyObserved,
                "oyatie-fd001-enterprise-dev",
                "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
                "evidence/cloud-deployment/enterprise-suite/network-policy.json",
            ),
            requirement(
                "service-account-observed",
                CloudDeploymentEvidenceRequirementKind::ServiceAccountObserved,
                "fd001-enterprise-workload",
                "https://kubernetes.io/docs/concepts/security/service-accounts/",
                "evidence/cloud-deployment/enterprise-suite/service-account.json",
            ),
            requirement(
                "deployment-available",
                CloudDeploymentEvidenceRequirementKind::DeploymentAvailable,
                "enterprise-suite-runtime",
                "https://kubernetes.io/docs/reference/kubernetes-api/apps/deployment-v1/",
                "evidence/cloud-deployment/enterprise-suite/deployment-available.json",
            ),
            requirement(
                "readiness-probe-green",
                CloudDeploymentEvidenceRequirementKind::ReadinessProbeGreen,
                "enterprise-suite-runtime",
                "https://kubernetes.io/docs/concepts/workloads/pods/probes/",
                "evidence/cloud-deployment/enterprise-suite/readiness-probe.json",
            ),
            requirement(
                "gateway-httproute-accepted",
                CloudDeploymentEvidenceRequirementKind::GatewayHttpRouteAccepted,
                "fd001-enterprise-workloads",
                "https://gateway-api.sigs.k8s.io/docs/introduction/",
                "evidence/cloud-deployment/enterprise-suite/gateway-httproute.json",
            ),
            requirement(
                "otel-resource-identity-observed",
                CloudDeploymentEvidenceRequirementKind::OtelResourceIdentityObserved,
                "fd001-enterprise-workloads",
                "https://opentelemetry.io/docs/specs/semconv/resource/service/",
                "evidence/cloud-deployment/enterprise-suite/otel-resource.json",
            ),
            requirement(
                "deployment-audit-event-recorded",
                CloudDeploymentEvidenceRequirementKind::DeploymentAuditEventRecorded,
                "fd001-enterprise-workloads",
                "https://cloudevents.io/",
                "evidence/cloud-deployment/enterprise-suite/audit-event.json",
            ),
            requirement(
                "rollback-plan-attached",
                CloudDeploymentEvidenceRequirementKind::RollbackPlanAttached,
                "fd001-enterprise-workloads",
                "https://argo-cd.readthedocs.io/en/stable/",
                "evidence/cloud-deployment/enterprise-suite/rollback-plan.json",
            ),
        ],
        fd001_product_delivery_master_goal_preserved: tenant_workload_manifest
            .fd001_product_goal_preserved,
        oyatie_cloud_substrate_proof_required: tenant_workload_manifest.oyatie_cloud_substrate_only,
        official_docs_required: true,
        argocd_sync_evidence_required: true,
        argocd_health_evidence_required: true,
        git_revision_pin_required: true,
        cosign_verification_required: true,
        namespace_observation_required: true,
        quota_observation_required: true,
        network_policy_observation_required: true,
        service_account_observation_required: true,
        deployment_available_required: true,
        readiness_probe_required: true,
        gateway_route_acceptance_required: true,
        otel_resource_identity_required: true,
        deployment_audit_event_required: true,
        rollback_plan_required: true,
        review_only_contract: true,
        argocd_controller_attached: false,
        kubernetes_cluster_attached: false,
        namespace_created_attached: false,
        quota_applied_attached: false,
        network_policy_applied_attached: false,
        gateway_route_attached: false,
        workload_runtime_deployed_attached: false,
        runtime_otel_export_attached: false,
        runtime_audit_chain_emission_attached: false,
        production_cloud_deployment_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_enterprise_cloud_deployment_evidence_plan(
    plan: &EnterpriseCloudDeploymentEvidencePlan,
) -> Result<(), EnterpriseCloudDeploymentEvidenceError> {
    validate_slug(
        plan.plan_name,
        EnterpriseCloudDeploymentEvidenceError::InvalidPlanName,
    )?;
    if plan.service_name != SERVICE_NAME {
        return Err(EnterpriseCloudDeploymentEvidenceError::InvalidServiceName);
    }
    if plan.substrate_name != SUBSTRATE_NAME {
        return Err(EnterpriseCloudDeploymentEvidenceError::InvalidSubstrateName);
    }
    validate_namespace(plan.tenant_namespace)?;
    if plan.deployment_manifest_name != "enterprise-suite-cloud-deployment-manifest" {
        return Err(EnterpriseCloudDeploymentEvidenceError::InvalidDeploymentManifestName);
    }
    if plan.tenant_workload_manifest_name != "fd001-enterprise-tenant-workload-manifest" {
        return Err(EnterpriseCloudDeploymentEvidenceError::InvalidTenantWorkloadManifestName);
    }
    if plan.requirements.len() < MIN_REQUIREMENT_COUNT {
        return Err(EnterpriseCloudDeploymentEvidenceError::MissingRequirements);
    }
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
            plan.argocd_sync_evidence_required,
            "argocd_sync_evidence_required",
        ),
        (
            plan.argocd_health_evidence_required,
            "argocd_health_evidence_required",
        ),
        (plan.git_revision_pin_required, "git_revision_pin_required"),
        (
            plan.cosign_verification_required,
            "cosign_verification_required",
        ),
        (
            plan.namespace_observation_required,
            "namespace_observation_required",
        ),
        (
            plan.quota_observation_required,
            "quota_observation_required",
        ),
        (
            plan.network_policy_observation_required,
            "network_policy_observation_required",
        ),
        (
            plan.service_account_observation_required,
            "service_account_observation_required",
        ),
        (
            plan.deployment_available_required,
            "deployment_available_required",
        ),
        (plan.readiness_probe_required, "readiness_probe_required"),
        (
            plan.gateway_route_acceptance_required,
            "gateway_route_acceptance_required",
        ),
        (
            plan.otel_resource_identity_required,
            "otel_resource_identity_required",
        ),
        (
            plan.deployment_audit_event_required,
            "deployment_audit_event_required",
        ),
        (plan.rollback_plan_required, "rollback_plan_required"),
        (plan.review_only_contract, "review_only_contract"),
    ] {
        require_control(control.0, control.1)?;
    }
    if plan.argocd_controller_attached
        || plan.kubernetes_cluster_attached
        || plan.namespace_created_attached
        || plan.quota_applied_attached
        || plan.network_policy_applied_attached
        || plan.gateway_route_attached
        || plan.workload_runtime_deployed_attached
        || plan.runtime_otel_export_attached
        || plan.runtime_audit_chain_emission_attached
        || plan.production_cloud_deployment_evidence_attached
    {
        return Err(EnterpriseCloudDeploymentEvidenceError::RuntimeAttachmentOverclaim);
    }

    let mut seen_requirements = BTreeSet::new();
    let mut seen_kinds = BTreeSet::new();
    for requirement in &plan.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(
                EnterpriseCloudDeploymentEvidenceError::DuplicateRequirement(
                    requirement.requirement_id.to_owned(),
                ),
            );
        }
        seen_kinds.insert(requirement.requirement_kind);
    }
    for kind in required_requirement_kinds() {
        if !seen_kinds.contains(&kind) {
            return Err(EnterpriseCloudDeploymentEvidenceError::MissingRequirementKind(kind));
        }
    }
    Ok(())
}

pub fn cloud_deployment_evidence_doc_urls(
    plan: &EnterpriseCloudDeploymentEvidencePlan,
) -> Vec<&'static str> {
    plan.requirements
        .iter()
        .map(|requirement| requirement.official_doc_url)
        .collect()
}

fn requirement(
    requirement_id: &'static str,
    requirement_kind: CloudDeploymentEvidenceRequirementKind,
    workload_scope: &'static str,
    official_doc_url: &'static str,
    expected_evidence_ref: &'static str,
) -> CloudDeploymentEvidenceRequirement {
    let requires_controller_observation = !matches!(
        requirement_kind,
        CloudDeploymentEvidenceRequirementKind::GitRevisionPinned
            | CloudDeploymentEvidenceRequirementKind::CosignImageVerified
            | CloudDeploymentEvidenceRequirementKind::RollbackPlanAttached
    );
    let requires_runtime_health_evidence = matches!(
        requirement_kind,
        CloudDeploymentEvidenceRequirementKind::ArgoCdApplicationHealthy
            | CloudDeploymentEvidenceRequirementKind::DeploymentAvailable
            | CloudDeploymentEvidenceRequirementKind::ReadinessProbeGreen
            | CloudDeploymentEvidenceRequirementKind::GatewayHttpRouteAccepted
            | CloudDeploymentEvidenceRequirementKind::OtelResourceIdentityObserved
    );

    CloudDeploymentEvidenceRequirement {
        requirement_id,
        requirement_kind,
        workload_scope,
        official_doc_url,
        expected_evidence_ref,
        source_manifest_ref: "crates/oya-enterprise-suite-cloud-deployment-manifest/src/lib.rs::enterprise_suite_cloud_deployment_manifest",
        tenant_namespace: TENANT_NAMESPACE,
        requires_controller_observation,
        requires_digest_pinned_revision: requirement_kind
            == CloudDeploymentEvidenceRequirementKind::GitRevisionPinned,
        requires_tenant_namespace: true,
        requires_runtime_health_evidence,
        requires_security_verification: requirement_kind
            == CloudDeploymentEvidenceRequirementKind::CosignImageVerified,
        runtime_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_requirement(
    requirement: &CloudDeploymentEvidenceRequirement,
) -> Result<(), EnterpriseCloudDeploymentEvidenceError> {
    validate_slug(
        requirement.requirement_id,
        EnterpriseCloudDeploymentEvidenceError::InvalidRequirementId,
    )?;
    validate_workload_scope(requirement.workload_scope)?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/cloud-deployment/enterprise-suite/",
        EnterpriseCloudDeploymentEvidenceError::InvalidExpectedEvidenceRef,
    )?;
    validate_prefixed_ref(
        requirement.source_manifest_ref,
        "crates/oya-enterprise-suite-cloud-deployment-manifest/",
        EnterpriseCloudDeploymentEvidenceError::InvalidSourceManifestRef,
    )?;
    if requirement.tenant_namespace != TENANT_NAMESPACE {
        return Err(EnterpriseCloudDeploymentEvidenceError::InvalidTenantNamespace);
    }
    require_control(
        requirement.requires_tenant_namespace,
        "requirement_requires_tenant_namespace",
    )?;
    if matches!(
        requirement.requirement_kind,
        CloudDeploymentEvidenceRequirementKind::ArgoCdApplicationSynced
            | CloudDeploymentEvidenceRequirementKind::ArgoCdApplicationHealthy
            | CloudDeploymentEvidenceRequirementKind::NamespaceObserved
            | CloudDeploymentEvidenceRequirementKind::ResourceQuotaObserved
            | CloudDeploymentEvidenceRequirementKind::NetworkPolicyObserved
            | CloudDeploymentEvidenceRequirementKind::ServiceAccountObserved
            | CloudDeploymentEvidenceRequirementKind::DeploymentAvailable
            | CloudDeploymentEvidenceRequirementKind::ReadinessProbeGreen
            | CloudDeploymentEvidenceRequirementKind::GatewayHttpRouteAccepted
            | CloudDeploymentEvidenceRequirementKind::OtelResourceIdentityObserved
            | CloudDeploymentEvidenceRequirementKind::DeploymentAuditEventRecorded
    ) {
        require_control(
            requirement.requires_controller_observation,
            "requirement_requires_controller_observation",
        )?;
    }
    if requirement.requirement_kind == CloudDeploymentEvidenceRequirementKind::GitRevisionPinned {
        require_control(
            requirement.requires_digest_pinned_revision,
            "requirement_requires_digest_pinned_revision",
        )?;
    }
    if requirement.requirement_kind == CloudDeploymentEvidenceRequirementKind::CosignImageVerified {
        require_control(
            requirement.requires_security_verification,
            "requirement_requires_security_verification",
        )?;
    }
    if requirement.runtime_evidence_attached {
        return Err(EnterpriseCloudDeploymentEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn required_requirement_kinds() -> [CloudDeploymentEvidenceRequirementKind; 14] {
    [
        CloudDeploymentEvidenceRequirementKind::ArgoCdApplicationSynced,
        CloudDeploymentEvidenceRequirementKind::ArgoCdApplicationHealthy,
        CloudDeploymentEvidenceRequirementKind::GitRevisionPinned,
        CloudDeploymentEvidenceRequirementKind::CosignImageVerified,
        CloudDeploymentEvidenceRequirementKind::NamespaceObserved,
        CloudDeploymentEvidenceRequirementKind::ResourceQuotaObserved,
        CloudDeploymentEvidenceRequirementKind::NetworkPolicyObserved,
        CloudDeploymentEvidenceRequirementKind::ServiceAccountObserved,
        CloudDeploymentEvidenceRequirementKind::DeploymentAvailable,
        CloudDeploymentEvidenceRequirementKind::ReadinessProbeGreen,
        CloudDeploymentEvidenceRequirementKind::GatewayHttpRouteAccepted,
        CloudDeploymentEvidenceRequirementKind::OtelResourceIdentityObserved,
        CloudDeploymentEvidenceRequirementKind::DeploymentAuditEventRecorded,
        CloudDeploymentEvidenceRequirementKind::RollbackPlanAttached,
    ]
}

fn validate_slug(
    value: &str,
    error: EnterpriseCloudDeploymentEvidenceError,
) -> Result<(), EnterpriseCloudDeploymentEvidenceError> {
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

fn validate_namespace(value: &str) -> Result<(), EnterpriseCloudDeploymentEvidenceError> {
    validate_slug(
        value,
        EnterpriseCloudDeploymentEvidenceError::InvalidTenantNamespace,
    )?;
    if !value.starts_with("oyatie-") || matches!(value, "default" | "kube-system" | "kube-public") {
        return Err(EnterpriseCloudDeploymentEvidenceError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_workload_scope(value: &str) -> Result<(), EnterpriseCloudDeploymentEvidenceError> {
    validate_slug(
        value,
        EnterpriseCloudDeploymentEvidenceError::InvalidWorkloadScope,
    )
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: EnterpriseCloudDeploymentEvidenceError,
) -> Result<(), EnterpriseCloudDeploymentEvidenceError> {
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

fn validate_doc_url(value: &str) -> Result<(), EnterpriseCloudDeploymentEvidenceError> {
    if !matches!(
        value,
        "https://argo-cd.readthedocs.io/en/stable/"
            | "https://argo-cd.readthedocs.io/en/stable/user-guide/auto_sync/"
            | "https://docs.sigstore.dev/cosign/verifying/verify/"
            | "https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/"
            | "https://kubernetes.io/docs/concepts/policy/resource-quotas/"
            | "https://kubernetes.io/docs/concepts/services-networking/network-policies/"
            | "https://kubernetes.io/docs/concepts/security/service-accounts/"
            | "https://kubernetes.io/docs/reference/kubernetes-api/apps/deployment-v1/"
            | "https://kubernetes.io/docs/concepts/workloads/pods/probes/"
            | "https://gateway-api.sigs.k8s.io/docs/introduction/"
            | "https://opentelemetry.io/docs/specs/semconv/resource/service/"
            | "https://cloudevents.io/"
    ) {
        return Err(EnterpriseCloudDeploymentEvidenceError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn require_control(
    value: bool,
    name: &'static str,
) -> Result<(), EnterpriseCloudDeploymentEvidenceError> {
    if value {
        Ok(())
    } else {
        Err(EnterpriseCloudDeploymentEvidenceError::MissingRequiredControl(name))
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
