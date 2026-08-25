#[test]
fn rejects_object_kms_binding_that_does_not_match_bucket_policy() {
    let bucket = active_bucket();
    let bad_version = StoredObject::new(
        &bucket,
        ObjectCreate {
            bucket_id: bucket.resource_id.value.value.clone(),
            tenant_id: "ten_alpha".to_string(),
            key: "workspace/bad-version.pdf".to_string(),
            size_bytes: 42,
            etag: "0123456789abcdef0123456789abcdef".to_string(),
            data_class: DataClass::PiiIdentifying,
            encryption: ObjectEncryptionBindingCreate {
                kms_key_version: 0,
                ..object_encryption()
            },
            stored_at_epoch_seconds: 1_700_000_010,
            last_accessed_at_epoch_seconds: None,
        },
    )
    .expect_err("object KMS binding must name a concrete key version");
    assert_eq!(bad_version, CloudStorageError::InvalidKmsKeyVersion);

    let wrong_key = StoredObject::new(
        &bucket,
        ObjectCreate {
            bucket_id: bucket.resource_id.value.value.clone(),
            tenant_id: "ten_alpha".to_string(),
            key: "workspace/wrong-key.pdf".to_string(),
            size_bytes: 42,
            etag: "0123456789abcdef0123456789abcdef".to_string(),
            data_class: DataClass::PiiIdentifying,
            encryption: ObjectEncryptionBindingCreate {
                kms_key: "byok/region-alpha1/ten_alpha/object-key".to_string(),
                kms_encrypt_event_id: "kmsuse_object_wrong_key_001".to_string(),
                ..object_encryption()
            },
            stored_at_epoch_seconds: 1_700_000_010,
            last_accessed_at_epoch_seconds: None,
        },
    )
    .expect_err("object KMS key origin must match bucket encryption mode");
    assert_eq!(wrong_key, CloudStorageError::KmsKeyModeMismatch);

    let wrong_purpose = StoredObject::new(
        &bucket,
        ObjectCreate {
            bucket_id: bucket.resource_id.value.value.clone(),
            tenant_id: "ten_alpha".to_string(),
            key: "workspace/wrong-purpose.pdf".to_string(),
            size_bytes: 42,
            etag: "0123456789abcdef0123456789abcdef".to_string(),
            data_class: DataClass::PiiIdentifying,
            encryption: ObjectEncryptionBindingCreate {
                purpose: KmsPurpose::CloudBlockStorage,
                kms_encrypt_event_id: "kmsuse_object_wrong_purpose_001".to_string(),
                ..object_encryption()
            },
            stored_at_epoch_seconds: 1_700_000_010,
            last_accessed_at_epoch_seconds: None,
        },
    )
    .expect_err("object KMS purpose is storage-object specific");
    assert_eq!(wrong_purpose, CloudStorageError::InvalidKmsPurpose);
}

#[test]
fn storage_provider_object_requests_validate_refs_bucket_shape_and_actor() {
    provider_put_request()
        .validate()
        .expect("provider put request is valid");
    provider_get_request()
        .validate()
        .expect("provider get request is valid");

    let mut bad_provider_ref = provider_put_request();
    bad_provider_ref.provider_bucket_ref = " ".to_string();
    assert_eq!(
        bad_provider_ref.validate(),
        Err(StorageProviderObjectError::InvalidProviderBucketRef)
    );

    let mut bad_bucket_kind = provider_put_request();
    bad_bucket_kind.bucket_id =
        "oyatie:cloud:region-alpha1:ten_alpha:volume:not-bucket".to_string();
    assert_eq!(
        bad_bucket_kind.validate(),
        Err(StorageProviderObjectError::InvalidRequestShape(
            CloudStorageError::ResourceKindMismatch,
        ))
    );

    let mut bad_actor = provider_get_request();
    bad_actor.actor = "storage".to_string();
    assert_eq!(
        bad_actor.validate(),
        Err(StorageProviderObjectError::InvalidActorRef)
    );
}

#[test]
fn storage_provider_object_receipts_redact_provider_payloads() {
    let put = StorageProviderObjectReceipt::put_object(
        StorageProviderKind::OciObjectStorage,
        provider_put_request(),
        "oci-object-put-001",
        "oci-object://axdotp9iv3ua/oyatie-audit-cold-backup/workspace/report.pdf/put",
    )
    .expect("put receipt keeps references only");
    let get = StorageProviderObjectReceipt::get_object(
        StorageProviderKind::OciObjectStorage,
        provider_get_request(),
        "oci-object-get-001",
        "oci-object://axdotp9iv3ua/oyatie-audit-cold-backup/workspace/report.pdf/get",
    )
    .expect("get receipt keeps references only");

    assert_eq!(put.provider.label(), "oci_object_storage");
    assert_eq!(put.operation.label(), "put_object");
    assert_eq!(get.operation.label(), "get_object");
    assert_eq!(put.object_body_ref, "objbody/ten_alpha/workspace/report");
    assert_eq!(
        get.object_body_ref,
        "objbody/ten_alpha/workspace/report-read"
    );
    assert_eq!(put.size_bytes, Some(42));
    assert_eq!(
        put.etag,
        Some("0123456789abcdef0123456789abcdef".to_string())
    );
    assert_eq!(
        put.kms_key,
        Some("kms/region-alpha1/ten_alpha/object-key".to_string())
    );
    assert_eq!(
        put.ciphertext_ref,
        Some("ct/ten_alpha/object/report".to_string())
    );
    assert_eq!(get.size_bytes, None);
    assert_eq!(get.etag, None);
    assert_eq!(get.kms_key, None);
    assert_eq!(get.ciphertext_ref, None);
}

#[test]
fn storage_provider_block_requests_validate_refs_volume_shape_and_actor() {
    provider_block_create_volume_request()
        .validate()
        .expect("provider block create request is valid");

    let mut bad_provider_ref = provider_block_create_volume_request();
    bad_provider_ref.provider_volume_ref = " ".to_string();
    assert_eq!(
        bad_provider_ref.validate(),
        Err(StorageProviderBlockError::InvalidProviderVolumeRef)
    );

    let mut bad_volume_kind = provider_block_create_volume_request();
    bad_volume_kind.volume_id =
        "oyatie:cloud:region-alpha1:ten_alpha:bucket:not-volume".to_string();
    assert_eq!(
        bad_volume_kind.validate(),
        Err(StorageProviderBlockError::InvalidRequestShape(
            CloudStorageError::ResourceKindMismatch,
        ))
    );

    let mut bad_actor = provider_block_create_volume_request();
    bad_actor.actor = "storage".to_string();
    assert_eq!(
        bad_actor.validate(),
        Err(StorageProviderBlockError::InvalidActorRef)
    );
}

#[test]
fn storage_provider_block_receipts_keep_refs_without_provider_credentials() {
    let receipt = StorageProviderBlockReceipt::create_volume(
        StorageProviderKind::OciBlockStorage,
        provider_block_create_volume_request(),
        "oci-block-create-001",
        "oci-block://ocid1.compartment.oc1..cloud/region-alpha1-a/db-primary/create",
    )
    .expect("block receipt keeps references only");

    assert_eq!(receipt.provider.label(), "oci_block_storage");
    assert_eq!(receipt.operation.label(), "create_volume");
    assert_eq!(
        receipt.provider_volume_ref,
        "oci-block://ocid1.compartment.oc1..cloud/region-alpha1-a/db-primary"
    );
    assert_eq!(
        receipt.volume_id,
        "oyatie:cloud:region-alpha1:ten_alpha:volume:db-primary"
    );
    assert_eq!(receipt.size_gib, 512);
    assert_eq!(receipt.performance.iops, 12_000);
    assert_eq!(receipt.encryption, EncryptionMode::Byok);
    assert_eq!(
        receipt.kms_key,
        Some("byok/region-alpha1/ten_alpha/db-key".to_string())
    );
    assert_eq!(receipt.actor, "sp_storage");
    assert_eq!(receipt.schema_version, STORAGE_SCHEMA_VERSION);
}

#[test]
fn rejects_operational_labels_on_storage_payloads_and_class_sets() {
    let class_set_error = Bucket::new(BucketCreate {
        allowed_data_classes: vec![DataClass::Audit],
        ..bucket_create()
    })
    .expect_err("allowed classes are privacy-program classes only");
    assert_eq!(class_set_error, CloudStorageError::InvalidDataClass);

    let volume_error = BlockVolume::new(VolumeCreate {
        data_class: DataClass::Secret,
        ..volume_create()
    })
    .expect_err("volume payload data class rejects operational labels");
    assert_eq!(volume_error, CloudStorageError::InvalidDataClass);
}

#[test]
fn creates_block_volume_with_az_cell_performance_and_byok_binding() {
    let volume = BlockVolume::new(volume_create()).expect("volume is valid");

    assert_eq!(volume.resource_id.value.kind_label().unwrap(), "volume");
    assert_eq!(volume.az.value.value, "region-alpha1-a");
    assert_eq!(volume.cell_id.value.value, "cell-region-alpha1-a-001");
    assert_eq!(volume.performance.value.iops, 12_000);
    assert_eq!(volume.encryption.value, EncryptionMode::Byok);
    assert_eq!(volume.schema_version.value, STORAGE_SCHEMA_VERSION);
}

#[test]
fn rejects_volume_location_and_performance_drift() {
    let az_error = BlockVolume::new(VolumeCreate {
        az: "region-gamma1-a".to_string(),
        cell_id: "cell-region-gamma1-a-001".to_string(),
        ..volume_create()
    })
    .expect_err("volume AZ must belong to region");
    assert_eq!(az_error, CloudStorageError::AzRegionMismatch);

    let cell_error = BlockVolume::new(VolumeCreate {
        cell_id: "cell-region-alpha1-b-001".to_string(),
        ..volume_create()
    })
    .expect_err("volume cell must belong to AZ namespace");
    assert_eq!(cell_error, CloudStorageError::CellLocationMismatch);

    let perf_error = BlockVolume::new(VolumeCreate {
        performance: VolumePerformance {
            iops: 0,
            throughput_mbps: 750,
        },
        ..volume_create()
    })
    .expect_err("performance is positive");
    assert_eq!(perf_error, CloudStorageError::InvalidPerformance);
}
