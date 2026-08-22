use iam_tenant_rbac_tenant_workload_identity_contract::{
    Fd001TenantWorkloadIdentityError, Fd001TenantWorkloadIdentityRequirementKind,
    fd001_tenant_workload_identity_contract, fd001_tenant_workload_identity_doc_urls,
    validate_fd001_tenant_workload_identity_contract,
};
use iam_tenant_rbac_tenant_workload_manifest::Fd001TenantWorkloadKind;

#[test]
fn tenant_workload_identity_contract_validates_controls_and_nonclaims() {
    let contract = fd001_tenant_workload_identity_contract().expect("contract builds");
    validate_fd001_tenant_workload_identity_contract(&contract).expect("contract validates");

    assert_eq!(
        contract.contract_name,
        "fd001-tenant-rbac-tenant-workload-identity-contract"
    );
    assert_eq!(contract.program_name, "fd-001-tenant-rbac-generic");
    assert_eq!(contract.substrate_name, "oyatie-cloud");
    assert_eq!(contract.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(contract.trust_domain, "oyatie.dev");
    assert_eq!(
        contract.spiffe_id_prefix,
        "spiffe://oyatie.dev/fd001-tenant-rbac/"
    );
    assert_eq!(contract.workload_manifest_count, 4);
    assert_eq!(
        contract.tenant_admission_policy_contract_name,
        "fd001-tenant-rbac-tenant-admission-policy-contract"
    );
    assert_eq!(contract.requirements.len(), 13);
    assert!(contract.official_docs_required);
    assert!(contract.all_manifest_workloads_in_scope);
    assert!(contract.spiffe_id_required);
    assert!(contract.trust_domain_pinned);
    assert!(contract.x509_svid_required);
    assert!(contract.jwt_svid_policy_required);
    assert!(contract.mutual_tls_required);
    assert!(contract.gateway_backend_tls_policy_required);
    assert!(contract.certificate_rotation_evidence_required);
    assert!(contract.trust_bundle_evidence_required);
    assert!(contract.workload_api_boundary_required);
    assert!(contract.workload_attestation_selector_required);
    assert!(contract.service_telemetry_identity_required);
    assert!(contract.authorization_policy_binding_required);
    assert!(contract.identity_audit_evidence_required);
    assert!(contract.review_only_contract);
    assert!(!contract.kubernetes_cluster_attached);
    assert!(!contract.spiffe_workload_api_attached);
    assert!(!contract.spire_server_runtime_attached);
    assert!(!contract.spire_agent_runtime_attached);
    assert!(!contract.svid_runtime_issued);
    assert!(!contract.mtls_handshake_observed);
    assert!(!contract.certificate_rotation_runtime_observed);
    assert!(!contract.gateway_backend_tls_applied);
    assert!(!contract.authorization_policy_runtime_attached);
    assert!(!contract.workload_runtime_deployed);
    assert!(!contract.cloud_substrate_runtime_attached);
    assert!(!contract.runtime_audit_chain_emission_attached);
}

#[test]
fn tenant_workload_identity_contract_covers_workloads_requirements_and_docs() {
    let contract = fd001_tenant_workload_identity_contract().expect("contract builds");
    let workloads = contract
        .requirements
        .iter()
        .map(|requirement| requirement.workload_kind)
        .collect::<std::collections::BTreeSet<_>>();
    for workload in [
        Fd001TenantWorkloadKind::TenantRbac,
        Fd001TenantWorkloadKind::HrEmployment,
        Fd001TenantWorkloadKind::PayrollRun,
        Fd001TenantWorkloadKind::AccountingJournal,
    ] {
        assert!(workloads.contains(&workload), "missing {workload:?}");
    }

    let requirement_kinds = contract
        .requirements
        .iter()
        .map(|requirement| requirement.requirement_kind)
        .collect::<std::collections::BTreeSet<_>>();
    for kind in [
        Fd001TenantWorkloadIdentityRequirementKind::SpiffeIdRequired,
        Fd001TenantWorkloadIdentityRequirementKind::TrustDomainPinned,
        Fd001TenantWorkloadIdentityRequirementKind::X509SvidRequired,
        Fd001TenantWorkloadIdentityRequirementKind::JwtSvidPolicyRequired,
        Fd001TenantWorkloadIdentityRequirementKind::MutualTlsRequired,
        Fd001TenantWorkloadIdentityRequirementKind::GatewayBackendTlsPolicyRequired,
        Fd001TenantWorkloadIdentityRequirementKind::CertificateRotationEvidenceRequired,
        Fd001TenantWorkloadIdentityRequirementKind::TrustBundleEvidenceRequired,
        Fd001TenantWorkloadIdentityRequirementKind::WorkloadApiBoundaryRequired,
        Fd001TenantWorkloadIdentityRequirementKind::WorkloadAttestationSelectorRequired,
        Fd001TenantWorkloadIdentityRequirementKind::ServiceTelemetryIdentityRequired,
        Fd001TenantWorkloadIdentityRequirementKind::AuthorizationPolicyBindingRequired,
        Fd001TenantWorkloadIdentityRequirementKind::IdentityAuditEvidenceRequired,
    ] {
        assert!(requirement_kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = fd001_tenant_workload_identity_doc_urls(&contract);
    assert!(docs.contains(&"https://spiffe.io/docs/latest/spiffe-about/overview/"));
    assert!(docs.contains(&"https://spiffe.io/docs/latest/spiffe-about/spiffe-concepts/"));
    assert!(docs.contains(&"https://spiffe.io/docs/latest/spire-about/spire-concepts/"));
    assert!(docs.contains(&"https://gateway-api.sigs.k8s.io/api-types/backendtlspolicy/"));
    assert!(docs.contains(&"https://opentelemetry.io/docs/specs/semconv/resource/service/"));
    assert!(docs.contains(&"https://kubernetes.io/docs/concepts/security/service-accounts/"));
}

#[test]
fn tenant_workload_identity_contract_preserves_refs_resources_and_scope() {
    let contract = fd001_tenant_workload_identity_contract().expect("contract builds");

    assert!(contract.requirements.iter().all(|requirement| {
        [
            "SpiffeId",
            "SpiffeTrustDomain",
            "X509Svid",
            "JwtSvid",
            "MutualTlsPolicy",
            "GatewayBackendTlsPolicy",
            "CertificateRotationPolicy",
            "TrustBundle",
            "SpiffeWorkloadApi",
            "WorkloadAttestationSelector",
            "OpenTelemetryServiceResource",
            "AuthorizationPolicyBinding",
            "IdentityAuditEvidence",
        ]
        .contains(&requirement.resource_kind)
            && requirement
                .policy_ref
                .starts_with("policy/workload-identity/fd001/")
            && requirement
                .expected_evidence_ref
                .starts_with("evidence/workload-identity/fd001-tenant-rbac/")
            && requirement
                .source_manifest_ref
                .starts_with("crates/tenant-rbac-tenant-workload-manifest/")
            && requirement
                .source_admission_policy_ref
                .starts_with("crates/tenant-rbac-tenant-admission-policy/")
            && requirement
                .spiffe_id
                .starts_with("spiffe://oyatie.dev/fd001-tenant-rbac/")
            && requirement.applies_to_all_manifest_workloads
            && !requirement.runtime_observation_attached
    }));
}

#[test]
fn tenant_workload_identity_contract_rejects_missing_duplicate_doc_and_ref_drift() {
    let mut contract = fd001_tenant_workload_identity_contract().expect("contract builds");
    contract.requirements.truncate(3);
    assert_eq!(
        validate_fd001_tenant_workload_identity_contract(&contract),
        Err(Fd001TenantWorkloadIdentityError::MissingRequirements)
    );

    let mut contract = fd001_tenant_workload_identity_contract().expect("contract builds");
    contract.requirements[1] = contract.requirements[0].clone();
    assert_eq!(
        validate_fd001_tenant_workload_identity_contract(&contract),
        Err(Fd001TenantWorkloadIdentityError::DuplicateRequirement(
            "spiffe-id-all-workloads".to_owned()
        ))
    );

    let mut contract = fd001_tenant_workload_identity_contract().expect("contract builds");
    contract.requirements[0].official_doc_url = "https://example.com/workload-identity";
    assert_eq!(
        validate_fd001_tenant_workload_identity_contract(&contract),
        Err(Fd001TenantWorkloadIdentityError::InvalidOfficialDocUrl)
    );

    let mut contract = fd001_tenant_workload_identity_contract().expect("contract builds");
    contract.requirements[0].policy_ref = "deploy/workload-identity/fd001/spiffe-id";
    assert_eq!(
        validate_fd001_tenant_workload_identity_contract(&contract),
        Err(Fd001TenantWorkloadIdentityError::InvalidPolicyRef)
    );
}

#[test]
fn tenant_workload_identity_contract_rejects_missing_controls_and_overclaims() {
    let mut contract = fd001_tenant_workload_identity_contract().expect("contract builds");
    contract.spiffe_id_required = false;
    assert_eq!(
        validate_fd001_tenant_workload_identity_contract(&contract),
        Err(Fd001TenantWorkloadIdentityError::MissingRequiredControl(
            "spiffe_id_required"
        ))
    );

    let mut contract = fd001_tenant_workload_identity_contract().expect("contract builds");
    contract.all_manifest_workloads_in_scope = false;
    assert_eq!(
        validate_fd001_tenant_workload_identity_contract(&contract),
        Err(Fd001TenantWorkloadIdentityError::MissingRequiredControl(
            "all_manifest_workloads_in_scope"
        ))
    );

    let mut contract = fd001_tenant_workload_identity_contract().expect("contract builds");
    contract.requirements[0].applies_to_all_manifest_workloads = false;
    assert_eq!(
        validate_fd001_tenant_workload_identity_contract(&contract),
        Err(Fd001TenantWorkloadIdentityError::MissingRequiredControl(
            "requirement_applies_to_all_manifest_workloads"
        ))
    );

    let mut contract = fd001_tenant_workload_identity_contract().expect("contract builds");
    contract.mtls_handshake_observed = true;
    assert_eq!(
        validate_fd001_tenant_workload_identity_contract(&contract),
        Err(Fd001TenantWorkloadIdentityError::RuntimeAttachmentOverclaim)
    );
}
