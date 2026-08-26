// Retirement-bound EFS-like compatibility model.
//
// ADR-0719 has no file-service owner. This shard preserves the historical
// root exports and behavior without misclassifying them as Drive or active
// storage-engine scope. New consumers are forbidden.

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FilesystemName {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FilesystemState {
    Creating,
    Available,
    Mounted,
    Deleting,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FilesystemCreate {
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub name: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub az: String,                    // data_class: PUBLIC
    pub cell_id: String,               // data_class: PUBLIC
    pub residency: ResidencyClass,     // data_class: INTERNAL_ONLY
    pub tier: FilesystemTier,          // data_class: PUBLIC
    pub size_gib: u64,                 // data_class: INTERNAL_ONLY
    pub throughput_mbps: u64,          // data_class: PUBLIC
    pub encryption: EncryptionMode,    // data_class: PUBLIC
    pub kms_key: Option<String>,       // data_class: INTERNAL_ONLY
    pub data_class: DataClass,         // data_class: INTERNAL_ONLY
    pub state: FilesystemState,        // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudFilesystem {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub name: Classified<FilesystemName>,    // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub az: Classified<AzCode>,              // data_class: PUBLIC
    pub cell_id: Classified<CellId>,         // data_class: PUBLIC
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub tier: Classified<FilesystemTier>,    // data_class: PUBLIC
    pub size_gib: Classified<u64>,           // data_class: INTERNAL_ONLY
    pub throughput_mbps: Classified<u64>,    // data_class: PUBLIC
    pub encryption: Classified<EncryptionMode>, // data_class: PUBLIC
    pub kms_key: Classified<Option<KmsKeyId>>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub state: Classified<FilesystemState>,  // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

impl FilesystemName {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudStorageError> {
        canonical_name(value.into(), CloudStorageError::InvalidResourceId)
            .map(|value| Self { value })
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
