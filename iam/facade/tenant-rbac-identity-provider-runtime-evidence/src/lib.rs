//! Tenant RBAC identity-provider runtime evidence contract.
//!
//! This review-only crate records the deployed identity-provider evidence that
//! must exist before FD-001 tenant workloads can claim external identity-provider
//! verification on the future Oyatie Cloud substrate. It validates source refs,
//! official OpenID Connect/JWS/JWK/JWT evidence requirements, claim boundaries,
//! and non-claim flags, but it does not fetch discovery metadata, fetch JWKS
//! documents, verify signatures, introspect access artifacts, persist sessions,
//! attach runtime auth middleware, enforce a cloud gateway, emit audit-chain
//! events, or claim production identity-provider evidence.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use iam_tenant_rbac_identity_provider_verification::{
    IdentityProviderVerificationError, TenantRbacIdentityProviderVerificationPlan,
    tenant_rbac_identity_provider_verification_plan,
    validate_tenant_rbac_identity_provider_verification_plan,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_REQUIREMENT_COUNT: usize = 15;
const PLAN_NAME: &str = "tenant-rbac-identity-provider-runtime-evidence-plan";
const SERVICE_NAME: &str = "tenant-rbac";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const SOURCE_PLAN_REF: &str = "crates/tenant-rbac-identity-provider-verification/src/lib.rs::tenant_rbac_identity_provider_verification_plan";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IdentityProviderRuntimeEvidenceRequirementKind {
    DiscoveryDocumentObserved,
    IssuerMetadataMatched,
    JwksFetched,
    JwksKidMatched,
    JwtSignatureVerified,
    AlgorithmAllowlistEnforced,
    IssuerClaimMatched,
    AudienceClaimMatched,
    TemporalClaimsChecked,
    NonceReplayDenied,
    TenantClaimMapped,
    RouteScopeAuthorized,
    SensitiveRouteMfaEnforced,
    KeyRotationOverlapObserved,
    AuthFailureAuditEventRecorded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityProviderRuntimeEvidenceRequirement {
    pub requirement_id: &'static str, // data_class: PUBLIC
    pub requirement_kind: IdentityProviderRuntimeEvidenceRequirementKind, // data_class: PUBLIC
    pub workload_scope: &'static str, // data_class: PUBLIC
    pub official_doc_url: &'static str, // data_class: PUBLIC
    pub expected_evidence_ref: &'static str, // data_class: INTERNAL_ONLY
    pub source_plan_ref: &'static str, // data_class: INTERNAL_ONLY
    pub tenant_namespace: &'static str, // data_class: INTERNAL_ONLY
    pub expected_issuer: &'static str, // data_class: INTERNAL_ONLY
    pub expected_audience: &'static str, // data_class: INTERNAL_ONLY
    pub requires_discovery_observation: bool, // data_class: PUBLIC
    pub requires_jwks_observation: bool, // data_class: PUBLIC
    pub requires_signature_validation: bool, // data_class: PUBLIC
    pub requires_claim_validation: bool, // data_class: PUBLIC
    pub requires_denial_evidence: bool, // data_class: PUBLIC
    pub runtime_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacIdentityProviderRuntimeEvidencePlan {
    pub plan_name: &'static str,        // data_class: PUBLIC
    pub service_name: &'static str,     // data_class: PUBLIC
    pub substrate_name: &'static str,   // data_class: PUBLIC
    pub tenant_namespace: &'static str, // data_class: INTERNAL_ONLY
    pub identity_provider_verification_plan_name: &'static str, // data_class: INTERNAL_ONLY
    pub issuer: &'static str,           // data_class: INTERNAL_ONLY
    pub audience: &'static str,         // data_class: INTERNAL_ONLY
    pub discovery_document_url: &'static str, // data_class: INTERNAL_ONLY
    pub jwks_uri: &'static str,         // data_class: INTERNAL_ONLY
    pub required_claim_count: usize,    // data_class: PUBLIC
    pub allowed_algorithm_count: usize, // data_class: PUBLIC
    pub requirements: Vec<IdentityProviderRuntimeEvidenceRequirement>, // data_class: INTERNAL_ONLY
    pub fd001_product_delivery_master_goal_preserved: bool, // data_class: PUBLIC
    pub oyatie_cloud_substrate_proof_required: bool, // data_class: PUBLIC
    pub official_docs_required: bool,   // data_class: PUBLIC
    pub discovery_document_observation_required: bool, // data_class: PUBLIC
    pub issuer_metadata_match_required: bool, // data_class: PUBLIC
    pub jwks_fetch_evidence_required: bool, // data_class: PUBLIC
    pub jwks_kid_match_required: bool,  // data_class: PUBLIC
    pub jwt_signature_verification_evidence_required: bool, // data_class: PUBLIC
    pub algorithm_allowlist_required: bool, // data_class: PUBLIC
    pub issuer_claim_match_required: bool, // data_class: PUBLIC
    pub audience_claim_match_required: bool, // data_class: PUBLIC
    pub temporal_claims_check_required: bool, // data_class: PUBLIC
    pub nonce_replay_denial_required: bool, // data_class: PUBLIC
    pub tenant_claim_mapping_required: bool, // data_class: PUBLIC
    pub route_scope_authorization_required: bool, // data_class: PUBLIC
    pub sensitive_route_mfa_enforcement_required: bool, // data_class: PUBLIC
    pub key_rotation_overlap_evidence_required: bool, // data_class: PUBLIC
    pub auth_failure_audit_event_required: bool, // data_class: PUBLIC
    pub review_only_contract: bool,     // data_class: PUBLIC
    pub discovery_fetch_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub jwks_fetch_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub oidc_signature_verification_attached: bool, // data_class: INTERNAL_ONLY
    pub external_identity_provider_attached: bool, // data_class: INTERNAL_ONLY
    pub token_introspection_attached: bool, // data_class: INTERNAL_ONLY
    pub durable_session_store_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_auth_middleware_attached: bool, // data_class: INTERNAL_ONLY
    pub cloud_gateway_enforcement_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub production_identity_provider_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacIdentityProviderRuntimeEvidenceError {
    IdentityProviderVerification(IdentityProviderVerificationError),
    InvalidPlanName,
    InvalidServiceName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidIdentityProviderVerificationPlanName,
    InvalidIssuer,
    InvalidAudience,
    InvalidDiscoveryDocumentUrl,
    InvalidJwksUri,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingRequirementKind(IdentityProviderRuntimeEvidenceRequirementKind),
    InvalidRequirementId,
    InvalidWorkloadScope,
    InvalidOfficialDocUrl,
    InvalidExpectedEvidenceRef,
    InvalidSourcePlanRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_identity_provider_runtime_evidence_plan() -> Result<
    TenantRbacIdentityProviderRuntimeEvidencePlan,
    TenantRbacIdentityProviderRuntimeEvidenceError,
> {
    let verification_plan = tenant_rbac_identity_provider_verification_plan();
    validate_tenant_rbac_identity_provider_verification_plan(&verification_plan)
        .map_err(TenantRbacIdentityProviderRuntimeEvidenceError::IdentityProviderVerification)?;

    Ok(TenantRbacIdentityProviderRuntimeEvidencePlan {
        plan_name: PLAN_NAME,
        service_name: SERVICE_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: TENANT_NAMESPACE,
        identity_provider_verification_plan_name: verification_plan.plan_name,
        issuer: verification_plan.issuer,
        audience: verification_plan.audience,
        discovery_document_url: verification_plan.discovery_document_url,
        jwks_uri: verification_plan.jwks_uri,
        required_claim_count: verification_plan.required_claims.len(),
        allowed_algorithm_count: verification_plan.allowed_signing_algorithms.len(),
        requirements: runtime_requirements(&verification_plan),
        fd001_product_delivery_master_goal_preserved: true,
        oyatie_cloud_substrate_proof_required: true,
        official_docs_required: true,
        discovery_document_observation_required: true,
        issuer_metadata_match_required: true,
        jwks_fetch_evidence_required: true,
        jwks_kid_match_required: true,
        jwt_signature_verification_evidence_required: true,
        algorithm_allowlist_required: true,
        issuer_claim_match_required: true,
        audience_claim_match_required: true,
        temporal_claims_check_required: true,
        nonce_replay_denial_required: true,
        tenant_claim_mapping_required: true,
        route_scope_authorization_required: true,
        sensitive_route_mfa_enforcement_required: true,
        key_rotation_overlap_evidence_required: true,
        auth_failure_audit_event_required: true,
        review_only_contract: true,
        discovery_fetch_runtime_attached: false,
        jwks_fetch_runtime_attached: false,
        oidc_signature_verification_attached: false,
        external_identity_provider_attached: false,
        token_introspection_attached: false,
        durable_session_store_attached: false,
        runtime_auth_middleware_attached: false,
        cloud_gateway_enforcement_attached: false,
        runtime_audit_chain_emission_attached: false,
        production_identity_provider_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_tenant_rbac_identity_provider_runtime_evidence_plan(
    plan: &TenantRbacIdentityProviderRuntimeEvidencePlan,
) -> Result<(), TenantRbacIdentityProviderRuntimeEvidenceError> {
    validate_slug(
        plan.plan_name,
        TenantRbacIdentityProviderRuntimeEvidenceError::InvalidPlanName,
    )?;
    if plan.service_name != SERVICE_NAME {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidServiceName);
    }
    if plan.substrate_name != SUBSTRATE_NAME {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidSubstrateName);
    }
    if plan.tenant_namespace != TENANT_NAMESPACE {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidTenantNamespace);
    }
    if plan.identity_provider_verification_plan_name
        != "tenant-rbac-identity-provider-verification-plan"
    {
        return Err(
            TenantRbacIdentityProviderRuntimeEvidenceError::InvalidIdentityProviderVerificationPlanName,
        );
    }
    if !valid_https_url(plan.issuer) {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidIssuer);
    }
    if !valid_label(plan.audience) {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidAudience);
    }
    if !valid_https_url(plan.discovery_document_url)
        || !plan
            .discovery_document_url
            .ends_with("/.well-known/openid-configuration")
    {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidDiscoveryDocumentUrl);
    }
    if !valid_https_url(plan.jwks_uri) || !plan.jwks_uri.ends_with("/.well-known/jwks.json") {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidJwksUri);
    }
    if plan.required_claim_count < 9
        || plan.allowed_algorithm_count < 2
        || plan.requirements.len() < MIN_REQUIREMENT_COUNT
        || plan.schema_version != SCHEMA_VERSION
    {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::MissingRequirements);
    }
    validate_required_controls(plan)?;
    validate_nonclaims(plan)?;
    validate_runtime_requirements(plan)?;
    Ok(())
}

pub fn identity_provider_runtime_evidence_doc_urls(
    plan: &TenantRbacIdentityProviderRuntimeEvidencePlan,
) -> Vec<&'static str> {
    plan.requirements
        .iter()
        .map(|requirement| requirement.official_doc_url)
        .collect()
}

fn runtime_requirements(
    verification_plan: &TenantRbacIdentityProviderVerificationPlan,
) -> Vec<IdentityProviderRuntimeEvidenceRequirement> {
    vec![
        requirement(
            "discovery-document-observed",
            IdentityProviderRuntimeEvidenceRequirementKind::DiscoveryDocumentObserved,
            "identity-provider-discovery",
            "https://openid.net/specs/openid-connect-discovery-1_0.html",
            "evidence/identity-provider-runtime/tenant-rbac/discovery-document.json",
            verification_plan,
        ),
        requirement(
            "issuer-metadata-matched",
            IdentityProviderRuntimeEvidenceRequirementKind::IssuerMetadataMatched,
            "identity-provider-discovery",
            "https://openid.net/specs/openid-connect-discovery-1_0.html",
            "evidence/identity-provider-runtime/tenant-rbac/issuer-metadata.json",
            verification_plan,
        ),
        requirement(
            "jwks-fetched",
            IdentityProviderRuntimeEvidenceRequirementKind::JwksFetched,
            "jwks-cache",
            "https://openid.net/specs/openid-connect-discovery-1_0.html",
            "evidence/identity-provider-runtime/tenant-rbac/jwks-fetch.json",
            verification_plan,
        ),
        requirement(
            "jwks-kid-matched",
            IdentityProviderRuntimeEvidenceRequirementKind::JwksKidMatched,
            "jwks-cache",
            "https://www.rfc-editor.org/rfc/rfc7515",
            "evidence/identity-provider-runtime/tenant-rbac/jwks-kid-match.json",
            verification_plan,
        ),
        requirement(
            "jwt-signature-verified",
            IdentityProviderRuntimeEvidenceRequirementKind::JwtSignatureVerified,
            "jwt-verifier",
            "https://openid.net/specs/openid-connect-core-1_0.html",
            "evidence/identity-provider-runtime/tenant-rbac/jwt-signature.json",
            verification_plan,
        ),
        requirement(
            "algorithm-allowlist-enforced",
            IdentityProviderRuntimeEvidenceRequirementKind::AlgorithmAllowlistEnforced,
            "jwt-verifier",
            "https://www.rfc-editor.org/rfc/rfc8725",
            "evidence/identity-provider-runtime/tenant-rbac/algorithm-allowlist.json",
            verification_plan,
        ),
        requirement(
            "issuer-claim-matched",
            IdentityProviderRuntimeEvidenceRequirementKind::IssuerClaimMatched,
            "claim-validator",
            "https://openid.net/specs/openid-connect-core-1_0.html",
            "evidence/identity-provider-runtime/tenant-rbac/issuer-claim.json",
            verification_plan,
        ),
        requirement(
            "audience-claim-matched",
            IdentityProviderRuntimeEvidenceRequirementKind::AudienceClaimMatched,
            "claim-validator",
            "https://www.rfc-editor.org/rfc/rfc7519",
            "evidence/identity-provider-runtime/tenant-rbac/audience-claim.json",
            verification_plan,
        ),
        requirement(
            "temporal-claims-checked",
            IdentityProviderRuntimeEvidenceRequirementKind::TemporalClaimsChecked,
            "claim-validator",
            "https://www.rfc-editor.org/rfc/rfc7519",
            "evidence/identity-provider-runtime/tenant-rbac/temporal-claims.json",
            verification_plan,
        ),
        requirement(
            "nonce-replay-denied",
            IdentityProviderRuntimeEvidenceRequirementKind::NonceReplayDenied,
            "nonce-store",
            "https://openid.net/specs/openid-connect-core-1_0.html",
            "evidence/identity-provider-runtime/tenant-rbac/nonce-replay-denied.json",
            verification_plan,
        ),
        requirement(
            "tenant-claim-mapped",
            IdentityProviderRuntimeEvidenceRequirementKind::TenantClaimMapped,
            "tenant-router",
            "https://www.rfc-editor.org/rfc/rfc7519",
            "evidence/identity-provider-runtime/tenant-rbac/tenant-claim-map.json",
            verification_plan,
        ),
        requirement(
            "route-scope-authorized",
            IdentityProviderRuntimeEvidenceRequirementKind::RouteScopeAuthorized,
            "route-policy",
            "https://www.rfc-editor.org/rfc/rfc7519",
            "evidence/identity-provider-runtime/tenant-rbac/route-scope-authorization.json",
            verification_plan,
        ),
        requirement(
            "sensitive-route-mfa-enforced",
            IdentityProviderRuntimeEvidenceRequirementKind::SensitiveRouteMfaEnforced,
            "sensitive-route-policy",
            "https://openid.net/specs/openid-connect-core-1_0.html",
            "evidence/identity-provider-runtime/tenant-rbac/sensitive-route-mfa.json",
            verification_plan,
        ),
        requirement(
            "key-rotation-overlap-observed",
            IdentityProviderRuntimeEvidenceRequirementKind::KeyRotationOverlapObserved,
            "key-rotation-watch",
            "https://www.rfc-editor.org/rfc/rfc7517",
            "evidence/identity-provider-runtime/tenant-rbac/key-rotation-overlap.json",
            verification_plan,
        ),
        requirement(
            "auth-failure-audit-event-recorded",
            IdentityProviderRuntimeEvidenceRequirementKind::AuthFailureAuditEventRecorded,
            "audit-chain-contract",
            "https://cloudevents.io/",
            "evidence/identity-provider-runtime/tenant-rbac/auth-failure-audit-event.json",
            verification_plan,
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    requirement_kind: IdentityProviderRuntimeEvidenceRequirementKind,
    workload_scope: &'static str,
    official_doc_url: &'static str,
    expected_evidence_ref: &'static str,
    verification_plan: &TenantRbacIdentityProviderVerificationPlan,
) -> IdentityProviderRuntimeEvidenceRequirement {
    let requires_discovery_observation = matches!(
        requirement_kind,
        IdentityProviderRuntimeEvidenceRequirementKind::DiscoveryDocumentObserved
            | IdentityProviderRuntimeEvidenceRequirementKind::IssuerMetadataMatched
            | IdentityProviderRuntimeEvidenceRequirementKind::JwksFetched
    );
    let requires_jwks_observation = matches!(
        requirement_kind,
        IdentityProviderRuntimeEvidenceRequirementKind::JwksFetched
            | IdentityProviderRuntimeEvidenceRequirementKind::JwksKidMatched
            | IdentityProviderRuntimeEvidenceRequirementKind::JwtSignatureVerified
            | IdentityProviderRuntimeEvidenceRequirementKind::KeyRotationOverlapObserved
    );
    let requires_signature_validation = matches!(
        requirement_kind,
        IdentityProviderRuntimeEvidenceRequirementKind::JwtSignatureVerified
            | IdentityProviderRuntimeEvidenceRequirementKind::AlgorithmAllowlistEnforced
            | IdentityProviderRuntimeEvidenceRequirementKind::JwksKidMatched
    );
    let requires_claim_validation = matches!(
        requirement_kind,
        IdentityProviderRuntimeEvidenceRequirementKind::IssuerClaimMatched
            | IdentityProviderRuntimeEvidenceRequirementKind::AudienceClaimMatched
            | IdentityProviderRuntimeEvidenceRequirementKind::TemporalClaimsChecked
            | IdentityProviderRuntimeEvidenceRequirementKind::NonceReplayDenied
            | IdentityProviderRuntimeEvidenceRequirementKind::TenantClaimMapped
            | IdentityProviderRuntimeEvidenceRequirementKind::RouteScopeAuthorized
            | IdentityProviderRuntimeEvidenceRequirementKind::SensitiveRouteMfaEnforced
    );
    let requires_denial_evidence = matches!(
        requirement_kind,
        IdentityProviderRuntimeEvidenceRequirementKind::NonceReplayDenied
            | IdentityProviderRuntimeEvidenceRequirementKind::RouteScopeAuthorized
            | IdentityProviderRuntimeEvidenceRequirementKind::SensitiveRouteMfaEnforced
            | IdentityProviderRuntimeEvidenceRequirementKind::AuthFailureAuditEventRecorded
    );

    IdentityProviderRuntimeEvidenceRequirement {
        requirement_id,
        requirement_kind,
        workload_scope,
        official_doc_url,
        expected_evidence_ref,
        source_plan_ref: SOURCE_PLAN_REF,
        tenant_namespace: TENANT_NAMESPACE,
        expected_issuer: verification_plan.issuer,
        expected_audience: verification_plan.audience,
        requires_discovery_observation,
        requires_jwks_observation,
        requires_signature_validation,
        requires_claim_validation,
        requires_denial_evidence,
        runtime_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_required_controls(
    plan: &TenantRbacIdentityProviderRuntimeEvidencePlan,
) -> Result<(), TenantRbacIdentityProviderRuntimeEvidenceError> {
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
            plan.discovery_document_observation_required,
            "discovery_document_observation_required",
        ),
        (
            plan.issuer_metadata_match_required,
            "issuer_metadata_match_required",
        ),
        (
            plan.jwks_fetch_evidence_required,
            "jwks_fetch_evidence_required",
        ),
        (plan.jwks_kid_match_required, "jwks_kid_match_required"),
        (
            plan.jwt_signature_verification_evidence_required,
            "jwt_signature_verification_evidence_required",
        ),
        (
            plan.algorithm_allowlist_required,
            "algorithm_allowlist_required",
        ),
        (
            plan.issuer_claim_match_required,
            "issuer_claim_match_required",
        ),
        (
            plan.audience_claim_match_required,
            "audience_claim_match_required",
        ),
        (
            plan.temporal_claims_check_required,
            "temporal_claims_check_required",
        ),
        (
            plan.nonce_replay_denial_required,
            "nonce_replay_denial_required",
        ),
        (
            plan.tenant_claim_mapping_required,
            "tenant_claim_mapping_required",
        ),
        (
            plan.route_scope_authorization_required,
            "route_scope_authorization_required",
        ),
        (
            plan.sensitive_route_mfa_enforcement_required,
            "sensitive_route_mfa_enforcement_required",
        ),
        (
            plan.key_rotation_overlap_evidence_required,
            "key_rotation_overlap_evidence_required",
        ),
        (
            plan.auth_failure_audit_event_required,
            "auth_failure_audit_event_required",
        ),
        (plan.review_only_contract, "review_only_contract"),
    ] {
        require_control(control.0, control.1)?;
    }
    Ok(())
}

fn validate_nonclaims(
    plan: &TenantRbacIdentityProviderRuntimeEvidencePlan,
) -> Result<(), TenantRbacIdentityProviderRuntimeEvidenceError> {
    if plan.discovery_fetch_runtime_attached
        || plan.jwks_fetch_runtime_attached
        || plan.oidc_signature_verification_attached
        || plan.external_identity_provider_attached
        || plan.token_introspection_attached
        || plan.durable_session_store_attached
        || plan.runtime_auth_middleware_attached
        || plan.cloud_gateway_enforcement_attached
        || plan.runtime_audit_chain_emission_attached
        || plan.production_identity_provider_evidence_attached
    {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_runtime_requirements(
    plan: &TenantRbacIdentityProviderRuntimeEvidencePlan,
) -> Result<(), TenantRbacIdentityProviderRuntimeEvidenceError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_kinds = BTreeSet::new();
    for requirement in &plan.requirements {
        validate_requirement(requirement, plan.issuer, plan.audience)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(
                TenantRbacIdentityProviderRuntimeEvidenceError::DuplicateRequirement(
                    requirement.requirement_id.to_owned(),
                ),
            );
        }
        seen_kinds.insert(requirement.requirement_kind);
    }
    for kind in required_requirement_kinds() {
        if !seen_kinds.contains(&kind) {
            return Err(
                TenantRbacIdentityProviderRuntimeEvidenceError::MissingRequirementKind(kind),
            );
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &IdentityProviderRuntimeEvidenceRequirement,
    expected_issuer: &'static str,
    expected_audience: &'static str,
) -> Result<(), TenantRbacIdentityProviderRuntimeEvidenceError> {
    validate_slug(
        requirement.requirement_id,
        TenantRbacIdentityProviderRuntimeEvidenceError::InvalidRequirementId,
    )?;
    validate_workload_scope(requirement.workload_scope)?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/identity-provider-runtime/tenant-rbac/",
        TenantRbacIdentityProviderRuntimeEvidenceError::InvalidExpectedEvidenceRef,
    )?;
    validate_prefixed_ref(
        requirement.source_plan_ref,
        "crates/tenant-rbac-identity-provider-verification/",
        TenantRbacIdentityProviderRuntimeEvidenceError::InvalidSourcePlanRef,
    )?;
    if requirement.tenant_namespace != TENANT_NAMESPACE {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidTenantNamespace);
    }
    if requirement.expected_issuer != expected_issuer {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidIssuer);
    }
    if requirement.expected_audience != expected_audience {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidAudience);
    }
    if matches!(
        requirement.requirement_kind,
        IdentityProviderRuntimeEvidenceRequirementKind::DiscoveryDocumentObserved
            | IdentityProviderRuntimeEvidenceRequirementKind::IssuerMetadataMatched
            | IdentityProviderRuntimeEvidenceRequirementKind::JwksFetched
    ) {
        require_control(
            requirement.requires_discovery_observation,
            "requirement_requires_discovery_observation",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        IdentityProviderRuntimeEvidenceRequirementKind::JwksFetched
            | IdentityProviderRuntimeEvidenceRequirementKind::JwksKidMatched
            | IdentityProviderRuntimeEvidenceRequirementKind::JwtSignatureVerified
            | IdentityProviderRuntimeEvidenceRequirementKind::KeyRotationOverlapObserved
    ) {
        require_control(
            requirement.requires_jwks_observation,
            "requirement_requires_jwks_observation",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        IdentityProviderRuntimeEvidenceRequirementKind::JwtSignatureVerified
            | IdentityProviderRuntimeEvidenceRequirementKind::AlgorithmAllowlistEnforced
            | IdentityProviderRuntimeEvidenceRequirementKind::JwksKidMatched
    ) {
        require_control(
            requirement.requires_signature_validation,
            "requirement_requires_signature_validation",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        IdentityProviderRuntimeEvidenceRequirementKind::IssuerClaimMatched
            | IdentityProviderRuntimeEvidenceRequirementKind::AudienceClaimMatched
            | IdentityProviderRuntimeEvidenceRequirementKind::TemporalClaimsChecked
            | IdentityProviderRuntimeEvidenceRequirementKind::NonceReplayDenied
            | IdentityProviderRuntimeEvidenceRequirementKind::TenantClaimMapped
            | IdentityProviderRuntimeEvidenceRequirementKind::RouteScopeAuthorized
            | IdentityProviderRuntimeEvidenceRequirementKind::SensitiveRouteMfaEnforced
    ) {
        require_control(
            requirement.requires_claim_validation,
            "requirement_requires_claim_validation",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        IdentityProviderRuntimeEvidenceRequirementKind::NonceReplayDenied
            | IdentityProviderRuntimeEvidenceRequirementKind::RouteScopeAuthorized
            | IdentityProviderRuntimeEvidenceRequirementKind::SensitiveRouteMfaEnforced
            | IdentityProviderRuntimeEvidenceRequirementKind::AuthFailureAuditEventRecorded
    ) {
        require_control(
            requirement.requires_denial_evidence,
            "requirement_requires_denial_evidence",
        )?;
    }
    if requirement.runtime_evidence_attached {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }
    if requirement.schema_version != SCHEMA_VERSION {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::MissingRequirements);
    }
    Ok(())
}

fn required_requirement_kinds() -> [IdentityProviderRuntimeEvidenceRequirementKind; 15] {
    [
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
    ]
}

fn validate_slug(
    value: &str,
    error: TenantRbacIdentityProviderRuntimeEvidenceError,
) -> Result<(), TenantRbacIdentityProviderRuntimeEvidenceError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || value
            .chars()
            .any(|ch| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'))
    {
        return Err(error);
    }
    Ok(())
}

fn validate_workload_scope(
    value: &str,
) -> Result<(), TenantRbacIdentityProviderRuntimeEvidenceError> {
    if value.is_empty() || has_unsafe_text(value) || has_path_traversal(value) {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidWorkloadScope);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), TenantRbacIdentityProviderRuntimeEvidenceError> {
    let allowed = [
        "https://openid.net/specs/openid-connect-discovery-1_0.html",
        "https://openid.net/specs/openid-connect-core-1_0.html",
        "https://www.rfc-editor.org/rfc/rfc7515",
        "https://www.rfc-editor.org/rfc/rfc7517",
        "https://www.rfc-editor.org/rfc/rfc7519",
        "https://www.rfc-editor.org/rfc/rfc8725",
        "https://cloudevents.io/",
    ];
    if !allowed.contains(&url) {
        return Err(TenantRbacIdentityProviderRuntimeEvidenceError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: TenantRbacIdentityProviderRuntimeEvidenceError,
) -> Result<(), TenantRbacIdentityProviderRuntimeEvidenceError> {
    if !value.starts_with(prefix) || has_unsafe_text(value) || has_path_traversal(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    enabled: bool,
    field: &'static str,
) -> Result<(), TenantRbacIdentityProviderRuntimeEvidenceError> {
    if enabled {
        Ok(())
    } else {
        Err(TenantRbacIdentityProviderRuntimeEvidenceError::MissingRequiredControl(field))
    }
}

fn valid_label(value: &str) -> bool {
    !value.is_empty()
        && !has_unsafe_text(value)
        && !has_path_traversal(value)
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn valid_https_url(value: &str) -> bool {
    value.starts_with("https://")
        && !value.contains(' ')
        && !value.contains('#')
        && !has_unsafe_text(value)
        && !has_path_traversal(value)
}

fn has_path_traversal(value: &str) -> bool {
    value.contains("..") || value.contains('~') || value.starts_with('/')
}

fn has_unsafe_text(value: &str) -> bool {
    value.contains("secret")
        || value.contains("password")
        || value.contains("credential")
        || value.contains("private-key")
}
