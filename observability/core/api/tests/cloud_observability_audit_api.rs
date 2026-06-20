// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use audit_chain_domain::{AuditChain, Plane};
use observability_api::{
    CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE, CloudObservabilityApiAuthorization,
    CloudObservabilityApiBoundaryContext, CloudObservabilityApiError,
    CloudObservabilityApiPrincipal, CloudObservabilityAuditReadApiRequest,
    CloudObservabilityAuditReadApiStatus, CloudObservabilityAuditReadRequest,
    CloudObservabilityAuditReadTopicRef, read_cloud_observability_audit_from_api,
};
use observability_aggregate::{
    CloudAuditEnvelopeCreate, CloudAuditOperation, CloudAuditTopic, CloudObservabilityCatalog,
    ObservabilityResidency, ObservabilityResidencyCreate, ObservabilityResidencyState,
};
use oya_data_boundary_kernel::{DataClass, Purpose};
use network_residency::ResidencyClass;

const TENANT: &str = "ten_alpha";
const REGION: &str = "region-home";
const CELL: &str = "cell-region-home-a-001";
const SIGNED_EXPORT: &str = "s3+signed://region-home/ten_alpha/audit?sig=abc123";
const RESOURCE_ID: &str = "oya:cloud:region-home:ten_alpha:instance:vm-a";
const HASH_A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

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
}

fn residency() -> ObservabilityResidency {
    ObservabilityResidency::new(ObservabilityResidencyCreate {
        tenant_id: TENANT.to_string(),
        region: REGION.to_string(),
        regional_pack: "oya-pack-alpha".to_string(),
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

fn authorization(principal_id: &str, surfaces: &[&str]) -> CloudObservabilityApiAuthorization {
    CloudObservabilityApiAuthorization {
        tenant_id: TENANT.to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
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
        principal: Some(principal("sp_audit_reader")),
        authorization: authorization("sp_audit_reader", &[CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE]),
        body: body(),
    }
}

#[test]
fn audit_read_api_projects_first_page_and_cursor_metadata() {
    let response = read_cloud_observability_audit_from_api(&catalog(), request())
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
    let first = read_cloud_observability_audit_from_api(&catalog(), request())
        .expect("first page succeeds");
    let mut second_request = request();
    second_request.body.cursor = first.metadata.next_cursor;
    second_request.body.page_size = Some(10);

    let second = read_cloud_observability_audit_from_api(&catalog(), second_request)
        .expect("second page succeeds");

    assert_eq!(second.data.len(), 1);
    assert_eq!(second.data[0].operation, "iam_policy_changed");
    assert_eq!(second.metadata.next_cursor, None);
}

#[test]
fn audit_read_api_separates_missing_authentication_from_denied_authorization() {
    let mut unauthenticated = request();
    unauthenticated.principal = None;
    let missing = read_cloud_observability_audit_from_api(&catalog(), unauthenticated)
        .expect_err("missing principal must be unauthorized");
    assert_eq!(missing.status_code(), 401);
    assert!(matches!(
        missing,
        CloudObservabilityApiError::MissingPrincipal
    ));

    let mut denied = request();
    denied.authorization.allowed_surfaces = vec!["cloud.finops.report".to_string()];
    let denied = read_cloud_observability_audit_from_api(&catalog(), denied)
        .expect_err("wrong surface must be forbidden");
    assert_eq!(denied.status_code(), 403);
    assert!(matches!(
        denied,
        CloudObservabilityApiError::AuthorizationDenied { .. }
    ));
}

#[test]
fn audit_read_api_rejects_required_header_and_tenant_drift_before_catalog_read() {
    let mut empty_header = request();
    empty_header.boundary.tenant_id.clear();
    let err = read_cloud_observability_audit_from_api(&catalog(), empty_header)
        .expect_err("tenant header is required");
    assert_eq!(err.status_code(), 400);
    assert!(matches!(err, CloudObservabilityApiError::EmptyTenantHeader));

    let mut drift = request();
    drift.body.tenant_id = "ten_other".to_string();
    let err = read_cloud_observability_audit_from_api(&catalog(), drift)
        .expect_err("body tenant must match header");
    assert_eq!(err.status_code(), 403);
    assert!(matches!(
        err,
        CloudObservabilityApiError::TenantMismatch { .. }
    ));
}

#[test]
fn audit_read_api_rejects_invalid_scope_and_topic_labels_before_kernel() {
    let mut invalid_scope = request();
    invalid_scope.body.scope = "everything".to_string();
    let err = read_cloud_observability_audit_from_api(&catalog(), invalid_scope)
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
    let err = read_cloud_observability_audit_from_api(&catalog(), invalid_topic)
        .expect_err("unknown topic label must fail closed");
    assert_eq!(err.status_code(), 400);
    assert!(matches!(
        err,
        CloudObservabilityApiError::InvalidAuditTopicLabel { .. }
    ));
}

#[test]
fn audit_read_api_maps_kernel_read_window_cursor_and_topic_scope_errors() {
    let mut invalid_window = request();
    invalid_window.body.start_epoch_seconds = 1_100;
    invalid_window.body.end_epoch_seconds = 900;
    let err = read_cloud_observability_audit_from_api(&catalog(), invalid_window)
        .expect_err("kernel rejects invalid read window");
    assert_eq!(err.status_code(), 400);

    let mut bad_cursor = request();
    bad_cursor.body.cursor = Some("cur/ten_other/region-home/1000/0".to_string());
    let err = read_cloud_observability_audit_from_api(&catalog(), bad_cursor)
        .expect_err("cursor tenant drift must be unprocessable");
    assert_eq!(err.status_code(), 422);

    let mut scope_topic = request();
    scope_topic.body.topics = vec![CloudObservabilityAuditReadTopicRef {
        value: "oya.audit.cloud_kms_use".to_string(),
    }];
    let err = read_cloud_observability_audit_from_api(&catalog(), scope_topic)
        .expect_err("data plane topic cannot be requested under control-plane scope");
    assert_eq!(err.status_code(), 400);
}

#[test]
fn audit_read_api_can_read_all_tenant_audit_by_actor_and_resource() {
    let mut all = request();
    all.body.scope = "all_tenant_audit".to_string();
    all.body.actor = Some("usr_admin".to_string());
    all.body.resource_id = Some(RESOURCE_ID.to_string());
    all.body.page_size = Some(100);

    let response = read_cloud_observability_audit_from_api(&catalog(), all)
        .expect("all tenant audit read succeeds");

    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].actor, "usr_admin");
    assert_eq!(
        response.data[0].source_resource_id.as_deref(),
        Some(RESOURCE_ID)
    );
    assert_eq!(response.data[0].plane, "control");
    assert_eq!(response.data[0].purpose, "core_service");
}
