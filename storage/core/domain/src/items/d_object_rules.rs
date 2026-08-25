impl ReplicationPolicy {
    pub const fn mode(&self) -> ReplicationMode {
        match self {
            Self::None => ReplicationMode::None,
            Self::Regional => ReplicationMode::Regional,
            Self::CrossRegion { .. } => ReplicationMode::CrossRegion,
        }
    }
}

const fn required_key_origin(mode: EncryptionMode) -> Option<KmsKeyOrigin> {
    match mode {
        EncryptionMode::Sse => None,
        EncryptionMode::SseKms => Some(KmsKeyOrigin::OyatieManaged),
        EncryptionMode::Byok => Some(KmsKeyOrigin::Byok),
        EncryptionMode::Hyok => Some(KmsKeyOrigin::Hyok),
    }
}

const fn object_key_origin(mode: EncryptionMode) -> KmsKeyOrigin {
    match mode {
        EncryptionMode::Sse | EncryptionMode::SseKms => KmsKeyOrigin::OyatieManaged,
        EncryptionMode::Byok => KmsKeyOrigin::Byok,
        EncryptionMode::Hyok => KmsKeyOrigin::Hyok,
    }
}

impl BucketName {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudStorageError> {
        let value = value.into();
        validate_dns_name(
            &value,
            MAX_BUCKET_NAME_LEN,
            CloudStorageError::InvalidBucketName,
        )?;
        Ok(Self { value })
    }
}

impl ObjectKey {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudStorageError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAX_OBJECT_KEY_LEN
            || value.starts_with('/')
            || value.split('/').any(|segment| segment == "..")
            || value.bytes().any(|byte| byte.is_ascii_control())
        {
            return Err(CloudStorageError::InvalidObjectKey);
        }
        Ok(Self { value })
    }
}

impl ETag {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudStorageError> {
        let value = value.into();
        let unquoted = value.trim_matches('"');
        if unquoted.len() == 32 && unquoted.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            Ok(Self { value })
        } else {
            Err(CloudStorageError::InvalidEtag)
        }
    }
}

impl VolumeName {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudStorageError> {
        canonical_name(value.into(), CloudStorageError::InvalidResourceId)
            .map(|value| Self { value })
    }
}

impl FilesystemName {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudStorageError> {
        canonical_name(value.into(), CloudStorageError::InvalidResourceId)
            .map(|value| Self { value })
    }
}

impl ArchiveVaultName {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudStorageError> {
        canonical_name(value.into(), CloudStorageError::InvalidResourceId)
            .map(|value| Self { value })
    }
}

impl SnapshotId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudStorageError> {
        let value = value.into();
        if value.starts_with(SNAPSHOT_ID_PREFIX) && value.len() > SNAPSHOT_ID_PREFIX.len() {
            Ok(Self { value })
        } else {
            Err(CloudStorageError::InvalidSnapshotId)
        }
    }
}

impl Bucket {
    pub fn new(input: BucketCreate) -> Result<Self, CloudStorageError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != BucketState::Creating {
            return Err(CloudStorageError::InvalidInitialState);
        }
        let region =
            RegionCode::new(input.region).map_err(|_| CloudStorageError::InvalidResourceId)?;
        validate_residency_allows_region(&input.residency, &region)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::Bucket(input.tier),
        )?;
        let name = BucketName::new(input.name)?;
        let replication = replication_policy(input.replication, &input.residency)?;
        let kms_key = encryption_key(input.encryption, input.kms_key, &region, &input.tenant_id)?;
        validate_object_lock(input.object_lock)?;
        let allowed_data_classes = privacy_class_set(input.allowed_data_classes)?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            name: internal(name),
            region: public(region),
            residency: internal(input.residency),
            tier: public(input.tier),
            replication: internal(replication),
            encryption: public(input.encryption),
            kms_key: internal(kms_key),
            object_lock: internal(input.object_lock),
            allowed_data_classes: internal(allowed_data_classes),
            state: public(input.state),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(STORAGE_SCHEMA_VERSION),
        })
    }
}
impl Bucket {
    pub fn activate(&self) -> Result<Self, CloudStorageError> {
        if self.state.value != BucketState::Creating {
            return Err(CloudStorageError::InvalidInitialState);
        }
        let mut bucket = self.clone();
        bucket.state = public(BucketState::Active);
        Ok(bucket)
    }
}

impl ObjectEncryptionBinding {
    pub fn new(
        bucket: &Bucket,
        input: ObjectEncryptionBindingCreate,
    ) -> Result<Self, CloudStorageError> {
        if input.kms_key_version == 0 {
            return Err(CloudStorageError::InvalidKmsKeyVersion);
        }
        if input.purpose != KmsPurpose::CloudObjectStorage {
            return Err(CloudStorageError::InvalidKmsPurpose);
        }
        let kms_key =
            KmsKeyId::new(input.kms_key).map_err(|_| CloudStorageError::InvalidKmsKeyId)?;
        if kms_key
            .origin()
            .map_err(|_| CloudStorageError::InvalidKmsKeyId)?
            != object_key_origin(bucket.encryption.value)
        {
            return Err(CloudStorageError::KmsKeyModeMismatch);
        }
        if kms_key
            .region()
            .map_err(|_| CloudStorageError::InvalidKmsKeyId)?
            != bucket.region.value
        {
            return Err(CloudStorageError::KmsKeyRegionMismatch);
        }
        if kms_key
            .tenant_id()
            .map_err(|_| CloudStorageError::InvalidKmsKeyId)?
            != bucket.tenant_id.value
        {
            return Err(CloudStorageError::KmsKeyTenantMismatch);
        }
        if bucket
            .kms_key
            .value
            .as_ref()
            .is_some_and(|bucket_key| bucket_key != &kms_key)
        {
            return Err(CloudStorageError::InvalidKmsKeyId);
        }
        Ok(Self {
            kms_key,
            kms_key_version: input.kms_key_version,
            material_ref: MaterialRef::new(input.material_ref)
                .map_err(|_| CloudStorageError::InvalidMaterialRef)?,
            ciphertext_ref: CiphertextRef::new(input.ciphertext_ref)
                .map_err(|_| CloudStorageError::InvalidCiphertextRef)?,
            kms_encrypt_event_id: KmsUseEventId::new(input.kms_encrypt_event_id)
                .map_err(|_| CloudStorageError::InvalidKmsUseEventId)?,
            purpose: input.purpose,
            shred_proof_ref: input
                .shred_proof_ref
                .map(DestructionProofRef::new)
                .transpose()
                .map_err(|_| CloudStorageError::InvalidDestructionProofRef)?,
        })
    }
}

impl StoredObject {
    pub fn new(bucket: &Bucket, input: ObjectCreate) -> Result<Self, CloudStorageError> {
        validate_tenant_id(&input.tenant_id)?;
        let bucket_id =
            ResourceId::new(input.bucket_id).map_err(|_| CloudStorageError::InvalidResourceId)?;
        if bucket_id != bucket.resource_id.value {
            return Err(CloudStorageError::UnknownBucket);
        }
        if input.tenant_id != bucket.tenant_id.value {
            return Err(CloudStorageError::ResourceTenantMismatch);
        }
        if !matches!(bucket.state.value, BucketState::Active) {
            return Err(CloudStorageError::InvalidInitialState);
        }
        let data_class = privacy_class(input.data_class)?;
        if !bucket.allowed_data_classes.value.contains(&data_class) {
            return Err(CloudStorageError::ObjectDataClassDenied);
        }
        if let Some(last_accessed_at) = input.last_accessed_at_epoch_seconds {
            validate_time_order(input.stored_at_epoch_seconds, last_accessed_at)?;
        }
        Ok(Self {
            bucket_id: internal(bucket_id),
            tenant_id: internal(input.tenant_id),
            key: internal(ObjectKey::new(input.key)?),
            size_bytes: internal(input.size_bytes),
            etag: internal(ETag::new(input.etag)?),
            data_class: internal(data_class),
            encryption: internal(ObjectEncryptionBinding::new(bucket, input.encryption)?),
            stored_at_epoch_seconds: internal(input.stored_at_epoch_seconds),
            last_accessed_at_epoch_seconds: internal(input.last_accessed_at_epoch_seconds),
            schema_version: public(STORAGE_SCHEMA_VERSION),
        })
    }
}
