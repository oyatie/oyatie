//! Enterprise Suite identity-provider verification foundation.
//!
//! This control-plane crate records the OpenID Connect discovery, JWKS, JWT
//! claim, key-rotation, and route-scope checks required before Enterprise Suite
//! can attach an external identity provider. It deliberately does not fetch
//! discovery metadata, fetch JWKS documents, verify signatures, introspect
//! tokens, issue credentials, persist sessions, attach runtime auth middleware,
//! enforce a cloud gateway, or emit runtime audit-chain events.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use oya_enterprise_suite_auth_runtime::{
    enterprise_auth_runtime_policy, validate_enterprise_auth_runtime_policy,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_REQUIRED_CLAIMS: usize = 9;
const MIN_ALLOWED_ALGORITHMS: usize = 2;
const MAX_CLOCK_SKEW_SECONDS: u64 = 300;
const MAX_TOKEN_AGE_SECONDS: u64 = 15 * 60;
const MAX_JWKS_CACHE_TTL_SECONDS: u64 = 10 * 60;
const MIN_KEY_ROTATION_OVERLAP_SECONDS: u64 = 24 * 60 * 60;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityProviderClaimRequirement {
    pub claim_name: &'static str,     // data_class: PUBLIC
    pub expected_value: &'static str, // data_class: INTERNAL_ONLY
    pub required: bool,               // data_class: PUBLIC
    pub source: &'static str,         // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterpriseSuiteIdentityProviderVerificationPlan {
    pub plan_name: &'static str,                       // data_class: PUBLIC
    pub issuer: &'static str,                          // data_class: INTERNAL_ONLY
    pub audience: &'static str,                        // data_class: INTERNAL_ONLY
    pub discovery_document_url: &'static str,          // data_class: INTERNAL_ONLY
    pub jwks_uri: &'static str,                        // data_class: INTERNAL_ONLY
    pub authorization_endpoint: &'static str,          // data_class: INTERNAL_ONLY
    pub token_endpoint: &'static str,                  // data_class: INTERNAL_ONLY
    pub userinfo_endpoint: &'static str,               // data_class: INTERNAL_ONLY
    pub subject_claim: &'static str,                   // data_class: INTERNAL_ONLY
    pub tenant_claim: &'static str,                    // data_class: INTERNAL_ONLY
    pub scope_claim: &'static str,                     // data_class: INTERNAL_ONLY
    pub nonce_claim: &'static str,                     // data_class: INTERNAL_ONLY
    pub mfa_claim: &'static str,                       // data_class: INTERNAL_ONLY
    pub assurance_claim: &'static str,                 // data_class: INTERNAL_ONLY
    pub allowed_signing_algorithms: Vec<&'static str>, // data_class: PUBLIC
    pub required_claims: Vec<IdentityProviderClaimRequirement>, // data_class: INTERNAL_ONLY
    pub clock_skew_seconds: u64,                       // data_class: PUBLIC
    pub max_token_age_seconds: u64,                    // data_class: PUBLIC
    pub jwks_cache_ttl_seconds: u64,                   // data_class: PUBLIC
    pub key_rotation_overlap_seconds: u64,             // data_class: PUBLIC
    pub oidc_discovery_required: bool,                 // data_class: PUBLIC
    pub jwks_required: bool,                           // data_class: PUBLIC
    pub tls_required: bool,                            // data_class: PUBLIC
    pub issuer_match_required: bool,                   // data_class: PUBLIC
    pub audience_match_required: bool,                 // data_class: PUBLIC
    pub expiration_required: bool,                     // data_class: PUBLIC
    pub not_before_and_issued_at_checked: bool,        // data_class: PUBLIC
    pub nonce_required: bool,                          // data_class: PUBLIC
    pub key_id_required: bool,                         // data_class: PUBLIC
    pub alg_none_forbidden: bool,                      // data_class: PUBLIC
    pub symmetric_algorithms_forbidden: bool,          // data_class: PUBLIC
    pub tenant_claim_required: bool,                   // data_class: PUBLIC
    pub subject_claim_required: bool,                  // data_class: PUBLIC
    pub mfa_claim_required_for_sensitive_routes: bool, // data_class: PUBLIC
    pub route_policy_scope_alignment_required: bool,   // data_class: PUBLIC
    pub discovery_fetch_runtime_attached: bool,        // data_class: INTERNAL_ONLY
    pub jwks_fetch_runtime_attached: bool,             // data_class: INTERNAL_ONLY
    pub oidc_signature_verification_attached: bool,    // data_class: INTERNAL_ONLY
    pub external_identity_provider_attached: bool,     // data_class: INTERNAL_ONLY
    pub token_introspection_attached: bool,            // data_class: INTERNAL_ONLY
    pub durable_session_store_attached: bool,          // data_class: INTERNAL_ONLY
    pub runtime_auth_middleware_attached: bool,        // data_class: INTERNAL_ONLY
    pub cloud_gateway_enforcement_attached: bool,      // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool,   // data_class: INTERNAL_ONLY
    pub schema_version: u32,                           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentityProviderVerificationError {
    InvalidPlan,
    InvalidIssuer,
    InvalidAudience,
    InvalidUrl,
    InvalidClaim,
    MissingRequiredClaim(String),
    DuplicateClaim(String),
    InvalidAlgorithm,
    DuplicateAlgorithm(String),
    InvalidCachePolicy,
    SecurityControlMissing,
    RuntimeAttachmentOverclaim,
    AuthRuntimePolicyInvalid,
}

pub fn enterprise_suite_identity_provider_verification_plan()
-> EnterpriseSuiteIdentityProviderVerificationPlan {
    let auth_policy = enterprise_auth_runtime_policy();
    EnterpriseSuiteIdentityProviderVerificationPlan {
        plan_name: "enterprise-suite-identity-provider-verification-plan",
        issuer: auth_policy.expected_issuer,
        audience: auth_policy.expected_audience,
        discovery_document_url: "https://identity.oyatie.dev/enterprise-suite/.well-known/openid-configuration",
        jwks_uri: "https://identity.oyatie.dev/enterprise-suite/.well-known/jwks.json",
        authorization_endpoint: "https://identity.oyatie.dev/enterprise-suite/oauth2/v1/authorize",
        token_endpoint: "https://identity.oyatie.dev/enterprise-suite/oauth2/v1/token",
        userinfo_endpoint: "https://identity.oyatie.dev/enterprise-suite/oauth2/v1/userinfo",
        subject_claim: "sub",
        tenant_claim: "tenant_id",
        scope_claim: "scp",
        nonce_claim: "nonce",
        mfa_claim: "amr",
        assurance_claim: "acr",
        allowed_signing_algorithms: vec!["RS256", "PS256", "ES256"],
        required_claims: required_claims(
            auth_policy.expected_issuer,
            auth_policy.expected_audience,
        ),
        clock_skew_seconds: MAX_CLOCK_SKEW_SECONDS,
        max_token_age_seconds: MAX_TOKEN_AGE_SECONDS,
        jwks_cache_ttl_seconds: MAX_JWKS_CACHE_TTL_SECONDS,
        key_rotation_overlap_seconds: MIN_KEY_ROTATION_OVERLAP_SECONDS,
        oidc_discovery_required: true,
        jwks_required: true,
        tls_required: true,
        issuer_match_required: true,
        audience_match_required: true,
        expiration_required: true,
        not_before_and_issued_at_checked: true,
        nonce_required: true,
        key_id_required: true,
        alg_none_forbidden: true,
        symmetric_algorithms_forbidden: true,
        tenant_claim_required: true,
        subject_claim_required: true,
        mfa_claim_required_for_sensitive_routes: true,
        route_policy_scope_alignment_required: true,
        discovery_fetch_runtime_attached: false,
        jwks_fetch_runtime_attached: false,
        oidc_signature_verification_attached: false,
        external_identity_provider_attached: false,
        token_introspection_attached: false,
        durable_session_store_attached: false,
        runtime_auth_middleware_attached: false,
        cloud_gateway_enforcement_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

pub fn validate_enterprise_suite_identity_provider_verification_plan(
    plan: &EnterpriseSuiteIdentityProviderVerificationPlan,
) -> Result<(), IdentityProviderVerificationError> {
    validate_plan_shape(plan)?;
    validate_urls(plan)?;
    validate_claims(plan)?;
    validate_algorithms(plan)?;
    validate_cache_policy(plan)?;
    validate_security_controls(plan)?;
    validate_nonclaims(plan)?;
    validate_auth_runtime_alignment(plan)?;
    Ok(())
}

pub fn render_identity_provider_verification_checklist(
    plan: &EnterpriseSuiteIdentityProviderVerificationPlan,
) -> Result<String, IdentityProviderVerificationError> {
    validate_enterprise_suite_identity_provider_verification_plan(plan)?;
    Ok(format!(
        "review_only: true\nissuer: {}\naudience: {}\ndiscovery_document_url: {}\njwks_uri: {}\nrequired_claims: {}\nallowed_signing_algorithms: {}\nchecks: tls issuer audience exp nbf iat nonce kid tenant scope mfa route_scope_alignment\nnon_claims: no_discovery_fetch no_jwks_fetch no_signature_verification no_external_idp no_token_introspection no_durable_session no_runtime_middleware no_cloud_gateway no_runtime_audit\n",
        plan.issuer,
        plan.audience,
        plan.discovery_document_url,
        plan.jwks_uri,
        plan.required_claims
            .iter()
            .map(|claim| claim.claim_name)
            .collect::<Vec<_>>()
            .join(","),
        plan.allowed_signing_algorithms.join(","),
    ))
}

fn required_claims(
    issuer: &'static str,
    audience: &'static str,
) -> Vec<IdentityProviderClaimRequirement> {
    vec![
        claim("iss", issuer, "oidc-core-id-token"),
        claim("aud", audience, "oidc-core-id-token"),
        claim("exp", "numeric-date", "jwt-registered-claim"),
        claim("nbf", "numeric-date", "jwt-registered-claim"),
        claim("iat", "numeric-date", "jwt-registered-claim"),
        claim("sub", "stable-subject", "oidc-core-id-token"),
        claim("nonce", "request-bound-nonce", "oidc-core-id-token"),
        claim(
            "tenant_id",
            "oyatie-tenant",
            "enterprise-suite-tenant-isolation",
        ),
        claim(
            "scp",
            "route-required-scope",
            "enterprise-suite-auth-runtime",
        ),
        claim("amr", "mfa-method-reference", "sensitive-route-mfa"),
        claim("acr", "assurance-context", "sensitive-route-assurance"),
    ]
}

fn claim(
    claim_name: &'static str,
    expected_value: &'static str,
    source: &'static str,
) -> IdentityProviderClaimRequirement {
    IdentityProviderClaimRequirement {
        claim_name,
        expected_value,
        required: true,
        source,
    }
}

fn validate_plan_shape(
    plan: &EnterpriseSuiteIdentityProviderVerificationPlan,
) -> Result<(), IdentityProviderVerificationError> {
    if !valid_label(plan.plan_name)
        || plan.schema_version != SCHEMA_VERSION
        || !valid_https_url(plan.issuer)
        || !valid_label(plan.audience)
        || !valid_claim_name(plan.subject_claim)
        || !valid_claim_name(plan.tenant_claim)
        || !valid_claim_name(plan.scope_claim)
        || !valid_claim_name(plan.nonce_claim)
        || !valid_claim_name(plan.mfa_claim)
        || !valid_claim_name(plan.assurance_claim)
        || plan.required_claims.len() < MIN_REQUIRED_CLAIMS
        || plan.allowed_signing_algorithms.len() < MIN_ALLOWED_ALGORITHMS
    {
        return Err(IdentityProviderVerificationError::InvalidPlan);
    }
    Ok(())
}

fn validate_urls(
    plan: &EnterpriseSuiteIdentityProviderVerificationPlan,
) -> Result<(), IdentityProviderVerificationError> {
    for value in [
        plan.discovery_document_url,
        plan.jwks_uri,
        plan.authorization_endpoint,
        plan.token_endpoint,
        plan.userinfo_endpoint,
    ] {
        if !valid_https_url(value) {
            return Err(IdentityProviderVerificationError::InvalidUrl);
        }
    }
    if !plan.discovery_document_url.starts_with(plan.issuer)
        || !plan
            .discovery_document_url
            .ends_with("/.well-known/openid-configuration")
        || !plan.jwks_uri.starts_with(plan.issuer)
        || !plan.jwks_uri.ends_with("/.well-known/jwks.json")
    {
        return Err(IdentityProviderVerificationError::InvalidUrl);
    }
    Ok(())
}

fn validate_claims(
    plan: &EnterpriseSuiteIdentityProviderVerificationPlan,
) -> Result<(), IdentityProviderVerificationError> {
    let required_names = [
        "iss",
        "aud",
        "exp",
        "nbf",
        "iat",
        plan.subject_claim,
        plan.nonce_claim,
        plan.tenant_claim,
        plan.scope_claim,
    ];
    let mut seen = BTreeSet::new();
    for claim in &plan.required_claims {
        if !claim.required
            || !valid_claim_name(claim.claim_name)
            || claim.expected_value.is_empty()
            || has_unsafe_text(claim.expected_value)
            || !valid_ref(claim.source)
        {
            return Err(IdentityProviderVerificationError::InvalidClaim);
        }
        if !seen.insert(claim.claim_name.to_string()) {
            return Err(IdentityProviderVerificationError::DuplicateClaim(
                claim.claim_name.to_string(),
            ));
        }
    }
    for required_name in required_names {
        if !seen.contains(required_name) {
            return Err(IdentityProviderVerificationError::MissingRequiredClaim(
                required_name.to_string(),
            ));
        }
    }
    let issuer_claim = plan
        .required_claims
        .iter()
        .find(|claim| claim.claim_name == "iss")
        .ok_or_else(|| IdentityProviderVerificationError::MissingRequiredClaim("iss".into()))?;
    let audience_claim = plan
        .required_claims
        .iter()
        .find(|claim| claim.claim_name == "aud")
        .ok_or_else(|| IdentityProviderVerificationError::MissingRequiredClaim("aud".into()))?;
    if issuer_claim.expected_value != plan.issuer || audience_claim.expected_value != plan.audience
    {
        return Err(IdentityProviderVerificationError::InvalidClaim);
    }
    Ok(())
}

fn validate_algorithms(
    plan: &EnterpriseSuiteIdentityProviderVerificationPlan,
) -> Result<(), IdentityProviderVerificationError> {
    let mut seen = BTreeSet::new();
    for alg in &plan.allowed_signing_algorithms {
        if !matches!(
            *alg,
            "RS256" | "RS384" | "RS512" | "PS256" | "PS384" | "PS512" | "ES256" | "ES384" | "ES512"
        ) {
            return Err(IdentityProviderVerificationError::InvalidAlgorithm);
        }
        if !seen.insert((*alg).to_string()) {
            return Err(IdentityProviderVerificationError::DuplicateAlgorithm(
                (*alg).to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_cache_policy(
    plan: &EnterpriseSuiteIdentityProviderVerificationPlan,
) -> Result<(), IdentityProviderVerificationError> {
    if plan.clock_skew_seconds == 0
        || plan.clock_skew_seconds > MAX_CLOCK_SKEW_SECONDS
        || plan.max_token_age_seconds == 0
        || plan.max_token_age_seconds > MAX_TOKEN_AGE_SECONDS
        || plan.jwks_cache_ttl_seconds == 0
        || plan.jwks_cache_ttl_seconds > MAX_JWKS_CACHE_TTL_SECONDS
        || plan.key_rotation_overlap_seconds < MIN_KEY_ROTATION_OVERLAP_SECONDS
    {
        return Err(IdentityProviderVerificationError::InvalidCachePolicy);
    }
    Ok(())
}

fn validate_security_controls(
    plan: &EnterpriseSuiteIdentityProviderVerificationPlan,
) -> Result<(), IdentityProviderVerificationError> {
    if !plan.oidc_discovery_required
        || !plan.jwks_required
        || !plan.tls_required
        || !plan.issuer_match_required
        || !plan.audience_match_required
        || !plan.expiration_required
        || !plan.not_before_and_issued_at_checked
        || !plan.nonce_required
        || !plan.key_id_required
        || !plan.alg_none_forbidden
        || !plan.symmetric_algorithms_forbidden
        || !plan.tenant_claim_required
        || !plan.subject_claim_required
        || !plan.mfa_claim_required_for_sensitive_routes
        || !plan.route_policy_scope_alignment_required
    {
        return Err(IdentityProviderVerificationError::SecurityControlMissing);
    }
    Ok(())
}

fn validate_nonclaims(
    plan: &EnterpriseSuiteIdentityProviderVerificationPlan,
) -> Result<(), IdentityProviderVerificationError> {
    if plan.discovery_fetch_runtime_attached
        || plan.jwks_fetch_runtime_attached
        || plan.oidc_signature_verification_attached
        || plan.external_identity_provider_attached
        || plan.token_introspection_attached
        || plan.durable_session_store_attached
        || plan.runtime_auth_middleware_attached
        || plan.cloud_gateway_enforcement_attached
        || plan.runtime_audit_chain_emission_attached
    {
        return Err(IdentityProviderVerificationError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_auth_runtime_alignment(
    plan: &EnterpriseSuiteIdentityProviderVerificationPlan,
) -> Result<(), IdentityProviderVerificationError> {
    let auth_policy = enterprise_auth_runtime_policy();
    validate_enterprise_auth_runtime_policy(&auth_policy)
        .map_err(|_| IdentityProviderVerificationError::AuthRuntimePolicyInvalid)?;
    if plan.issuer != auth_policy.expected_issuer || plan.audience != auth_policy.expected_audience
    {
        return Err(IdentityProviderVerificationError::AuthRuntimePolicyInvalid);
    }
    if plan.route_policy_scope_alignment_required
        && !auth_policy
            .route_policies
            .iter()
            .all(|route| valid_scope(route.required_scope))
    {
        return Err(IdentityProviderVerificationError::AuthRuntimePolicyInvalid);
    }
    Ok(())
}

fn valid_https_url(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("https://") else {
        return false;
    };
    if rest.is_empty()
        || rest.contains('?')
        || rest.contains('#')
        || rest.contains('@')
        || rest.contains("..")
        || rest.contains("//")
        || rest.contains('\\')
        || has_unsafe_text(rest)
        || has_credential_shape(rest)
    {
        return false;
    }
    let host = rest.split('/').next().unwrap_or_default();
    host.contains('.')
        && !host.starts_with('.')
        && !host.ends_with('.')
        && !looks_like_ipv4_literal(host)
        && host
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '.')
}

fn valid_label(value: &str) -> bool {
    !value.is_empty()
        && !has_unsafe_text(value)
        && !has_credential_shape(value)
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn valid_ref(value: &str) -> bool {
    !value.is_empty()
        && !has_unsafe_text(value)
        && !has_credential_shape(value)
        && value.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_' || ch == ':'
        })
}

fn valid_claim_name(value: &str) -> bool {
    !value.is_empty()
        && !has_unsafe_text(value)
        && !has_credential_shape(value)
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn valid_scope(value: &str) -> bool {
    !value.is_empty()
        && value.contains(':')
        && !has_unsafe_text(value)
        && !has_credential_shape(value)
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == ':' || ch == '-')
}

fn has_unsafe_text(value: &str) -> bool {
    value.chars().any(char::is_whitespace) || value.chars().any(char::is_control)
}

fn has_credential_shape(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("client_secret")
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("api_key")
}

fn looks_like_ipv4_literal(value: &str) -> bool {
    let labels = value.split('.').collect::<Vec<_>>();
    labels.len() == 4
        && labels.iter().all(|label| {
            !label.is_empty()
                && label.len() <= 3
                && label.chars().all(|ch| ch.is_ascii_digit())
                && label.parse::<u8>().is_ok()
        })
}
