impl StorageRepo for CloudStorageCatalog {
    fn create_bucket(&mut self, input: BucketCreate) -> Result<Bucket, CloudStorageError> {
        let bucket = Bucket::new(input)?;
        if self.buckets.contains_key(&bucket.resource_id.value) {
            return Err(CloudStorageError::DuplicateBucket);
        }
        self.buckets
            .insert(bucket.resource_id.value.clone(), bucket.clone());
        Ok(bucket)
    }

    fn put_object(&mut self, input: ObjectCreate) -> Result<StoredObject, CloudStorageError> {
        let bucket_id = ResourceId::new(input.bucket_id.clone())
            .map_err(|_| CloudStorageError::InvalidResourceId)?;
        let bucket = self
            .buckets
            .get(&bucket_id)
            .ok_or(CloudStorageError::UnknownBucket)?;
        let object = StoredObject::new(bucket, input)?;
        let key = (object.bucket_id.value.clone(), object.key.value.clone());
        if self.objects.contains_key(&key) {
            return Err(CloudStorageError::DuplicateObject);
        }
        self.objects.insert(key, object.clone());
        Ok(object)
    }

    fn create_volume(&mut self, input: VolumeCreate) -> Result<BlockVolume, CloudStorageError> {
        let volume = BlockVolume::new(input)?;
        if self.volumes.contains_key(&volume.resource_id.value) {
            return Err(CloudStorageError::DuplicateVolume);
        }
        self.volumes
            .insert(volume.resource_id.value.clone(), volume.clone());
        Ok(volume)
    }

    fn create_filesystem(
        &mut self,
        input: FilesystemCreate,
    ) -> Result<CloudFilesystem, CloudStorageError> {
        let filesystem = CloudFilesystem::new(input)?;
        if self.filesystems.contains_key(&filesystem.resource_id.value) {
            return Err(CloudStorageError::DuplicateFilesystem);
        }
        self.filesystems
            .insert(filesystem.resource_id.value.clone(), filesystem.clone());
        Ok(filesystem)
    }

    fn create_archive_vault(
        &mut self,
        input: ArchiveVaultCreate,
    ) -> Result<ArchiveVault, CloudStorageError> {
        let vault = ArchiveVault::new(input)?;
        if self.archive_vaults.contains_key(&vault.resource_id.value) {
            return Err(CloudStorageError::DuplicateArchiveVault);
        }
        self.archive_vaults
            .insert(vault.resource_id.value.clone(), vault.clone());
        Ok(vault)
    }

    fn create_snapshot(
        &mut self,
        input: SnapshotCreate,
    ) -> Result<VolumeSnapshot, CloudStorageError> {
        let source_volume_id = ResourceId::new(input.source_volume_id.clone())
            .map_err(|_| CloudStorageError::InvalidResourceId)?;
        let source_volume = self
            .volumes
            .get(&source_volume_id)
            .ok_or(CloudStorageError::UnknownVolume)?;
        let snapshot = VolumeSnapshot::new(source_volume, input)?;
        if self.snapshots.contains_key(&snapshot.id.value) {
            return Err(CloudStorageError::DuplicateSnapshot);
        }
        self.snapshots
            .insert(snapshot.id.value.clone(), snapshot.clone());
        Ok(snapshot)
    }
}

impl CloudStorageCatalog {
    pub fn activate_bucket(&mut self, bucket_id: &str) -> Result<Bucket, CloudStorageError> {
        let bucket_id = ResourceId::new(bucket_id.to_string())
            .map_err(|_| CloudStorageError::InvalidResourceId)?;
        let bucket = self
            .buckets
            .get(&bucket_id)
            .ok_or(CloudStorageError::UnknownBucket)?;
        let active = bucket.activate()?;
        self.buckets.insert(bucket_id, active.clone());
        Ok(active)
    }

    pub fn buckets(&self) -> impl Iterator<Item = &Bucket> {
        self.buckets.values()
    }

    pub fn objects(&self) -> impl Iterator<Item = &StoredObject> {
        self.objects.values()
    }

    pub fn volumes(&self) -> impl Iterator<Item = &BlockVolume> {
        self.volumes.values()
    }

    pub fn filesystems(&self) -> impl Iterator<Item = &CloudFilesystem> {
        self.filesystems.values()
    }

    pub fn archive_vaults(&self) -> impl Iterator<Item = &ArchiveVault> {
        self.archive_vaults.values()
    }

    pub fn snapshots(&self) -> impl Iterator<Item = &VolumeSnapshot> {
        self.snapshots.values()
    }
}

fn resource_id_for(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    kind: ResourceKind,
) -> Result<ResourceId, CloudStorageError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudStorageError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudStorageError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != kind.type_label() {
        return Err(CloudStorageError::ResourceKindMismatch);
    }
    Ok(id)
}

fn resource_id_for_kind_label(
    value: &str,
    tenant_id: &str,
    region: &RegionCode,
    kind_label: &str,
) -> Result<ResourceId, CloudStorageError> {
    let id = ResourceId::new(value.to_string()).map_err(map_resource_error)?;
    if id.tenant_id().map_err(map_resource_error)? != tenant_id {
        return Err(CloudStorageError::ResourceTenantMismatch);
    }
    if id.region().map_err(map_resource_error)? != *region {
        return Err(CloudStorageError::ResourceRegionMismatch);
    }
    if id.kind_label().map_err(map_resource_error)? != kind_label {
        return Err(CloudStorageError::ResourceKindMismatch);
    }
    Ok(id)
}

fn replication_policy(
    input: ReplicationPolicyCreate,
    residency: &ResidencyClass,
) -> Result<ReplicationPolicy, CloudStorageError> {
    match input {
        ReplicationPolicyCreate::None => Ok(ReplicationPolicy::None),
        ReplicationPolicyCreate::Regional => Ok(ReplicationPolicy::Regional),
        ReplicationPolicyCreate::CrossRegion {
            destination_regions,
        } => {
            if destination_regions.is_empty() {
                return Err(CloudStorageError::InvalidReplicationPolicy);
            }
            let mut seen = BTreeSet::new();
            let mut typed = Vec::with_capacity(destination_regions.len());
            for value in destination_regions {
                let region = RegionCode::new(value)
                    .map_err(|_| CloudStorageError::InvalidReplicationPolicy)?;
                if !seen.insert(region.clone()) {
                    return Err(CloudStorageError::DuplicateReplicationRegion);
                }
                if !residency_class_allows_home_region_label(residency, &region.value) {
                    return Err(CloudStorageError::ReplicationResidencyDenied);
                }
                typed.push(region);
            }
            Ok(ReplicationPolicy::CrossRegion {
                destination_regions: typed,
            })
        }
    }
}

fn encryption_key(
    mode: EncryptionMode,
    key: Option<String>,
    region: &RegionCode,
    tenant_id: &str,
) -> Result<Option<KmsKeyId>, CloudStorageError> {
    let Some(expected_origin) = mode.required_key_origin() else {
        if key.is_some() {
            return Err(CloudStorageError::UnexpectedKmsKey);
        }
        return Ok(None);
    };
    let Some(key) = key else {
        return Err(CloudStorageError::MissingKmsKey);
    };
    let key = KmsKeyId::new(key).map_err(|_| CloudStorageError::InvalidKmsKeyId)?;
    if key
        .origin()
        .map_err(|_| CloudStorageError::InvalidKmsKeyId)?
        != expected_origin
    {
        return Err(CloudStorageError::KmsKeyModeMismatch);
    }
    if key
        .region()
        .map_err(|_| CloudStorageError::InvalidKmsKeyId)?
        != *region
    {
        return Err(CloudStorageError::KmsKeyRegionMismatch);
    }
    if key
        .tenant_id()
        .map_err(|_| CloudStorageError::InvalidKmsKeyId)?
        != tenant_id
    {
        return Err(CloudStorageError::KmsKeyTenantMismatch);
    }
    Ok(Some(key))
}

fn privacy_class(data_class: DataClass) -> Result<PrivacyDataClass, CloudStorageError> {
    PrivacyDataClass::new(data_class).map_err(|_| CloudStorageError::InvalidDataClass)
}
