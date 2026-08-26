// Archive lifecycle remains object behavior, not a standalone product owner.

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ArchiveVaultName {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ArchiveTier {
    Instant,
    Cold,
    DeepCold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ArchiveVaultState {
    Creating,
    Active,
    Deleting,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveVaultCreate {
    pub resource_id: String,                  // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub name: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: PUBLIC
    pub residency: ResidencyClass,            // data_class: INTERNAL_ONLY
    pub tier: ArchiveTier,                    // data_class: PUBLIC
    pub encryption: EncryptionMode,           // data_class: PUBLIC
    pub kms_key: Option<String>,              // data_class: INTERNAL_ONLY
    pub allowed_data_classes: Vec<DataClass>, // data_class: INTERNAL_ONLY
    pub state: ArchiveVaultState,             // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveVault {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub name: Classified<ArchiveVaultName>,  // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub tier: Classified<ArchiveTier>,       // data_class: PUBLIC
    pub encryption: Classified<EncryptionMode>, // data_class: PUBLIC
    pub kms_key: Classified<Option<KmsKeyId>>, // data_class: INTERNAL_ONLY
    pub allowed_data_classes: Classified<BTreeSet<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub state: Classified<ArchiveVaultState>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

impl ArchiveVaultName {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudStorageError> {
        canonical_name(value.into(), CloudStorageError::InvalidResourceId)
            .map(|value| Self { value })
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
