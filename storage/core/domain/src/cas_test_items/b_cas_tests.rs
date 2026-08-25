#[test]
fn reference_validation_rejects_trim_mismatched_values() {
    let tenant_id = tenant("ten_alpha");
    assert_eq!(
        TenantKekBoundary::new(
            tenant_id.clone(),
            " kms/ten_alpha/object-store",
            1,
            "ct/ten_alpha/object-store",
            None,
        )
        .unwrap_err(),
        ObjectStoreError::InvalidKekBoundary
    );

    let (mut request, _) = put_request(&tenant_id, b"metadata payload", 1_800_000_000);
    request
        .user_metadata
        .insert(" object-store-key".to_string(), "ok".to_string());
    assert_eq!(
        request.validate().unwrap_err(),
        ObjectStoreError::InvalidUserMetadata
    );

    let delete = CasDeleteRequest::new(
        tenant_id.clone(),
        TenantScopedBlake3Address::for_payload(tenant_id, b"metadata payload"),
        1_800_000_001,
        " audit_evt_delete_with_space",
    );
    assert_eq!(
        delete.validate().unwrap_err(),
        ObjectStoreError::InvalidDeleteRequest
    );
}

#[test]
fn put_rejects_worm_policy_already_expired_at_write_time() {
    let tenant_id = tenant("ten_alpha");
    let error = CasPutRequest::new(
        tenant_id.clone(),
        b"expired worm".to_vec(),
        kek_boundary(&tenant_id),
        CasWormPolicy::compliance_until(1_700_000_009, false),
        audit_anchor(),
        1_700_000_010,
    )
    .unwrap_err();

    assert_eq!(
        error,
        ObjectStoreError::ExpiredWormPolicy {
            retain_until_epoch_seconds: 1_700_000_009,
            requested_at_epoch_seconds: 1_700_000_010,
        }
    );
}

#[test]
fn worm_retention_blocks_delete_until_retention_expires() {
    let tenant_id = tenant("ten_alpha");
    let (request, mut reader) = put_request(&tenant_id, b"worm payload", 1_800_000_000);
    let address = request.address.clone();
    let store = InMemoryObjectStore::new();
    store.put_cas(request, &mut reader).unwrap();

    let protected = store
        .delete_cas(CasDeleteRequest::new(
            tenant_id.clone(),
            address.clone(),
            1_799_999_999,
            "audit_evt_delete_attempt",
        ))
        .unwrap_err();
    assert_eq!(
        protected,
        ObjectStoreError::WormRetentionActive {
            retain_until_epoch_seconds: 1_800_000_000,
            legal_hold: false,
        }
    );

    store
        .delete_cas(CasDeleteRequest::new(
            tenant_id.clone(),
            address.clone(),
            1_800_000_001,
            "audit_evt_delete_after_retention",
        ))
        .unwrap();
    assert!(matches!(
        store
            .head_cas(CasReadRequest::new(tenant_id, address))
            .unwrap_err(),
        ObjectStoreError::NotFound { .. }
    ));
}

#[test]
fn conformance_suite_proves_reference_adapter_contract() {
    let store = InMemoryObjectStore::new();

    let report = run_object_store_conformance_suite(&store).unwrap();

    assert_eq!(
        report.checks,
        vec![
            "put_immediate_head_visibility",
            "put_immediate_get_visibility",
            "tenant_isolation",
            "worm_delete_refusal",
            "same_payload_cross_tenant_isolation",
        ]
    );
}

#[test]
fn transitional_adapter_boundary_is_diagnostic_not_bridge_shaped() {
    let tenant_id = tenant("ten_alpha");
    let (request, mut reader) = put_request(&tenant_id, b"transitional payload", 1_800_000_000);
    let address = request.address.clone();
    let store = InMemoryObjectStore::with_transitional_adapter(
        TransitionalAdapterClass::ProtocolCompatible,
        "object-protocol-bridge",
    )
    .unwrap();

    let record = store.put_cas(request, &mut reader).unwrap();
    assert_eq!(record.address, address);
    assert_eq!(
        ObjectStoreDiagnostics::backend_kind(&store),
        ObjectStoreBackendKind::TransitionalAdapter
    );
    let boundary =
        ObjectStoreDiagnostics::adapter_boundary(&store, CasReadRequest::new(tenant_id, address))
            .unwrap()
            .expect("transitional boundary is present");
    assert_eq!(
        boundary.adapter_class,
        TransitionalAdapterClass::ProtocolCompatible
    );
    assert_eq!(boundary.adapter_id, "object-protocol-bridge");
    assert!(
        boundary
            .adapter_namespace
            .starts_with("adapter/protocol_compatible/object-protocol-bridge/")
    );
    assert!(boundary.adapter_object_ref.starts_with("cas/ten_alpha/"));
    assert!(
        boundary
            .adapter_evidence_ref
            .starts_with("evidence/object-store/ten_alpha/")
    );
}
