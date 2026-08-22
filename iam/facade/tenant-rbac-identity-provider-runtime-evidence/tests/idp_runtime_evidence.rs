use iam_tenant_rbac_identity_provider_runtime_evidence::{
    IdentityProviderRuntimeEvidenceRequirementKind, TenantRbacIdentityProviderRuntimeEvidenceError,
    identity_provider_runtime_evidence_doc_urls,
    tenant_rbac_identity_provider_runtime_evidence_plan,
    validate_tenant_rbac_identity_provider_runtime_evidence_plan,
};

#[test]
fn idp_runtime_evidence_plan_validates_controls_and_nonclaims() {
    let plan = tenant_rbac_identity_provider_runtime_evidence_plan().expect("plan builds");
    validate_tenant_rbac_identity_provider_runtime_evidence_plan(&plan).expect("plan validates");

    assert_eq!(
        plan.plan_name,
        "tenant-rbac-identity-provider-runtime-evidence-plan"
    );
    assert_eq!(plan.service_name, "tenant-rbac");
    assert_eq!(plan.substrate_name, "oyatie-cloud");
    assert_eq!(plan.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(
        plan.identity_provider_verification_plan_name,
        "tenant-rbac-identity-provider-verification-plan"
    );
    assert_eq!(plan.issuer, "https://identity.oyatie.com/tenant-rbac");
    assert_eq!(plan.audience, "tenant-rbac-api");
    assert!(
        plan.discovery_document_url
            .ends_with("/.well-known/openid-configuration")
    );
    assert!(plan.jwks_uri.ends_with("/.well-known/jwks.json"));
    assert_eq!(plan.required_claim_count, 11);
    assert_eq!(plan.allowed_algorithm_count, 3);
    assert_eq!(plan.requirements.len(), 15);
    assert!(plan.fd001_product_delivery_master_goal_preserved);
    assert!(plan.oyatie_cloud_substrate_proof_required);
    assert!(plan.official_docs_required);
    assert!(plan.discovery_document_observation_required);
    assert!(plan.issuer_metadata_match_required);
    assert!(plan.jwks_fetch_evidence_required);
    assert!(plan.jwks_kid_match_required);
    assert!(plan.jwt_signature_verification_evidence_required);
    assert!(plan.algorithm_allowlist_required);
    assert!(plan.issuer_claim_match_required);
    assert!(plan.audience_claim_match_required);
    assert!(plan.temporal_claims_check_required);
    assert!(plan.nonce_replay_denial_required);
    assert!(plan.tenant_claim_mapping_required);
    assert!(plan.route_scope_authorization_required);
    assert!(plan.sensitive_route_mfa_enforcement_required);
    assert!(plan.key_rotation_overlap_evidence_required);
    assert!(plan.auth_failure_audit_event_required);
    assert!(plan.review_only_contract);
    assert!(!plan.discovery_fetch_runtime_attached);
    assert!(!plan.jwks_fetch_runtime_attached);
    assert!(!plan.oidc_signature_verification_attached);
    assert!(!plan.external_identity_provider_attached);
    assert!(!plan.token_introspection_attached);
    assert!(!plan.durable_session_store_attached);
    assert!(!plan.runtime_auth_middleware_attached);
    assert!(!plan.cloud_gateway_enforcement_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
    assert!(!plan.production_identity_provider_evidence_attached);
}

#[test]
fn idp_runtime_evidence_plan_covers_required_requirement_kinds_and_docs() {
    let plan = tenant_rbac_identity_provider_runtime_evidence_plan().expect("plan builds");
    let kinds = plan
        .requirements
        .iter()
        .map(|requirement| requirement.requirement_kind)
        .collect::<std::collections::BTreeSet<_>>();

    for kind in [
        IdentityProviderRuntimeEvidenceRequirementKind::DiscoveryDocumentObserved,
        IdentityProviderRuntimeEvidenceRequirementKind::IssuerMetadataMatched,
        IdentityProviderRuntimeEvidenceRequirementKind::JwksFetched,
        IdentityProviderRuntimeEvidenceRequirementKind::JwksKidMatched,
        IdentityProviderRuntimeEvidenceRequirementKind::JwtSignatureVerified,
        IdentityProviderRuntimeEvidenceRequirementKind::AlgorithmAllowlistEnforced,
        IdentityProviderRuntimeEvidenceRequirementKind::IssuerClaimMatched,
        IdentityProviderRuntimeEvidenceRequirementKind::AudienceClaimMatched,
        IdentityProviderRuntimeEvidenceRequirementKind::TemporalClaimsChecked,
        IdentityProviderRuntimeEvidenceRequirementKind::NonceReplayDenied,
        IdentityProviderRuntimeEvidenceRequirementKind::TenantClaimMapped,
        IdentityProviderRuntimeEvidenceRequirementKind::RouteScopeAuthorized,
        IdentityProviderRuntimeEvidenceRequirementKind::SensitiveRouteMfaEnforced,
        IdentityProviderRuntimeEvidenceRequirementKind::KeyRotationOverlapObserved,
        IdentityProviderRuntimeEvidenceRequirementKind::AuthFailureAuditEventRecorded,
    ] {
        assert!(kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = identity_provider_runtime_evidence_doc_urls(&plan);
    assert!(docs.contains(&"https://openid.net/specs/openid-connect-discovery-1_0.html"));
    assert!(docs.contains(&"https://openid.net/specs/openid-connect-core-1_0.html"));
    assert!(docs.contains(&"https://www.rfc-editor.org/rfc/rfc7515"));
    assert!(docs.contains(&"https://www.rfc-editor.org/rfc/rfc7517"));
    assert!(docs.contains(&"https://www.rfc-editor.org/rfc/rfc7519"));
    assert!(docs.contains(&"https://www.rfc-editor.org/rfc/rfc8725"));
}

#[test]
fn idp_runtime_evidence_plan_preserves_ref_and_claim_boundaries() {
    let plan = tenant_rbac_identity_provider_runtime_evidence_plan().expect("plan builds");

    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .expected_evidence_ref
            .starts_with("evidence/identity-provider-runtime/tenant-rbac/")
            && requirement
                .source_plan_ref
                .starts_with("crates/tenant-rbac-identity-provider-verification/")
            && requirement.tenant_namespace == "oyatie-fd001-tenant-rbac-dev"
            && requirement.expected_issuer == "https://identity.oyatie.com/tenant-rbac"
            && requirement.expected_audience == "tenant-rbac-api"
            && !requirement.runtime_evidence_attached
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == IdentityProviderRuntimeEvidenceRequirementKind::DiscoveryDocumentObserved
            && requirement.requires_discovery_observation
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == IdentityProviderRuntimeEvidenceRequirementKind::JwtSignatureVerified
            && requirement.requires_signature_validation
            && requirement.requires_jwks_observation
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == IdentityProviderRuntimeEvidenceRequirementKind::NonceReplayDenied
            && requirement.requires_claim_validation
            && requirement.requires_denial_evidence
    }));
}

#[test]
fn idp_runtime_evidence_plan_rejects_missing_duplicate_and_doc_drift() {
    let mut plan = tenant_rbac_identity_provider_runtime_evidence_plan().expect("plan builds");
    plan.requirements.truncate(3);
    assert_eq!(
        validate_tenant_rbac_identity_provider_runtime_evidence_plan(&plan),
        Err(TenantRbacIdentityProviderRuntimeEvidenceError::MissingRequirements)
    );

    let mut plan = tenant_rbac_identity_provider_runtime_evidence_plan().expect("plan builds");
    plan.requirements[1].requirement_id = plan.requirements[0].requirement_id;
    assert_eq!(
        validate_tenant_rbac_identity_provider_runtime_evidence_plan(&plan),
        Err(
            TenantRbacIdentityProviderRuntimeEvidenceError::DuplicateRequirement(
                "discovery-document-observed".to_owned()
            )
        )
    );

    let mut plan = tenant_rbac_identity_provider_runtime_evidence_plan().expect("plan builds");
    plan.requirements[0].official_doc_url = "https://example.com/oidc";
    assert_eq!(
        validate_tenant_rbac_identity_provider_runtime_evidence_plan(&plan),
        Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidOfficialDocUrl)
    );
}

#[test]
fn idp_runtime_evidence_plan_rejects_unsafe_refs_missing_controls_and_overclaims() {
    let mut plan = tenant_rbac_identity_provider_runtime_evidence_plan().expect("plan builds");
    plan.requirements[0].expected_evidence_ref =
        "evidence/identity-provider-runtime/tenant-rbac/password-material";
    assert_eq!(
        validate_tenant_rbac_identity_provider_runtime_evidence_plan(&plan),
        Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidExpectedEvidenceRef)
    );

    let mut plan = tenant_rbac_identity_provider_runtime_evidence_plan().expect("plan builds");
    plan.nonce_replay_denial_required = false;
    assert_eq!(
        validate_tenant_rbac_identity_provider_runtime_evidence_plan(&plan),
        Err(
            TenantRbacIdentityProviderRuntimeEvidenceError::MissingRequiredControl(
                "nonce_replay_denial_required"
            )
        )
    );

    let mut plan = tenant_rbac_identity_provider_runtime_evidence_plan().expect("plan builds");
    plan.production_identity_provider_evidence_attached = true;
    assert_eq!(
        validate_tenant_rbac_identity_provider_runtime_evidence_plan(&plan),
        Err(TenantRbacIdentityProviderRuntimeEvidenceError::RuntimeAttachmentOverclaim)
    );
}
