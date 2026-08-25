use super::*;

fn tenant(value: &str) -> TenantId {
    TenantId::parse(value).unwrap_or_else(|_| panic!("tenant parse: {value}"))
}

fn digest_for(value: &[u8]) -> Blake3Digest {
    Blake3Digest::for_payload(value)
}

fn kek_boundary(tenant_id: &TenantId) -> TenantKekBoundary {
    TenantKekBoundary::new(
        tenant_id.clone(),
        format!("kms/{tenant_id}/object-store"),
        1,
        format!("ct/{tenant_id}/object-store"),
        Some(format!("shred/{tenant_id}/object-store")),
    )
    .unwrap()
}

fn audit_anchor() -> CasAuditAnchor {
    CasAuditAnchor::new(
        "audit_evt_object_store_001",
        digest_for(b"audit-chain-head"),
        1_700_000_001,
    )
    .unwrap()
}

fn put_request(
    tenant_id: &TenantId,
    bytes: &[u8],
    retain_until: u64,
) -> (CasPutRequest, InMemoryPayloadReader) {
    let bytes = bytes.to_vec();
    let request = CasPutRequest::new(
        tenant_id.clone(),
        bytes.clone(),
        kek_boundary(tenant_id),
        CasWormPolicy::compliance_until(retain_until, false),
        audit_anchor(),
        1_700_000_010,
    )
    .unwrap();
    (request, InMemoryPayloadReader::from_bytes(bytes))
}

#[test]
fn tenant_scoped_address_uses_blake3_hex() {
    let bytes = b"oyatie object-store payload";
    let address = TenantScopedBlake3Address::for_payload(tenant("ten_alpha"), bytes);
    assert_eq!(address.tenant_id.as_str(), "ten_alpha");
    assert_eq!(
        address.digest.as_str(),
        blake3::hash(bytes).to_hex().as_str()
    );
    assert_eq!(
        address.canonical(),
        format!("cas://ten_alpha/blake3/{}", blake3::hash(bytes).to_hex())
    );
}

#[test]
fn digest_validation_rejects_uppercase_or_wrong_length() {
    assert_eq!(
        Blake3Digest::parse("ABCDEF").unwrap_err(),
        ObjectStoreError::InvalidBlake3Digest
    );
    assert_eq!(
        Blake3Digest::parse("abc").unwrap_err(),
        ObjectStoreError::InvalidBlake3Digest
    );
}

#[test]
fn put_head_get_records_cas_worm_audit_contract() {
    let tenant_id = tenant("ten_alpha");
    let bytes = b"audit payload";
    let (request, mut reader) = put_request(&tenant_id, bytes, 1_800_000_000);
    let address = request.address.clone();
    let store = InMemoryObjectStore::new();

    let put_record = store.put_cas(request, &mut reader).unwrap();
    assert_eq!(put_record.address, address);
    assert_eq!(put_record.size_bytes, bytes.len() as u64);
    assert_eq!(put_record.worm_policy.mode, CasWormMode::Compliance);
    assert_eq!(
        put_record.audit_anchor.audit_event_id,
        "audit_evt_object_store_001"
    );
    assert_eq!(put_record.durability, CasDurabilityPolicy::default());
    let read = CasReadRequest::new(tenant_id.clone(), address.clone());
    assert_eq!(store.head_cas(read.clone()).unwrap(), put_record);
    let mut sink = InMemoryPayloadSink::default();
    let get_record = store.get_cas(read, &mut sink).unwrap();
    assert_eq!(get_record.address, address);
    assert_eq!(sink.to_bytes_for_reference(), bytes);
}

#[test]
fn duplicate_identical_cas_put_is_idempotent() {
    let tenant_id = tenant("ten_alpha");
    let (request, mut reader) = put_request(&tenant_id, b"idempotent payload", 1_800_000_000);
    let replay = request.clone();
    let mut replay_reader = reader.clone();
    let store = InMemoryObjectStore::new();

    let first = store.put_cas(request, &mut reader).unwrap();
    let second = store.put_cas(replay, &mut replay_reader).unwrap();

    assert_eq!(second, first);
    assert_eq!(store.len(), 1);
}

#[test]
fn duplicate_same_bytes_with_different_worm_policy_is_conflict() {
    let tenant_id = tenant("ten_alpha");
    let bytes = b"same bytes stronger retention";
    let (first, mut first_reader) = put_request(&tenant_id, bytes, 1_800_000_000);
    let address = first.address.clone();
    let (stronger, mut stronger_reader) = put_request(&tenant_id, bytes, 1_900_000_000);
    let store = InMemoryObjectStore::new();

    store.put_cas(first, &mut first_reader).unwrap();
    let error = store.put_cas(stronger, &mut stronger_reader).unwrap_err();

    assert_eq!(
        error,
        ObjectStoreError::DuplicateCasWriteConflict {
            tenant_id: "ten_alpha".to_string(),
            digest: address.digest.as_str().to_string(),
        }
    );
    assert_eq!(
        store
            .head_cas(CasReadRequest::new(tenant_id, address))
            .unwrap()
            .worm_policy
            .retain_until_epoch_seconds,
        1_800_000_000
    );
}

#[test]
fn cross_tenant_reads_are_denied_even_when_digest_is_known() {
    let tenant_alpha = tenant("ten_alpha");
    let tenant_beta = tenant("ten_beta");
    let (request, mut reader) = put_request(&tenant_alpha, b"same bytes", 1_800_000_000);
    let address = request.address.clone();
    let store = InMemoryObjectStore::new();
    store.put_cas(request, &mut reader).unwrap();

    let mut sink = InMemoryPayloadSink::default();
    let error = store
        .get_cas(CasReadRequest::new(tenant_beta, address), &mut sink)
        .unwrap_err();
    assert_eq!(error, ObjectStoreError::CrossTenantAccessDenied);
}

#[test]
fn same_digest_in_different_tenants_is_stored_as_separate_cas_objects() {
    let payload = b"identical object payload";
    let alpha = tenant("ten_alpha");
    let beta = tenant("ten_beta");
    let (alpha_request, mut alpha_reader) = put_request(&alpha, payload, 1_800_000_000);
    let (beta_request, mut beta_reader) = put_request(&beta, payload, 1_800_000_000);
    assert_eq!(alpha_request.address.digest, beta_request.address.digest);
    assert_ne!(alpha_request.address, beta_request.address);

    let store = InMemoryObjectStore::new();
    let alpha_record = store.put_cas(alpha_request, &mut alpha_reader).unwrap();
    let beta_record = store.put_cas(beta_request, &mut beta_reader).unwrap();

    assert_eq!(store.len(), 2);
    assert_eq!(alpha_record.address.tenant_id.as_str(), "ten_alpha");
    assert_eq!(beta_record.address.tenant_id.as_str(), "ten_beta");
}

#[test]
fn put_rejects_digest_that_does_not_match_payload() {
    let tenant_id = tenant("ten_alpha");
    let wrong_digest =
        Blake3Digest::parse("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap();
    let request = CasPutRequest {
        address: TenantScopedBlake3Address {
            tenant_id: tenant_id.clone(),
            digest: wrong_digest,
        },
        payload: CasPayload::from_bytes(b"payload with different digest").unwrap(),
        kms_boundary: kek_boundary(&tenant_id),
        worm_policy: CasWormPolicy::compliance_until(1_800_000_000, false),
        audit_anchor: audit_anchor(),
        durability: CasDurabilityPolicy::default(),
        user_metadata: BTreeMap::new(),
        requested_at_epoch_seconds: 1_700_000_010,
    };
    let mut reader = InMemoryPayloadReader::from_bytes(b"payload with different digest".to_vec());
    let error = InMemoryObjectStore::new()
        .put_cas(request, &mut reader)
        .unwrap_err();
    assert!(matches!(
        error,
        ObjectStoreError::AddressDigestMismatch { .. }
    ));
}

#[test]
fn chunked_payload_preserves_root_digest_without_trait_buffer_requirement() {
    let tenant_id = tenant("ten_alpha");
    let chunks = vec![b"chunk-a".to_vec(), b"chunk-b".to_vec()];
    let payload = CasPayload::from_chunks(&chunks).unwrap();
    let request = CasPutRequest::new_with_payload(
        tenant_id.clone(),
        payload.clone(),
        kek_boundary(&tenant_id),
        CasWormPolicy::compliance_until(1_800_000_000, false),
        audit_anchor(),
        1_700_000_010,
    )
    .unwrap();

    assert_eq!(
        request.address.digest,
        Blake3Digest::for_payload(b"chunk-achunk-b")
    );
    assert_eq!(request.payload, payload);
}
