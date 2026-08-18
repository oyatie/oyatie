// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_workspace_meet_api::{
    WORKSPACE_MEET_OPENAPI_CONTRACT, WORKSPACE_MEET_SESSION_START_SURFACE,
    WorkspaceMeetApiAuthorization, WorkspaceMeetApiError, WorkspaceMeetApiPrincipal,
    WorkspaceMeetParticipantRequest, WorkspaceMeetSessionDirectory, WorkspaceMeetSessionMetadata,
    WorkspaceMeetSessionRecord, WorkspaceMeetSessionStartApiRequest,
    WorkspaceMeetSessionStartApiStatus, WorkspaceMeetSessionStartIdempotencyLedger,
    WorkspaceMeetSessionStartRequest, WorkspaceMeetStartBoundaryContext,
    start_workspace_meet_session_from_api,
};

const SESSION_ID: &str = "meet_session_001";
const TENANT_ID: &str = "ten_workspace_alpha";
const HOST: &str = "user:host@example.com";
/// Placement the request carries; the API echoes it back rather than choosing one, so the
/// fixture and the replay assertion must name the same cell. They were two separate literals
/// and had drifted apart — the fixture moved to the `kr` placement alongside `sfu-pool-kr-001`
/// while the assertion still expected an older `alpha` cell.
const CELL_ID: &str = "cell-workspace-kr-001";

fn boundary(request_id: &str, idempotency_key: &str) -> WorkspaceMeetStartBoundaryContext {
    WorkspaceMeetStartBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal(principal_id: &str) -> WorkspaceMeetApiPrincipal {
    WorkspaceMeetApiPrincipal {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization(principal_id: &str, surfaces: &[&str]) -> WorkspaceMeetApiAuthorization {
    WorkspaceMeetApiAuthorization {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn participants() -> Vec<WorkspaceMeetParticipantRequest> {
    vec![
        WorkspaceMeetParticipantRequest {
            actor_ref: HOST.to_string(),
            display_name: Some("Host User".to_string()),
            role: "host".to_string(),
            connection_state: "joined".to_string(),
            joined_at_epoch_seconds: Some(1_700_000_000),
            left_at_epoch_seconds: None,
        },
        WorkspaceMeetParticipantRequest {
            actor_ref: "user:attendee@example.com".to_string(),
            display_name: Some("Attendee User".to_string()),
            role: "attendee".to_string(),
            connection_state: "invited".to_string(),
            joined_at_epoch_seconds: None,
            left_at_epoch_seconds: None,
        },
    ]
}

fn start_body(session_id: &str) -> WorkspaceMeetSessionStartRequest {
    WorkspaceMeetSessionStartRequest {
        session_id: session_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        region: "region-home".to_string(),
        cell_id: CELL_ID.to_string(),
        sfu_pool_id: "sfu-pool-kr-001".to_string(),
        data_class: "PII_IDENTIFYING".to_string(),
        started_at_epoch_seconds: 1_700_000_000,
        participants: participants(),
        recording_consent: "not_requested".to_string(),
        transcript_session_id: Some("transcript_session_001".to_string()),
        summary_id: None,
    }
}

fn start_request(request_id: &str, idempotency_key: &str) -> WorkspaceMeetSessionStartApiRequest {
    WorkspaceMeetSessionStartApiRequest {
        path_session_id: SESSION_ID.to_string(),
        boundary: boundary(request_id, idempotency_key),
        principal: principal(HOST),
        authorization: authorization(HOST, &[WORKSPACE_MEET_SESSION_START_SURFACE]),
        body: start_body(SESSION_ID),
    }
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(
        WORKSPACE_MEET_SESSION_START_SURFACE,
        "workspace.meet.session.start"
    );
    assert_eq!(
        WORKSPACE_MEET_OPENAPI_CONTRACT,
        "contracts/openapi/workspace/workspace-meet-v1.yaml"
    );
    assert_eq!(WorkspaceMeetSessionStartApiStatus::Created.code(), 201);
    assert_eq!(WorkspaceMeetSessionStartApiStatus::BadRequest.code(), 400);
    assert_eq!(WorkspaceMeetSessionStartApiStatus::Forbidden.code(), 403);
    assert_eq!(WorkspaceMeetSessionStartApiStatus::Conflict.code(), 409);
    assert_eq!(
        WorkspaceMeetSessionStartApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn start_meet_session_creates_once_and_replays_same_idempotent_result() {
    let mut directory = WorkspaceMeetSessionDirectory::default();
    let mut ledger = WorkspaceMeetSessionStartIdempotencyLedger::default();
    let request = start_request("req-workspace-meet-start", "idem-workspace-meet-start");

    let first = start_workspace_meet_session_from_api(&mut directory, &mut ledger, request.clone())
        .expect("authorized Meet session start succeeds");
    let second = start_workspace_meet_session_from_api(&mut directory, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(directory.len(), 1);
    assert_eq!(first.metadata.request_id, "req-workspace-meet-start");
    assert_eq!(first.metadata.surface, WORKSPACE_MEET_SESSION_START_SURFACE);
    assert_eq!(first.data.session_id, SESSION_ID);
    assert_eq!(first.data.cell_id, CELL_ID);
    assert_eq!(first.data.participants[0].role, "host");
    assert_eq!(first.data.recording_consent, "not_requested");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn start_meet_session_rejects_path_body_tenant_and_host_drift_before_mutation() {
    let mut directory = WorkspaceMeetSessionDirectory::default();
    let mut ledger = WorkspaceMeetSessionStartIdempotencyLedger::default();
    let mut request = start_request("req-workspace-meet-drift", "idem-workspace-meet-drift");
    request.body.session_id = "meet_session_other".to_string();

    let error = start_workspace_meet_session_from_api(&mut directory, &mut ledger, request)
        .expect_err("path/body session id drift is rejected");
    assert_eq!(
        error,
        WorkspaceMeetApiError::SessionIdMismatch {
            path_session_id: SESSION_ID.to_string(),
            body_session_id: "meet_session_other".to_string(),
        }
    );
    assert_eq!(error.session_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(directory.len(), 0);

    let mut tenant_drift = start_request("req-workspace-meet-tenant", "idem-workspace-meet-tenant");
    tenant_drift.boundary.tenant_id = "ten_other".to_string();
    let error = start_workspace_meet_session_from_api(&mut directory, &mut ledger, tenant_drift)
        .expect_err("tenant drift is rejected");
    assert_eq!(error.session_status_code(), 403);
    assert!(matches!(
        error,
        WorkspaceMeetApiError::TenantMismatch { .. }
    ));
    assert!(ledger.is_empty());

    let mut host_drift = start_request("req-workspace-meet-host", "idem-workspace-meet-host");
    host_drift.principal = principal("user:observer@example.com");
    host_drift.authorization = authorization(
        "user:observer@example.com",
        &[WORKSPACE_MEET_SESSION_START_SURFACE],
    );
    let error = start_workspace_meet_session_from_api(&mut directory, &mut ledger, host_drift)
        .expect_err("starting principal must be a host participant");
    assert_eq!(error.session_status_code(), 403);
    assert!(matches!(
        error,
        WorkspaceMeetApiError::HostPermissionDenied { .. }
    ));
    assert!(ledger.is_empty());
    assert_eq!(directory.len(), 0);
}

#[test]
fn start_meet_session_rejects_authorization_and_reused_idempotency_key() {
    let mut directory = WorkspaceMeetSessionDirectory::default();
    let mut ledger = WorkspaceMeetSessionStartIdempotencyLedger::default();
    let mut request = start_request("req-workspace-meet-authz", "idem-workspace-meet-authz");
    request.authorization.allowed_surfaces = vec!["workspace.drive.get".to_string()];

    let error = start_workspace_meet_session_from_api(&mut directory, &mut ledger, request)
        .expect_err("authorization decision does not allow Meet session start");
    assert_eq!(
        error,
        WorkspaceMeetApiError::AuthorizationDenied {
            surface: WORKSPACE_MEET_SESSION_START_SURFACE.to_string(),
        }
    );
    assert_eq!(error.session_status_code(), 403);
    assert!(ledger.is_empty());

    let request = start_request("req-workspace-meet-idem", "idem-workspace-meet-idem");
    start_workspace_meet_session_from_api(&mut directory, &mut ledger, request.clone())
        .expect("initial Meet session start succeeds");
    let mut drifted = request;
    drifted.body.sfu_pool_id = "sfu-pool-alpha-002".to_string();
    assert_eq!(
        start_workspace_meet_session_from_api(&mut directory, &mut ledger, drifted),
        Err(WorkspaceMeetApiError::IdempotencyKeyReused {
            idempotency_key: "idem-workspace-meet-idem".to_string(),
        })
    );
    assert_eq!(directory.len(), 1);
}

#[test]
fn start_meet_session_maps_duplicate_invalid_role_and_data_class() {
    let mut directory = WorkspaceMeetSessionDirectory::default();
    let mut ledger = WorkspaceMeetSessionStartIdempotencyLedger::default();
    start_workspace_meet_session_from_api(
        &mut directory,
        &mut ledger,
        start_request("req-workspace-meet-dup-1", "idem-workspace-meet-dup-1"),
    )
    .expect("first Meet session start succeeds");

    let duplicate = start_workspace_meet_session_from_api(
        &mut directory,
        &mut ledger,
        start_request("req-workspace-meet-dup-2", "idem-workspace-meet-dup-2"),
    )
    .expect_err("same session through new idempotency key is conflict");
    assert_eq!(duplicate.session_status_code(), 409);
    assert!(matches!(
        duplicate,
        WorkspaceMeetApiError::SessionAlreadyExists { .. }
    ));

    let mut invalid_role = start_request("req-workspace-meet-role", "idem-workspace-meet-role");
    invalid_role.body.participants[0].role = "moderator".to_string();
    let error = start_workspace_meet_session_from_api(&mut directory, &mut ledger, invalid_role)
        .expect_err("unknown role is rejected before kernel");
    assert_eq!(error.session_status_code(), 400);
    assert!(matches!(
        error,
        WorkspaceMeetApiError::InvalidParticipantRole { .. }
    ));

    let mut invalid_class = start_request("req-workspace-meet-class", "idem-workspace-meet-class");
    invalid_class.body.data_class = "AUDIT".to_string();
    let error = start_workspace_meet_session_from_api(&mut directory, &mut ledger, invalid_class)
        .expect_err("operational data class is rejected for public privacy field");
    assert_eq!(error.session_status_code(), 400);
    assert!(matches!(
        error,
        WorkspaceMeetApiError::InvalidDataClassLabel { .. }
    ));
}

#[test]
fn stable_error_response_shape_uses_request_id_and_field_details() {
    let error = WorkspaceMeetApiError::InvalidDataClassLabel {
        data_class: "AUDIT".to_string(),
    };

    let response = error.error_response("req-workspace-meet-error");

    assert_eq!(response.error.code, "WORKSPACE_MEET_DATA_CLASS_INVALID");
    assert_eq!(response.error.request_id, "req-workspace-meet-error");
    assert_eq!(response.error.retry_after_seconds, None);
    assert_eq!(response.error.details[0].field, "body.data_class");
}

#[test]
fn public_response_structs_keep_contract_names_stable() {
    let _metadata = WorkspaceMeetSessionMetadata {
        request_id: "req".to_string(),
        surface: WORKSPACE_MEET_SESSION_START_SURFACE.to_string(),
        openapi_contract: WORKSPACE_MEET_OPENAPI_CONTRACT.to_string(),
    };
    let _record = WorkspaceMeetSessionRecord {
        session_id: "session".to_string(),
        tenant_id: TENANT_ID.to_string(),
        region: "region-home".to_string(),
        cell_id: "cell".to_string(),
        sfu_pool_id: "sfu".to_string(),
        data_class: "PII_IDENTIFYING".to_string(),
        started_at_epoch_seconds: 1,
        ended_at_epoch_seconds: None,
        participants: vec![],
        recording_consent: "not_requested".to_string(),
        transcript_session_id: None,
        summary_id: None,
        schema_version: 1,
    };
}
