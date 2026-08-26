use data_classification::DataClass;
use network_residency::ResidencyClass;
pub(super) use storage_domain::{
    BucketCreate, BucketState, BucketTier, CloudStorageCatalog, CloudStorageError, EncryptionMode,
    ObjectLockMode, ObjectLockPolicy, ReplicationPolicyCreate, StorageRepo,
};
pub(super) use storage_object_api::{
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

pub(super) const BUCKET_ID: &str = "oyatie:cloud:region-home:ten_alpha:bucket:tenant-assets";
pub(super) const OBJECT_KEY: &str = "workspace/report.pdf";

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

pub(super) fn catalog_with_active_bucket() -> CloudStorageCatalog {
    let mut catalog = CloudStorageCatalog::default();
    catalog
        .create_bucket(bucket_create())
        .expect("bucket create fixture registers");
    catalog
        .activate_bucket(BUCKET_ID)
        .expect("bucket fixture activates before object IO");
    catalog
}

pub(super) fn mutation_boundary_for(
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

pub(super) fn principal_for(principal_id: &str) -> CloudStorageObjectApiPrincipal {
    CloudStorageObjectApiPrincipal {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
    }
}

pub(super) fn authorization_for(
    principal_id: &str,
    surfaces: &[&str],
) -> CloudStorageObjectApiAuthorization {
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

pub(super) fn put_body(bucket_id: &str, key: &str) -> CloudStorageObjectPutRequest {
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

pub(super) fn put_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudStorageObjectPutApiRequest {
    CloudStorageObjectPutApiRequest {
        path_bucket_id: BUCKET_ID.to_string(),
        path_object_key: OBJECT_KEY.to_string(),
        boundary: mutation_boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_storage"),
        authorization: authorization_for("sp_storage", &[CLOUD_STORAGE_OBJECT_PUT_SURFACE]),
        body: put_body(BUCKET_ID, OBJECT_KEY),
    }
}

pub(super) fn get_request(bucket_id: &str, object_key: &str) -> CloudStorageObjectGetApiRequest {
    CloudStorageObjectGetApiRequest {
        path_bucket_id: bucket_id.to_string(),
        path_object_key: object_key.to_string(),
        boundary: read_boundary_for("req-storage-object-get"),
        principal: principal_for("sp_storage"),
        authorization: authorization_for("sp_storage", &[CLOUD_STORAGE_OBJECT_GET_SURFACE]),
    }
}

pub(super) fn put_fixture_object(catalog: &mut CloudStorageCatalog) {
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    put_cloud_storage_object_from_api(
        catalog,
        &mut ledger,
        put_request("req-storage-object-fixture", "idem-storage-object-fixture"),
    )
    .expect("object fixture writes through API");
}
