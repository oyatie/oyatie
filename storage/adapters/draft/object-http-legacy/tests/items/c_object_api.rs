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
