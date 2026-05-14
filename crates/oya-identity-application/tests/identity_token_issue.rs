use oya_identity_application::{
    IDENTITY_TOKEN_ISSUE_OPENAPI_CONTRACT, IDENTITY_TOKEN_ISSUE_SURFACE, IdentityApiAuthorization,
    IdentityApiBoundaryContext, IdentityApiPrincipal, IdentityScopeRef, IdentityTokenIssueApiError,
    IdentityTokenIssueApiRequest, IdentityTokenIssueApiStatus, IdentityTokenIssueIdempotencyLedger,
    IdentityTokenIssueRequest, issue_identity_token_from_app,
};

const REQUEST_ID: &str = "req_identity_token_001";
const IDEMPOTENCY_KEY: &str = "idem_identity_token_001";

#[test]
fn identity_token_issue_contract_runtime_constants_are_covered() {
    assert_eq!(IDENTITY_TOKEN_ISSUE_SURFACE, "identity.token.issue");
    assert_eq!(
        IDENTITY_TOKEN_ISSUE_OPENAPI_CONTRACT,
        "contracts/openapi/platform/platform-identity-token-v1.yaml"
    );
    assert_eq!(IdentityTokenIssueApiStatus::Ok.code(), 200);
    assert_eq!(IdentityTokenIssueApiStatus::BadRequest.code(), 400);
    assert_eq!(IdentityTokenIssueApiStatus::Unauthorized.code(), 401);
    assert_eq!(IdentityTokenIssueApiStatus::Forbidden.code(), 403);
    assert_eq!(IdentityTokenIssueApiStatus::UnprocessableEntity.code(), 422);
}

#[test]
fn identity_token_issue_returns_purpose_bound_sts_fingerprint_and_replays_idempotently() {
    let mut idempotency = IdentityTokenIssueIdempotencyLedger::default();
    let request = token_request(REQUEST_ID, IDEMPOTENCY_KEY);

    let first = issue_identity_token_from_app(&mut idempotency, request.clone())
        .expect("first STS issue succeeds");
    let second = issue_identity_token_from_app(&mut idempotency, request)
        .expect("same request fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(first.data.tenant_id, "ten_kr");
    assert_eq!(first.data.subject_id, "usr_admin");
    assert_eq!(first.data.subject_kind, "human");
    assert_eq!(first.data.credential_kind, "sts");
    assert_eq!(first.data.purpose, "CapabilityInvocation");
    assert_eq!(first.data.scopes[0].value, "foundry.invoke");
    assert_eq!(first.data.issued_at_epoch_seconds, 1_700_000_000);
    assert_eq!(first.data.expires_at_epoch_seconds, 1_700_000_900);
    assert!(first.data.token_fingerprint.starts_with("sts1:"));
    assert_eq!(first.data.schema_version, 1);
    assert_eq!(first.metadata.request_id, REQUEST_ID);
}

#[test]
fn identity_token_issue_rejects_tenant_and_principal_drift_before_issuance() {
    let mut idempotency = IdentityTokenIssueIdempotencyLedger::default();
    let mut tenant_drift = token_request(
        "req_identity_token_drift_tenant",
        "idem_identity_token_drift_tenant",
    );
    tenant_drift.body.tenant_id = "ten_other".to_string();

    let tenant_error = issue_identity_token_from_app(&mut idempotency, tenant_drift)
        .expect_err("tenant drift is forbidden before credential issue");
    assert!(matches!(
        tenant_error,
        IdentityTokenIssueApiError::TenantMismatch { .. }
    ));
    assert_eq!(
        tenant_error.identity_token_issue_status(),
        IdentityTokenIssueApiStatus::Forbidden
    );

    let mut principal_drift = token_request(
        "req_identity_token_drift_principal",
        "idem_identity_token_drift_principal",
    );
    principal_drift.body.subject_id = "usr_other".to_string();
    let principal_error = issue_identity_token_from_app(&mut idempotency, principal_drift)
        .expect_err("principal drift is forbidden before credential issue");
    assert!(matches!(
        principal_error,
        IdentityTokenIssueApiError::PrincipalMismatch { .. }
    ));
    assert_eq!(principal_error.identity_token_issue_status_code(), 403);

    assert!(idempotency.is_empty());
}

#[test]
fn identity_token_issue_separates_missing_principal_from_denied_authorization() {
    let mut idempotency = IdentityTokenIssueIdempotencyLedger::default();
    let mut unauthenticated =
        token_request("req_identity_token_authn", "idem_identity_token_authn");
    unauthenticated.principal.principal_id.clear();

    let authn_error = issue_identity_token_from_app(&mut idempotency, unauthenticated)
        .expect_err("missing principal is authentication failure");
    assert_eq!(
        authn_error.identity_token_issue_status(),
        IdentityTokenIssueApiStatus::Unauthorized
    );

    let mut denied = token_request("req_identity_token_authz", "idem_identity_token_authz");
    denied.authorization.allowed_surfaces = vec!["identity.user.upsert".to_string()];
    let authz_error = issue_identity_token_from_app(&mut idempotency, denied)
        .expect_err("missing surface grant is authorization failure");
    assert!(matches!(
        authz_error,
        IdentityTokenIssueApiError::AuthorizationDenied { ref surface }
            if surface == IDENTITY_TOKEN_ISSUE_SURFACE
    ));
    assert_eq!(
        authz_error.identity_token_issue_status(),
        IdentityTokenIssueApiStatus::Forbidden
    );
    assert!(idempotency.is_empty());
}

#[test]
fn identity_token_issue_rejects_non_sts_invalid_purpose_ttl_and_unscoped_requests() {
    let mut idempotency = IdentityTokenIssueIdempotencyLedger::default();

    let mut long_lived = token_request(
        "req_identity_token_long_lived",
        "idem_identity_token_long_lived",
    );
    long_lived.body.credential_kind = "long_lived_api_key".to_string();
    assert!(matches!(
        issue_identity_token_from_app(&mut idempotency, long_lived),
        Err(IdentityTokenIssueApiError::Identity(_))
    ));

    let mut invalid_purpose = token_request(
        "req_identity_token_bad_purpose",
        "idem_identity_token_bad_purpose",
    );
    invalid_purpose.body.purpose = "Banana".to_string();
    assert!(matches!(
        issue_identity_token_from_app(&mut idempotency, invalid_purpose),
        Err(IdentityTokenIssueApiError::InvalidPurpose { .. })
    ));

    let mut too_long = token_request("req_identity_token_ttl", "idem_identity_token_ttl");
    too_long.body.ttl_seconds = 3_601;
    let ttl_error = issue_identity_token_from_app(&mut idempotency, too_long)
        .expect_err("STS tokens must be at most one hour");
    assert_eq!(ttl_error.identity_token_issue_status_code(), 400);

    let mut unscoped = token_request(
        "req_identity_token_unscoped",
        "idem_identity_token_unscoped",
    );
    unscoped.body.scopes.clear();
    assert!(matches!(
        issue_identity_token_from_app(&mut idempotency, unscoped),
        Err(IdentityTokenIssueApiError::Identity(_))
    ));
    assert!(idempotency.is_empty());
}

#[test]
fn identity_token_issue_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut idempotency = IdentityTokenIssueIdempotencyLedger::default();
    let mut request = token_request("req_identity_token_reused", "idem_identity_token_reused");

    issue_identity_token_from_app(&mut idempotency, request.clone())
        .expect("first idempotent request succeeds");

    request.body.scopes.push(IdentityScopeRef {
        value: "cloud.iam.read".to_string(),
    });
    let error = issue_identity_token_from_app(&mut idempotency, request)
        .expect_err("same idempotency key with changed body is rejected");

    assert_eq!(
        error,
        IdentityTokenIssueApiError::IdempotencyKeyReused {
            idempotency_key: "idem_identity_token_reused".to_string()
        }
    );
    assert_eq!(
        error.identity_token_issue_status(),
        IdentityTokenIssueApiStatus::UnprocessableEntity
    );
    assert_eq!(idempotency.len(), 1);
}

fn token_request(request_id: &str, idempotency_key: &str) -> IdentityTokenIssueApiRequest {
    IdentityTokenIssueApiRequest {
        boundary: IdentityApiBoundaryContext {
            request_id: request_id.to_string(),
            tenant_id: "ten_kr".to_string(),
            idempotency_key: idempotency_key.to_string(),
        },
        principal: IdentityApiPrincipal {
            tenant_id: "ten_kr".to_string(),
            principal_id: "usr_admin".to_string(),
            principal_kind: "human".to_string(),
            owning_capability_id: None,
        },
        authorization: IdentityApiAuthorization {
            tenant_id: "ten_kr".to_string(),
            principal_id: "usr_admin".to_string(),
            decision_id: "authz_identity_token_issue".to_string(),
            allowed_surfaces: vec![IDENTITY_TOKEN_ISSUE_SURFACE.to_string()],
        },
        body: IdentityTokenIssueRequest {
            tenant_id: "ten_kr".to_string(),
            subject_id: "usr_admin".to_string(),
            subject_kind: "human".to_string(),
            owning_capability_id: None,
            credential_kind: "sts".to_string(),
            purpose: "CapabilityInvocation".to_string(),
            ttl_seconds: 900,
            scopes: vec![IdentityScopeRef {
                value: "foundry.invoke".to_string(),
            }],
            issued_at_epoch_seconds: 1_700_000_000,
        },
    }
}
