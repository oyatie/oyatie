#[test]
fn catalog_rejects_duplicate_resources_and_creates_snapshot_from_known_volume() {
    let mut catalog = CloudStorageCatalog::default();
    let bucket = catalog
        .create_bucket(bucket_create())
        .expect("first bucket create succeeds");
    assert_eq!(
        catalog
            .create_bucket(bucket_create())
            .expect_err("duplicate bucket resource id rejected"),
        CloudStorageError::DuplicateBucket
    );

    let volume = catalog
        .create_volume(volume_create())
        .expect("volume create succeeds");
    let snapshot = catalog
        .create_snapshot(SnapshotCreate {
            id: "snap_db_primary_001".to_string(),
            tenant_id: "ten_alpha".to_string(),
            source_volume_id: volume.resource_id.value.value.clone(),
            region: "region-alpha1".to_string(),
            data_class: DataClass::PiiIdentifying,
            state: SnapshotState::Creating,
            created_at_epoch_seconds: 1_700_000_030,
        })
        .expect("snapshot source volume is known");

    assert_eq!(snapshot.source_volume_id.value, volume.resource_id.value);
    assert_eq!(catalog.buckets().count(), 1);
    assert_eq!(
        bucket.resource_id.value.resource_name().unwrap(),
        "tenant-assets"
    );
}

#[test]
fn rejects_snapshot_data_class_downgrade_from_source_volume() {
    let volume = BlockVolume::new(volume_create()).expect("volume create request is valid");
    let error = VolumeSnapshot::new(
        &volume,
        SnapshotCreate {
            id: "snap_db_primary_public".to_string(),
            tenant_id: "ten_alpha".to_string(),
            source_volume_id: volume.resource_id.value.value.clone(),
            region: "region-alpha1".to_string(),
            data_class: DataClass::Public,
            state: SnapshotState::Creating,
            created_at_epoch_seconds: 1_700_000_030,
        },
    )
    .expect_err("snapshot data class must match source volume class");
    assert_eq!(error, CloudStorageError::InvalidDataClass);
}

#[test]
fn creates_filesystem_and_archive_vault_surfaces() {
    let filesystem = CloudFilesystem::new(FilesystemCreate {
        resource_id: "oyatie:cloud:region-alpha1:ten_alpha:filesystem:shared-docs".to_string(),
        tenant_id: "ten_alpha".to_string(),
        name: "shared-docs".to_string(),
        region: "region-alpha1".to_string(),
        az: "region-alpha1-a".to_string(),
        cell_id: "cell-region-alpha1-a-001".to_string(),
        residency: residency_class(),
        tier: FilesystemTier::ThroughputOptimized,
        size_gib: 2048,
        throughput_mbps: 1024,
        encryption: EncryptionMode::Sse,
        kms_key: None,
        data_class: DataClass::PiiIdentifying,
        state: FilesystemState::Creating,
        created_at_epoch_seconds: 1_700_000_000,
    })
    .expect("filesystem is valid");
    assert_eq!(
        filesystem.resource_id.value.kind_label().unwrap(),
        "filesystem"
    );

    let vault = ArchiveVault::new(ArchiveVaultCreate {
        resource_id: "oyatie:cloud:region-alpha1:ten_alpha:archive-vault:cold-records".to_string(),
        tenant_id: "ten_alpha".to_string(),
        name: "cold-records".to_string(),
        region: "region-alpha1".to_string(),
        residency: residency_class(),
        tier: ArchiveTier::DeepCold,
        encryption: EncryptionMode::Hyok,
        kms_key: Some("hyok/region-alpha1/ten_alpha/archive-key".to_string()),
        allowed_data_classes: vec![DataClass::PiiIdentifying, DataClass::Phi],
        state: ArchiveVaultState::Creating,
        created_at_epoch_seconds: 1_700_000_000,
    })
    .expect("archive vault is valid");

    assert_eq!(
        vault.resource_id.value.kind_label().unwrap(),
        "archive-vault"
    );
    assert_eq!(vault.encryption.value, EncryptionMode::Hyok);
}
