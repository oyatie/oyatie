use data_classification::DataClass;
use network_residency::ResidencyClass;
use storage_domain::{
    BucketCreate, BucketState, BucketTier, CloudStorageCatalog, CloudStorageError, EncryptionMode,
    ObjectLockMode, ObjectLockPolicy, ReplicationPolicyCreate, StorageRepo,
};
use storage_object_http_legacy_draft::{
    CLOUD_STORAGE_OBJECT_GET_SURFACE, CLOUD_STORAGE_OBJECT_PUT_SURFACE,
    CloudStorageObjectApiAuthorization, CloudStorageObjectApiError, CloudStorageObjectApiErrorCode,
    CloudStorageObjectApiPrincipal, CloudStorageObjectEncryptionBindingRequest,
    CloudStorageObjectGetApiRequest, CloudStorageObjectGetApiStatus,
    CloudStorageObjectMutationBoundaryContext, CloudStorageObjectPutApiRequest,
    CloudStorageObjectPutApiStatus, CloudStorageObjectPutIdempotencyEntry,
    CloudStorageObjectPutIdempotencyLedger, CloudStorageObjectPutRequest,
    CloudStorageObjectReadBoundaryContext, CloudStorageObjectReplayOutcome,
    get_cloud_storage_object_from_api, put_cloud_storage_object_from_api,
};

const BUCKET_ID: &str = "oyatie:cloud:region-home:ten_alpha:bucket:tenant-assets";
const OBJECT_KEY: &str = "workspace/report.pdf";

fn bucket_create() -> BucketCreate {
    BucketCreate {
        resource_id: BUCKET_ID.to_string(),
        tenant_id: "ten_alpha".to_string(),
        name: "tenant-assets".to_string(),
        region: "region-home".to_string(),
        residency: ResidencyClass::StrictHomeRegion,
        tier: BucketTier::Standard,
        replication: ReplicationPolicyCreate::Regional,
        encryption: EncryptionMode::SseKms,
        kms_key: Some("kms/region-home/ten_alpha/object-key".to_string()),
        object_lock: Some(ObjectLockPolicy {
            mode: ObjectLockMode::Compliance,
            retain_until_epoch_seconds: 1_800_000_000,
            legal_hold: true,
        }),
        allowed_data_classes: vec![DataClass::Public, DataClass::PiiIdentifying],
        state: BucketState::Creating,
        created_at_epoch_seconds: 1_700_000_000,
    }
}

fn catalog_with_active_bucket() -> CloudStorageCatalog {
    let mut catalog = CloudStorageCatalog::default();
    catalog
        .create_bucket(bucket_create())
        .expect("bucket create fixture registers");
    catalog
        .activate_bucket(BUCKET_ID)
        .expect("bucket fixture activates before object IO");
    catalog
}

fn mutation_boundary_for(
    request_id: &str,
    idempotency_key: &str,
) -> CloudStorageObjectMutationBoundaryContext {
    CloudStorageObjectMutationBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn read_boundary_for(request_id: &str) -> CloudStorageObjectReadBoundaryContext {
    CloudStorageObjectReadBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudStorageObjectApiPrincipal {
    CloudStorageObjectApiPrincipal {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudStorageObjectApiAuthorization {
    CloudStorageObjectApiAuthorization {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn object_encryption() -> CloudStorageObjectEncryptionBindingRequest {
    CloudStorageObjectEncryptionBindingRequest {
        kms_key: "kms/region-home/ten_alpha/object-key".to_string(),
        kms_key_version: 1,
        material_ref: "matref/ten_alpha/object/report".to_string(),
        ciphertext_ref: "ct/ten_alpha/object/report".to_string(),
        kms_encrypt_event_id: "kmsuse_object_report_001".to_string(),
        purpose: "cloud_object_storage".to_string(),
        shred_proof_ref: None,
    }
}

fn put_body(bucket_id: &str, key: &str) -> CloudStorageObjectPutRequest {
    CloudStorageObjectPutRequest {
        bucket_id: bucket_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        key: key.to_string(),
        size_bytes: 42,
        etag: "0123456789abcdef0123456789abcdef".to_string(),
        data_class: "PII_IDENTIFYING".to_string(),
        encryption: object_encryption(),
        stored_at_epoch_seconds: 1_700_000_010,
        last_accessed_at_epoch_seconds: Some(1_700_000_020),
    }
}

fn put_request(request_id: &str, idempotency_key: &str) -> CloudStorageObjectPutApiRequest {
    CloudStorageObjectPutApiRequest {
        path_bucket_id: BUCKET_ID.to_string(),
        path_object_key: OBJECT_KEY.to_string(),
        boundary: mutation_boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_storage"),
        authorization: authorization_for("sp_storage", &[CLOUD_STORAGE_OBJECT_PUT_SURFACE]),
        body: put_body(BUCKET_ID, OBJECT_KEY),
    }
}

fn get_request(bucket_id: &str, object_key: &str) -> CloudStorageObjectGetApiRequest {
    CloudStorageObjectGetApiRequest {
        path_bucket_id: bucket_id.to_string(),
        path_object_key: object_key.to_string(),
        boundary: read_boundary_for("req-storage-object-get"),
        principal: principal_for("sp_storage"),
        authorization: authorization_for("sp_storage", &[CLOUD_STORAGE_OBJECT_GET_SURFACE]),
    }
}

fn put_fixture_object(catalog: &mut CloudStorageCatalog) {
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    put_cloud_storage_object_from_api(
        catalog,
        &mut ledger,
        put_request("req-storage-object-fixture", "idem-storage-object-fixture"),
    )
    .expect("object fixture writes through API");
}

#[test]
fn legacy_http_runtime_binding_contracts_are_covered() {
    assert_eq!(CLOUD_STORAGE_OBJECT_PUT_SURFACE, "cloud.storage.object.put");
    assert_eq!(CLOUD_STORAGE_OBJECT_GET_SURFACE, "cloud.storage.object.get");
    assert_eq!(CloudStorageObjectPutApiStatus::Created.code(), 201);
    assert_eq!(CloudStorageObjectPutApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudStorageObjectPutApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudStorageObjectPutApiStatus::NotFound.code(), 404);
    assert_eq!(CloudStorageObjectPutApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudStorageObjectPutApiStatus::UnprocessableEntity.code(),
        422
    );
    assert_eq!(CloudStorageObjectGetApiStatus::Ok.code(), 200);
    assert_eq!(CloudStorageObjectGetApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudStorageObjectGetApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudStorageObjectGetApiStatus::NotFound.code(), 404);
}

#[test]
fn put_object_api_creates_object_once_and_replays_same_idempotent_result() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    let request = put_request("req-storage-object-put", "idem-storage-object-put");

    let first = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("authorized object PUT succeeds");
    let second = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.objects().count(), 1);
    assert_eq!(first.metadata.request_id, "req-storage-object-put");
    assert_eq!(first.data.bucket_id, BUCKET_ID);
    assert_eq!(first.data.key, OBJECT_KEY);
    assert_eq!(first.data.size_bytes, 42);
    assert_eq!(first.data.data_class, "PII_IDENTIFYING");
    assert_eq!(first.data.encryption.kms_key_version, 1);
    assert_eq!(first.data.encryption.purpose, "cloud_object_storage");
    assert_eq!(
        first.data.last_accessed_at_epoch_seconds,
        Some(1_700_000_020)
    );
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn put_object_api_rejects_path_body_drift_before_catalog_mutation() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    let mut request = put_request("req-storage-object-drift", "idem-storage-object-drift");
    request.body.key = "workspace/other.pdf".to_string();

    let error = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, request)
        .expect_err("path/body object key drift is rejected");

    assert_eq!(
        error,
        CloudStorageObjectApiError::ObjectKeyMismatch {
            path_object_key: OBJECT_KEY.to_string(),
            body_key: "workspace/other.pdf".to_string(),
        }
    );
    assert_eq!(error.object_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.objects().count(), 0);
}

#[test]
fn put_object_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    let mut empty_request = put_request(" ", "idem-storage-object-empty-header");
    assert_eq!(
        put_cloud_storage_object_from_api(&mut catalog, &mut ledger, empty_request.clone()),
        Err(CloudStorageObjectApiError::EmptyRequestId)
    );

    empty_request.boundary.request_id = "req-storage-object-tenant-drift".to_string();
    empty_request.boundary.tenant_id = "ten_other".to_string();
    let error = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, empty_request)
        .expect_err("tenant drift is rejected before idempotency ledger write");

    assert_eq!(error.object_status_code(), 403);
    assert!(matches!(
        error,
        CloudStorageObjectApiError::TenantMismatch { .. }
    ));
    assert!(ledger.is_empty());
    assert_eq!(catalog.objects().count(), 0);
}

#[test]
fn put_object_api_rejects_unauthorized_same_tenant_principal_before_ledger() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    let mut request = put_request("req-storage-object-authz", "idem-storage-object-authz");
    request.authorization.allowed_surfaces = vec![CLOUD_STORAGE_OBJECT_GET_SURFACE.to_string()];

    let error = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, request)
        .expect_err("authorization decision does not allow object PUT");

    assert_eq!(
        error,
        CloudStorageObjectApiError::AuthorizationDenied {
            surface: CLOUD_STORAGE_OBJECT_PUT_SURFACE.to_string(),
        }
    );
    assert_eq!(error.object_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.objects().count(), 0);
}

#[test]
fn put_object_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    let request = put_request("req-storage-object-idem", "idem-storage-object-idem");
    put_cloud_storage_object_from_api(&mut catalog, &mut ledger, request.clone())
        .expect("initial object PUT succeeds");

    let mut drifted = request;
    drifted.body.etag = "fedcba9876543210fedcba9876543210".to_string();
    assert_eq!(
        put_cloud_storage_object_from_api(&mut catalog, &mut ledger, drifted),
        Err(CloudStorageObjectApiError::IdempotencyKeyReused {
            idempotency_key: "idem-storage-object-idem".to_string(),
        })
    );
    assert_eq!(catalog.objects().count(), 1);
}
