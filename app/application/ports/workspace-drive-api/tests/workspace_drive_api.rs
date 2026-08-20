// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use workspace_drive_api::{
    WORKSPACE_DRIVE_GET_SURFACE, WORKSPACE_DRIVE_OPENAPI_CONTRACT, WORKSPACE_DRIVE_PUT_SURFACE,
    WorkspaceDriveApiAuthorization, WorkspaceDriveApiError, WorkspaceDriveApiPrincipal,
    WorkspaceDriveMutationBoundaryContext, WorkspaceDriveObjectDirectory,
    WorkspaceDriveObjectGetApiRequest, WorkspaceDriveObjectGetApiStatus,
    WorkspaceDriveObjectPutApiRequest, WorkspaceDriveObjectPutApiStatus,
    WorkspaceDriveObjectPutIdempotencyLedger, WorkspaceDriveObjectPutRequest,
    WorkspaceDrivePermissionGrantRequest, WorkspaceDriveReadBoundaryContext,
    get_workspace_drive_object_from_api, put_workspace_drive_object_from_api,
};

const OBJECT_ID: &str = "drive_object_001";
const TENANT_ID: &str = "ten_workspace_alpha";
const OWNER: &str = "user:owner@example.com";

fn mutation_boundary(
    request_id: &str,
    idempotency_key: &str,
) -> WorkspaceDriveMutationBoundaryContext {
    WorkspaceDriveMutationBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn read_boundary(request_id: &str) -> WorkspaceDriveReadBoundaryContext {
    WorkspaceDriveReadBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: TENANT_ID.to_string(),
    }
}

fn principal(principal_id: &str) -> WorkspaceDriveApiPrincipal {
    WorkspaceDriveApiPrincipal {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization(principal_id: &str, surfaces: &[&str]) -> WorkspaceDriveApiAuthorization {
    WorkspaceDriveApiAuthorization {
        tenant_id: TENANT_ID.to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn grants() -> Vec<WorkspaceDrivePermissionGrantRequest> {
    vec![
        WorkspaceDrivePermissionGrantRequest {
            subject_ref: OWNER.to_string(),
            role: "owner".to_string(),
            granted_at_epoch_seconds: 1_700_000_000,
        },
        WorkspaceDrivePermissionGrantRequest {
            subject_ref: "user:viewer@example.com".to_string(),
            role: "viewer".to_string(),
            granted_at_epoch_seconds: 1_700_000_001,
        },
    ]
}

fn put_body(object_id: &str) -> WorkspaceDriveObjectPutRequest {
    WorkspaceDriveObjectPutRequest {
        object_id: object_id.to_string(),
        folder_id: "folder_team".to_string(),
        path: "/team/roadmap.md".to_string(),
        tenant_id: TENANT_ID.to_string(),
        region: "region-home".to_string(),
        data_class: "PII_IDENTIFYING".to_string(),
        object_storage_key: "ten_workspace_alpha/drive/team/roadmap.md".to_string(),
        size_bytes: 2048,
        mime_type: "text/markdown".to_string(),
        kms_shred_key_id: "kms/region-home/ten_workspace_kr/drive/object".to_string(),
        permissions: grants(),
        created_at_epoch_seconds: 1_700_000_010,
    }
}

fn put_request(request_id: &str, idempotency_key: &str) -> WorkspaceDriveObjectPutApiRequest {
    WorkspaceDriveObjectPutApiRequest {
        path_object_id: OBJECT_ID.to_string(),
        boundary: mutation_boundary(request_id, idempotency_key),
        principal: principal(OWNER),
        authorization: authorization(OWNER, &[WORKSPACE_DRIVE_PUT_SURFACE]),
        body: put_body(OBJECT_ID),
    }
}

fn get_request(object_id: &str, principal_id: &str) -> WorkspaceDriveObjectGetApiRequest {
    WorkspaceDriveObjectGetApiRequest {
        path_object_id: object_id.to_string(),
        boundary: read_boundary("req-workspace-drive-get"),
        principal: principal(principal_id),
        authorization: authorization(principal_id, &[WORKSPACE_DRIVE_GET_SURFACE]),
    }
}

fn put_fixture(directory: &mut WorkspaceDriveObjectDirectory) {
    let mut ledger = WorkspaceDriveObjectPutIdempotencyLedger::default();
    put_workspace_drive_object_from_api(
        directory,
        &mut ledger,
        put_request(
            "req-workspace-drive-fixture",
            "idem-workspace-drive-fixture",
        ),
    )
    .expect("drive object fixture writes through API");
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(WORKSPACE_DRIVE_PUT_SURFACE, "workspace.drive.put");
    assert_eq!(WORKSPACE_DRIVE_GET_SURFACE, "workspace.drive.get");
    assert_eq!(
        WORKSPACE_DRIVE_OPENAPI_CONTRACT,
        "contracts/openapi/workspace/workspace-drive-v1.yaml"
    );
    assert_eq!(WorkspaceDriveObjectPutApiStatus::Created.code(), 201);
    assert_eq!(WorkspaceDriveObjectPutApiStatus::BadRequest.code(), 400);
    assert_eq!(WorkspaceDriveObjectPutApiStatus::Forbidden.code(), 403);
    assert_eq!(WorkspaceDriveObjectPutApiStatus::Conflict.code(), 409);
    assert_eq!(
        WorkspaceDriveObjectPutApiStatus::UnprocessableEntity.code(),
        422
    );
    assert_eq!(WorkspaceDriveObjectGetApiStatus::Ok.code(), 200);
    assert_eq!(WorkspaceDriveObjectGetApiStatus::BadRequest.code(), 400);
    assert_eq!(WorkspaceDriveObjectGetApiStatus::Forbidden.code(), 403);
    assert_eq!(WorkspaceDriveObjectGetApiStatus::NotFound.code(), 404);
}

#[test]
fn put_drive_object_creates_once_and_replays_same_idempotent_result() {
    let mut directory = WorkspaceDriveObjectDirectory::default();
    let mut ledger = WorkspaceDriveObjectPutIdempotencyLedger::default();
    let request = put_request("req-workspace-drive-put", "idem-workspace-drive-put");

    let first = put_workspace_drive_object_from_api(&mut directory, &mut ledger, request.clone())
        .expect("authorized drive PUT succeeds");
    let second = put_workspace_drive_object_from_api(&mut directory, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(directory.len(), 1);
    assert_eq!(first.metadata.request_id, "req-workspace-drive-put");
    assert_eq!(first.metadata.surface, WORKSPACE_DRIVE_PUT_SURFACE);
    assert_eq!(first.data.object_id, OBJECT_ID);
    assert_eq!(first.data.path, "/team/roadmap.md");
    assert_eq!(first.data.data_class, "PII_IDENTIFYING");
    assert_eq!(first.data.permissions[0].role, "owner");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn put_drive_object_rejects_path_body_tenant_and_acl_drift_before_mutation() {
    let mut directory = WorkspaceDriveObjectDirectory::default();
    let mut ledger = WorkspaceDriveObjectPutIdempotencyLedger::default();
    let mut request = put_request("req-workspace-drive-drift", "idem-workspace-drive-drift");
    request.body.object_id = "drive_object_other".to_string();

    let error = put_workspace_drive_object_from_api(&mut directory, &mut ledger, request)
        .expect_err("path/body object id drift is rejected");

    assert_eq!(
        error,
        WorkspaceDriveApiError::ObjectIdMismatch {
            path_object_id: OBJECT_ID.to_string(),
            body_object_id: "drive_object_other".to_string(),
        }
    );
    assert_eq!(error.object_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(directory.len(), 0);

    let mut tenant_drift = put_request("req-workspace-drive-tenant", "idem-workspace-drive-tenant");
    tenant_drift.boundary.tenant_id = "ten_other".to_string();
    let error = put_workspace_drive_object_from_api(&mut directory, &mut ledger, tenant_drift)
        .expect_err("tenant drift is rejected");
    assert_eq!(error.object_status_code(), 403);
    assert!(matches!(
        error,
        WorkspaceDriveApiError::TenantMismatch { .. }
    ));
    assert!(ledger.is_empty());
    assert_eq!(directory.len(), 0);

    let mut acl_drift = put_request("req-workspace-drive-acl", "idem-workspace-drive-acl");
    acl_drift.principal = principal("user:editor@example.com");
    acl_drift.authorization =
        authorization("user:editor@example.com", &[WORKSPACE_DRIVE_PUT_SURFACE]);
    let error = put_workspace_drive_object_from_api(&mut directory, &mut ledger, acl_drift)
        .expect_err("creating principal must be granted owner on the drive object");
    assert_eq!(error.object_status_code(), 403);
    assert!(matches!(
        error,
        WorkspaceDriveApiError::PermissionDenied { .. }
    ));
    assert!(ledger.is_empty());
    assert_eq!(directory.len(), 0);
}

#[test]
fn put_drive_object_rejects_authorization_and_reused_idempotency_key() {
    let mut directory = WorkspaceDriveObjectDirectory::default();
    let mut ledger = WorkspaceDriveObjectPutIdempotencyLedger::default();
    let mut request = put_request("req-workspace-drive-authz", "idem-workspace-drive-authz");
    request.authorization.allowed_surfaces = vec![WORKSPACE_DRIVE_GET_SURFACE.to_string()];

    let error = put_workspace_drive_object_from_api(&mut directory, &mut ledger, request)
        .expect_err("authorization decision does not allow drive PUT");
    assert_eq!(
        error,
        WorkspaceDriveApiError::AuthorizationDenied {
            surface: WORKSPACE_DRIVE_PUT_SURFACE.to_string(),
        }
    );
    assert_eq!(error.object_status_code(), 403);
    assert!(ledger.is_empty());

    let request = put_request("req-workspace-drive-idem", "idem-workspace-drive-idem");
    put_workspace_drive_object_from_api(&mut directory, &mut ledger, request.clone())
        .expect("initial drive PUT succeeds");
    let mut drifted = request;
    drifted.body.size_bytes = 4096;
    assert_eq!(
        put_workspace_drive_object_from_api(&mut directory, &mut ledger, drifted),
        Err(WorkspaceDriveApiError::IdempotencyKeyReused {
            idempotency_key: "idem-workspace-drive-idem".to_string(),
        })
    );
    assert_eq!(directory.len(), 1);
}

#[test]
fn get_drive_object_projects_authorized_metadata_and_enforces_acl_before_data_projection() {
    let mut directory = WorkspaceDriveObjectDirectory::default();
    put_fixture(&mut directory);

    let owner = get_workspace_drive_object_from_api(&directory, get_request(OBJECT_ID, OWNER))
        .expect("owner can read drive object");
    assert_eq!(owner.metadata.request_id, "req-workspace-drive-get");
    assert_eq!(owner.metadata.surface, WORKSPACE_DRIVE_GET_SURFACE);
    assert_eq!(
        owner.data.object_storage_key,
        "ten_workspace_alpha/drive/team/roadmap.md"
    );

    let viewer = get_workspace_drive_object_from_api(
        &directory,
        get_request(OBJECT_ID, "user:viewer@example.com"),
    )
    .expect("viewer grant can read drive object");
    assert_eq!(viewer.data.object_id, OBJECT_ID);

    let denied = get_workspace_drive_object_from_api(
        &directory,
        get_request(OBJECT_ID, "user:blocked@example.com"),
    )
    .expect_err("principal without ACL grant cannot read metadata");
    assert_eq!(denied.object_status_code(), 403);
    assert!(matches!(
        denied,
        WorkspaceDriveApiError::PermissionDenied { .. }
    ));
}

#[test]
fn get_drive_object_rejects_authorization_before_existence_lookup_and_maps_not_found() {
    let directory = WorkspaceDriveObjectDirectory::default();
    let mut request = get_request("missing_object", OWNER);
    request.authorization.allowed_surfaces = vec![WORKSPACE_DRIVE_PUT_SURFACE.to_string()];

    let error = get_workspace_drive_object_from_api(&directory, request)
        .expect_err("authorization denial must win over object existence checks");
    assert_eq!(
        error,
        WorkspaceDriveApiError::AuthorizationDenied {
            surface: WORKSPACE_DRIVE_GET_SURFACE.to_string(),
        }
    );
    assert_eq!(error.object_status_code(), 403);

    let missing =
        get_workspace_drive_object_from_api(&directory, get_request("missing_object", OWNER))
            .expect_err("missing object maps to not found");
    assert_eq!(missing.object_status_code(), 404);
    assert!(matches!(
        missing,
        WorkspaceDriveApiError::ObjectNotFound { .. }
    ));
}

#[test]
fn stable_error_response_shape_uses_request_id_and_field_details() {
    let error = WorkspaceDriveApiError::InvalidDataClassLabel {
        data_class: "AUDIT".to_string(),
    };

    let response = error.error_response("req-workspace-drive-error");

    assert_eq!(response.error.code, "WORKSPACE_DRIVE_DATA_CLASS_INVALID");
    assert_eq!(response.error.request_id, "req-workspace-drive-error");
    assert_eq!(response.error.retry_after_seconds, None);
    assert_eq!(response.error.details[0].field, "body.data_class");
}
