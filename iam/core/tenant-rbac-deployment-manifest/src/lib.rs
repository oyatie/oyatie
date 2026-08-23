//! Tenant RBAC cloud deployment manifest foundation.
//!
//! This control-plane crate models the declarative Kubernetes, ArgoCD, Jenkins,
//! image-signature, observability, and SLO evidence surfaces required before a
//! future Oyatie cloud deployment can be claimed. It deliberately does not apply
//! manifests, contact a cluster, publish images, run ArgoCD/Jenkins, verify
//! Cosign signatures, emit telemetry, create namespaces, or claim a deployed
//! listener, cloud deployment evidence, or production SLO.
#![forbid(unsafe_code)]

const SCHEMA_VERSION: u32 = 1;
const MIN_REPLICAS: u16 = 2;
const MAX_REPLICAS: u16 = 100;
const MIN_SUCCESS_RATE_BPS: u16 = 9_900;
const RELEASE_INJECTED_IMAGE_REF: &str = "registry.oyatie.internal/tenant-rbac/runtime@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacCloudDeploymentManifest {
    pub manifest_name: &'static str,                // data_class: PUBLIC
    pub service_name: &'static str,                 // data_class: PUBLIC
    pub namespace: &'static str,                    // data_class: INTERNAL_ONLY
    pub deployment_name: &'static str,              // data_class: INTERNAL_ONLY
    pub service_account_name: &'static str,         // data_class: INTERNAL_ONLY
    pub container_image_ref: &'static str,          // data_class: INTERNAL_ONLY
    pub container_port: u16,                        // data_class: PUBLIC
    pub replicas_min: u16,                          // data_class: PUBLIC
    pub replicas_max: u16,                          // data_class: PUBLIC
    pub request_cpu_millicores: u16,                // data_class: PUBLIC
    pub limit_cpu_millicores: u16,                  // data_class: PUBLIC
    pub request_memory_mib: u16,                    // data_class: PUBLIC
    pub limit_memory_mib: u16,                      // data_class: PUBLIC
    pub readiness_probe_path: &'static str,         // data_class: PUBLIC
    pub liveness_probe_path: &'static str,          // data_class: PUBLIC
    pub argocd_application_ref: &'static str,       // data_class: INTERNAL_ONLY
    pub argocd_project: &'static str,               // data_class: INTERNAL_ONLY
    pub gitops_repo_path: &'static str,             // data_class: INTERNAL_ONLY
    pub jenkins_quality_gate_ref: &'static str,     // data_class: INTERNAL_ONLY
    pub cosign_policy_ref: &'static str,            // data_class: INTERNAL_ONLY
    pub network_policy_ref: &'static str,           // data_class: INTERNAL_ONLY
    pub otel_collector_ref: &'static str,           // data_class: INTERNAL_ONLY
    pub slo_success_rate_bps: u16,                  // data_class: PUBLIC
    pub manual_kubectl_apply_allowed: bool,         // data_class: PUBLIC
    pub helm_cli_deploy_allowed: bool,              // data_class: PUBLIC
    pub argocd_controller_attached: bool,           // data_class: INTERNAL_ONLY
    pub kubernetes_cluster_attached: bool,          // data_class: INTERNAL_ONLY
    pub image_published_attached: bool,             // data_class: INTERNAL_ONLY
    pub cosign_verification_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_otel_export_attached: bool,         // data_class: INTERNAL_ONLY
    pub cloud_deployment_evidence_attached: bool,   // data_class: INTERNAL_ONLY
    pub production_slo_evidence_attached: bool,     // data_class: INTERNAL_ONLY
    pub schema_version: u32,                        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudDeploymentManifestError {
    InvalidManifestName,
    InvalidServiceName,
    InvalidNamespace,
    InvalidDeploymentName,
    InvalidServiceAccountName,
    InvalidImageRef,
    InvalidPort,
    InvalidReplicaRange,
    InvalidResources,
    InvalidProbePath,
    InvalidArgoCdApplicationRef,
    InvalidArgoCdProject,
    InvalidGitOpsRepoPath,
    InvalidJenkinsQualityGateRef,
    InvalidCosignPolicyRef,
    InvalidNetworkPolicyRef,
    InvalidOtelCollectorRef,
    InvalidSloTarget,
    ImperativeDeployForbidden,
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_deployment_manifest() -> TenantRbacCloudDeploymentManifest {
    TenantRbacCloudDeploymentManifest {
        manifest_name: "tenant-rbac-deployment-manifest",
        service_name: "tenant-rbac",
        namespace: "oyatie-tenant-rbac-dev",
        deployment_name: "tenant-rbac-runtime",
        service_account_name: "sa-tenant-rbac-runtime",
        container_image_ref: RELEASE_INJECTED_IMAGE_REF,
        container_port: 8080,
        replicas_min: MIN_REPLICAS,
        replicas_max: 12,
        request_cpu_millicores: 250,
        limit_cpu_millicores: 1_000,
        request_memory_mib: 512,
        limit_memory_mib: 2_048,
        readiness_probe_path: "/health",
        liveness_probe_path: "/health",
        argocd_application_ref: "argocd/applications/tenant-rbac-dev",
        argocd_project: "oyatie-tenant-rbac-apps",
        gitops_repo_path: "deploy/tenant-rbac/overlays/dev",
        jenkins_quality_gate_ref: "microservices/tenant-rbac/ci/Jenkinsfile",
        cosign_policy_ref: "policy/cosign/tenant-rbac-runtime-required",
        network_policy_ref: "policy/network/tenant-rbac-default-deny-egress-allowlist",
        otel_collector_ref: "otel/collector/tenant-rbac-dev",
        slo_success_rate_bps: MIN_SUCCESS_RATE_BPS,
        manual_kubectl_apply_allowed: false,
        helm_cli_deploy_allowed: false,
        argocd_controller_attached: false,
        kubernetes_cluster_attached: false,
        image_published_attached: false,
        cosign_verification_runtime_attached: false,
        runtime_otel_export_attached: false,
        cloud_deployment_evidence_attached: false,
        production_slo_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

pub fn validate_cloud_deployment_manifest(
    manifest: &TenantRbacCloudDeploymentManifest,
) -> Result<(), CloudDeploymentManifestError> {
    validate_slug(
        manifest.manifest_name,
        CloudDeploymentManifestError::InvalidManifestName,
    )?;
    validate_slug(
        manifest.service_name,
        CloudDeploymentManifestError::InvalidServiceName,
    )?;
    validate_namespace(manifest.namespace)?;
    validate_slug(
        manifest.deployment_name,
        CloudDeploymentManifestError::InvalidDeploymentName,
    )?;
    validate_prefixed_ref(
        manifest.service_account_name,
        "sa-",
        CloudDeploymentManifestError::InvalidServiceAccountName,
    )?;
    validate_digest_pinned_image(manifest.container_image_ref)?;
    validate_port(manifest.container_port)?;
    validate_replica_range(manifest.replicas_min, manifest.replicas_max)?;
    validate_resources(manifest)?;
    validate_probe_path(manifest.readiness_probe_path)?;
    validate_probe_path(manifest.liveness_probe_path)?;
    validate_prefixed_ref(
        manifest.argocd_application_ref,
        "argocd/applications/",
        CloudDeploymentManifestError::InvalidArgoCdApplicationRef,
    )?;
    validate_slug(
        manifest.argocd_project,
        CloudDeploymentManifestError::InvalidArgoCdProject,
    )?;
    validate_prefixed_ref(
        manifest.gitops_repo_path,
        "deploy/",
        CloudDeploymentManifestError::InvalidGitOpsRepoPath,
    )?;
    validate_prefixed_ref(
        manifest.jenkins_quality_gate_ref,
        "microservices/tenant-rbac/ci/",
        CloudDeploymentManifestError::InvalidJenkinsQualityGateRef,
    )?;
    validate_prefixed_ref(
        manifest.cosign_policy_ref,
        "policy/cosign/",
        CloudDeploymentManifestError::InvalidCosignPolicyRef,
    )?;
    validate_prefixed_ref(
        manifest.network_policy_ref,
        "policy/network/",
        CloudDeploymentManifestError::InvalidNetworkPolicyRef,
    )?;
    validate_prefixed_ref(
        manifest.otel_collector_ref,
        "otel/collector/",
        CloudDeploymentManifestError::InvalidOtelCollectorRef,
    )?;
    if manifest.slo_success_rate_bps < MIN_SUCCESS_RATE_BPS
        || manifest.slo_success_rate_bps > 10_000
    {
        return Err(CloudDeploymentManifestError::InvalidSloTarget);
    }
    if manifest.manual_kubectl_apply_allowed || manifest.helm_cli_deploy_allowed {
        return Err(CloudDeploymentManifestError::ImperativeDeployForbidden);
    }
    if manifest.argocd_controller_attached
        || manifest.kubernetes_cluster_attached
        || manifest.image_published_attached
        || manifest.cosign_verification_runtime_attached
        || manifest.runtime_otel_export_attached
        || manifest.cloud_deployment_evidence_attached
        || manifest.production_slo_evidence_attached
    {
        return Err(CloudDeploymentManifestError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: CloudDeploymentManifestError,
) -> Result<(), CloudDeploymentManifestError> {
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

fn validate_namespace(value: &str) -> Result<(), CloudDeploymentManifestError> {
    validate_slug(value, CloudDeploymentManifestError::InvalidNamespace)?;
    if !value.starts_with("oyatie-") || matches!(value, "default" | "kube-system" | "kube-public") {
        return Err(CloudDeploymentManifestError::InvalidNamespace);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: CloudDeploymentManifestError,
) -> Result<(), CloudDeploymentManifestError> {
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

fn validate_digest_pinned_image(value: &str) -> Result<(), CloudDeploymentManifestError> {
    let Some((registry_path, digest)) = value.split_once("@sha256:") else {
        return Err(CloudDeploymentManifestError::InvalidImageRef);
    };
    let first_digest_char = digest.as_bytes().first().copied().unwrap_or_default() as char;
    if registry_path.is_empty()
        || !registry_path.starts_with("registry.oyatie.internal/")
        || has_unsafe_text(registry_path)
        || has_path_traversal(registry_path)
        || digest.len() != 64
        || !digest
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
        || !digest.chars().any(|ch| ch != '0')
        || digest.chars().all(|ch| ch == first_digest_char)
    {
        return Err(CloudDeploymentManifestError::InvalidImageRef);
    }
    Ok(())
}

fn validate_port(value: u16) -> Result<(), CloudDeploymentManifestError> {
    if value == 0 || value < 1024 {
        return Err(CloudDeploymentManifestError::InvalidPort);
    }
    Ok(())
}

fn validate_replica_range(min: u16, max: u16) -> Result<(), CloudDeploymentManifestError> {
    if min < MIN_REPLICAS || max < min || max > MAX_REPLICAS {
        return Err(CloudDeploymentManifestError::InvalidReplicaRange);
    }
    Ok(())
}

fn validate_resources(
    manifest: &TenantRbacCloudDeploymentManifest,
) -> Result<(), CloudDeploymentManifestError> {
    if manifest.request_cpu_millicores == 0
        || manifest.request_memory_mib == 0
        || manifest.limit_cpu_millicores < manifest.request_cpu_millicores
        || manifest.limit_memory_mib < manifest.request_memory_mib
    {
        return Err(CloudDeploymentManifestError::InvalidResources);
    }
    Ok(())
}

fn validate_probe_path(value: &str) -> Result<(), CloudDeploymentManifestError> {
    if !value.starts_with('/')
        || value.len() < 2
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || has_credential_shape(value)
    {
        return Err(CloudDeploymentManifestError::InvalidProbePath);
    }
    Ok(())
}

fn has_unsafe_text(value: &str) -> bool {
    value.chars().any(char::is_whitespace) || value.chars().any(char::is_control)
}

fn has_path_traversal(value: &str) -> bool {
    value.contains("..") || value.contains('\\') || value.contains("//")
}

fn has_credential_shape(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("secret")
        || lower.contains("token")
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("api_key")
}
