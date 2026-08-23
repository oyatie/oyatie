use iam_tenant_rbac_deployment_manifest::{
    CloudDeploymentManifestError, tenant_rbac_deployment_manifest,
    validate_cloud_deployment_manifest,
};

#[test]
fn cloud_deployment_manifest_preserves_gitops_security_and_nonclaims() {
    let manifest = tenant_rbac_deployment_manifest();
    validate_cloud_deployment_manifest(&manifest).expect("manifest validates");

    assert_eq!(manifest.service_name, "tenant-rbac");
    assert!(manifest.namespace.starts_with("oyatie-"));
    assert!(manifest.container_image_ref.contains("@sha256:"));
    assert_eq!(manifest.readiness_probe_path, "/health");
    assert_eq!(manifest.liveness_probe_path, "/health");
    assert!(manifest.replicas_min >= 2);
    assert!(manifest.limit_cpu_millicores >= manifest.request_cpu_millicores);
    assert!(manifest.limit_memory_mib >= manifest.request_memory_mib);
    assert!(
        manifest
            .argocd_application_ref
            .starts_with("argocd/applications/")
    );
    assert!(manifest.jenkins_quality_gate_ref.ends_with("Jenkinsfile"));
    assert!(manifest.cosign_policy_ref.starts_with("policy/cosign/"));
    assert!(manifest.network_policy_ref.starts_with("policy/network/"));
    assert!(!manifest.manual_kubectl_apply_allowed);
    assert!(!manifest.helm_cli_deploy_allowed);
    assert!(!manifest.argocd_controller_attached);
    assert!(!manifest.kubernetes_cluster_attached);
    assert!(!manifest.image_published_attached);
    assert!(!manifest.cosign_verification_runtime_attached);
    assert!(!manifest.runtime_otel_export_attached);
    assert!(!manifest.cloud_deployment_evidence_attached);
    assert!(!manifest.production_slo_evidence_attached);
}

#[test]
fn cloud_deployment_manifest_rejects_imperative_deploy_overclaims() {
    let mut manifest = tenant_rbac_deployment_manifest();
    manifest.manual_kubectl_apply_allowed = true;
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::ImperativeDeployForbidden)
    );

    let mut manifest = tenant_rbac_deployment_manifest();
    manifest.argocd_controller_attached = true;
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::RuntimeAttachmentOverclaim)
    );
}

#[test]
fn cloud_deployment_manifest_validates_namespace_image_resources_and_refs() {
    let mut manifest = tenant_rbac_deployment_manifest();
    manifest.namespace = "default";
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::InvalidNamespace)
    );

    let mut manifest = tenant_rbac_deployment_manifest();
    manifest.container_image_ref = "registry.oyatie.internal/tenant-rbac/runtime:latest";
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::InvalidImageRef)
    );

    let mut manifest = tenant_rbac_deployment_manifest();
    let zero_digest = format!("sha256:{}", "0".repeat(64));
    let zero_image_ref = format!("registry.oyatie.internal/tenant-rbac/runtime@{zero_digest}");
    manifest.container_image_ref = Box::leak(zero_image_ref.into_boxed_str());
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::InvalidImageRef)
    );

    let mut manifest = tenant_rbac_deployment_manifest();
    let repeated_digest = format!("sha256:{}", "1".repeat(64));
    let repeated_image_ref =
        format!("registry.oyatie.internal/tenant-rbac/runtime@{repeated_digest}");
    manifest.container_image_ref = Box::leak(repeated_image_ref.into_boxed_str());
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::InvalidImageRef)
    );

    let mut manifest = tenant_rbac_deployment_manifest();
    manifest.limit_memory_mib = manifest.request_memory_mib - 1;
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::InvalidResources)
    );

    let mut manifest = tenant_rbac_deployment_manifest();
    manifest.cosign_policy_ref = "policy/cosign/secret-token";
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::InvalidCosignPolicyRef)
    );
}

#[test]
fn cloud_deployment_manifest_validates_replica_probe_and_slo_bounds() {
    let mut manifest = tenant_rbac_deployment_manifest();
    manifest.replicas_min = 1;
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::InvalidReplicaRange)
    );

    let mut manifest = tenant_rbac_deployment_manifest();
    manifest.readiness_probe_path = "health";
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::InvalidProbePath)
    );

    let mut manifest = tenant_rbac_deployment_manifest();
    manifest.slo_success_rate_bps = 9_899;
    assert_eq!(
        validate_cloud_deployment_manifest(&manifest),
        Err(CloudDeploymentManifestError::InvalidSloTarget)
    );
}
