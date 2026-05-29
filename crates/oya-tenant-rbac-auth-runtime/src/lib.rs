//! Tenant RBAC authentication and authorization runtime foundation.
//!
//! This crate evaluates tenant-bound, scope-bound access decisions for the
//! Tenant RBAC local route catalog. It deliberately does not validate OIDC
//! signatures, fetch JWKS documents, contact an external identity provider,
//! issue credentials, persist sessions, attach a deployed gateway, mutate route
//! handlers, or claim cloud authentication readiness.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

const SCHEMA_VERSION: u32 = 1;
const MAX_CLOCK_SKEW_SECONDS: u64 = 300;
const MAX_SESSION_AGE_SECONDS: u64 = 8 * 60 * 60;
const MIN_ROUTE_POLICY_COUNT: usize = 19;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AssuranceLevel {
    Aal1,
    Aal2,
    Aal3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacAuthRoutePolicy {
    pub method: &'static str,          // data_class: PUBLIC
    pub path: &'static str,            // data_class: PUBLIC
    pub required_scope: &'static str,  // data_class: INTERNAL_ONLY
    pub sensitive_data: bool,          // data_class: INTERNAL_ONLY
    pub min_assurance: AssuranceLevel, // data_class: INTERNAL_ONLY
    pub mfa_required: bool,            // data_class: INTERNAL_ONLY
    pub break_glass_allowed: bool,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacAuthRuntimePolicy {
    pub policy_name: &'static str,       // data_class: PUBLIC
    pub expected_issuer: &'static str,   // data_class: INTERNAL_ONLY
    pub expected_audience: &'static str, // data_class: INTERNAL_ONLY
    pub replay_nonce: &'static str,      // data_class: INTERNAL_ONLY
    pub max_clock_skew_seconds: u64,     // data_class: PUBLIC
    pub max_session_age_seconds: u64,    // data_class: PUBLIC
    pub route_policies: Vec<TenantRbacAuthRoutePolicy>, // data_class: INTERNAL_ONLY
    pub deny_by_default: bool,           // data_class: PUBLIC
    pub tenant_isolation_required: bool, // data_class: PUBLIC
    pub mfa_for_sensitive_required: bool, // data_class: PUBLIC
    pub break_glass_audit_required: bool, // data_class: PUBLIC
    pub oidc_signature_verification_attached: bool, // data_class: INTERNAL_ONLY
    pub jwks_provider_attached: bool,    // data_class: INTERNAL_ONLY
    pub external_identity_provider_attached: bool, // data_class: INTERNAL_ONLY
    pub durable_session_store_attached: bool, // data_class: INTERNAL_ONLY
    pub deployed_gateway_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedPrincipal {
    pub subject_id: &'static str,           // data_class: INTERNAL_ONLY
    pub tenant_id: &'static str,            // data_class: INTERNAL_ONLY
    pub issuer: &'static str,               // data_class: INTERNAL_ONLY
    pub audience: &'static str,             // data_class: INTERNAL_ONLY
    pub issued_at_unix: u64,                // data_class: INTERNAL_ONLY
    pub expires_at_unix: u64,               // data_class: INTERNAL_ONLY
    pub nonce: &'static str,                // data_class: INTERNAL_ONLY
    pub assurance_level: AssuranceLevel,    // data_class: INTERNAL_ONLY
    pub mfa_present: bool,                  // data_class: INTERNAL_ONLY
    pub scopes: Vec<&'static str>,          // data_class: INTERNAL_ONLY
    pub roles: Vec<&'static str>,           // data_class: INTERNAL_ONLY
    pub break_glass: bool,                  // data_class: INTERNAL_ONLY
    pub federated_identity_verified: bool,  // data_class: INTERNAL_ONLY
    pub durable_session_ref_attached: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacAuthRequest {
    pub method: &'static str,           // data_class: PUBLIC
    pub path: &'static str,             // data_class: PUBLIC
    pub target_tenant_id: &'static str, // data_class: INTERNAL_ONLY
    pub now_unix: u64,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacAuthDecision {
    pub authorized: bool,                              // data_class: PUBLIC
    pub method: &'static str,                          // data_class: PUBLIC
    pub path: &'static str,                            // data_class: PUBLIC
    pub tenant_id: &'static str,                       // data_class: INTERNAL_ONLY
    pub subject_id: &'static str,                      // data_class: INTERNAL_ONLY
    pub required_scope: &'static str,                  // data_class: INTERNAL_ONLY
    pub sensitive_data: bool,                          // data_class: INTERNAL_ONLY
    pub audit_required: bool,                          // data_class: INTERNAL_ONLY
    pub external_identity_verification_required: bool, // data_class: INTERNAL_ONLY
    pub runtime_audit_emission_attached: bool,         // data_class: INTERNAL_ONLY
    pub schema_version: u32,                           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacAuthRuntimeError {
    InvalidPolicy,
    InvalidRoutePolicy,
    DuplicateRoutePolicy(String),
    RuntimeAttachmentOverclaim,
    InvalidPrincipal,
    InvalidRequest,
    InvalidIssuer,
    InvalidAudience,
    NonceMismatch,
    IssuedInFuture,
    SessionExpired,
    SessionTooOld,
    TenantIsolationRequired,
    RouteNotRegistered,
    MissingScope,
    AssuranceTooLow,
    MfaRequired,
    BreakGlassNotAllowed,
}

pub fn tenant_rbac_auth_runtime_policy() -> TenantRbacAuthRuntimePolicy {
    TenantRbacAuthRuntimePolicy {
        policy_name: "tenant-rbac-auth-runtime",
        expected_issuer: "https://identity.oyatie.com/tenant-rbac",
        expected_audience: "tenant-rbac-api",
        replay_nonce: "tenant-rbac-local-rehearsal-nonce",
        max_clock_skew_seconds: MAX_CLOCK_SKEW_SECONDS,
        max_session_age_seconds: MAX_SESSION_AGE_SECONDS,
        route_policies: tenant_rbac_auth_route_policies(),
        deny_by_default: true,
        tenant_isolation_required: true,
        mfa_for_sensitive_required: true,
        break_glass_audit_required: true,
        oidc_signature_verification_attached: false,
        jwks_provider_attached: false,
        external_identity_provider_attached: false,
        durable_session_store_attached: false,
        deployed_gateway_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

pub fn tenant_rbac_auth_route_policies() -> Vec<TenantRbacAuthRoutePolicy> {
    vec![
        route_policy("POST", "/hr/v1/employees", "hr:employees:write", false),
        route_policy(
            "POST",
            "/hr/v1/labor-compliance-workflow-plans",
            "hr:labor-compliance:plan",
            false,
        ),
        sensitive_route_policy(
            "POST",
            "/hr/v1/sensitive-read-policy-decisions",
            "hr:sensitive-read:decide",
        ),
        route_policy(
            "POST",
            "/hr/v1/leave-payroll-impact-plans",
            "hr:leave-payroll-impact:plan",
            false,
        ),
        route_policy("GET", "/hr/v1/healthz", "hr:health:read", false),
        sensitive_route_policy(
            "POST",
            "/payroll/v1/trial-closes",
            "payroll:trial-close:write",
        ),
        sensitive_route_policy(
            "POST",
            "/payroll/v1/accounting-journal-drafts",
            "payroll:journal-draft:write",
        ),
        route_policy(
            "POST",
            "/payroll/v1/hr-leave-impact-intakes",
            "payroll:leave-impact:intake",
            false,
        ),
        route_policy("GET", "/payroll/v1/healthz", "payroll:health:read", false),
        sensitive_route_policy(
            "POST",
            "/accounting/v1/journals",
            "accounting:journal:write",
        ),
        sensitive_route_policy(
            "POST",
            "/accounting/v1/payroll-postings",
            "accounting:payroll-posting:write",
        ),
        route_policy(
            "POST",
            "/accounting/v1/vat-workflow-plans",
            "accounting:vat-workflow:plan",
            false,
        ),
        route_policy(
            "GET",
            "/accounting/v1/healthz",
            "accounting:health:read",
            false,
        ),
        sensitive_route_policy(
            "POST",
            "/tenant-rbac/v1/policy-admissions",
            "tenant-rbac:policy-admission:write",
        ),
        sensitive_route_policy(
            "POST",
            "/tenant-rbac/v1/group-close-rollups",
            "tenant-rbac:group-close:rollup",
        ),
        route_policy(
            "POST",
            "/tenant-rbac/v1/cross-service-workflow-plans",
            "tenant-rbac:workflow:plan",
            false,
        ),
        route_policy(
            "POST",
            "/tenant-rbac/v1/incident-rollback-plans",
            "tenant-rbac:incident-rollback:plan",
            false,
        ),
        route_policy(
            "POST",
            "/tenant-rbac/v1/ops-commands",
            "tenant-rbac:ops-command:prepare",
            false,
        ),
        route_policy(
            "GET",
            "/tenant-rbac/v1/healthz",
            "tenant-rbac:health:read",
            false,
        ),
    ]
}

pub fn validate_tenant_rbac_auth_runtime_policy(
    policy: &TenantRbacAuthRuntimePolicy,
) -> Result<(), TenantRbacAuthRuntimeError> {
    if !valid_label(policy.policy_name)
        || !valid_url(policy.expected_issuer)
        || !valid_label(policy.expected_audience)
        || !valid_ref(policy.replay_nonce)
        || policy.max_clock_skew_seconds == 0
        || policy.max_clock_skew_seconds > MAX_CLOCK_SKEW_SECONDS
        || policy.max_session_age_seconds == 0
        || policy.max_session_age_seconds > MAX_SESSION_AGE_SECONDS
        || policy.route_policies.len() < MIN_ROUTE_POLICY_COUNT
        || policy.schema_version != SCHEMA_VERSION
    {
        return Err(TenantRbacAuthRuntimeError::InvalidPolicy);
    }
    if !policy.deny_by_default
        || !policy.tenant_isolation_required
        || !policy.mfa_for_sensitive_required
        || !policy.break_glass_audit_required
    {
        return Err(TenantRbacAuthRuntimeError::InvalidPolicy);
    }
    if policy.oidc_signature_verification_attached
        || policy.jwks_provider_attached
        || policy.external_identity_provider_attached
        || policy.durable_session_store_attached
        || policy.deployed_gateway_attached
        || policy.runtime_audit_chain_emission_attached
    {
        return Err(TenantRbacAuthRuntimeError::RuntimeAttachmentOverclaim);
    }
    let mut seen = BTreeSet::new();
    for route in &policy.route_policies {
        validate_route_policy(route)?;
        let key = format!("{} {}", route.method, route.path);
        if !seen.insert(key.clone()) {
            return Err(TenantRbacAuthRuntimeError::DuplicateRoutePolicy(key));
        }
    }
    Ok(())
}

pub fn authorize_tenant_rbac_route(
    policy: &TenantRbacAuthRuntimePolicy,
    principal: &AuthenticatedPrincipal,
    request: &TenantRbacAuthRequest,
) -> Result<TenantRbacAuthDecision, TenantRbacAuthRuntimeError> {
    validate_tenant_rbac_auth_runtime_policy(policy)?;
    validate_principal(principal)?;
    validate_request(request)?;
    validate_claims(policy, principal, request)?;

    let route = policy
        .route_policies
        .iter()
        .find(|route| route.method == request.method && route.path == request.path)
        .ok_or(TenantRbacAuthRuntimeError::RouteNotRegistered)?;

    if !principal.scopes.contains(&route.required_scope) {
        return Err(TenantRbacAuthRuntimeError::MissingScope);
    }
    if principal.assurance_level < route.min_assurance {
        return Err(TenantRbacAuthRuntimeError::AssuranceTooLow);
    }
    if route.mfa_required && !principal.mfa_present {
        return Err(TenantRbacAuthRuntimeError::MfaRequired);
    }
    if principal.break_glass && !route.break_glass_allowed {
        return Err(TenantRbacAuthRuntimeError::BreakGlassNotAllowed);
    }

    Ok(TenantRbacAuthDecision {
        authorized: true,
        method: request.method,
        path: request.path,
        tenant_id: principal.tenant_id,
        subject_id: principal.subject_id,
        required_scope: route.required_scope,
        sensitive_data: route.sensitive_data,
        audit_required: principal.break_glass || route.sensitive_data,
        external_identity_verification_required: !principal.federated_identity_verified
            || !policy.oidc_signature_verification_attached
            || !policy.jwks_provider_attached
            || !policy.external_identity_provider_attached,
        runtime_audit_emission_attached: policy.runtime_audit_chain_emission_attached,
        schema_version: SCHEMA_VERSION,
    })
}

fn route_policy(
    method: &'static str,
    path: &'static str,
    required_scope: &'static str,
    sensitive_data: bool,
) -> TenantRbacAuthRoutePolicy {
    TenantRbacAuthRoutePolicy {
        method,
        path,
        required_scope,
        sensitive_data,
        min_assurance: AssuranceLevel::Aal1,
        mfa_required: false,
        break_glass_allowed: false,
    }
}

fn sensitive_route_policy(
    method: &'static str,
    path: &'static str,
    required_scope: &'static str,
) -> TenantRbacAuthRoutePolicy {
    TenantRbacAuthRoutePolicy {
        method,
        path,
        required_scope,
        sensitive_data: true,
        min_assurance: AssuranceLevel::Aal2,
        mfa_required: true,
        break_glass_allowed: true,
    }
}

fn validate_route_policy(
    route: &TenantRbacAuthRoutePolicy,
) -> Result<(), TenantRbacAuthRuntimeError> {
    if !valid_method(route.method)
        || !valid_path(route.path)
        || !valid_scope(route.required_scope)
        || (route.sensitive_data && route.min_assurance < AssuranceLevel::Aal2)
        || (route.sensitive_data && !route.mfa_required)
    {
        return Err(TenantRbacAuthRuntimeError::InvalidRoutePolicy);
    }
    Ok(())
}

fn validate_principal(
    principal: &AuthenticatedPrincipal,
) -> Result<(), TenantRbacAuthRuntimeError> {
    if !valid_ref(principal.subject_id)
        || !valid_ref(principal.tenant_id)
        || !valid_url(principal.issuer)
        || !valid_label(principal.audience)
        || !valid_ref(principal.nonce)
        || principal.expires_at_unix <= principal.issued_at_unix
        || principal.scopes.is_empty()
        || principal.scopes.iter().any(|scope| !valid_scope(scope))
        || principal.roles.iter().any(|role| !valid_ref(role))
        || principal.durable_session_ref_attached
    {
        return Err(TenantRbacAuthRuntimeError::InvalidPrincipal);
    }
    Ok(())
}

fn validate_request(request: &TenantRbacAuthRequest) -> Result<(), TenantRbacAuthRuntimeError> {
    if !valid_method(request.method)
        || !valid_path(request.path)
        || !valid_ref(request.target_tenant_id)
        || request.now_unix == 0
    {
        return Err(TenantRbacAuthRuntimeError::InvalidRequest);
    }
    Ok(())
}

fn validate_claims(
    policy: &TenantRbacAuthRuntimePolicy,
    principal: &AuthenticatedPrincipal,
    request: &TenantRbacAuthRequest,
) -> Result<(), TenantRbacAuthRuntimeError> {
    if principal.issuer != policy.expected_issuer {
        return Err(TenantRbacAuthRuntimeError::InvalidIssuer);
    }
    if principal.audience != policy.expected_audience {
        return Err(TenantRbacAuthRuntimeError::InvalidAudience);
    }
    if principal.nonce != policy.replay_nonce {
        return Err(TenantRbacAuthRuntimeError::NonceMismatch);
    }
    if principal.tenant_id != request.target_tenant_id {
        return Err(TenantRbacAuthRuntimeError::TenantIsolationRequired);
    }
    if request
        .now_unix
        .saturating_add(policy.max_clock_skew_seconds)
        < principal.issued_at_unix
    {
        return Err(TenantRbacAuthRuntimeError::IssuedInFuture);
    }
    if request.now_unix
        > principal
            .expires_at_unix
            .saturating_add(policy.max_clock_skew_seconds)
    {
        return Err(TenantRbacAuthRuntimeError::SessionExpired);
    }
    if request.now_unix.saturating_sub(principal.issued_at_unix) > policy.max_session_age_seconds {
        return Err(TenantRbacAuthRuntimeError::SessionTooOld);
    }
    Ok(())
}

fn valid_method(value: &str) -> bool {
    matches!(value, "GET" | "POST")
}

fn valid_path(value: &str) -> bool {
    value.starts_with('/')
        && value.len() > 1
        && !unsafe_text(value)
        && !path_traversal(value)
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '/' | '-'))
}

fn valid_scope(value: &str) -> bool {
    !value.is_empty()
        && !unsafe_text(value)
        && !path_traversal(value)
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, ':' | '-'))
}

fn valid_label(value: &str) -> bool {
    !value.is_empty()
        && !unsafe_text(value)
        && !path_traversal(value)
        && value
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn valid_ref(value: &str) -> bool {
    !value.is_empty()
        && !unsafe_text(value)
        && !path_traversal(value)
        && value.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_' | ':')
        })
}

fn valid_url(value: &str) -> bool {
    let Some(host_and_path) = value.strip_prefix("https://") else {
        return false;
    };
    !host_and_path.is_empty()
        && !unsafe_text(value)
        && !path_traversal(host_and_path)
        && host_and_path.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '/' | '-')
        })
}

fn unsafe_text(value: &str) -> bool {
    value.chars().any(char::is_whitespace) || value.chars().any(char::is_control)
}

fn path_traversal(value: &str) -> bool {
    value.contains("..") || value.contains('\\') || value.contains("//")
}
