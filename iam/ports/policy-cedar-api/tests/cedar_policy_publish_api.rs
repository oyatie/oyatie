// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_policy_cedar_api::{
    CEDAR_POLICY_PUBLISH_OPENAPI_CONTRACT, CEDAR_POLICY_PUBLISH_SURFACE,
    CedarPolicyApiAuthorization, CedarPolicyApiBoundaryContext, CedarPolicyApiPrincipal,
    CedarPolicyPublishApiError, CedarPolicyPublishApiRequest, CedarPolicyPublishApiStatus,
    CedarPolicyPublishIdempotencyLedger, CedarPolicyPublishRequest, CedarPolicyRequiredAttribute,
    CedarPolicyRuleRef, CedarPolicyScopeRef,
    authz::{
        CallerCredential, ConfiguredBearerPrincipalVerifier, PrincipalVerifier, VerifiedPrincipal,
    },
    publish_cedar_policy_from_api,
};
use iam_policy_cedar_domain::{AuthorizationQuery, AuthorizationSubject, PolicySet};
use std::collections::BTreeMap;

const REQUEST_ID: &str = "req_cedar_policy_001";
const IDEMPOTENCY_KEY: &str = "idem_cedar_policy_001";
const OPERATOR_TENANT_ID: &str = "ten_platform";
const POLICY_ID: &str = "pol_tenant_admin";
const VERSION: &str = "1.0.0";

/// Bearer secret used exclusively by the test verifier. Not a production secret.
const TEST_BEARER_SECRET: &str = "test-bearer-secret-for-unit-tests";
/// Principal id bound to the test verifier (matches `policy_request` body).
const TEST_PRINCIPAL_ID: &str = "usr_platform_admin";

/// Test helper: mint a [`VerifiedPrincipal`] by running through the legitimate
/// [`ConfiguredBearerPrincipalVerifier`] path — the same path production uses.
/// This proves that external crates CANNOT forge a `VerifiedPrincipal` by struct
/// literal (the fields are private); they MUST go through a real verifier.
fn test_principal() -> VerifiedPrincipal {
    ConfiguredBearerPrincipalVerifier::new(
        TEST_BEARER_SECRET,
        TEST_PRINCIPAL_ID,
        OPERATOR_TENANT_ID,
    )
    .expect("test verifier constructs with non-empty secret and identity")
    .verify_principal(&CallerCredential {
        authorization: Some(format!("Bearer {TEST_BEARER_SECRET}")),
        claimed_principal_id: TEST_PRINCIPAL_ID.to_string(),
        claimed_tenant_id: OPERATOR_TENANT_ID.to_string(),
    })
    .expect("test credential verifies")
}

#[test]
fn cedar_policy_publish_contract_runtime_constants_are_covered() {
    assert_eq!(CEDAR_POLICY_PUBLISH_SURFACE, "cedar.policy.publish");
    assert_eq!(
        CEDAR_POLICY_PUBLISH_OPENAPI_CONTRACT,
        "contracts/openapi/platform/platform-policy-cedar-v1.yaml"
    );
    assert_eq!(CedarPolicyPublishApiStatus::Created.code(), 201);
    assert_eq!(CedarPolicyPublishApiStatus::BadRequest.code(), 400);
    assert_eq!(CedarPolicyPublishApiStatus::Unauthorized.code(), 401);
    assert_eq!(CedarPolicyPublishApiStatus::Forbidden.code(), 403);
    assert_eq!(CedarPolicyPublishApiStatus::Conflict.code(), 409);
    assert_eq!(CedarPolicyPublishApiStatus::UnprocessableEntity.code(), 422);
}

#[test]
fn cedar_policy_publish_publishes_tenant_policy_and_replays_idempotently() {
    let mut policies = PolicySet::default();
    let mut idempotency = CedarPolicyPublishIdempotencyLedger::default();
    let request = policy_request(REQUEST_ID, IDEMPOTENCY_KEY, POLICY_ID, VERSION);

    let first = publish_cedar_policy_from_api(
        &test_principal(),
        &mut policies,
        &mut idempotency,
        request.clone(),
    )
    .expect("first policy publish succeeds");
    let second =
        publish_cedar_policy_from_api(&test_principal(), &mut policies, &mut idempotency, request)
            .expect("same policy publish request replays");

    assert_eq!(first, second);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(first.data.policy_id, POLICY_ID);
    assert_eq!(first.data.version, VERSION);
    assert_eq!(first.data.scope.kind, "tenant");
    assert_eq!(first.data.scope.tenant_id.as_deref(), Some("ten_alpha"));
    assert_eq!(first.data.rules[0].effect, "allow");
    assert_eq!(first.data.schema_version, 1);
    assert_eq!(first.metadata.request_id, REQUEST_ID);

    let decision = policies.authorize(&AuthorizationQuery {
        subject: AuthorizationSubject {
            tenant_id: "ten_alpha".to_string(),
            roles: vec!["tenant-admin".to_string()],
        },
        action: "tenant.settings.update".to_string(),
        resource: "tenant:ten_kr:settings".to_string(),
        attributes: BTreeMap::from([("region".to_string(), "region-home".to_string())]),
    });
    assert!(decision.allowed);
}

#[test]
fn cedar_policy_publish_supports_supersedes_chain_for_new_semver_versions() {
    let mut policies = PolicySet::default();
    let mut idempotency = CedarPolicyPublishIdempotencyLedger::default();
    publish_cedar_policy_from_api(
        &test_principal(),
        &mut policies,
        &mut idempotency,
        policy_request(
            "req_cedar_policy_first",
            "idem_cedar_policy_first",
            POLICY_ID,
            VERSION,
        ),
    )
    .expect("initial policy publish succeeds");

    let mut request = policy_request(
        "req_cedar_policy_second",
        "idem_cedar_policy_second",
        POLICY_ID,
        "1.1.0",
    );
    request.body.supersedes = Some(VERSION.to_string());
    let response =
        publish_cedar_policy_from_api(&test_principal(), &mut policies, &mut idempotency, request)
            .expect("new version can supersede prior version");

    assert_eq!(response.data.version, "1.1.0");
    assert_eq!(response.data.supersedes.as_deref(), Some(VERSION));
    assert_eq!(idempotency.len(), 2);
}

#[test]
fn cedar_policy_publish_supports_global_scope_without_tenant_binding() {
    let mut policies = PolicySet::default();
    let mut idempotency = CedarPolicyPublishIdempotencyLedger::default();
    let mut request = policy_request(
        "req_cedar_policy_global",
        "idem_cedar_policy_global",
        "pol_global_reader",
        VERSION,
    );
    request.body.scope = CedarPolicyScopeRef {
        kind: "global".to_string(),
        tenant_id: None,
    };

    let response =
        publish_cedar_policy_from_api(&test_principal(), &mut policies, &mut idempotency, request)
            .expect("global policy publish succeeds");

    assert_eq!(response.data.scope.kind, "global");
    assert_eq!(response.data.scope.tenant_id, None);

    let decision = policies.authorize(&AuthorizationQuery {
        subject: AuthorizationSubject {
            tenant_id: "ten_any".to_string(),
            roles: vec!["tenant-admin".to_string()],
        },
        action: "tenant.settings.update".to_string(),
        resource: "tenant:ten_any:settings".to_string(),
        attributes: BTreeMap::from([("region".to_string(), "region-home".to_string())]),
    });
    assert!(decision.allowed);
}

#[test]
fn cedar_policy_publish_rejects_path_body_scope_and_effect_drift_before_kernel() {
    let mut policies = PolicySet::default();
    let mut idempotency = CedarPolicyPublishIdempotencyLedger::default();
    let mut path_drift = policy_request(
        "req_cedar_policy_drift",
        "idem_cedar_policy_drift",
        POLICY_ID,
        VERSION,
    );
    path_drift.body.policy_id = "pol_other".to_string();

    let path_error = publish_cedar_policy_from_api(
        &test_principal(),
        &mut policies,
        &mut idempotency,
        path_drift,
    )
    .expect_err("path/body policy drift is rejected");
    assert!(matches!(
        path_error,
        CedarPolicyPublishApiError::PolicyPathBodyMismatch { .. }
    ));
    assert_eq!(path_error.cedar_policy_publish_status_code(), 400);

    let mut invalid_scope = policy_request(
        "req_cedar_policy_scope",
        "idem_cedar_policy_scope",
        POLICY_ID,
        VERSION,
    );
    invalid_scope.body.scope.kind = "workspace".to_string();
    assert!(matches!(
        publish_cedar_policy_from_api(
            &test_principal(),
            &mut policies,
            &mut idempotency,
            invalid_scope
        ),
        Err(CedarPolicyPublishApiError::InvalidScopeKind { .. })
    ));

    let mut invalid_effect = policy_request(
        "req_cedar_policy_effect",
        "idem_cedar_policy_effect",
        POLICY_ID,
        VERSION,
    );
    invalid_effect.body.rules[0].effect = "permit".to_string();
    assert!(matches!(
        publish_cedar_policy_from_api(
            &test_principal(),
            &mut policies,
            &mut idempotency,
            invalid_effect
        ),
        Err(CedarPolicyPublishApiError::InvalidRuleEffect { .. })
    ));
    assert!(idempotency.is_empty());
}

#[test]
fn cedar_policy_publish_separates_missing_principal_from_denied_authorization() {
    let mut policies = PolicySet::default();
    let mut idempotency = CedarPolicyPublishIdempotencyLedger::default();
    let mut unauthenticated = policy_request(
        "req_cedar_policy_authn",
        "idem_cedar_policy_authn",
        POLICY_ID,
        VERSION,
    );
    unauthenticated.principal.principal_id.clear();

    let authn_error = publish_cedar_policy_from_api(
        &test_principal(),
        &mut policies,
        &mut idempotency,
        unauthenticated,
    )
    .expect_err("missing principal is authentication failure");
    assert_eq!(
        authn_error.cedar_policy_publish_status(),
        CedarPolicyPublishApiStatus::Unauthorized
    );

    let mut denied = policy_request(
        "req_cedar_policy_authz",
        "idem_cedar_policy_authz",
        POLICY_ID,
        VERSION,
    );
    denied.authorization.allowed_surfaces = vec!["identity.user.upsert".to_string()];
    let authz_error =
        publish_cedar_policy_from_api(&test_principal(), &mut policies, &mut idempotency, denied)
            .expect_err("missing cedar.policy.publish grant is authorization failure");
    assert!(
        matches!(authz_error, CedarPolicyPublishApiError::AuthorizationDenied { ref surface } if surface == CEDAR_POLICY_PUBLISH_SURFACE)
    );
    assert_eq!(
        authz_error.cedar_policy_publish_status(),
        CedarPolicyPublishApiStatus::Forbidden
    );
    assert!(idempotency.is_empty());
}

#[test]
fn cedar_policy_publish_maps_duplicate_invalid_semver_empty_rules_and_reused_idempotency() {
    let mut policies = PolicySet::default();
    let mut idempotency = CedarPolicyPublishIdempotencyLedger::default();
    let mut request = policy_request(
        "req_cedar_policy_first",
        "idem_cedar_policy_first",
        POLICY_ID,
        VERSION,
    );
    publish_cedar_policy_from_api(
        &test_principal(),
        &mut policies,
        &mut idempotency,
        request.clone(),
    )
    .expect("first policy publish succeeds");

    let duplicate = publish_cedar_policy_from_api(
        &test_principal(),
        &mut policies,
        &mut idempotency,
        policy_request(
            "req_cedar_policy_duplicate",
            "idem_cedar_policy_duplicate",
            POLICY_ID,
            VERSION,
        ),
    )
    .expect_err("duplicate policy version conflicts");
    assert!(matches!(duplicate, CedarPolicyPublishApiError::Policy(_)));
    assert_eq!(
        duplicate.cedar_policy_publish_status(),
        CedarPolicyPublishApiStatus::Conflict
    );

    let invalid_semver = policy_request(
        "req_cedar_policy_semver",
        "idem_cedar_policy_semver",
        POLICY_ID,
        "v1",
    );
    assert!(matches!(
        publish_cedar_policy_from_api(
            &test_principal(),
            &mut policies,
            &mut idempotency,
            invalid_semver
        ),
        Err(CedarPolicyPublishApiError::Policy(_))
    ));

    let mut empty_rules = policy_request(
        "req_cedar_policy_empty",
        "idem_cedar_policy_empty",
        POLICY_ID,
        "2.0.0",
    );
    empty_rules.body.rules.clear();
    assert!(matches!(
        publish_cedar_policy_from_api(
            &test_principal(),
            &mut policies,
            &mut idempotency,
            empty_rules
        ),
        Err(CedarPolicyPublishApiError::Policy(_))
    ));

    request.body.rules[0].action = "tenant.settings.read".to_string();
    let reused =
        publish_cedar_policy_from_api(&test_principal(), &mut policies, &mut idempotency, request)
            .expect_err("same idempotency key with changed body is rejected");
    assert_eq!(
        reused.cedar_policy_publish_status(),
        CedarPolicyPublishApiStatus::UnprocessableEntity
    );
    assert_eq!(idempotency.len(), 1);
}

fn policy_request(
    request_id: &str,
    idempotency_key: &str,
    policy_id: &str,
    version: &str,
) -> CedarPolicyPublishApiRequest {
    CedarPolicyPublishApiRequest {
        path_policy_id: policy_id.to_string(),
        path_version: version.to_string(),
        boundary: CedarPolicyApiBoundaryContext {
            request_id: request_id.to_string(),
            tenant_id: OPERATOR_TENANT_ID.to_string(),
            idempotency_key: idempotency_key.to_string(),
        },
        principal: CedarPolicyApiPrincipal {
            tenant_id: OPERATOR_TENANT_ID.to_string(),
            principal_id: "usr_platform_admin".to_string(),
        },
        authorization: CedarPolicyApiAuthorization {
            tenant_id: OPERATOR_TENANT_ID.to_string(),
            principal_id: "usr_platform_admin".to_string(),
            decision_id: "authz_cedar_policy_publish".to_string(),
            allowed_surfaces: vec![CEDAR_POLICY_PUBLISH_SURFACE.to_string()],
        },
        body: CedarPolicyPublishRequest {
            policy_id: policy_id.to_string(),
            version: version.to_string(),
            scope: CedarPolicyScopeRef {
                kind: "tenant".to_string(),
                tenant_id: Some("ten_alpha".to_string()),
            },
            supersedes: None,
            rules: vec![CedarPolicyRuleRef {
                effect: "allow".to_string(),
                principal_role: "tenant-admin".to_string(),
                action: "tenant.settings.update".to_string(),
                resource_prefix: "tenant:".to_string(),
                required_attribute: Some(CedarPolicyRequiredAttribute {
                    key: "region".to_string(),
                    value: "region-home".to_string(),
                }),
            }],
        },
    }
}
