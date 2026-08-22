//! FD-001 tenant-workload manifest for later Oyatie Cloud dogfooding.
//!
//! This control-plane crate records the review-only workload contract that lets
//! Tenant RBAC, HR, Payroll, and Accounting run as tenant workloads on the
//! future Oyatie Cloud substrate. It deliberately does not create namespaces,
//! apply quotas or network policies, attach Gateway routes, deploy workloads,
//! emit runtime audit-chain events, or claim a live cloud substrate.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

const SCHEMA_VERSION: u32 = 1;
const MIN_WORKLOAD_COUNT: usize = 4;
const MANIFEST_NAME: &str = "fd001-tenant-rbac-tenant-workload-manifest";
const PROGRAM_NAME: &str = "fd-001-tenant-rbac-generic";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const TENANT_CELL_ID: &str = "cell-us-east-001";
const RESIDENCY_REGION: &str = "us-east-1";
const OTEL_SERVICE_NAMESPACE: &str = "fd001-tenant-rbac";
const TENANT_CLAIM: &str = "tenant_id";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Fd001TenantWorkloadKind {
    TenantRbac,
    HrEmployment,
    PayrollRun,
    AccountingJournal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantWorkload {
    pub workload_id: &'static str,                   // data_class: PUBLIC
    pub service_name: &'static str,                  // data_class: PUBLIC
    pub package_name: &'static str,                  // data_class: PUBLIC
    pub workload_kind: Fd001TenantWorkloadKind,      // data_class: PUBLIC
    pub tenant_namespace: &'static str,              // data_class: INTERNAL_ONLY
    pub tenant_cell_id: &'static str,                // data_class: INTERNAL_ONLY
    pub residency_region: &'static str,              // data_class: INTERNAL_ONLY
    pub runtime_package_ref: &'static str,           // data_class: INTERNAL_ONLY
    pub route_scope_ref: &'static str,               // data_class: INTERNAL_ONLY
    pub deployment_manifest_ref: &'static str,       // data_class: INTERNAL_ONLY
    pub resource_quota_ref: &'static str,            // data_class: INTERNAL_ONLY
    pub network_policy_ref: &'static str,            // data_class: INTERNAL_ONLY
    pub service_account_ref: &'static str,           // data_class: INTERNAL_ONLY
    pub gateway_route_ref: &'static str,             // data_class: INTERNAL_ONLY
    pub otel_service_namespace: &'static str,        // data_class: INTERNAL_ONLY
    pub tenant_claim: &'static str,                  // data_class: INTERNAL_ONLY
    pub evidence_ref: &'static str,                  // data_class: INTERNAL_ONLY
    pub namespace_isolation_required: bool,          // data_class: PUBLIC
    pub resource_quota_required: bool,               // data_class: PUBLIC
    pub network_policy_required: bool,               // data_class: PUBLIC
    pub service_account_boundary_required: bool,     // data_class: PUBLIC
    pub gateway_route_required: bool,                // data_class: PUBLIC
    pub route_auth_scope_required: bool,             // data_class: PUBLIC
    pub rls_tenant_claim_required: bool,             // data_class: PUBLIC
    pub otel_resource_identity_required: bool,       // data_class: PUBLIC
    pub production_runtime_attached: bool,           // data_class: INTERNAL_ONLY
    pub cloud_deployment_attached: bool,             // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                         // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fd001TenantWorkloadManifest {
    pub manifest_name: &'static str,                 // data_class: PUBLIC
    pub program_name: &'static str,                  // data_class: PUBLIC
    pub substrate_name: &'static str,                // data_class: PUBLIC
    pub tenant_namespace: &'static str,              // data_class: INTERNAL_ONLY
    pub tenant_cell_id: &'static str,                // data_class: INTERNAL_ONLY
    pub residency_region: &'static str,              // data_class: INTERNAL_ONLY
    pub workloads: Vec<Fd001TenantWorkload>,         // data_class: INTERNAL_ONLY
    pub official_doc_urls: Vec<&'static str>,        // data_class: PUBLIC
    pub fd001_product_goal_preserved: bool,          // data_class: PUBLIC
    pub oyatie_cloud_substrate_only: bool,           // data_class: PUBLIC
    pub review_only_contract: bool,                  // data_class: PUBLIC
    pub namespace_isolation_required: bool,          // data_class: PUBLIC
    pub resource_quota_required: bool,               // data_class: PUBLIC
    pub network_policy_required: bool,               // data_class: PUBLIC
    pub service_account_boundary_required: bool,     // data_class: PUBLIC
    pub gateway_route_required: bool,                // data_class: PUBLIC
    pub route_auth_scope_required: bool,             // data_class: PUBLIC
    pub tenant_claim_required: bool,                 // data_class: PUBLIC
    pub legal_entity_claim_required: bool,           // data_class: PUBLIC
    pub otel_resource_identity_required: bool,       // data_class: PUBLIC
    pub per_workload_evidence_required: bool,        // data_class: PUBLIC
    pub production_tenant_attached: bool,            // data_class: INTERNAL_ONLY
    pub kubernetes_namespace_created: bool,          // data_class: INTERNAL_ONLY
    pub resource_quota_applied: bool,                // data_class: INTERNAL_ONLY
    pub network_policy_applied: bool,                // data_class: INTERNAL_ONLY
    pub gateway_route_attached: bool,                // data_class: INTERNAL_ONLY
    pub workload_runtime_deployed: bool,             // data_class: INTERNAL_ONLY
    pub cloud_substrate_runtime_attached: bool,      // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                         // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fd001TenantWorkloadManifestError {
    InvalidManifestName,
    InvalidProgramName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidTenantCellId,
    InvalidResidencyRegion,
    InvalidWorkloadId,
    InvalidServiceName,
    InvalidPackageName,
    InvalidRuntimePackageRef,
    InvalidRouteScopeRef,
    InvalidDeploymentManifestRef,
    InvalidResourceQuotaRef,
    InvalidNetworkPolicyRef,
    InvalidServiceAccountRef,
    InvalidGatewayRouteRef,
    InvalidOtelServiceNamespace,
    InvalidTenantClaim,
    InvalidEvidenceRef,
    InvalidOfficialDocUrl,
    MissingWorkloads,
    DuplicateWorkload(String),
    MissingWorkloadKind(Fd001TenantWorkloadKind),
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn fd001_tenant_workload_manifest() -> Fd001TenantWorkloadManifest {
    Fd001TenantWorkloadManifest {
        manifest_name: MANIFEST_NAME,
        program_name: PROGRAM_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: TENANT_NAMESPACE,
        tenant_cell_id: TENANT_CELL_ID,
        residency_region: RESIDENCY_REGION,
        workloads: vec![
            workload(
                Fd001TenantWorkloadKind::TenantRbac,
                "tenant-rbac-runtime",
                "tenant-rbac",
                "tenant-rbac-app",
                "crates/tenant-rbac-app",
                "evidence/multispectrum/cs-ent-platform-runtime-adapter-foundation-1779537600.json",
            ),
            workload(
                Fd001TenantWorkloadKind::HrEmployment,
                "hr-employment-runtime",
                "hr-employment",
                "hr-employment-infrastructure",
                "app/hr/adapters/employment-infrastructure",
                "evidence/multispectrum/cs-ent-hr-runtime-adapter-foundation-1779536400.json",
            ),
            workload(
                Fd001TenantWorkloadKind::PayrollRun,
                "payroll-run-runtime",
                "payroll-run",
                "payroll-run-infrastructure",
                "app/payroll/adapters/run-infrastructure",
                "evidence/multispectrum/cs-ent-payroll-runtime-adapter-foundation-1779535800.json",
            ),
            workload(
                Fd001TenantWorkloadKind::AccountingJournal,
                "accounting-journal-runtime",
                "accounting-journal",
                "accounting-journal-infrastructure",
                "crates/accounting-journal-infrastructure",
                "evidence/multispectrum/cs-ent-accounting-runtime-adapter-foundation-1779537000.json",
            ),
        ],
        official_doc_urls: vec![
            "https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/",
            "https://kubernetes.io/docs/concepts/policy/resource-quotas/",
            "https://kubernetes.io/docs/concepts/services-networking/network-policies/",
            "https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/",
            "https://kubernetes.io/docs/concepts/workloads/pods/probes/",
            "https://gateway-api.sigs.k8s.io/docs/introduction/",
            "https://opentelemetry.io/docs/specs/semconv/resource/service/",
        ],
        fd001_product_goal_preserved: true,
        oyatie_cloud_substrate_only: true,
        review_only_contract: true,
        namespace_isolation_required: true,
        resource_quota_required: true,
        network_policy_required: true,
        service_account_boundary_required: true,
        gateway_route_required: true,
        route_auth_scope_required: true,
        tenant_claim_required: true,
        legal_entity_claim_required: true,
        otel_resource_identity_required: true,
        per_workload_evidence_required: true,
        production_tenant_attached: false,
        kubernetes_namespace_created: false,
        resource_quota_applied: false,
        network_policy_applied: false,
        gateway_route_attached: false,
        workload_runtime_deployed: false,
        cloud_substrate_runtime_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

pub fn validate_fd001_tenant_workload_manifest(
    manifest: &Fd001TenantWorkloadManifest,
) -> Result<(), Fd001TenantWorkloadManifestError> {
    validate_slug(
        manifest.manifest_name,
        Fd001TenantWorkloadManifestError::InvalidManifestName,
    )?;
    if manifest.program_name != PROGRAM_NAME {
        return Err(Fd001TenantWorkloadManifestError::InvalidProgramName);
    }
    if manifest.substrate_name != SUBSTRATE_NAME {
        return Err(Fd001TenantWorkloadManifestError::InvalidSubstrateName);
    }
    validate_tenant_namespace(manifest.tenant_namespace)?;
    validate_cell_id(manifest.tenant_cell_id)?;
    validate_region(manifest.residency_region)?;
    if manifest.workloads.len() < MIN_WORKLOAD_COUNT {
        return Err(Fd001TenantWorkloadManifestError::MissingWorkloads);
    }
    for url in &manifest.official_doc_urls {
        validate_doc_url(url)?;
    }
    for control in [
        (
            manifest.fd001_product_goal_preserved,
            "fd001_product_goal_preserved",
        ),
        (
            manifest.oyatie_cloud_substrate_only,
            "oyatie_cloud_substrate_only",
        ),
        (manifest.review_only_contract, "review_only_contract"),
        (
            manifest.namespace_isolation_required,
            "namespace_isolation_required",
        ),
        (manifest.resource_quota_required, "resource_quota_required"),
        (manifest.network_policy_required, "network_policy_required"),
        (
            manifest.service_account_boundary_required,
            "service_account_boundary_required",
        ),
        (manifest.gateway_route_required, "gateway_route_required"),
        (
            manifest.route_auth_scope_required,
            "route_auth_scope_required",
        ),
        (manifest.tenant_claim_required, "tenant_claim_required"),
        (
            manifest.legal_entity_claim_required,
            "legal_entity_claim_required",
        ),
        (
            manifest.otel_resource_identity_required,
            "otel_resource_identity_required",
        ),
        (
            manifest.per_workload_evidence_required,
            "per_workload_evidence_required",
        ),
    ] {
        require_control(control.0, control.1)?;
    }
    if manifest.production_tenant_attached
        || manifest.kubernetes_namespace_created
        || manifest.resource_quota_applied
        || manifest.network_policy_applied
        || manifest.gateway_route_attached
        || manifest.workload_runtime_deployed
        || manifest.cloud_substrate_runtime_attached
        || manifest.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantWorkloadManifestError::RuntimeAttachmentOverclaim);
    }

    let mut seen_workloads = BTreeSet::new();
    let mut seen_kinds = BTreeSet::new();
    for workload in &manifest.workloads {
        validate_workload(workload, manifest)?;
        if !seen_workloads.insert(workload.workload_id) {
            return Err(Fd001TenantWorkloadManifestError::DuplicateWorkload(
                workload.workload_id.to_owned(),
            ));
        }
        seen_kinds.insert(workload.workload_kind);
    }
    for kind in required_workload_kinds() {
        if !seen_kinds.contains(&kind) {
            return Err(Fd001TenantWorkloadManifestError::MissingWorkloadKind(kind));
        }
    }
    Ok(())
}

pub fn tenant_workload_official_doc_urls(
    manifest: &Fd001TenantWorkloadManifest,
) -> Vec<&'static str> {
    manifest.official_doc_urls.clone()
}

pub fn fd001_workload_count(manifest: &Fd001TenantWorkloadManifest) -> usize {
    manifest.workloads.len()
}

fn workload(
    workload_kind: Fd001TenantWorkloadKind,
    workload_id: &'static str,
    service_name: &'static str,
    package_name: &'static str,
    runtime_package_ref: &'static str,
    evidence_ref: &'static str,
) -> Fd001TenantWorkload {
    Fd001TenantWorkload {
        workload_id,
        service_name,
        package_name,
        workload_kind,
        tenant_namespace: TENANT_NAMESPACE,
        tenant_cell_id: TENANT_CELL_ID,
        residency_region: RESIDENCY_REGION,
        runtime_package_ref,
        route_scope_ref: "crates/tenant-rbac-local-runtime-composition/src/lib.rs::tenant_rbac_local_runtime_composition",
        deployment_manifest_ref: "crates/tenant-rbac-cloud-deployment-manifest/src/lib.rs::tenant_rbac_cloud_deployment_manifest",
        resource_quota_ref: "deploy/oyatie-cloud/fd001-tenant-rbac/resource-quota.yaml",
        network_policy_ref: "deploy/oyatie-cloud/fd001-tenant-rbac/network-policy.yaml",
        service_account_ref: "serviceaccount/fd001-tenant-rbac-workload",
        gateway_route_ref: "gateway/httproute/fd001-tenant-rbac-workloads",
        otel_service_namespace: OTEL_SERVICE_NAMESPACE,
        tenant_claim: TENANT_CLAIM,
        evidence_ref,
        namespace_isolation_required: true,
        resource_quota_required: true,
        network_policy_required: true,
        service_account_boundary_required: true,
        gateway_route_required: true,
        route_auth_scope_required: true,
        rls_tenant_claim_required: true,
        otel_resource_identity_required: true,
        production_runtime_attached: false,
        cloud_deployment_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_workload(
    workload: &Fd001TenantWorkload,
    manifest: &Fd001TenantWorkloadManifest,
) -> Result<(), Fd001TenantWorkloadManifestError> {
    validate_slug(
        workload.workload_id,
        Fd001TenantWorkloadManifestError::InvalidWorkloadId,
    )?;
    validate_slug(
        workload.service_name,
        Fd001TenantWorkloadManifestError::InvalidServiceName,
    )?;
    validate_package_name(workload.package_name)?;
    if workload.tenant_namespace != manifest.tenant_namespace {
        return Err(Fd001TenantWorkloadManifestError::InvalidTenantNamespace);
    }
    if workload.tenant_cell_id != manifest.tenant_cell_id {
        return Err(Fd001TenantWorkloadManifestError::InvalidTenantCellId);
    }
    if workload.residency_region != manifest.residency_region {
        return Err(Fd001TenantWorkloadManifestError::InvalidResidencyRegion);
    }
    // The runtime package ref used to be pinned to `crates/`, which is the
    // pre-ADR-0562 layout. A capability-first crate lives at
    // `<root>/<face>/<crate>`, so a workload whose crate has been absorbed into
    // app/<product>/<face>/ can no longer satisfy a `crates/` prefix. Accept
    // either: `crates/` for rows whose crate has not moved yet, and `app/` for
    // rows that have. validate_prefixed_ref still guards traversal and shape.
    if validate_prefixed_ref(
        workload.runtime_package_ref,
        "crates/",
        Fd001TenantWorkloadManifestError::InvalidRuntimePackageRef,
    )
    .is_err()
    {
        validate_prefixed_ref(
            workload.runtime_package_ref,
            "app/",
            Fd001TenantWorkloadManifestError::InvalidRuntimePackageRef,
        )?;
    }
    validate_prefixed_ref(
        workload.route_scope_ref,
        "crates/tenant-rbac-local-runtime-composition/",
        Fd001TenantWorkloadManifestError::InvalidRouteScopeRef,
    )?;
    validate_prefixed_ref(
        workload.deployment_manifest_ref,
        "crates/tenant-rbac-cloud-deployment-manifest/",
        Fd001TenantWorkloadManifestError::InvalidDeploymentManifestRef,
    )?;
    validate_prefixed_ref(
        workload.resource_quota_ref,
        "deploy/oyatie-cloud/",
        Fd001TenantWorkloadManifestError::InvalidResourceQuotaRef,
    )?;
    validate_prefixed_ref(
        workload.network_policy_ref,
        "deploy/oyatie-cloud/",
        Fd001TenantWorkloadManifestError::InvalidNetworkPolicyRef,
    )?;
    validate_prefixed_ref(
        workload.service_account_ref,
        "serviceaccount/",
        Fd001TenantWorkloadManifestError::InvalidServiceAccountRef,
    )?;
    validate_prefixed_ref(
        workload.gateway_route_ref,
        "gateway/httproute/",
        Fd001TenantWorkloadManifestError::InvalidGatewayRouteRef,
    )?;
    if workload.otel_service_namespace != OTEL_SERVICE_NAMESPACE {
        return Err(Fd001TenantWorkloadManifestError::InvalidOtelServiceNamespace);
    }
    if workload.tenant_claim != TENANT_CLAIM {
        return Err(Fd001TenantWorkloadManifestError::InvalidTenantClaim);
    }
    validate_prefixed_ref(
        workload.evidence_ref,
        "evidence/multispectrum/",
        Fd001TenantWorkloadManifestError::InvalidEvidenceRef,
    )?;
    for control in [
        (
            workload.namespace_isolation_required,
            "workload_namespace_isolation_required",
        ),
        (
            workload.resource_quota_required,
            "workload_resource_quota_required",
        ),
        (
            workload.network_policy_required,
            "workload_network_policy_required",
        ),
        (
            workload.service_account_boundary_required,
            "workload_service_account_boundary_required",
        ),
        (
            workload.gateway_route_required,
            "workload_gateway_route_required",
        ),
        (
            workload.route_auth_scope_required,
            "workload_route_auth_scope_required",
        ),
        (
            workload.rls_tenant_claim_required,
            "workload_rls_tenant_claim_required",
        ),
        (
            workload.otel_resource_identity_required,
            "workload_otel_resource_identity_required",
        ),
    ] {
        require_control(control.0, control.1)?;
    }
    if workload.production_runtime_attached
        || workload.cloud_deployment_attached
        || workload.runtime_audit_chain_emission_attached
    {
        return Err(Fd001TenantWorkloadManifestError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn required_workload_kinds() -> [Fd001TenantWorkloadKind; 4] {
    [
        Fd001TenantWorkloadKind::TenantRbac,
        Fd001TenantWorkloadKind::HrEmployment,
        Fd001TenantWorkloadKind::PayrollRun,
        Fd001TenantWorkloadKind::AccountingJournal,
    ]
}

fn validate_slug(
    value: &str,
    error: Fd001TenantWorkloadManifestError,
) -> Result<(), Fd001TenantWorkloadManifestError> {
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

fn validate_tenant_namespace(value: &str) -> Result<(), Fd001TenantWorkloadManifestError> {
    validate_slug(
        value,
        Fd001TenantWorkloadManifestError::InvalidTenantNamespace,
    )?;
    if !value.starts_with("oyatie-") || matches!(value, "default" | "kube-system" | "kube-public") {
        return Err(Fd001TenantWorkloadManifestError::InvalidTenantNamespace);
    }
    Ok(())
}

fn validate_cell_id(value: &str) -> Result<(), Fd001TenantWorkloadManifestError> {
    validate_slug(value, Fd001TenantWorkloadManifestError::InvalidTenantCellId)?;
    if !value.starts_with("cell-") {
        return Err(Fd001TenantWorkloadManifestError::InvalidTenantCellId);
    }
    Ok(())
}

fn validate_region(value: &str) -> Result<(), Fd001TenantWorkloadManifestError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
        || !value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(Fd001TenantWorkloadManifestError::InvalidResidencyRegion);
    }
    Ok(())
}

fn validate_package_name(value: &str) -> Result<(), Fd001TenantWorkloadManifestError> {
    validate_slug(value, Fd001TenantWorkloadManifestError::InvalidPackageName)?;
    // The `oya-` prefix is NOT required. This validator used to demand it, which
    // made a brand the contract rather than a naming convention — and the repo is
    // actively removing that prefix: `ci/facade/cloud-name-ratchet` is a
    // shrink-only ratchet over exactly these names, so requiring it here would
    // break every workload row as its crate debrands. Rows still carrying `oya-`
    // (accounting-journal, payroll-run, tenant-rbac-app) are crates that have not
    // moved yet, not a rule.
    //
    // What the contract actually needs is a well-formed package slug, which
    // validate_slug already enforces: lowercase, digits and hyphens only, no path
    // traversal, no credential shape.
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: Fd001TenantWorkloadManifestError,
) -> Result<(), Fd001TenantWorkloadManifestError> {
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

fn validate_doc_url(value: &str) -> Result<(), Fd001TenantWorkloadManifestError> {
    if !matches!(
        value,
        "https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/"
            | "https://kubernetes.io/docs/concepts/policy/resource-quotas/"
            | "https://kubernetes.io/docs/concepts/services-networking/network-policies/"
            | "https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/"
            | "https://kubernetes.io/docs/concepts/workloads/pods/probes/"
            | "https://gateway-api.sigs.k8s.io/docs/introduction/"
            | "https://opentelemetry.io/docs/specs/semconv/resource/service/"
    ) {
        return Err(Fd001TenantWorkloadManifestError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn require_control(
    value: bool,
    name: &'static str,
) -> Result<(), Fd001TenantWorkloadManifestError> {
    if value {
        Ok(())
    } else {
        Err(Fd001TenantWorkloadManifestError::MissingRequiredControl(
            name,
        ))
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
