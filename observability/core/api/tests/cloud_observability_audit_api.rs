// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use audit_chain_domain::{AuditChain, Plane};
use network_residency::ResidencyClass;
use observability_aggregate::{
    CloudAuditEnvelopeCreate, CloudAuditOperation, CloudAuditTopic, CloudObservabilityCatalog,
    ObservabilityResidency, ObservabilityResidencyCreate, ObservabilityResidencyState,
};
use observability_api::{
    AuditReadAction, AuditReadAuthorizationError, AuditReadAuthorizer, AuditReadResource,
    CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE, CallerCredential, CloudObservabilityApiAuthorization,
    CloudObservabilityApiBoundaryContext, CloudObservabilityApiError,
    CloudObservabilityApiPrincipal, CloudObservabilityAuditReadApiRequest,
    CloudObservabilityAuditReadApiStatus, CloudObservabilityAuditReadRequest,
    CloudObservabilityAuditReadTopicRef, ConfiguredBearerPrincipalVerifier, PrincipalVerifier,
    VerifiedPrincipal, read_cloud_observability_audit_from_api,
};
use data_boundary_kernel::{DataClass, Purpose};

const TENANT: &str = "ten_alpha";
const OTHER_TENANT: &str = "ten_other";
const REGION: &str = "region-home";
const CELL: &str = "cell-region-home-a-001";
const SIGNED_EXPORT: &str = "s3+signed://region-home/ten_alpha/audit?sig=abc123";
const RESOURCE_ID: &str = "oya:cloud:region-home:ten_alpha:instance:vm-a";
const HASH_A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BEARER_SECRET: &str = "obs-audit-reader-break-glass-secret";
const PRINCIPAL_ID: &str = "sp_audit_reader";

// ---------------------------------------------------------------------------
// Authz test doubles. The ONLY way an external crate obtains a VerifiedPrincipal
// is by running a real PrincipalVerifier — proving the token is unforgeable (no
// public constructor / no public fields).
// ---------------------------------------------------------------------------

fn verifier(principal_id: &str, tenant_id: &str) -> ConfiguredBearerPrincipalVerifier {
    ConfiguredBearerPrincipalVerifier::new(BEARER_SECRET, principal_id, tenant_id)
        .expect("non-empty bearer secret and bound identity")
}

fn verified(principal_id: &str, tenant_id: &str) -> VerifiedPrincipal {
    verifier(principal_id, tenant_id)
        .verify_principal(&CallerCredential {
            authorization: Some(format!("Bearer {BEARER_SECRET}")),
            claimed_principal_id: principal_id.to_string(),
            claimed_tenant_id: tenant_id.to_string(),
        })
        .expect("valid bearer mints a verified principal")
}

/// An authorizer that ALLOWS everything. Used to prove that the OTHER gates
/// (tenant binding, cross-check, coarse-scope action) still deny — i.e. the
/// blast-radius binding is not the only thing keeping cross-tenant out.
struct AllowAllAuthorizer;
impl AuditReadAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &AuditReadResource,
    ) -> Result<(), AuditReadAuthorizationError> {
        Ok(())
    }
}

/// An authorizer that allows ONLY the control-plane action for a specific tenant
/// and DENIES the all-tenant action — modelling a least-privilege grant. Proves
/// the coarse-scope fix: a control-plane grant does NOT auto-confer all-tenant.
struct ControlPlaneOnlyAuthorizer {
    tenant_id: String,
}
impl AuditReadAuthorizer for ControlPlaneOnlyAuthorizer {
    fn ensure_authorized(
        &self,
        principal: &VerifiedPrincipal,
        resource: &AuditReadResource,
    ) -> Result<(), AuditReadAuthorizationError> {
        if principal.tenant_id() == self.tenant_id
            && resource.tenant_id == self.tenant_id
            && resource.action == AuditReadAction::ControlPlaneAuditRead
        {
            Ok(())
        } else {
            Err(AuditReadAuthorizationError::Denied)
        }
    }
}

/// An authorizer that always denies (models an explicit PDP deny).
struct DenyAllAuthorizer;
impl AuditReadAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &AuditReadResource,
    ) -> Result<(), AuditReadAuthorizationError> {
        Err(AuditReadAuthorizationError::Denied)
    }
}

/// An authorizer that refuses (models a PDP fault / unavailability). Fail-closed:
/// a refusal must map to 403 exactly like a deny.
struct RefuseAllAuthorizer;
impl AuditReadAuthorizer for RefuseAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &AuditReadResource,
    ) -> Result<(), AuditReadAuthorizationError> {
        Err(AuditReadAuthorizationError::Refused)
    }
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(
        CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE,
        "cloud.observability.audit.read"
    );
    assert_eq!(CloudObservabilityAuditReadApiStatus::Ok.code(), 200);
    assert_eq!(CloudObservabilityAuditReadApiStatus::BadRequest.code(), 400);
    assert_eq!(
        CloudObservabilityAuditReadApiStatus::Unauthorized.code(),
        401
    );
    assert_eq!(CloudObservabilityAuditReadApiStatus::Forbidden.code(), 403);
    assert_eq!(
        CloudObservabilityAuditReadApiStatus::UnprocessableEntity.code(),
        422
    );
    assert_eq!(
        AuditReadAction::ControlPlaneAuditRead.as_str(),
        "cloud.observability.audit.read.control_plane"
    );
    assert_eq!(
        AuditReadAction::AllTenantAuditRead.as_str(),
        "cloud.observability.audit.read.all_tenant"
    );
}

fn residency() -> ObservabilityResidency {
    ObservabilityResidency::new(ObservabilityResidencyCreate {
        tenant_id: TENANT.to_string(),
        region: REGION.to_string(),
        regional_pack: "pack-alpha".to_string(),
        residency: ResidencyClass::StrictHomeRegion,
        metric_storage_region: REGION.to_string(),
        log_storage_region: REGION.to_string(),
        trace_storage_region: REGION.to_string(),
        audit_storage_region: REGION.to_string(),
        signed_audit_export_uri: SIGNED_EXPORT.to_string(),
        retention_days: 2555,
        state: ObservabilityResidencyState::Enforcing,
    })
    .expect("valid residency")
}

fn chain() -> AuditChain {
    let mut chain = AuditChain::default();
    chain
        .append_classifications(
            TENANT,
            CloudAuditTopic::CloudResourceCreated.as_str(),
            Plane::Control,
            Purpose::CoreService,
            [DataClass::InternalOnly, DataClass::Public, DataClass::Audit],
            "ALLOW",
        )
        .unwrap();
    chain
        .append_classifications(
            TENANT,
            CloudAuditTopic::CloudIamPolicy.as_str(),
            Plane::Control,
            Purpose::CoreService,
            [DataClass::InternalOnly, DataClass::Audit],
            "ALLOW",
        )
        .unwrap();
    chain
        .append_classifications(
            TENANT,
            CloudAuditTopic::CloudKmsUse.as_str(),
            Plane::Data,
            Purpose::CoreService,
            [DataClass::InternalOnly, DataClass::Audit],
            "ALLOW",
        )
        .unwrap();
    assert!(chain.verify());
    chain
}

fn envelopes() -> Vec<CloudAuditEnvelopeCreate> {
    vec![
        CloudAuditEnvelopeCreate {
            event_sequence: 0,
            topic: CloudAuditTopic::CloudResourceCreated,
            operation: CloudAuditOperation::ResourceCreated,
            region: REGION.to_string(),
            cell_id: Some(CELL.to_string()),
            resource_id: Some(RESOURCE_ID.to_string()),
            actor: "usr_admin".to_string(),
            iam_role: Some("role_cloud_admin".to_string()),
            occurred_at_epoch_seconds: 1_000,
            payload_hash: HASH_A.to_string(),
            idempotency_key: "idem/create-vm-a".to_string(),
            signed_export_uri: SIGNED_EXPORT.to_string(),
        },
        CloudAuditEnvelopeCreate {
            event_sequence: 1,
            topic: CloudAuditTopic::CloudIamPolicy,
            operation: CloudAuditOperation::IamPolicyChanged,
            region: REGION.to_string(),
            cell_id: Some(CELL.to_string()),
            resource_id: None,
            actor: "sp_foundry".to_string(),
            iam_role: Some("role_cloud_admin".to_string()),
            occurred_at_epoch_seconds: 1_010,
            payload_hash: HASH_B.to_string(),
            idempotency_key: "idem/iam-policy".to_string(),
            signed_export_uri: SIGNED_EXPORT.to_string(),
        },
        CloudAuditEnvelopeCreate {
            event_sequence: 2,
            topic: CloudAuditTopic::CloudKmsUse,
            operation: CloudAuditOperation::KmsKeyUsed,
            region: REGION.to_string(),
            cell_id: Some(CELL.to_string()),
            resource_id: None,
            actor: "role_cloud_admin".to_string(),
            iam_role: Some("role_cloud_admin".to_string()),
            occurred_at_epoch_seconds: 1_020,
            payload_hash: HASH_A.to_string(),
            idempotency_key: "idem/kms-use".to_string(),
            signed_export_uri: SIGNED_EXPORT.to_string(),
        },
    ]
}

fn catalog() -> CloudObservabilityCatalog {
    let mut catalog = CloudObservabilityCatalog::default();
    catalog
        .ingest_verified_chain(&chain(), envelopes(), &residency())
        .expect("audit fixture ingests");
    catalog
}

fn boundary() -> CloudObservabilityApiBoundaryContext {
    CloudObservabilityApiBoundaryContext {
        request_id: "req-observability-1".to_string(),
        tenant_id: TENANT.to_string(),
    }
}

fn principal(principal_id: &str) -> CloudObservabilityApiPrincipal {
    CloudObservabilityApiPrincipal {
        tenant_id: TENANT.to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn correlation(principal_id: &str) -> CloudObservabilityApiAuthorization {
    CloudObservabilityApiAuthorization {
        tenant_id: TENANT.to_string(),
        principal_id: principal_id.to_string(),
        correlation_id: format!("trace_{principal_id}"),
    }
}

fn body() -> CloudObservabilityAuditReadRequest {
    CloudObservabilityAuditReadRequest {
        tenant_id: TENANT.to_string(),
        region: REGION.to_string(),
        cell_id: Some(CELL.to_string()),
        scope: "control_plane_mutations".to_string(),
        start_epoch_seconds: 900,
        end_epoch_seconds: 1_100,
        topics: Vec::new(),
        actor: None,
        resource_id: None,
        cursor: None,
        page_size: Some(1),
        require_complete_chain: true,
    }
}

fn request() -> CloudObservabilityAuditReadApiRequest {
    CloudObservabilityAuditReadApiRequest {
        boundary: boundary(),
        principal: Some(principal(PRINCIPAL_ID)),
        authorization: correlation(PRINCIPAL_ID),
        body: body(),
    }
}

#[test]
fn audit_read_api_projects_first_page_and_cursor_metadata() {
    let response = read_cloud_observability_audit_from_api(
        &verified(PRINCIPAL_ID, TENANT),
        &AllowAllAuthorizer,
        &catalog(),
        request(),
    )
    .expect("authorized audit read succeeds");

    assert_eq!(response.metadata.request_id, "req-observability-1");
    assert_eq!(response.metadata.tenant_id, TENANT);
    assert_eq!(response.metadata.region, REGION);
    assert_eq!(response.metadata.record_count, 1);
    assert!(response.metadata.next_cursor.is_some());
    assert!(response.metadata.chain_complete);
    assert_eq!(response.metadata.high_watermark_sequence, Some(2));
    assert_eq!(response.data[0].operation, "resource_created");
    assert_eq!(response.data[0].topic, "oya.audit.cloud_resource_created");
    assert_eq!(response.data[0].record_class, "control_plane_mutation");
    assert_eq!(response.data[0].audit_marker, "AUDIT");
    assert_eq!(response.data[0].data_classes_referenced[2].label, "AUDIT");
}

#[test]
fn audit_read_api_uses_cursor_for_second_page() {
    let verified = verified(PRINCIPAL_ID, TENANT);
    let first = read_cloud_observability_audit_from_api(
        &verified,
        &AllowAllAuthorizer,
        &catalog(),
        request(),
    )
    .expect("first page succeeds");
    let mut second_request = request();
    second_request.body.cursor = first.metadata.next_cursor;
    second_request.body.page_size = Some(10);

    let second = read_cloud_observability_audit_from_api(
        &verified,
        &AllowAllAuthorizer,
        &catalog(),
        second_request,
    )
    .expect("second page succeeds");

    assert_eq!(second.data.len(), 1);
    assert_eq!(second.data[0].operation, "iam_policy_changed");
    assert_eq!(second.metadata.next_cursor, None);
}

// === RED/GREEN: the fail-closed seam (these MUST fail if the gate is removed) ===

#[test]
fn audit_read_api_forged_or_absent_credential_is_unauthorized() {
    // An absent credential never mints a VerifiedPrincipal: the verifier refuses
    // at the 401 boundary BEFORE the boundary fn is reachable. We assert the
    // verifier — the only producer of a VerifiedPrincipal — refuses.
    let absent = verifier(PRINCIPAL_ID, TENANT).verify_principal(&CallerCredential {
        authorization: None,
        claimed_principal_id: PRINCIPAL_ID.to_string(),
        claimed_tenant_id: TENANT.to_string(),
    });
    assert!(absent.is_err(), "absent credential must not verify (401)");

    // A forged bearer (wrong secret) likewise never verifies — constant-time
    // compare, never `==`.
    let forged = verifier(PRINCIPAL_ID, TENANT).verify_principal(&CallerCredential {
        authorization: Some("Bearer not-the-secret".to_string()),
        claimed_principal_id: PRINCIPAL_ID.to_string(),
        claimed_tenant_id: TENANT.to_string(),
    });
    assert!(forged.is_err(), "forged bearer must not verify (401)");

    // A caller cannot forge a VerifiedPrincipal by struct literal: the type has
    // private fields and no public constructor. (Compile-time guarantee — this
    // test documents the property; an attempted `VerifiedPrincipal { .. }` would
    // not compile from this external crate.)
}

#[test]
fn audit_read_api_self_grant_via_dto_does_not_authorize() {
    // The PRE-FIX self-grant: a caller populating "authorization" surfaces no
    // longer authorizes anything. With a DENY authorizer the read is 403
    // regardless of what the caller put in the DTO — proving the DTO is inert.
    let mut self_granting = request();
    // The DTO no longer carries allowed_surfaces; even a fully-consistent
    // correlation cannot grant.
    self_granting.authorization = correlation(PRINCIPAL_ID);
    let denied = read_cloud_observability_audit_from_api(
        &verified(PRINCIPAL_ID, TENANT),
        &DenyAllAuthorizer,
        &catalog(),
        self_granting,
    )
    .expect_err("caller-supplied authorization must not self-grant");
    assert_eq!(denied.status_code(), 403);
    assert!(matches!(
        denied,
        CloudObservabilityApiError::AuthorizationDenied { .. }
    ));
}

#[test]
fn audit_read_api_pdp_deny_and_refuse_both_forbid() {
    let deny = read_cloud_observability_audit_from_api(
        &verified(PRINCIPAL_ID, TENANT),
        &DenyAllAuthorizer,
        &catalog(),
        request(),
    )
    .expect_err("PDP deny must forbid");
    assert_eq!(deny.status_code(), 403);

    // Fail-closed: a PDP fault/refusal is treated as deny, also 403.
    let refuse = read_cloud_observability_audit_from_api(
        &verified(PRINCIPAL_ID, TENANT),
        &RefuseAllAuthorizer,
        &catalog(),
        request(),
    )
    .expect_err("PDP refuse must forbid (fail-closed)");
    assert_eq!(refuse.status_code(), 403);
}

#[test]
fn audit_read_api_cross_tenant_is_forbidden_even_with_permissive_authorizer() {
    // The verified principal belongs to OTHER_TENANT but tries to read ten_alpha
    // audit (body/header tenant = ten_alpha). Even with an allow-all authorizer,
    // the tenant binding rejects it 403 — the served data is scoped to the
    // VERIFIED tenant, never the caller-supplied target (no IDOR).
    let attacker = verified(PRINCIPAL_ID, OTHER_TENANT);
    // The request body/header/DTO all target ten_alpha. The attacker's verified
    // tenant is ten_other. The boundary rejects 403 — either at the DTO
    // cross-check (the self-attested ten_alpha tenant != verified ten_other) or
    // at the body tenant binding; both are correct blast-radius denials and both
    // are 403. A removed gate would instead serve ten_alpha audit.
    let forbidden = read_cloud_observability_audit_from_api(
        &attacker,
        &AllowAllAuthorizer,
        &catalog(),
        request(),
    )
    .expect_err("cross-tenant read must be forbidden");
    assert_eq!(forbidden.status_code(), 403);
    assert!(matches!(
        forbidden,
        CloudObservabilityApiError::TenantMismatch { .. }
            | CloudObservabilityApiError::AuthorizationTenantMismatch { .. }
    ));

    // Drop the self-attested DTO entirely so the ONLY tenant signal is the body
    // tenant (ten_alpha) vs the verified tenant (ten_other): the body tenant
    // binding fires TenantMismatch.
    let mut no_dto = request();
    no_dto.principal = None;
    no_dto.authorization = CloudObservabilityApiAuthorization::default();
    let forbidden_binding =
        read_cloud_observability_audit_from_api(&attacker, &AllowAllAuthorizer, &catalog(), no_dto)
            .expect_err("body-tenant binding must forbid cross-tenant read");
    assert_eq!(forbidden_binding.status_code(), 403);
    assert!(matches!(
        forbidden_binding,
        CloudObservabilityApiError::TenantMismatch { .. }
    ));

    // And the PDP also sees the TARGET tenant from the verified principal, so a
    // tenant-bound authorizer denies a cross-tenant attempt at the PDP too.
    let pdp_for_alpha = ControlPlaneOnlyAuthorizer {
        tenant_id: TENANT.to_string(),
    };
    let mut as_other = request();
    as_other.boundary.tenant_id = OTHER_TENANT.to_string();
    as_other.body.tenant_id = OTHER_TENANT.to_string();
    as_other.principal = Some(CloudObservabilityApiPrincipal {
        tenant_id: OTHER_TENANT.to_string(),
        principal_id: PRINCIPAL_ID.to_string(),
    });
    as_other.authorization = CloudObservabilityApiAuthorization {
        tenant_id: OTHER_TENANT.to_string(),
        principal_id: PRINCIPAL_ID.to_string(),
        correlation_id: "trace".to_string(),
    };
    let denied_at_pdp =
        read_cloud_observability_audit_from_api(&attacker, &pdp_for_alpha, &catalog(), as_other)
            .expect_err("PDP bound to ten_alpha denies ten_other");
    assert_eq!(denied_at_pdp.status_code(), 403);
    assert!(matches!(
        denied_at_pdp,
        CloudObservabilityApiError::AuthorizationDenied { .. }
    ));
}

#[test]
fn audit_read_api_control_plane_grant_cannot_read_all_tenant_audit() {
    // The coarse-scope fix: a principal granted ONLY control-plane reads cannot
    // escalate to the all_tenant_audit scope (data-plane security, KMS use,
    // billing). The two scopes map to DISTINCT PDP actions.
    let pdp = ControlPlaneOnlyAuthorizer {
        tenant_id: TENANT.to_string(),
    };
    let verified = verified(PRINCIPAL_ID, TENANT);

    // Control-plane read is allowed.
    let ok = read_cloud_observability_audit_from_api(&verified, &pdp, &catalog(), request())
        .expect("control-plane grant reads control-plane audit");
    assert_eq!(ok.metadata.tenant_id, TENANT);

    // The SAME principal/grant requesting all_tenant_audit is DENIED 403.
    let mut all = request();
    all.body.scope = "all_tenant_audit".to_string();
    all.body.actor = Some("usr_admin".to_string());
    all.body.resource_id = Some(RESOURCE_ID.to_string());
    all.body.page_size = Some(100);
    let denied = read_cloud_observability_audit_from_api(&verified, &pdp, &catalog(), all)
        .expect_err("control-plane grant must NOT confer all-tenant audit");
    assert_eq!(denied.status_code(), 403);
    assert!(matches!(
        denied,
        CloudObservabilityApiError::AuthorizationDenied { .. }
    ));
}

#[test]
fn audit_read_api_substituted_dto_principal_is_rejected() {
    // A caller who substitutes a different principal id in the DTO than the
    // verified identity is rejected — the DTO never overrides the credential.
    let mut substituted = request();
    substituted.principal = Some(CloudObservabilityApiPrincipal {
        tenant_id: TENANT.to_string(),
        principal_id: "sp_someone_else".to_string(),
    });
    let err = read_cloud_observability_audit_from_api(
        &verified(PRINCIPAL_ID, TENANT),
        &AllowAllAuthorizer,
        &catalog(),
        substituted,
    )
    .expect_err("substituted DTO principal must be rejected");
    assert_eq!(err.status_code(), 403);
    assert!(matches!(
        err,
        CloudObservabilityApiError::AuthorizationPrincipalMismatch { .. }
    ));
}

#[test]
fn audit_read_api_rejects_required_header_and_tenant_drift_before_catalog_read() {
    let verified = verified(PRINCIPAL_ID, TENANT);
    let mut empty_header = request();
    empty_header.boundary.tenant_id.clear();
    let err = read_cloud_observability_audit_from_api(
        &verified,
        &AllowAllAuthorizer,
        &catalog(),
        empty_header,
    )
    .expect_err("tenant header is required");
    assert_eq!(err.status_code(), 400);
    assert!(matches!(err, CloudObservabilityApiError::EmptyTenantHeader));

    let mut drift = request();
    drift.body.tenant_id = OTHER_TENANT.to_string();
    let err =
        read_cloud_observability_audit_from_api(&verified, &AllowAllAuthorizer, &catalog(), drift)
            .expect_err("body tenant must match verified principal");
    assert_eq!(err.status_code(), 403);
    assert!(matches!(
        err,
        CloudObservabilityApiError::TenantMismatch { .. }
    ));
}

#[test]
fn audit_read_api_rejects_invalid_scope_and_topic_labels_before_kernel() {
    let verified = verified(PRINCIPAL_ID, TENANT);
    let mut invalid_scope = request();
    invalid_scope.body.scope = "everything".to_string();
    let err = read_cloud_observability_audit_from_api(
        &verified,
        &AllowAllAuthorizer,
        &catalog(),
        invalid_scope,
    )
    .expect_err("unknown scope label must fail closed");
    assert_eq!(err.status_code(), 400);
    assert!(matches!(
        err,
        CloudObservabilityApiError::InvalidAuditScopeLabel { .. }
    ));

    let mut invalid_topic = request();
    invalid_topic.body.topics = vec![CloudObservabilityAuditReadTopicRef {
        value: "oya.audit.unknown".to_string(),
    }];
    let err = read_cloud_observability_audit_from_api(
        &verified,
        &AllowAllAuthorizer,
        &catalog(),
        invalid_topic,
    )
    .expect_err("unknown topic label must fail closed");
    assert_eq!(err.status_code(), 400);
    assert!(matches!(
        err,
        CloudObservabilityApiError::InvalidAuditTopicLabel { .. }
    ));
}

#[test]
fn audit_read_api_maps_kernel_read_window_cursor_and_topic_scope_errors() {
    let verified = verified(PRINCIPAL_ID, TENANT);
    let mut invalid_window = request();
    invalid_window.body.start_epoch_seconds = 1_100;
    invalid_window.body.end_epoch_seconds = 900;
    let err = read_cloud_observability_audit_from_api(
        &verified,
        &AllowAllAuthorizer,
        &catalog(),
        invalid_window,
    )
    .expect_err("kernel rejects invalid read window");
    assert_eq!(err.status_code(), 400);

    let mut bad_cursor = request();
    bad_cursor.body.cursor = Some("cur/ten_other/region-home/1000/0".to_string());
    let err = read_cloud_observability_audit_from_api(
        &verified,
        &AllowAllAuthorizer,
        &catalog(),
        bad_cursor,
    )
    .expect_err("cursor tenant drift must be unprocessable");
    assert_eq!(err.status_code(), 422);

    let mut scope_topic = request();
    scope_topic.body.topics = vec![CloudObservabilityAuditReadTopicRef {
        value: "oya.audit.cloud_kms_use".to_string(),
    }];
    let err = read_cloud_observability_audit_from_api(
        &verified,
        &AllowAllAuthorizer,
        &catalog(),
        scope_topic,
    )
    .expect_err("data plane topic cannot be requested under control-plane scope");
    assert_eq!(err.status_code(), 400);
}

#[test]
fn audit_read_api_can_read_all_tenant_audit_when_authorized_for_that_action() {
    // With an authorizer that DOES grant the all-tenant action, the broader read
    // succeeds and is still scoped to the verified tenant.
    let mut all = request();
    all.body.scope = "all_tenant_audit".to_string();
    all.body.actor = Some("usr_admin".to_string());
    all.body.resource_id = Some(RESOURCE_ID.to_string());
    all.body.page_size = Some(100);

    let response = read_cloud_observability_audit_from_api(
        &verified(PRINCIPAL_ID, TENANT),
        &AllowAllAuthorizer,
        &catalog(),
        all,
    )
    .expect("all tenant audit read succeeds when authorized");

    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].actor, "usr_admin");
    assert_eq!(
        response.data[0].source_resource_id.as_deref(),
        Some(RESOURCE_ID)
    );
    assert_eq!(response.data[0].plane, "control");
    assert_eq!(response.data[0].purpose, "core_service");
    assert_eq!(response.metadata.tenant_id, TENANT);
}

#[test]
fn authz_provider_assembles_from_ports() {
    // The AuditReadAuthzProvider composes the two ports; the boundary refuses to
    // serve without one (no default-allow). Smoke-test the composition root.
    use observability_api::AuditReadAuthzProvider;
    let provider = AuditReadAuthzProvider::new(
        Arc::new(verifier(PRINCIPAL_ID, TENANT)),
        Arc::new(AllowAllAuthorizer),
    );
    let principal = provider
        .verify_principal(&CallerCredential {
            authorization: Some(format!("Bearer {BEARER_SECRET}")),
            claimed_principal_id: PRINCIPAL_ID.to_string(),
            claimed_tenant_id: TENANT.to_string(),
        })
        .expect("valid bearer verifies via provider");
    let resource = AuditReadResource {
        tenant_id: TENANT.to_string(),
        region: REGION.to_string(),
        action: AuditReadAction::ControlPlaneAuditRead,
        request_hash: "h:0000000000000000".to_string(),
    };
    provider
        .ensure_authorized(&principal, &resource)
        .expect("allow-all authorizer grants via provider");
}
