use oya_enterprise_suite_identity_provider_verification::{
    IdentityProviderVerificationError, enterprise_suite_identity_provider_verification_plan,
    render_identity_provider_verification_checklist,
    validate_enterprise_suite_identity_provider_verification_plan,
};

#[test]
fn identity_provider_verification_plan_covers_oidc_discovery_jwks_and_claims() {
    let plan = enterprise_suite_identity_provider_verification_plan();
    validate_enterprise_suite_identity_provider_verification_plan(&plan)
        .expect("identity provider verification plan validates");

    assert_eq!(plan.issuer, "https://identity.oyatie.dev/enterprise-suite");
    assert_eq!(plan.audience, "enterprise-suite-api");
    assert!(
        plan.discovery_document_url
            .ends_with("/.well-known/openid-configuration")
    );
    assert!(plan.jwks_uri.ends_with("/.well-known/jwks.json"));
    assert!(plan.allowed_signing_algorithms.contains(&"RS256"));
    assert!(plan.allowed_signing_algorithms.contains(&"ES256"));
    assert!(
        plan.required_claims
            .iter()
            .any(|claim| claim.claim_name == "iss")
    );
    assert!(
        plan.required_claims
            .iter()
            .any(|claim| claim.claim_name == "aud")
    );
    assert!(
        plan.required_claims
            .iter()
            .any(|claim| claim.claim_name == "exp")
    );
    assert!(
        plan.required_claims
            .iter()
            .any(|claim| claim.claim_name == "sub")
    );
    assert!(
        plan.required_claims
            .iter()
            .any(|claim| claim.claim_name == "tenant_id")
    );
    assert!(
        plan.required_claims
            .iter()
            .any(|claim| claim.claim_name == "nonce")
    );
    assert!(plan.oidc_discovery_required);
    assert!(plan.jwks_required);
    assert!(plan.tls_required);
    assert!(plan.key_id_required);
    assert!(plan.alg_none_forbidden);
    assert!(plan.symmetric_algorithms_forbidden);
    assert!(plan.route_policy_scope_alignment_required);
    assert!(!plan.discovery_fetch_runtime_attached);
    assert!(!plan.jwks_fetch_runtime_attached);
    assert!(!plan.oidc_signature_verification_attached);
    assert!(!plan.external_identity_provider_attached);
    assert!(!plan.runtime_auth_middleware_attached);
    assert!(!plan.cloud_gateway_enforcement_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
}

#[test]
fn identity_provider_verification_plan_renders_review_only_checklist() {
    let plan = enterprise_suite_identity_provider_verification_plan();
    let checklist = render_identity_provider_verification_checklist(&plan)
        .expect("review-only checklist renders");

    assert!(checklist.contains("review_only: true"));
    assert!(checklist.contains(".well-known/openid-configuration"));
    assert!(checklist.contains(".well-known/jwks.json"));
    assert!(checklist.contains("required_claims: iss,aud,exp"));
    assert!(checklist.contains("allowed_signing_algorithms: RS256,PS256,ES256"));
    assert!(
        checklist.contains("checks: tls issuer audience exp nbf iat nonce kid tenant scope mfa")
    );
    assert!(checklist.contains("no_signature_verification"));
    assert!(!checklist.contains("client_secret"));
}

#[test]
fn identity_provider_verification_plan_rejects_bad_issuer_audience_or_algorithm() {
    let mut plan = enterprise_suite_identity_provider_verification_plan();
    plan.issuer = "http://identity.oyatie.dev/enterprise-suite";
    assert_eq!(
        validate_enterprise_suite_identity_provider_verification_plan(&plan),
        Err(IdentityProviderVerificationError::InvalidPlan)
    );

    let mut plan = enterprise_suite_identity_provider_verification_plan();
    plan.audience = "enterprise suite api";
    assert_eq!(
        validate_enterprise_suite_identity_provider_verification_plan(&plan),
        Err(IdentityProviderVerificationError::InvalidPlan)
    );

    let mut plan = enterprise_suite_identity_provider_verification_plan();
    plan.allowed_signing_algorithms.push("HS256");
    assert_eq!(
        validate_enterprise_suite_identity_provider_verification_plan(&plan),
        Err(IdentityProviderVerificationError::InvalidAlgorithm)
    );
}

#[test]
fn identity_provider_verification_plan_rejects_claim_drift_and_duplicates() {
    let mut plan = enterprise_suite_identity_provider_verification_plan();
    plan.required_claims
        .retain(|claim| claim.claim_name != "tenant_id");
    assert_eq!(
        validate_enterprise_suite_identity_provider_verification_plan(&plan),
        Err(IdentityProviderVerificationError::MissingRequiredClaim(
            "tenant_id".to_string()
        ))
    );

    let mut plan = enterprise_suite_identity_provider_verification_plan();
    let duplicate = plan.required_claims[0].clone();
    plan.required_claims.push(duplicate);
    assert!(matches!(
        validate_enterprise_suite_identity_provider_verification_plan(&plan),
        Err(IdentityProviderVerificationError::DuplicateClaim(_))
    ));

    let mut plan = enterprise_suite_identity_provider_verification_plan();
    plan.required_claims[0].expected_value = "https://identity.other.example/enterprise-suite";
    assert_eq!(
        validate_enterprise_suite_identity_provider_verification_plan(&plan),
        Err(IdentityProviderVerificationError::InvalidClaim)
    );
}

#[test]
fn identity_provider_verification_plan_rejects_security_control_gaps_and_runtime_overclaims() {
    let mut plan = enterprise_suite_identity_provider_verification_plan();
    plan.jwks_required = false;
    assert_eq!(
        validate_enterprise_suite_identity_provider_verification_plan(&plan),
        Err(IdentityProviderVerificationError::SecurityControlMissing)
    );

    let mut plan = enterprise_suite_identity_provider_verification_plan();
    plan.jwks_fetch_runtime_attached = true;
    assert_eq!(
        validate_enterprise_suite_identity_provider_verification_plan(&plan),
        Err(IdentityProviderVerificationError::RuntimeAttachmentOverclaim)
    );

    let mut plan = enterprise_suite_identity_provider_verification_plan();
    plan.jwks_cache_ttl_seconds = 60 * 60;
    assert_eq!(
        validate_enterprise_suite_identity_provider_verification_plan(&plan),
        Err(IdentityProviderVerificationError::InvalidCachePolicy)
    );
}
