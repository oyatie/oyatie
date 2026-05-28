use oya_tenant_rbac_auth_runtime::{
    AssuranceLevel, AuthenticatedPrincipal, TenantRbacAuthRequest, TenantRbacAuthRuntimeError,
    authorize_tenant_rbac_route, tenant_rbac_auth_runtime_policy,
    validate_tenant_rbac_auth_runtime_policy,
};

const NOW: u64 = 1_779_552_000;

#[test]
fn auth_runtime_authorizes_scoped_tenant_route_and_denies_cross_tenant() {
    let policy = tenant_rbac_auth_runtime_policy();
    validate_tenant_rbac_auth_runtime_policy(&policy).expect("policy validates");

    let principal = sample_principal(vec!["tenant-rbac:policy-admission:write"]);
    let route_request = request("POST", "/tenant-rbac/v1/policy-admissions", "tenant_acme");
    let decision = authorize_tenant_rbac_route(&policy, &principal, &route_request)
        .expect("scoped same-tenant request is authorized");

    assert!(decision.authorized);
    assert_eq!(decision.tenant_id, "tenant_acme");
    assert_eq!(
        decision.required_scope,
        "tenant-rbac:policy-admission:write"
    );
    assert!(decision.sensitive_data);
    assert!(decision.audit_required);
    assert!(decision.external_identity_verification_required);
    assert!(!decision.runtime_audit_emission_attached);

    let cross_tenant = request("POST", "/tenant-rbac/v1/policy-admissions", "tenant_other");
    assert_eq!(
        authorize_tenant_rbac_route(&policy, &principal, &cross_tenant),
        Err(TenantRbacAuthRuntimeError::TenantIsolationRequired)
    );
}

#[test]
fn auth_runtime_refuses_expired_bad_issuer_audience_and_nonce() {
    let policy = tenant_rbac_auth_runtime_policy();
    let request = request("POST", "/tenant-rbac/v1/policy-admissions", "tenant_acme");

    let mut principal = sample_principal(vec!["tenant-rbac:policy-admission:write"]);
    principal.issued_at_unix = NOW - 900;
    principal.expires_at_unix = NOW - 400;
    assert_eq!(
        authorize_tenant_rbac_route(&policy, &principal, &request),
        Err(TenantRbacAuthRuntimeError::SessionExpired)
    );

    let mut principal = sample_principal(vec!["tenant-rbac:policy-admission:write"]);
    principal.issuer = "https://identity.other.example/tenant-rbac";
    assert_eq!(
        authorize_tenant_rbac_route(&policy, &principal, &request),
        Err(TenantRbacAuthRuntimeError::InvalidIssuer)
    );

    let mut principal = sample_principal(vec!["tenant-rbac:policy-admission:write"]);
    principal.audience = "other-api";
    assert_eq!(
        authorize_tenant_rbac_route(&policy, &principal, &request),
        Err(TenantRbacAuthRuntimeError::InvalidAudience)
    );

    let mut principal = sample_principal(vec!["tenant-rbac:policy-admission:write"]);
    principal.nonce = "tenant-rbac-wrong-nonce";
    assert_eq!(
        authorize_tenant_rbac_route(&policy, &principal, &request),
        Err(TenantRbacAuthRuntimeError::NonceMismatch)
    );
}

#[test]
fn auth_runtime_denies_by_default_and_requires_route_scope() {
    let policy = tenant_rbac_auth_runtime_policy();
    let principal = sample_principal(vec!["tenant-rbac:policy-admission:write"]);

    let unknown_route = request("POST", "/tenant-rbac/v1/not-registered", "tenant_acme");
    assert_eq!(
        authorize_tenant_rbac_route(&policy, &principal, &unknown_route),
        Err(TenantRbacAuthRuntimeError::RouteNotRegistered)
    );

    let missing_scope = request("POST", "/payroll/v1/trial-closes", "tenant_acme");
    assert_eq!(
        authorize_tenant_rbac_route(&policy, &principal, &missing_scope),
        Err(TenantRbacAuthRuntimeError::MissingScope)
    );
}

#[test]
fn auth_runtime_requires_mfa_and_assurance_for_sensitive_routes() {
    let policy = tenant_rbac_auth_runtime_policy();
    let request = request(
        "POST",
        "/hr/v1/sensitive-read-policy-decisions",
        "tenant_acme",
    );

    let mut low_assurance = sample_principal(vec!["hr:sensitive-read:decide"]);
    low_assurance.assurance_level = AssuranceLevel::Aal1;
    assert_eq!(
        authorize_tenant_rbac_route(&policy, &low_assurance, &request),
        Err(TenantRbacAuthRuntimeError::AssuranceTooLow)
    );

    let mut missing_mfa = sample_principal(vec!["hr:sensitive-read:decide"]);
    missing_mfa.mfa_present = false;
    assert_eq!(
        authorize_tenant_rbac_route(&policy, &missing_mfa, &request),
        Err(TenantRbacAuthRuntimeError::MfaRequired)
    );
}

#[test]
fn auth_runtime_allows_break_glass_only_with_audit_on_allowed_route() {
    let policy = tenant_rbac_auth_runtime_policy();
    let mut principal = sample_principal(vec!["hr:sensitive-read:decide", "hr:health:read"]);
    principal.break_glass = true;

    let sensitive_request = request(
        "POST",
        "/hr/v1/sensitive-read-policy-decisions",
        "tenant_acme",
    );
    let decision = authorize_tenant_rbac_route(&policy, &principal, &sensitive_request)
        .expect("break-glass sensitive route is authorized with audit requirement");
    assert!(decision.authorized);
    assert!(decision.audit_required);

    let health_request = request("GET", "/hr/v1/healthz", "tenant_acme");
    assert_eq!(
        authorize_tenant_rbac_route(&policy, &principal, &health_request),
        Err(TenantRbacAuthRuntimeError::BreakGlassNotAllowed)
    );
}

#[test]
fn auth_runtime_rejects_runtime_attachment_overclaims() {
    let mut policy = tenant_rbac_auth_runtime_policy();
    policy.external_identity_provider_attached = true;
    assert_eq!(
        validate_tenant_rbac_auth_runtime_policy(&policy),
        Err(TenantRbacAuthRuntimeError::RuntimeAttachmentOverclaim)
    );

    let mut policy = tenant_rbac_auth_runtime_policy();
    policy.deployed_gateway_attached = true;
    assert_eq!(
        validate_tenant_rbac_auth_runtime_policy(&policy),
        Err(TenantRbacAuthRuntimeError::RuntimeAttachmentOverclaim)
    );
}

fn sample_principal(scopes: Vec<&'static str>) -> AuthenticatedPrincipal {
    AuthenticatedPrincipal {
        subject_id: "subject_tenant_rbac_operator",
        tenant_id: "tenant_acme",
        issuer: "https://identity.oyatie.com/tenant-rbac",
        audience: "tenant-rbac-api",
        issued_at_unix: NOW - 60,
        expires_at_unix: NOW + 600,
        nonce: "tenant-rbac-local-rehearsal-nonce",
        assurance_level: AssuranceLevel::Aal2,
        mfa_present: true,
        scopes,
        roles: vec!["tenant_rbac_operator"],
        break_glass: false,
        federated_identity_verified: false,
        durable_session_ref_attached: false,
    }
}

fn request(
    method: &'static str,
    path: &'static str,
    target_tenant_id: &'static str,
) -> TenantRbacAuthRequest {
    TenantRbacAuthRequest {
        method,
        path,
        target_tenant_id,
        now_unix: NOW,
    }
}
