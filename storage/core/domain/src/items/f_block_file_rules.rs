impl BlockVolume {
    pub fn new(input: VolumeCreate) -> Result<Self, CloudStorageError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != VolumeState::Creating {
            return Err(CloudStorageError::InvalidInitialState);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudStorageError::InvalidResourceId)?;
        let az = AzCode::new(input.az).map_err(|_| CloudStorageError::InvalidAzCode)?;
        let cell_id = CellId::new(input.cell_id).map_err(|_| CloudStorageError::InvalidCellId)?;
        validate_az_region(&az, &region)?;
        validate_cell_location(&cell_id, &region, Some(&az))?;
        validate_residency_allows_region(&input.residency, &region)?;
        validate_size(input.size_gib)?;
        validate_performance(input.performance)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::Volume(input.tier),
        )?;
        let kms_key = encryption_key(input.encryption, input.kms_key, &region, &input.tenant_id)?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            name: internal(VolumeName::new(input.name)?),
            region: public(region),
            az: public(az),
            cell_id: public(cell_id),
            residency: internal(input.residency),
            tier: public(input.tier),
            size_gib: internal(input.size_gib),
            performance: public(input.performance),
            encryption: public(input.encryption),
            kms_key: internal(kms_key),
            data_class: internal(privacy_class(input.data_class)?),
            state: public(input.state),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(STORAGE_SCHEMA_VERSION),
        })
    }
}
impl CloudFilesystem {
    pub fn new(input: FilesystemCreate) -> Result<Self, CloudStorageError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != FilesystemState::Creating {
            return Err(CloudStorageError::InvalidInitialState);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudStorageError::InvalidResourceId)?;
        let az = AzCode::new(input.az).map_err(|_| CloudStorageError::InvalidAzCode)?;
        let cell_id = CellId::new(input.cell_id).map_err(|_| CloudStorageError::InvalidCellId)?;
        validate_az_region(&az, &region)?;
        validate_cell_location(&cell_id, &region, Some(&az))?;
        validate_residency_allows_region(&input.residency, &region)?;
        validate_size(input.size_gib)?;
        if input.throughput_mbps == 0 {
            return Err(CloudStorageError::InvalidPerformance);
        }
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::Filesystem(input.tier),
        )?;
        let kms_key = encryption_key(input.encryption, input.kms_key, &region, &input.tenant_id)?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            name: internal(FilesystemName::new(input.name)?),
            region: public(region),
            az: public(az),
            cell_id: public(cell_id),
            residency: internal(input.residency),
            tier: public(input.tier),
            size_gib: internal(input.size_gib),
            throughput_mbps: public(input.throughput_mbps),
            encryption: public(input.encryption),
            kms_key: internal(kms_key),
            data_class: internal(privacy_class(input.data_class)?),
            state: public(input.state),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(STORAGE_SCHEMA_VERSION),
        })
    }
}

impl ArchiveVault {
    pub fn new(input: ArchiveVaultCreate) -> Result<Self, CloudStorageError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != ArchiveVaultState::Creating {
            return Err(CloudStorageError::InvalidInitialState);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudStorageError::InvalidResourceId)?;
        validate_residency_allows_region(&input.residency, &region)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::ArchiveVault,
        )?;
        let kms_key = encryption_key(input.encryption, input.kms_key, &region, &input.tenant_id)?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            name: internal(ArchiveVaultName::new(input.name)?),
            region: public(region),
            residency: internal(input.residency),
            tier: public(input.tier),
            encryption: public(input.encryption),
            kms_key: internal(kms_key),
            allowed_data_classes: internal(privacy_class_set(input.allowed_data_classes)?),
            state: public(input.state),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(STORAGE_SCHEMA_VERSION),
        })
    }
}

impl VolumeSnapshot {
    pub fn new(
        source_volume: &BlockVolume,
        input: SnapshotCreate,
    ) -> Result<Self, CloudStorageError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != SnapshotState::Creating {
            return Err(CloudStorageError::InvalidInitialState);
        }
        let source_volume_id = ResourceId::new(input.source_volume_id)
            .map_err(|_| CloudStorageError::InvalidResourceId)?;
        if source_volume_id != source_volume.resource_id.value {
            return Err(CloudStorageError::UnknownVolume);
        }
        if input.tenant_id != source_volume.tenant_id.value {
            return Err(CloudStorageError::ResourceTenantMismatch);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudStorageError::InvalidResourceId)?;
        if region != source_volume.region.value {
            return Err(CloudStorageError::ResourceRegionMismatch);
        }
        let data_class = privacy_class(input.data_class)?;
        if data_class != source_volume.data_class.value {
            return Err(CloudStorageError::InvalidDataClass);
        }
        Ok(Self {
            id: internal(SnapshotId::new(input.id)?),
            tenant_id: internal(input.tenant_id),
            source_volume_id: internal(source_volume_id),
            region: public(region),
            data_class: internal(data_class),
            state: public(input.state),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(STORAGE_SCHEMA_VERSION),
        })
    }
}

impl StorageTenantCellGuardrail {
    pub fn new(input: StorageTenantCellGuardrailCreate) -> Result<Self, CloudStorageError> {
        let StorageTenantCellGuardrailCreate {
            tenant_id,
            region,
            primary_cell_id,
            bucket_id,
            object_key_prefix,
            object_versioning_enabled,
            object_lock_enabled,
            default_object_lock,
            volume_id,
            volume_tier,
            snapshot_required,
            snapshot_evidence_ref,
            filesystem_id,
            filesystem_tier,
            filesystem_mount_policy_ref,
            provider_evidence_refs,
        } = input;

        validate_tenant_id(&tenant_id)?;
        let region = RegionCode::new(region).map_err(|_| CloudStorageError::InvalidResourceId)?;
        let primary_cell_id =
            CellId::new(primary_cell_id).map_err(|_| CloudStorageError::InvalidCellId)?;
        validate_cell_location(&primary_cell_id, &region, None)?;

        let bucket_id = resource_id_for_kind_label(&bucket_id, &tenant_id, &region, "bucket")?;
        let volume_id = resource_id_for(
            &volume_id,
            &tenant_id,
            &region,
            ResourceKind::Volume(volume_tier),
        )?;
        let filesystem_id = resource_id_for(
            &filesystem_id,
            &tenant_id,
            &region,
            ResourceKind::Filesystem(filesystem_tier),
        )?;

        let object_key_prefix = ObjectKey::new(object_key_prefix)?;
        validate_object_key_prefix(&object_key_prefix, &tenant_id, &primary_cell_id)?;
        if !object_versioning_enabled {
            return Err(CloudStorageError::InvalidStorageNamespacePolicy);
        }
        if !object_lock_enabled {
            return Err(CloudStorageError::InvalidObjectLockPolicy);
        }
        validate_object_lock(Some(default_object_lock))?;
        if !snapshot_required {
            return Err(CloudStorageError::InvalidStorageNamespacePolicy);
        }
        let snapshot_evidence_ref =
            validate_metadata_ref(snapshot_evidence_ref, REF_SNAPSHOT_EVIDENCE_PREFIX)?;
        let filesystem_mount_policy_ref =
            validate_metadata_ref(filesystem_mount_policy_ref, REF_MOUNT_POLICY_PREFIX)?;
        let provider_evidence_refs =
            validate_metadata_refs(provider_evidence_refs, REF_EVIDENCE_PREFIX)?;

        Ok(Self {
            tenant_id: internal(tenant_id),
            region: public(region),
            primary_cell_id: public(primary_cell_id),
            bucket_id: internal(bucket_id),
            object_key_prefix: internal(object_key_prefix),
            object_versioning_enabled: internal(object_versioning_enabled),
            object_lock_enabled: internal(object_lock_enabled),
            default_object_lock: internal(default_object_lock),
            volume_id: internal(volume_id),
            volume_tier: public(volume_tier),
            snapshot_required: internal(snapshot_required),
            snapshot_evidence_ref: internal(snapshot_evidence_ref),
            filesystem_id: internal(filesystem_id),
            filesystem_tier: public(filesystem_tier),
            filesystem_mount_policy_ref: internal(filesystem_mount_policy_ref),
            provider_evidence_refs: internal(provider_evidence_refs),
            schema_version: public(STORAGE_SCHEMA_VERSION),
        })
    }
}
