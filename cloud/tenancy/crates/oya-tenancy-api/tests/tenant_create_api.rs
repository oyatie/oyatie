// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_shared_pdp_kernel::{
    DecisionAuditRecord, EntitySlice, PdpError, PdpOutcome, PolicyDecisionPoint,
};
use oya_shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, PolicyVersion,
};
use oya_tenancy_api::{
    IssueTenantApiKeyRequest, ListTenantEnvironmentsRequest, PROD_DESTRUCTIVE_ACK_HEADER,
    TENANT_API_KEY_ISSUE_ENDPOINT, TENANT_API_KEY_ISSUE_SURFACE, TENANT_CREATE_OPENAPI_CONTRACT,
    TENANT_CREATE_SURFACE, TENANT_ENVIRONMENT_TIERS_OPENAPI_CONTRACT,
    TENANT_ENVIRONMENTS_ENDPOINT_TEMPLATE, TENANT_ENVIRONMENTS_READ_SURFACE,
    TENANT_OUTBOUND_CONFIG_ENDPOINT_TEMPLATE, TENANT_OUTBOUND_CONFIG_UPDATE_SURFACE,
    TenantApiAuthorization, TenantApiBoundaryContext, TenantApiKeyIssuerRole, TenantApiKeyKind,
    TenantApiPrincipal, TenantCreateApiError, TenantCreateApiRequest, TenantCreateApiStatus,
    TenantCreateIdempotencyLedger, TenantCreateRequest, TenantDirectory,
    TenantEnvironmentApiContext, TenantEnvironmentApiError, TenantEnvironmentDirectory,
    TenantEnvironmentTier, TenantOutboundConfig, TenantOutboundMode, TenantRegulatoryPackRef,
    UpdateTenantOutboundConfigRequest, create_tenant_from_api, destructive_operation_acknowledged,
    issue_tenant_api_key_from_api, list_tenant_environments_from_api, parse_tenant_api_key_prefix,
    tenant_api_key_issuer_role_allowed, tenant_api_key_issuer_role_label_allowed,
    tenant_prod_destructive_operation_authorization_request,
    update_tenant_outbound_config_from_api,
};
use std::sync::Mutex;

const REQUEST_ID: &str = "req_tenant_create_001";
const IDEMPOTENCY_KEY: &str = "idem_tenant_create_001";
const OPERATOR_TENANT_ID: &str = "ten_platform";
const TARGET_TENANT_ID: &str = "ten_alpha";

#[test]
fn tenant_create_contract_runtime_constants_are_covered() {
    assert_eq!(TENANT_CREATE_SURFACE, "tenant.create");
    assert_eq!(
        TENANT_CREATE_OPENAPI_CONTRACT,
        "contracts/openapi/platform/platform-tenant-v1.yaml"
    );
    assert_eq!(TenantCreateApiStatus::Created.code(), 201);
    assert_eq!(TenantCreateApiStatus::BadRequest.code(), 400);
    assert_eq!(TenantCreateApiStatus::Unauthorized.code(), 401);
    assert_eq!(TenantCreateApiStatus::Forbidden.code(), 403);
    assert_eq!(TenantCreateApiStatus::Conflict.code(), 409);
    assert_eq!(TenantCreateApiStatus::UnprocessableEntity.code(), 422);
}

#[test]
fn tenant_environment_tier_endpoint_constants_are_covered() {
    assert_eq!(
        TENANT_ENVIRONMENT_TIERS_OPENAPI_CONTRACT,
        "cloud/tenancy/contracts/openapi/tenancy.yaml"
    );
    assert_eq!(TENANT_API_KEY_ISSUE_ENDPOINT, "/v1/tenancy/api-keys");
    assert_eq!(
        TENANT_ENVIRONMENTS_ENDPOINT_TEMPLATE,
        "/v1/tenancy/tenants/{tenant_id}/environments"
    );
    assert_eq!(
        TENANT_OUTBOUND_CONFIG_ENDPOINT_TEMPLATE,
        "/v1/tenancy/tenants/{tenant_id}/environments/{tier}/outbound-config"
    );
    assert_eq!(TENANT_API_KEY_ISSUE_SURFACE, "tenancy.api_key.issue");
    assert_eq!(
        TENANT_ENVIRONMENTS_READ_SURFACE,
        "tenancy.environments.read"
    );
    assert_eq!(
        TENANT_OUTBOUND_CONFIG_UPDATE_SURFACE,
        "tenancy.environment.outbound_config.update"
    );
    assert_eq!(PROD_DESTRUCTIVE_ACK_HEADER, "x-oya-prod-destructive-ack");
}

#[test]
fn tenant_api_key_prefixes_parse_to_environment_tiers_without_crossing_prod() {
    for (sample_key, expected_kind, expected_tier, expected_prefix) in [
        (
            TenantEnvironmentTier::Test.server_key_prefix(),
            TenantApiKeyKind::Server,
            TenantEnvironmentTier::Test,
            "sk_test_",
        ),
        (
            TenantEnvironmentTier::Test.public_key_prefix(),
            TenantApiKeyKind::Public,
            TenantEnvironmentTier::Test,
            "pk_test_",
        ),
        (
            TenantEnvironmentTier::Staging.server_key_prefix(),
            TenantApiKeyKind::Server,
            TenantEnvironmentTier::Staging,
            "sk_stage_",
        ),
        (
            TenantEnvironmentTier::Staging.public_key_prefix(),
            TenantApiKeyKind::Public,
            TenantEnvironmentTier::Staging,
            "pk_stage_",
        ),
        (
            TenantEnvironmentTier::Prod.server_key_prefix(),
            TenantApiKeyKind::Server,
            TenantEnvironmentTier::Prod,
            "sk_live_",
        ),
        (
            TenantEnvironmentTier::Prod.public_key_prefix(),
            TenantApiKeyKind::Public,
            TenantEnvironmentTier::Prod,
            "pk_live_",
        ),
    ] {
        let parsed = parse_tenant_api_key_prefix(sample_key).expect("known prefix parses");
        assert_eq!(parsed.kind, expected_kind);
        assert_eq!(parsed.environment_tier, expected_tier);
        assert_eq!(parsed.prefix, expected_prefix);
        if sample_key.starts_with("sk_test_") || sample_key.starts_with("pk_test_") {
            assert_ne!(parsed.environment_tier, TenantEnvironmentTier::Prod);
        }
    }

    assert!(parse_tenant_api_key_prefix("server_prod_wrong_01H").is_none());
    assert!(parse_tenant_api_key_prefix("pk_staging_wrong_01H").is_none());
    assert!(parse_tenant_api_key_prefix("tenant_01HENV").is_none());
}

#[test]
fn tenant_api_key_issuance_roles_follow_adr_0163_gate_ladder() {
    assert!(tenant_api_key_issuer_role_allowed(
        TenantEnvironmentTier::Test,
        TenantApiKeyIssuerRole::Developer
    ));
    assert!(tenant_api_key_issuer_role_allowed(
        TenantEnvironmentTier::Test,
        TenantApiKeyIssuerRole::Maintainer
    ));
    assert!(!tenant_api_key_issuer_role_allowed(
        TenantEnvironmentTier::Staging,
        TenantApiKeyIssuerRole::Developer
    ));
    assert!(tenant_api_key_issuer_role_allowed(
        TenantEnvironmentTier::Staging,
        TenantApiKeyIssuerRole::Maintainer
    ));
    assert!(!tenant_api_key_issuer_role_allowed(
        TenantEnvironmentTier::Prod,
        TenantApiKeyIssuerRole::Maintainer
    ));
    assert!(tenant_api_key_issuer_role_allowed(
        TenantEnvironmentTier::Prod,
        TenantApiKeyIssuerRole::Admin
    ));
    assert!(!tenant_api_key_issuer_role_allowed(
        TenantEnvironmentTier::Prod,
        TenantApiKeyIssuerRole::Owner
    ));
    assert!(tenant_api_key_issuer_role_allowed(
        TenantEnvironmentTier::Staging,
        TenantApiKeyIssuerRole::Owner
    ));
    assert!(!tenant_api_key_issuer_role_label_allowed(
        TenantEnvironmentTier::Prod,
        "maintainer"
    ));
    assert!(!tenant_api_key_issuer_role_label_allowed(
        TenantEnvironmentTier::Prod,
        "owner"
    ));
    assert!(!tenant_api_key_issuer_role_label_allowed(
        TenantEnvironmentTier::Test,
        "viewer"
    ));
}

#[test]
fn tenant_environment_tiers_define_outbound_modes_and_prod_ack_header() {
    assert_eq!(
        TenantEnvironmentTier::Test.outbound_mode(),
        TenantOutboundMode::Intercept
    );
    assert_eq!(
        TenantEnvironmentTier::Staging.outbound_mode(),
        TenantOutboundMode::TestRecipients
    );
    assert_eq!(
        TenantEnvironmentTier::Prod.outbound_mode(),
        TenantOutboundMode::Live
    );
    assert_eq!(
        TenantEnvironmentTier::Test.audit_chain_tag(),
        "env_tier=test"
    );
    assert_eq!(
        TenantEnvironmentTier::Staging.audit_chain_tag(),
        "env_tier=staging"
    );
    assert_eq!(
        TenantEnvironmentTier::Prod.audit_chain_tag(),
        "env_tier=prod"
    );
    assert!(TenantEnvironmentTier::Test.outbound_config_patch_allowed());
    assert!(TenantEnvironmentTier::Staging.outbound_config_patch_allowed());
    assert!(!TenantEnvironmentTier::Prod.outbound_config_patch_allowed());

    assert!(!destructive_operation_acknowledged(
        TenantEnvironmentTier::Test,
        None
    ));
    assert!(destructive_operation_acknowledged(
        TenantEnvironmentTier::Test,
        Some(" true ")
    ));
    assert!(!destructive_operation_acknowledged(
        TenantEnvironmentTier::Prod,
        None
    ));
    assert!(!destructive_operation_acknowledged(
        TenantEnvironmentTier::Prod,
        Some("false")
    ));
    assert!(destructive_operation_acknowledged(
        TenantEnvironmentTier::Prod,
        Some(" true ")
    ));
}

#[test]
fn tenant_environment_runtime_issues_api_key_through_pdp_without_persisting_secret() {
    let pdp = RecordingPdp::allow();
    let mut directory = TenantEnvironmentDirectory::default();
    directory.register_tenant(TARGET_TENANT_ID);

    let response = issue_tenant_api_key_from_api(
        &mut directory,
        &pdp,
        IssueTenantApiKeyRequest {
            context: tenant_environment_context(TenantApiKeyIssuerRole::Admin),
            tenant_id: TARGET_TENANT_ID.to_string(),
            environment_tier: TenantEnvironmentTier::Prod,
            key_kind: TenantApiKeyKind::Server,
            label: Some("prod server key".to_string()),
            created_at: "2026-07-01T00:00:00Z".to_string(),
        },
    )
    .expect("PDP-allowed prod server key issuance succeeds");

    assert_eq!(response.metadata.tenant_id, TARGET_TENANT_ID);
    assert_eq!(
        response.metadata.environment_tier,
        TenantEnvironmentTier::Prod
    );
    assert_eq!(response.metadata.key_kind, TenantApiKeyKind::Server);
    assert_eq!(response.metadata.prefix, "sk_live_");
    assert_eq!(response.secret_once.as_deref(), Some("sk_live_req_env_001"));
    assert_eq!(directory.api_key_metadata_len(), 1);
    assert_eq!(
        directory
            .api_key_metadata(&response.metadata.api_key_id)
            .expect("safe API-key metadata persisted"),
        &response.metadata
    );

    let requests = pdp.requests.lock().expect("recorded PDP requests");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].action, "issue_api_key");
    assert_eq!(requests[0].tenant_id, TARGET_TENANT_ID);
    assert_eq!(
        requests[0].context.get("env_tier"),
        Some(&serde_json::json!("prod"))
    );
    assert_eq!(
        requests[0].context.get("api_key_prefix"),
        Some(&serde_json::json!("sk_live_"))
    );
    drop(requests);

    let entity_slices = pdp.entity_slices.lock().expect("recorded entity slices");
    let api_key_entities = &entity_slices[0].entities;
    assert!(api_key_entities.iter().any(|record| {
        record.uid.entity_type == "TenantOperator"
            && record.uid.entity_id == "usr_tenant_admin"
            && record.attributes.get("tenant_id") == Some(&serde_json::json!(TARGET_TENANT_ID))
            && record.attributes.get("plan_tier_role") == Some(&serde_json::json!("admin"))
    }));
    assert!(api_key_entities.iter().any(|record| {
        record.uid.entity_type == "TenantEnvironment"
            && record.uid.entity_id == "ten_alpha:prod"
            && record.attributes.get("tenant_id") == Some(&serde_json::json!(TARGET_TENANT_ID))
            && record.attributes.get("env_tier") == Some(&serde_json::json!("prod"))
            && record.attributes.get("api_key_prefix") == Some(&serde_json::json!("sk_live_"))
    }));
}

#[test]
fn tenant_environment_runtime_denies_api_key_issuance_when_pdp_denies() {
    let pdp = RecordingPdp::deny();
    let mut directory = TenantEnvironmentDirectory::default();
    directory.register_tenant(TARGET_TENANT_ID);

    let error = issue_tenant_api_key_from_api(
        &mut directory,
        &pdp,
        IssueTenantApiKeyRequest {
            context: tenant_environment_context(TenantApiKeyIssuerRole::Maintainer),
            tenant_id: TARGET_TENANT_ID.to_string(),
            environment_tier: TenantEnvironmentTier::Prod,
            key_kind: TenantApiKeyKind::Server,
            label: None,
            created_at: "2026-07-01T00:00:00Z".to_string(),
        },
    )
    .expect_err("PDP deny blocks prod server key issuance");

    assert_eq!(error, TenantEnvironmentApiError::AuthorizationDenied);
    assert_eq!(directory.api_key_metadata_len(), 0);
}

#[test]
fn tenant_environment_runtime_fails_closed_on_pdp_errors_without_mutation() {
    let pdp = RecordingPdp::error();
    let mut directory = TenantEnvironmentDirectory::default();
    directory.register_tenant(TARGET_TENANT_ID);

    let api_key_error = issue_tenant_api_key_from_api(
        &mut directory,
        &pdp,
        IssueTenantApiKeyRequest {
            context: tenant_environment_context(TenantApiKeyIssuerRole::Admin),
            tenant_id: TARGET_TENANT_ID.to_string(),
            environment_tier: TenantEnvironmentTier::Prod,
            key_kind: TenantApiKeyKind::Server,
            label: None,
            created_at: "2026-07-01T00:00:00Z".to_string(),
        },
    )
    .expect_err("PDP errors fail closed before API-key metadata is persisted");
    assert!(matches!(api_key_error, TenantEnvironmentApiError::Pdp(_)));
    assert_eq!(directory.api_key_metadata_len(), 0);

    let outbound_error = update_tenant_outbound_config_from_api(
        &mut directory,
        &pdp,
        UpdateTenantOutboundConfigRequest {
            context: tenant_environment_context(TenantApiKeyIssuerRole::Owner),
            path_tenant_id: TARGET_TENANT_ID.to_string(),
            environment_tier: TenantEnvironmentTier::Staging,
            outbound_config: TenantOutboundConfig {
                mode: TenantOutboundMode::TestRecipients,
                test_recipient_allowlist: vec!["qa@example.com".to_string()],
                intercept_sink: None,
            },
        },
    )
    .expect_err("PDP errors fail closed before outbound config mutation");
    assert!(matches!(outbound_error, TenantEnvironmentApiError::Pdp(_)));

    let list = list_tenant_environments_from_api(
        &directory,
        &RecordingPdp::allow(),
        ListTenantEnvironmentsRequest {
            context: tenant_environment_context(TenantApiKeyIssuerRole::Admin),
            path_tenant_id: TARGET_TENANT_ID.to_string(),
        },
    )
    .expect("default environment records remain readable after failed mutation");
    let staging = list
        .environments
        .iter()
        .find(|environment| environment.environment_tier == TenantEnvironmentTier::Staging)
        .expect("staging environment is present");
    assert!(staging.outbound_config.test_recipient_allowlist.is_empty());
}

#[test]
fn tenant_environment_runtime_lists_tiers_and_patches_only_non_prod_outbound_config() {
    let pdp = RecordingPdp::allow();
    let mut directory = TenantEnvironmentDirectory::default();
    directory.register_tenant(TARGET_TENANT_ID);

    let list = list_tenant_environments_from_api(
        &directory,
        &pdp,
        ListTenantEnvironmentsRequest {
            context: tenant_environment_context(TenantApiKeyIssuerRole::Admin),
            path_tenant_id: TARGET_TENANT_ID.to_string(),
        },
    )
    .expect("PDP-allowed environment listing succeeds");
    assert_eq!(list.tenant_id, TARGET_TENANT_ID);
    assert_eq!(list.environments.len(), 3);
    assert!(
        list.environments
            .iter()
            .any(|environment| environment.environment_tier == TenantEnvironmentTier::Test)
    );
    assert!(
        list.environments
            .iter()
            .any(|environment| environment.environment_tier == TenantEnvironmentTier::Staging)
    );
    assert!(
        list.environments
            .iter()
            .any(|environment| environment.environment_tier == TenantEnvironmentTier::Prod)
    );

    let updated = update_tenant_outbound_config_from_api(
        &mut directory,
        &pdp,
        UpdateTenantOutboundConfigRequest {
            context: tenant_environment_context(TenantApiKeyIssuerRole::Owner),
            path_tenant_id: TARGET_TENANT_ID.to_string(),
            environment_tier: TenantEnvironmentTier::Staging,
            outbound_config: TenantOutboundConfig {
                mode: TenantOutboundMode::TestRecipients,
                test_recipient_allowlist: vec!["qa@example.com".to_string()],
                intercept_sink: None,
            },
        },
    )
    .expect("staging outbound config is patchable through PDP");
    assert_eq!(
        updated.outbound_config.test_recipient_allowlist,
        vec!["qa@example.com"]
    );

    let prod_error = update_tenant_outbound_config_from_api(
        &mut directory,
        &pdp,
        UpdateTenantOutboundConfigRequest {
            context: tenant_environment_context(TenantApiKeyIssuerRole::Admin),
            path_tenant_id: TARGET_TENANT_ID.to_string(),
            environment_tier: TenantEnvironmentTier::Prod,
            outbound_config: TenantOutboundConfig::default_for_tier(TenantEnvironmentTier::Prod),
        },
    )
    .expect_err("prod live outbound config is immutable through this endpoint");
    assert_eq!(
        prod_error,
        TenantEnvironmentApiError::ProdOutboundConfigImmutable
    );
}

#[test]
fn prod_destructive_ack_header_projects_only_to_cedar_context() {
    let mut context = tenant_environment_context(TenantApiKeyIssuerRole::Admin);
    let without_header = tenant_prod_destructive_operation_authorization_request(
        &context,
        TARGET_TENANT_ID,
        "tenant-offboarding",
        TenantEnvironmentTier::Prod,
    )
    .expect("prod destructive authorization projection builds");
    assert_eq!(
        without_header.context.get("prod_destructive_acknowledged"),
        Some(&serde_json::json!(false))
    );

    let non_prod_without_header = tenant_prod_destructive_operation_authorization_request(
        &context,
        TARGET_TENANT_ID,
        "tenant-offboarding",
        TenantEnvironmentTier::Test,
    )
    .expect("non-prod destructive authorization projection builds");
    assert_eq!(
        non_prod_without_header
            .context
            .get("prod_destructive_acknowledged"),
        Some(&serde_json::json!(false))
    );

    context.prod_destructive_ack_header = Some(" true ".to_string());
    let with_header = tenant_prod_destructive_operation_authorization_request(
        &context,
        TARGET_TENANT_ID,
        "tenant-offboarding",
        TenantEnvironmentTier::Prod,
    )
    .expect("prod destructive authorization projection builds");
    assert_eq!(
        with_header.context.get("prod_destructive_acknowledged"),
        Some(&serde_json::json!(true))
    );
}

#[test]
fn tenant_create_creates_once_and_replays_same_idempotent_result() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let request = tenant_request(REQUEST_ID, IDEMPOTENCY_KEY, TARGET_TENANT_ID);

    let first = create_tenant_from_api(&mut directory, &mut idempotency, request.clone())
        .expect("first tenant creation succeeds");
    let second = create_tenant_from_api(&mut directory, &mut idempotency, request)
        .expect("same tenant creation request replays");

    assert_eq!(first, second);
    assert_eq!(directory.len(), 1);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(first.data.tenant_id, TARGET_TENANT_ID);
    assert_eq!(first.data.legal_name, "Alpha Tenant Ltd");
    assert_eq!(first.data.home_region, "region-home");
    assert_eq!(first.data.residency_class, "strict_home_region");
    assert_eq!(first.data.regulatory_packs[0].value, "oya-pack-alpha");
    assert_eq!(first.data.schema_version, 1);
    assert_eq!(first.metadata.request_id, REQUEST_ID);
    assert!(directory.get(TARGET_TENANT_ID).is_some());
}

#[test]
fn tenant_create_rejects_path_body_drift_before_directory_mutation() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let mut request = tenant_request("req_tenant_drift", "idem_tenant_drift", TARGET_TENANT_ID);
    request.body.tenant_id = "ten_other".to_string();

    let error = create_tenant_from_api(&mut directory, &mut idempotency, request)
        .expect_err("path/body tenant drift is rejected");

    assert!(matches!(
        error,
        TenantCreateApiError::TenantPathBodyMismatch { .. }
    ));
    assert_eq!(error.tenant_create_status_code(), 400);
    assert!(directory.is_empty());
    assert!(idempotency.is_empty());
}

#[test]
fn tenant_create_separates_missing_principal_from_denied_authorization() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let mut unauthenticated =
        tenant_request("req_tenant_authn", "idem_tenant_authn", TARGET_TENANT_ID);
    unauthenticated.principal.principal_id.clear();

    let authn_error = create_tenant_from_api(&mut directory, &mut idempotency, unauthenticated)
        .expect_err("missing principal is authentication failure");
    assert_eq!(
        authn_error.tenant_create_status(),
        TenantCreateApiStatus::Unauthorized
    );

    let mut denied = tenant_request("req_tenant_authz", "idem_tenant_authz", TARGET_TENANT_ID);
    denied.authorization.allowed_surfaces = vec!["identity.token.issue".to_string()];
    let authz_error = create_tenant_from_api(&mut directory, &mut idempotency, denied)
        .expect_err("missing tenant.create grant is authorization failure");
    assert!(matches!(
        authz_error,
        TenantCreateApiError::AuthorizationDenied { ref surface }
            if surface == TENANT_CREATE_SURFACE
    ));
    assert_eq!(
        authz_error.tenant_create_status(),
        TenantCreateApiStatus::Forbidden
    );
    assert!(directory.is_empty());
    assert!(idempotency.is_empty());
}

#[test]
fn tenant_create_maps_duplicate_invalid_residency_and_kernel_errors() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        tenant_request("req_tenant_first", "idem_tenant_first", TARGET_TENANT_ID),
    )
    .expect("initial tenant creation succeeds");

    let duplicate = create_tenant_from_api(
        &mut directory,
        &mut idempotency,
        tenant_request(
            "req_tenant_duplicate",
            "idem_tenant_duplicate",
            TARGET_TENANT_ID,
        ),
    )
    .expect_err("duplicate tenant id conflicts");
    assert!(matches!(
        duplicate,
        TenantCreateApiError::DuplicateTenant { .. }
    ));
    assert_eq!(
        duplicate.tenant_create_status(),
        TenantCreateApiStatus::Conflict
    );

    let mut invalid_residency = tenant_request(
        "req_tenant_bad_residency",
        "idem_tenant_bad_residency",
        "ten_bad_residency",
    );
    invalid_residency.body.residency_class = "moon_base".to_string();
    assert!(matches!(
        create_tenant_from_api(&mut directory, &mut idempotency, invalid_residency),
        Err(TenantCreateApiError::InvalidResidencyClass { .. })
    ));

    let mut bad_home_region = tenant_request(
        "req_tenant_bad_region",
        "idem_tenant_bad_region",
        "ten_bad_region",
    );
    bad_home_region.body.home_region = "region-recovery".to_string();
    assert!(matches!(
        create_tenant_from_api(&mut directory, &mut idempotency, bad_home_region),
        Err(TenantCreateApiError::Tenant(_))
    ));
    assert_eq!(directory.len(), 1);
}

#[test]
fn tenant_create_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut directory = TenantDirectory::default();
    let mut idempotency = TenantCreateIdempotencyLedger::default();
    let mut request = tenant_request("req_tenant_reused", "idem_tenant_reused", "ten_reused");

    create_tenant_from_api(&mut directory, &mut idempotency, request.clone())
        .expect("first idempotent tenant creation succeeds");

    request.body.legal_name = "Changed Tenant Ltd".to_string();
    let error = create_tenant_from_api(&mut directory, &mut idempotency, request)
        .expect_err("same idempotency key with changed body is rejected");

    assert_eq!(
        error,
        TenantCreateApiError::IdempotencyKeyReused {
            idempotency_key: "idem_tenant_reused".to_string()
        }
    );
    assert_eq!(
        error.tenant_create_status(),
        TenantCreateApiStatus::UnprocessableEntity
    );
    assert_eq!(directory.len(), 1);
    assert_eq!(idempotency.len(), 1);
}

fn tenant_request(
    request_id: &str,
    idempotency_key: &str,
    tenant_id: &str,
) -> TenantCreateApiRequest {
    TenantCreateApiRequest {
        path_tenant_id: tenant_id.to_string(),
        boundary: TenantApiBoundaryContext {
            request_id: request_id.to_string(),
            tenant_id: OPERATOR_TENANT_ID.to_string(),
            idempotency_key: idempotency_key.to_string(),
        },
        principal: TenantApiPrincipal {
            tenant_id: OPERATOR_TENANT_ID.to_string(),
            principal_id: "usr_platform_admin".to_string(),
        },
        authorization: TenantApiAuthorization {
            tenant_id: OPERATOR_TENANT_ID.to_string(),
            principal_id: "usr_platform_admin".to_string(),
            decision_id: "authz_tenant_create".to_string(),
            allowed_surfaces: vec![TENANT_CREATE_SURFACE.to_string()],
        },
        body: TenantCreateRequest {
            tenant_id: tenant_id.to_string(),
            legal_name: "Alpha Tenant Ltd".to_string(),
            home_region: "region-home".to_string(),
            residency_class: "strict_home_region".to_string(),
            regulatory_packs: vec![TenantRegulatoryPackRef {
                value: "oya-pack-alpha".to_string(),
            }],
        },
    }
}

fn tenant_environment_context(role: TenantApiKeyIssuerRole) -> TenantEnvironmentApiContext {
    TenantEnvironmentApiContext {
        request_id: "req_env_001".to_string(),
        principal: TenantApiPrincipal {
            tenant_id: TARGET_TENANT_ID.to_string(),
            principal_id: "usr_tenant_admin".to_string(),
        },
        plan_tier_role: role,
        prod_destructive_ack_header: None,
    }
}

struct RecordingPdp {
    decision: Decision,
    error: Option<PdpError>,
    requests: Mutex<Vec<AuthorizationRequest>>,
    entity_slices: Mutex<Vec<EntitySlice>>,
}

impl RecordingPdp {
    fn allow() -> Self {
        Self::new(Decision::Allow)
    }

    fn deny() -> Self {
        Self::new(Decision::Deny)
    }

    fn error() -> Self {
        Self {
            decision: Decision::Deny,
            error: Some(PdpError::Evaluation {
                detail: "tenant-scope bundle unavailable".to_string(),
            }),
            requests: Mutex::new(Vec::new()),
            entity_slices: Mutex::new(Vec::new()),
        }
    }

    fn new(decision: Decision) -> Self {
        Self {
            decision,
            error: None,
            requests: Mutex::new(Vec::new()),
            entity_slices: Mutex::new(Vec::new()),
        }
    }
}

impl PolicyDecisionPoint for RecordingPdp {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        if let Some(error) = self.error.clone() {
            return Err(error);
        }
        self.requests
            .lock()
            .expect("request recorder lock")
            .push(request.clone());
        self.entity_slices
            .lock()
            .expect("entity recorder lock")
            .push(entities.clone());
        let determining_policy_ids = if self.decision == Decision::Allow {
            vec!["tenant-scope.cedar".to_string()]
        } else {
            Vec::new()
        };
        let response = AuthorizationResponse {
            decision_id: format!("dec_{}", request.action),
            request_id: request.request_id.clone(),
            decision: self.decision,
            policy_version: PolicyVersion::new("psv-tenant-env-test")
                .expect("valid policy version"),
            determining_policy_ids: determining_policy_ids.clone(),
            obligations: Vec::new(),
        };
        Ok(PdpOutcome {
            response: response.clone(),
            audit: DecisionAuditRecord {
                decision_id: response.decision_id,
                request_id: response.request_id,
                tenant_id: request.tenant_id.clone(),
                principal: request.principal.clone(),
                action: request.action.clone(),
                resource: request.resource.clone(),
                decision: self.decision,
                policy_version: response.policy_version,
                determining_policy_ids,
                cache_hit: false,
            },
            cache_hit: false,
        })
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-tenant-env-test").expect("valid policy version")
    }
}
