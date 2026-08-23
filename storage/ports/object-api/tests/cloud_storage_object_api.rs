// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::DataClass;
use network_residency::ResidencyClass;
use storage_domain::{
    BucketCreate, BucketState, BucketTier, CloudStorageCatalog, CloudStorageError, EncryptionMode,
    ObjectLockMode, ObjectLockPolicy, ReplicationPolicyCreate, StorageRepo,
};
use storage_object_api::{
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

const BUCKET_ID: &str = "oya:cloud:region-home:ten_alpha:bucket:tenant-assets";
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
fn openapi_runtime_binding_contracts_are_covered() {
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

#[test]
fn get_object_api_projects_authorized_object_metadata() {
    let mut catalog = catalog_with_active_bucket();
    put_fixture_object(&mut catalog);

    let response = get_cloud_storage_object_from_api(&catalog, get_request(BUCKET_ID, OBJECT_KEY))
        .expect("authorized object GET succeeds");

    assert_eq!(response.metadata.request_id, "req-storage-object-get");
    assert_eq!(response.data.bucket_id, BUCKET_ID);
    assert_eq!(response.data.key, OBJECT_KEY);
    assert_eq!(
        response.data.encryption.ciphertext_ref,
        "ct/ten_alpha/object/report"
    );
    assert_eq!(response.data.schema_version, 1);
}

#[test]
fn get_object_api_rejects_authorization_before_existence_lookup() {
    let catalog = catalog_with_active_bucket();
    let mut request = get_request(BUCKET_ID, "workspace/missing.pdf");
    request.authorization.allowed_surfaces = vec![CLOUD_STORAGE_OBJECT_PUT_SURFACE.to_string()];

    let error = get_cloud_storage_object_from_api(&catalog, request)
        .expect_err("authorization denial must win over object existence checks");

    assert_eq!(
        error,
        CloudStorageObjectApiError::AuthorizationDenied {
            surface: CLOUD_STORAGE_OBJECT_GET_SURFACE.to_string(),
        }
    );
    assert_eq!(error.object_status_code(), 403);
}

#[test]
fn get_object_api_maps_not_found_and_tenant_drift_explicitly() {
    let catalog = catalog_with_active_bucket();
    let missing = get_cloud_storage_object_from_api(
        &catalog,
        get_request(BUCKET_ID, "workspace/missing.pdf"),
    )
    .expect_err("missing object maps to not found");
    assert_eq!(missing.object_status_code(), 404);
    assert!(matches!(
        missing,
        CloudStorageObjectApiError::ObjectNotFound { .. }
    ));

    let tenant_drift = get_cloud_storage_object_from_api(
        &catalog,
        get_request(
            "oya:cloud:region-home:ten_other:bucket:tenant-assets",
            OBJECT_KEY,
        ),
    )
    .expect_err("bucket tenant drift is rejected before catalog lookup");
    assert_eq!(tenant_drift.object_status_code(), 403);
    assert!(matches!(
        tenant_drift,
        CloudStorageObjectApiError::TenantMismatch { .. }
    ));
}

// ---------------------------------------------------------------------------
// cso-1 / cso-2 / cso-3: expanded idempotency replay surface coverage
// These tests pin behavioral contracts from the slice spec that are not yet
// asserted in the existing suite.
// ---------------------------------------------------------------------------

/// Replayed response preserves the ORIGINAL request's `request_id`, not the
/// second caller's.  This is the deterministic-replay contract: the recorded
/// success is returned verbatim regardless of what `request_id` the replay
/// caller supplies.
#[test]
fn replayed_response_preserves_first_request_id_not_second_callers() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();

    // First PUT — request_id "req-replay-first".
    let first = put_cloud_storage_object_from_api(
        &mut catalog,
        &mut ledger,
        put_request("req-replay-first", "idem-replay-id-check"),
    )
    .expect("first PUT succeeds");
    assert_eq!(first.metadata.request_id, "req-replay-first");

    // Second PUT — same idempotency key, DIFFERENT request_id.
    let replayed = put_cloud_storage_object_from_api(
        &mut catalog,
        &mut ledger,
        put_request("req-replay-second", "idem-replay-id-check"),
    )
    .expect("replay with same fingerprint succeeds");

    // The replayed response must carry the FIRST request_id, not the second.
    assert_eq!(
        replayed.metadata.request_id, "req-replay-first",
        "replay must return the stored response verbatim (S3/GCS deterministic-replay semantics)"
    );
    assert_eq!(first, replayed);
    assert_eq!(catalog.objects().count(), 1);
}

/// Two principals in the same tenant with the same idempotency key string are
/// recorded as INDEPENDENT ledger entries.  Each entry tracks its own
/// fingerprint and result; they must not interfere with each other.
#[test]
fn composite_key_isolates_principal_scope() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();

    // Build a request for principal "sp_alpha".
    let alpha_request = CloudStorageObjectPutApiRequest {
        path_bucket_id: BUCKET_ID.to_string(),
        path_object_key: "workspace/alpha.pdf".to_string(),
        boundary: CloudStorageObjectMutationBoundaryContext {
            request_id: "req-alpha".to_string(),
            tenant_id: "ten_alpha".to_string(),
            idempotency_key: "idem-shared-key".to_string(),
        },
        principal: principal_for("sp_alpha"),
        authorization: authorization_for("sp_alpha", &[CLOUD_STORAGE_OBJECT_PUT_SURFACE]),
        body: put_body(BUCKET_ID, "workspace/alpha.pdf"),
    };

    // Build a request for principal "sp_beta" — SAME idempotency key string.
    let beta_request = CloudStorageObjectPutApiRequest {
        path_bucket_id: BUCKET_ID.to_string(),
        path_object_key: "workspace/beta.pdf".to_string(),
        boundary: CloudStorageObjectMutationBoundaryContext {
            request_id: "req-beta".to_string(),
            tenant_id: "ten_alpha".to_string(),
            idempotency_key: "idem-shared-key".to_string(),
        },
        principal: principal_for("sp_beta"),
        authorization: authorization_for("sp_beta", &[CLOUD_STORAGE_OBJECT_PUT_SURFACE]),
        body: put_body(BUCKET_ID, "workspace/beta.pdf"),
    };

    put_cloud_storage_object_from_api(&mut catalog, &mut ledger, alpha_request)
        .expect("sp_alpha PUT succeeds");
    put_cloud_storage_object_from_api(&mut catalog, &mut ledger, beta_request)
        .expect("sp_beta PUT with same idempotency key string but different principal succeeds");

    // Ledger must hold TWO independent entries.
    assert_eq!(
        ledger.len(),
        2,
        "distinct (principal, surface, idempotency_key) composite keys are independent entries"
    );
    assert_eq!(catalog.objects().count(), 2);

    // peek for each principal returns their own entry.
    let alpha_entry = ledger
        .peek(
            "ten_alpha",
            "sp_alpha",
            CLOUD_STORAGE_OBJECT_PUT_SURFACE,
            "idem-shared-key",
        )
        .expect("sp_alpha peek returns Some");
    let beta_entry = ledger
        .peek(
            "ten_alpha",
            "sp_beta",
            CLOUD_STORAGE_OBJECT_PUT_SURFACE,
            "idem-shared-key",
        )
        .expect("sp_beta peek returns Some");

    assert!(matches!(
        alpha_entry.outcome,
        CloudStorageObjectReplayOutcome::Replayed { .. }
    ));
    assert!(matches!(
        beta_entry.outcome,
        CloudStorageObjectReplayOutcome::Replayed { .. }
    ));
}

/// `shred_proof_ref` is included in the fingerprint canonical form.  A request
/// that changes only `shred_proof_ref` while reusing the same idempotency key
/// must be classified as a fingerprint conflict and return
/// `IdempotencyKeyReused`.
#[test]
fn shred_proof_ref_change_yields_idempotency_key_reused_conflict() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();

    let original = put_request("req-shred-1", "idem-shred");
    put_cloud_storage_object_from_api(&mut catalog, &mut ledger, original.clone())
        .expect("first PUT without shred_proof_ref succeeds");

    // Construct a drifted request: same idempotency key, shred_proof_ref added.
    let mut drifted = original;
    drifted.boundary.request_id = "req-shred-2".to_string();
    drifted.body.encryption.shred_proof_ref = Some("proof/ten_alpha/shred/001".to_string());

    let err = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, drifted)
        .expect_err("adding shred_proof_ref changes the fingerprint and triggers conflict");

    assert_eq!(
        err,
        CloudStorageObjectApiError::IdempotencyKeyReused {
            idempotency_key: "idem-shred".to_string(),
        },
        "shred_proof_ref must be part of the fingerprint canonical form"
    );
    assert_eq!(
        catalog.objects().count(),
        1,
        "catalog unchanged on conflict"
    );
}

/// `IdempotencyKeyReused` maps to HTTP 422 and the canonical error code string
/// `CLOUD_STORAGE_OBJECT_IDEMPOTENCY_KEY_REUSED`.  This pins the OpenAPI
/// contract for idempotency-key conflict responses.
#[test]
fn idempotency_key_reused_error_maps_to_422_and_canonical_error_code() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();
    let original = put_request("req-ec-1", "idem-error-code");
    put_cloud_storage_object_from_api(&mut catalog, &mut ledger, original.clone())
        .expect("first PUT succeeds");

    let mut drifted = original;
    drifted.body.size_bytes = 9999;
    let err = put_cloud_storage_object_from_api(&mut catalog, &mut ledger, drifted)
        .expect_err("fingerprint drift yields IdempotencyKeyReused");

    assert_eq!(
        err.object_status_code(),
        422,
        "IdempotencyKeyReused must map to 422 Unprocessable Entity per OpenAPI contract"
    );
    assert_eq!(
        err.code(),
        CloudStorageObjectApiErrorCode::IdempotencyKeyReused,
        "error code variant must be IdempotencyKeyReused"
    );
    assert_eq!(
        err.code().as_str(),
        "CLOUD_STORAGE_OBJECT_IDEMPOTENCY_KEY_REUSED",
        "canonical error code string must match OpenAPI spec"
    );

    let response = err.error_response("req-ec-2");
    assert_eq!(
        response.error.code,
        "CLOUD_STORAGE_OBJECT_IDEMPOTENCY_KEY_REUSED"
    );
    assert_eq!(response.error.request_id, "req-ec-2");
}

/// `peek` returns a `CloudStorageObjectPutIdempotencyEntry` whose
/// `idempotency_key` field exactly matches the key that was passed to `peek`.
/// The `Replayed` variant's inner response must equal the stored PUT response.
#[test]
fn peek_entry_fields_match_recorded_put_response_exactly() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();

    let stored = put_cloud_storage_object_from_api(
        &mut catalog,
        &mut ledger,
        put_request("req-peek-fields", "idem-peek-exact"),
    )
    .expect("PUT succeeds");

    let CloudStorageObjectPutIdempotencyEntry {
        idempotency_key,
        outcome,
    } = ledger
        .peek(
            "ten_alpha",
            "sp_storage",
            CLOUD_STORAGE_OBJECT_PUT_SURFACE,
            "idem-peek-exact",
        )
        .expect("peek returns Some after successful PUT");

    assert_eq!(
        idempotency_key, "idem-peek-exact",
        "entry idempotency_key must match the key used in the lookup"
    );
    match outcome {
        CloudStorageObjectReplayOutcome::Replayed { response } => {
            assert_eq!(
                *response, stored,
                "Replayed response must equal the stored PUT response verbatim"
            );
        }
        CloudStorageObjectReplayOutcome::Conflict { .. } => {
            panic!("peek after successful PUT must return Replayed, not Conflict");
        }
    }
}

/// Multiple independent idempotency keys on the same ledger are tracked
/// separately.  A replay on key A does not affect the state of key B.
#[test]
fn multiple_independent_keys_on_same_ledger_do_not_interfere() {
    let mut catalog = catalog_with_active_bucket();
    let mut ledger = CloudStorageObjectPutIdempotencyLedger::default();

    // Two different object keys so the catalog accepts both.
    let req_a = CloudStorageObjectPutApiRequest {
        path_bucket_id: BUCKET_ID.to_string(),
        path_object_key: "workspace/file-a.bin".to_string(),
        boundary: mutation_boundary_for("req-multi-a", "idem-multi-a"),
        principal: principal_for("sp_storage"),
        authorization: authorization_for("sp_storage", &[CLOUD_STORAGE_OBJECT_PUT_SURFACE]),
        body: put_body(BUCKET_ID, "workspace/file-a.bin"),
    };
    let req_b = CloudStorageObjectPutApiRequest {
        path_bucket_id: BUCKET_ID.to_string(),
        path_object_key: "workspace/file-b.bin".to_string(),
        boundary: mutation_boundary_for("req-multi-b", "idem-multi-b"),
        principal: principal_for("sp_storage"),
        authorization: authorization_for("sp_storage", &[CLOUD_STORAGE_OBJECT_PUT_SURFACE]),
        body: put_body(BUCKET_ID, "workspace/file-b.bin"),
    };

    put_cloud_storage_object_from_api(&mut catalog, &mut ledger, req_a.clone())
        .expect("key-A first PUT succeeds");
    put_cloud_storage_object_from_api(&mut catalog, &mut ledger, req_b.clone())
        .expect("key-B first PUT succeeds");

    assert_eq!(ledger.len(), 2);
    assert_eq!(catalog.objects().count(), 2);

    // Replay key-A does not change ledger size or key-B state.
    put_cloud_storage_object_from_api(&mut catalog, &mut ledger, req_a)
        .expect("key-A replay succeeds");

    assert_eq!(ledger.len(), 2, "replay must not create a new ledger entry");
    assert_eq!(catalog.objects().count(), 2, "catalog unchanged on replay");

    // peek for key-B is still independently accessible.
    let b_entry = ledger
        .peek(
            "ten_alpha",
            "sp_storage",
            CLOUD_STORAGE_OBJECT_PUT_SURFACE,
            "idem-multi-b",
        )
        .expect("key-B peek is still present after key-A replay");
    assert!(matches!(
        b_entry.outcome,
        CloudStorageObjectReplayOutcome::Replayed { .. }
    ));
}
