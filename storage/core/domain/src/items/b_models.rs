#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredObject {
    pub bucket_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub key: Classified<ObjectKey>,        // data_class: INTERNAL_ONLY
    pub size_bytes: Classified<u64>,       // data_class: INTERNAL_ONLY
    pub etag: Classified<ETag>,            // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub encryption: Classified<ObjectEncryptionBinding>, // data_class: INTERNAL_ONLY
    pub stored_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub last_accessed_at_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeCreate {
    pub resource_id: String,            // data_class: INTERNAL_ONLY
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub name: String,                   // data_class: INTERNAL_ONLY
    pub region: String,                 // data_class: PUBLIC
    pub az: String,                     // data_class: PUBLIC
    pub cell_id: String,                // data_class: PUBLIC
    pub residency: ResidencyClass,      // data_class: INTERNAL_ONLY
    pub tier: VolumeTier,               // data_class: PUBLIC
    pub size_gib: u64,                  // data_class: INTERNAL_ONLY
    pub performance: VolumePerformance, // data_class: PUBLIC
    pub encryption: EncryptionMode,     // data_class: PUBLIC
    pub kms_key: Option<String>,        // data_class: INTERNAL_ONLY
    pub data_class: DataClass,          // data_class: INTERNAL_ONLY
    pub state: VolumeState,             // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockVolume {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub name: Classified<VolumeName>,        // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub az: Classified<AzCode>,              // data_class: PUBLIC
    pub cell_id: Classified<CellId>,         // data_class: PUBLIC
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub tier: Classified<VolumeTier>,        // data_class: PUBLIC
    pub size_gib: Classified<u64>,           // data_class: INTERNAL_ONLY
    pub performance: Classified<VolumePerformance>, // data_class: PUBLIC
    pub encryption: Classified<EncryptionMode>, // data_class: PUBLIC
    pub kms_key: Classified<Option<KmsKeyId>>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub state: Classified<VolumeState>,      // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}
