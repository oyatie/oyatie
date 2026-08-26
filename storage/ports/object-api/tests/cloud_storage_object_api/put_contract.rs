use super::common::*;

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

#[test]
fn put_object_api_maps_duplicate_object_to_conflict() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    put_cloud_storage_object_from_api(
        &mut catalog,
        &mut ledger,
        put_request("req-storage-object-dup-1", "idem-storage-object-dup-1"),
    )
    .expect("first object PUT succeeds");

    let error = put_cloud_storage_object_from_api(
        &mut catalog,
        &mut ledger,
        put_request("req-storage-object-dup-2", "idem-storage-object-dup-2"),
    )
    .expect_err("same object through new idempotency key is a conflict");

    assert_eq!(
        error,
        CloudStorageObjectApiError::Storage(CloudStorageError::DuplicateObject)
    );
    assert_eq!(error.object_status_code(), 409);
    assert_eq!(catalog.objects().count(), 1);
}

#[test]
fn put_object_api_maps_bucket_data_class_policy_denial_to_forbidden() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    let mut request = put_request("req-storage-object-class", "idem-storage-object-class");
    request.body.data_class = "PCI".to_string();

    let error = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, request)
        .expect_err("bucket allowed data-class set denies PCI object metadata");

    assert_eq!(
        error,
        CloudStorageObjectApiError::Storage(CloudStorageError::ObjectDataClassDenied)
    );
    assert_eq!(error.object_status_code(), 403);
    assert_eq!(catalog.objects().count(), 0);
}

#[test]
fn put_object_api_maps_wrong_kms_purpose_without_masking() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    let mut request = put_request("req-storage-object-purpose", "idem-storage-object-purpose");
    request.body.encryption.purpose = "cloud_block_storage".to_string();

    let error = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, request)
        .expect_err("object storage accepts only object KMS purpose at kernel boundary");

    assert_eq!(
        error,
        CloudStorageObjectApiError::Storage(CloudStorageError::InvalidKmsPurpose)
    );
    assert_eq!(error.object_status_code(), 400);
    assert_eq!(catalog.objects().count(), 0);
}
