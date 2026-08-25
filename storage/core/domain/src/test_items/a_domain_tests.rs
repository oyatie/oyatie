use network_residency::{
    PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
};

use super::*;

fn residency_class() -> ResidencyClass {
    ResidencyClass::PerPack(Box::new(
        PerPackResidency::new(PerPackResidencyCreate {
            allowed_primary_regions: vec!["region-alpha1".to_string()],
            allowed_replica_regions: vec!["region-beta1".to_string()],
            forbidden_regions: vec!["region-gamma1".to_string()],
            regulator_overlay: RegulatorOverlay::new(RegulatorOverlayCreate {
                regulator_refs: vec!["regulator/cloud-storage".to_string()],
                evidence_ref: "evidence/residency/cloud-storage".to_string(),
            })
            .expect("regulator overlay fixture is valid"),
        })
        .expect("per-pack residency fixture is valid"),
    ))
}

fn bucket_create() -> BucketCreate {
    BucketCreate {
        resource_id: "oyatie:cloud:region-alpha1:ten_alpha:bucket:tenant-assets".to_string(),
        tenant_id: "ten_alpha".to_string(),
        name: "tenant-assets".to_string(),
        region: "region-alpha1".to_string(),
        residency: residency_class(),
        tier: BucketTier::Standard,
        replication: ReplicationPolicyCreate::Regional,
        encryption: EncryptionMode::SseKms,
        kms_key: Some("kms/region-alpha1/ten_alpha/object-key".to_string()),
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

fn active_bucket() -> Bucket {
    Bucket::new(bucket_create())
        .expect("bucket create request is valid")
        .activate()
        .expect("created bucket can become active")
}

fn object_encryption() -> ObjectEncryptionBindingCreate {
    ObjectEncryptionBindingCreate {
        kms_key: "kms/region-alpha1/ten_alpha/object-key".to_string(),
        kms_key_version: 1,
        material_ref: "matref/ten_alpha/object/report".to_string(),
        ciphertext_ref: "ct/ten_alpha/object/report".to_string(),
        kms_encrypt_event_id: "kmsuse_object_report_001".to_string(),
        purpose: KmsPurpose::CloudObjectStorage,
        shred_proof_ref: None,
    }
}

fn provider_put_request() -> StorageProviderObjectPutRequest {
    StorageProviderObjectPutRequest {
        request_id: "storageprov_req_put_001".to_string(),
        provider_bucket_ref: "oci-object://axdotp9iv3ua/oyatie-audit-cold-backup".to_string(),
        bucket_id: "oyatie:cloud:region-alpha1:ten_alpha:bucket:tenant-assets".to_string(),
        tenant_id: "ten_alpha".to_string(),
        object_key: "workspace/report.pdf".to_string(),
        object_body_ref: "objbody/ten_alpha/workspace/report".to_string(),
        size_bytes: 42,
        etag: "0123456789abcdef0123456789abcdef".to_string(),
        data_class: DataClass::PiiIdentifying,
        kms_key: "kms/region-alpha1/ten_alpha/object-key".to_string(),
        ciphertext_ref: "ct/ten_alpha/object/report".to_string(),
        actor: "sp_storage".to_string(),
        idempotency_key: "idem-storage-object-put".to_string(),
        requested_at_epoch_seconds: 1_700_000_010,
    }
}

fn provider_get_request() -> StorageProviderObjectGetRequest {
    StorageProviderObjectGetRequest {
        request_id: "storageprov_req_get_001".to_string(),
        provider_bucket_ref: "oci-object://axdotp9iv3ua/oyatie-audit-cold-backup".to_string(),
        bucket_id: "oyatie:cloud:region-alpha1:ten_alpha:bucket:tenant-assets".to_string(),
        tenant_id: "ten_alpha".to_string(),
        object_key: "workspace/report.pdf".to_string(),
        result_body_ref: "objbody/ten_alpha/workspace/report-read".to_string(),
        actor: "sp_storage".to_string(),
        requested_at_epoch_seconds: 1_700_000_020,
    }
}

fn volume_create() -> VolumeCreate {
    VolumeCreate {
        resource_id: "oyatie:cloud:region-alpha1:ten_alpha:volume:db-primary".to_string(),
        tenant_id: "ten_alpha".to_string(),
        name: "db-primary".to_string(),
        region: "region-alpha1".to_string(),
        az: "region-alpha1-a".to_string(),
        cell_id: "cell-region-alpha1-a-001".to_string(),
        residency: residency_class(),
        tier: VolumeTier::ProvisionedIopsSsd,
        size_gib: 512,
        performance: VolumePerformance {
            iops: 12_000,
            throughput_mbps: 750,
        },
        encryption: EncryptionMode::Byok,
        kms_key: Some("byok/region-alpha1/ten_alpha/db-key".to_string()),
        data_class: DataClass::PiiIdentifying,
        state: VolumeState::Creating,
        created_at_epoch_seconds: 1_700_000_000,
    }
}

fn provider_block_create_volume_request() -> StorageProviderBlockCreateVolumeRequest {
    let volume = volume_create();
    StorageProviderBlockCreateVolumeRequest {
        request_id: "storageprov_req_block_create_001".to_string(),
        provider_volume_ref: "oci-block://ocid1.compartment.oc1..cloud/region-alpha1-a/db-primary"
            .to_string(),
        volume_id: volume.resource_id,
        tenant_id: volume.tenant_id,
        name: volume.name,
        region: volume.region,
        az: volume.az,
        cell_id: volume.cell_id,
        residency: volume.residency,
        tier: volume.tier,
        size_gib: volume.size_gib,
        performance: volume.performance,
        encryption: volume.encryption,
        kms_key: volume.kms_key,
        data_class: volume.data_class,
        actor: "sp_storage".to_string(),
        idempotency_key: "idem-storage-block-create".to_string(),
        requested_at_epoch_seconds: 1_700_000_010,
    }
}

#[test]
fn creates_bucket_with_resource_residency_encryption_and_data_class_policy() {
    let bucket = Bucket::new(bucket_create()).expect("bucket is valid");

    assert_eq!(bucket.resource_id.value.kind_label().unwrap(), "bucket");
    assert_eq!(bucket.region.value.value, "region-alpha1");
    assert_eq!(bucket.name.value.value, "tenant-assets");
    assert_eq!(bucket.replication.value.mode(), ReplicationMode::Regional);
    assert_eq!(bucket.encryption.value, EncryptionMode::SseKms);
    assert!(bucket.kms_key.value.is_some());
    assert!(
        bucket
            .allowed_data_classes
            .value
            .contains(&PrivacyDataClass::pii_identifying())
    );
    assert_eq!(bucket.schema_version.value, STORAGE_SCHEMA_VERSION);
}

#[test]
fn create_contracts_reject_caller_forged_terminal_or_runtime_state() {
    assert_eq!(
        Bucket::new(BucketCreate {
            state: BucketState::Active,
            ..bucket_create()
        })
        .expect_err("bucket create starts in Creating"),
        CloudStorageError::InvalidInitialState
    );
    assert_eq!(
        BlockVolume::new(VolumeCreate {
            state: VolumeState::Attached,
            ..volume_create()
        })
        .expect_err("volume create starts in Creating"),
        CloudStorageError::InvalidInitialState
    );
    assert_eq!(
        ArchiveVault::new(ArchiveVaultCreate {
            resource_id: "oyatie:cloud:region-alpha1:ten_alpha:archive-vault:state-test"
                .to_string(),
            tenant_id: "ten_alpha".to_string(),
            name: "state-test".to_string(),
            region: "region-alpha1".to_string(),
            residency: residency_class(),
            tier: ArchiveTier::Cold,
            encryption: EncryptionMode::Sse,
            kms_key: None,
            allowed_data_classes: vec![DataClass::Public],
            state: ArchiveVaultState::Active,
            created_at_epoch_seconds: 1_700_000_000,
        })
        .expect_err("archive-vault create starts in Creating"),
        CloudStorageError::InvalidInitialState
    );
}

#[test]
fn rejects_bucket_identity_encryption_and_object_lock_drift() {
    let wrong_kind = Bucket::new(BucketCreate {
        resource_id: "oyatie:cloud:region-alpha1:ten_alpha:volume:tenant-assets".to_string(),
        ..bucket_create()
    })
    .expect_err("resource id kind must match bucket");
    assert_eq!(wrong_kind, CloudStorageError::ResourceKindMismatch);

    let missing_key = Bucket::new(BucketCreate {
        kms_key: None,
        ..bucket_create()
    })
    .expect_err("SSE-KMS requires key material binding");
    assert_eq!(missing_key, CloudStorageError::MissingKmsKey);

    let invalid_lock = Bucket::new(BucketCreate {
        object_lock: Some(ObjectLockPolicy {
            mode: ObjectLockMode::Governance,
            retain_until_epoch_seconds: 0,
            legal_hold: false,
        }),
        ..bucket_create()
    })
    .expect_err("object lock cannot be empty");
    assert_eq!(invalid_lock, CloudStorageError::InvalidObjectLockPolicy);
}

#[test]
fn rejects_cross_region_replication_that_violates_residency() {
    let error = Bucket::new(BucketCreate {
        replication: ReplicationPolicyCreate::CrossRegion {
            destination_regions: vec!["region-gamma1".to_string()],
        },
        ..bucket_create()
    })
    .expect_err("pack residency forbids replication to a forbidden region");

    assert_eq!(error, CloudStorageError::ReplicationResidencyDenied);
}

#[test]
fn puts_object_only_when_bucket_allows_the_object_data_class() {
    let bucket = active_bucket();
    let object = StoredObject::new(
        &bucket,
        ObjectCreate {
            bucket_id: bucket.resource_id.value.value.clone(),
            tenant_id: "ten_alpha".to_string(),
            key: "workspace/report.pdf".to_string(),
            size_bytes: 42,
            etag: "0123456789abcdef0123456789abcdef".to_string(),
            data_class: DataClass::PiiIdentifying,
            encryption: object_encryption(),
            stored_at_epoch_seconds: 1_700_000_010,
            last_accessed_at_epoch_seconds: Some(1_700_000_020),
        },
    )
    .expect("object data class is admitted by bucket policy");

    assert_eq!(object.key.value.value, "workspace/report.pdf");
    assert_eq!(object.size_bytes.value, 42);
    assert_eq!(object.encryption.value.kms_key_version, 1);
    assert_eq!(
        object.encryption.value.purpose,
        KmsPurpose::CloudObjectStorage
    );

    let denied = StoredObject::new(
        &bucket,
        ObjectCreate {
            bucket_id: bucket.resource_id.value.value.clone(),
            tenant_id: "ten_alpha".to_string(),
            key: "workspace/card.txt".to_string(),
            size_bytes: 42,
            etag: "0123456789abcdef0123456789abcdef".to_string(),
            data_class: DataClass::Pci,
            encryption: ObjectEncryptionBindingCreate {
                kms_encrypt_event_id: "kmsuse_object_card_001".to_string(),
                ..object_encryption()
            },
            stored_at_epoch_seconds: 1_700_000_010,
            last_accessed_at_epoch_seconds: None,
        },
    )
    .expect_err("bucket allowed class set is a hard admission gate");
    assert_eq!(denied, CloudStorageError::ObjectDataClassDenied);
}
