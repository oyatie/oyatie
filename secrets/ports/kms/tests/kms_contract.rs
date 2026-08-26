use secrets_kms::{
    CiphertextRef, CloudKmsError, DestructionProofRef, KmsKeyId, KmsKeyOrigin, KmsPurpose,
    KmsUseEventId, MaterialRef,
};

#[test]
fn key_id_contract_preserves_origin_region_tenant_and_name() {
    for (value, expected_origin) in [
        (
            "kms/ap-seoul-1/ten_acme/object-primary",
            KmsKeyOrigin::OyatieManaged,
        ),
        (
            "byok/ap-seoul-1/ten_acme/object-primary",
            KmsKeyOrigin::Byok,
        ),
        (
            "hyok/ap-seoul-1/ten_acme/object-primary",
            KmsKeyOrigin::Hyok,
        ),
    ] {
        let key_id = KmsKeyId::new(value).expect("canonical key id should construct");
        assert_eq!(
            key_id.origin().expect("origin should parse"),
            expected_origin
        );
        assert_eq!(
            key_id.region().expect("region should parse").value,
            "ap-seoul-1"
        );
        assert_eq!(key_id.tenant_id().expect("tenant should parse"), "ten_acme");
        assert_eq!(key_id.name().expect("name should parse"), "object-primary");
    }
}

#[test]
fn malformed_key_ids_fail_closed() {
    for value in [
        "",
        "kms/ap-seoul-1/ten_acme",
        "external/ap-seoul-1/ten_acme/object-primary",
        "kms//ten_acme/object-primary",
        "kms/ap-seoul-1/acme/object-primary",
        "kms/ap-seoul-1/ten_acme/",
    ] {
        assert_eq!(
            KmsKeyId::new(value),
            Err(CloudKmsError::InvalidKeyId),
            "accepted invalid key id: {value}"
        );
    }
}

#[test]
fn opaque_references_keep_prefix_validation() {
    assert!(MaterialRef::new("matref/object-body").is_ok());
    assert!(CiphertextRef::new("ct/object-body").is_ok());
    assert!(KmsUseEventId::new("kmsuse_object-write").is_ok());
    assert!(DestructionProofRef::new("kproof_object-delete").is_ok());

    assert_eq!(
        MaterialRef::new("object-body"),
        Err(CloudKmsError::InvalidMaterialRef)
    );
    assert_eq!(
        CiphertextRef::new("object-body"),
        Err(CloudKmsError::InvalidCiphertextRef)
    );
    assert_eq!(
        KmsUseEventId::new("kmsuse_"),
        Err(CloudKmsError::InvalidEventId)
    );
    assert_eq!(
        DestructionProofRef::new("kproof_"),
        Err(CloudKmsError::InvalidDestructionProofRef)
    );
}

#[test]
fn storage_purpose_and_origin_vocabulary_remains_stable() {
    assert_eq!(
        KmsPurpose::CloudObjectStorage.label(),
        "cloud_object_storage"
    );
    assert_eq!(KmsPurpose::CloudBlockStorage.label(), "cloud_block_storage");
    assert_eq!(KmsPurpose::CloudFileStorage.label(), "cloud_file_storage");
    assert_eq!(
        KmsPurpose::CloudArchiveStorage.label(),
        "cloud_archive_storage"
    );
    assert_eq!(KmsKeyOrigin::OyatieManaged.id_prefix(), "kms");
    assert_eq!(KmsKeyOrigin::Byok.id_prefix(), "byok");
    assert_eq!(KmsKeyOrigin::Hyok.id_prefix(), "hyok");
}
