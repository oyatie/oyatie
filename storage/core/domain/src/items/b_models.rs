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
pub struct StorageProviderObjectPutRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_bucket_ref: String,     // data_class: INTERNAL_ONLY
    pub bucket_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub object_key: String,              // data_class: INTERNAL_ONLY
    pub object_body_ref: String,         // data_class: INTERNAL_ONLY
    pub size_bytes: u64,                 // data_class: INTERNAL_ONLY
    pub etag: String,                    // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: INTERNAL_ONLY
    pub kms_key: String,                 // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProviderObjectGetRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_bucket_ref: String,     // data_class: INTERNAL_ONLY
    pub bucket_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub object_key: String,              // data_class: INTERNAL_ONLY
    pub result_body_ref: String,         // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProviderObjectReceipt {
    pub provider: StorageProviderKind,     // data_class: PUBLIC
    pub operation: StorageObjectOperation, // data_class: PUBLIC
    pub request_id: String,                // data_class: INTERNAL_ONLY
    pub provider_request_id: String,       // data_class: INTERNAL_ONLY
    pub provider_bucket_ref: String,       // data_class: INTERNAL_ONLY
    pub bucket_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub object_key: String,                // data_class: INTERNAL_ONLY
    pub object_body_ref: String,           // data_class: INTERNAL_ONLY
    pub size_bytes: Option<u64>,           // data_class: INTERNAL_ONLY
    pub etag: Option<String>,              // data_class: INTERNAL_ONLY
    pub data_class: Option<DataClass>,     // data_class: INTERNAL_ONLY
    pub kms_key: Option<String>,           // data_class: INTERNAL_ONLY
    pub ciphertext_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub actor: String,                     // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,     // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,               // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProviderBlockCreateVolumeRequest {
    pub request_id: String,              // data_class: INTERNAL_ONLY
    pub provider_volume_ref: String,     // data_class: INTERNAL_ONLY
    pub volume_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub name: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                  // data_class: PUBLIC
    pub az: String,                      // data_class: PUBLIC
    pub cell_id: String,                 // data_class: PUBLIC
    pub residency: ResidencyClass,       // data_class: INTERNAL_ONLY
    pub tier: VolumeTier,                // data_class: PUBLIC
    pub size_gib: u64,                   // data_class: INTERNAL_ONLY
    pub performance: VolumePerformance,  // data_class: PUBLIC
    pub encryption: EncryptionMode,      // data_class: PUBLIC
    pub kms_key: Option<String>,         // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: INTERNAL_ONLY
    pub actor: String,                   // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProviderBlockReceipt {
    pub provider: StorageProviderKind,    // data_class: PUBLIC
    pub operation: StorageBlockOperation, // data_class: PUBLIC
    pub request_id: String,               // data_class: INTERNAL_ONLY
    pub provider_request_id: String,      // data_class: INTERNAL_ONLY
    pub provider_volume_ref: String,      // data_class: INTERNAL_ONLY
    pub volume_id: String,                // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub name: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                   // data_class: PUBLIC
    pub az: String,                       // data_class: PUBLIC
    pub cell_id: String,                  // data_class: PUBLIC
    pub residency: ResidencyClass,        // data_class: INTERNAL_ONLY
    pub tier: VolumeTier,                 // data_class: PUBLIC
    pub size_gib: u64,                    // data_class: INTERNAL_ONLY
    pub performance: VolumePerformance,   // data_class: PUBLIC
    pub encryption: EncryptionMode,       // data_class: PUBLIC
    pub kms_key: Option<String>,          // data_class: INTERNAL_ONLY
    pub data_class: DataClass,            // data_class: INTERNAL_ONLY
    pub actor: String,                    // data_class: INTERNAL_ONLY
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub provider_evidence_ref: String,    // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub schema_version: u32,              // data_class: PUBLIC
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotCreate {
    pub id: String,                    // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub source_volume_id: String,      // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub data_class: DataClass,         // data_class: INTERNAL_ONLY
    pub state: SnapshotState,          // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VolumeSnapshot {
    pub id: Classified<SnapshotId>,    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub source_volume_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub state: Classified<SnapshotState>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageTenantCellGuardrailCreate {
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                        // data_class: PUBLIC
    pub primary_cell_id: String,               // data_class: PUBLIC
    pub bucket_id: String,                     // data_class: INTERNAL_ONLY
    pub object_key_prefix: String,             // data_class: INTERNAL_ONLY
    pub object_versioning_enabled: bool,       // data_class: INTERNAL_ONLY
    pub object_lock_enabled: bool,             // data_class: INTERNAL_ONLY
    pub default_object_lock: ObjectLockPolicy, // data_class: INTERNAL_ONLY
    pub volume_id: String,                     // data_class: INTERNAL_ONLY
    pub volume_tier: VolumeTier,               // data_class: PUBLIC
    pub snapshot_required: bool,               // data_class: INTERNAL_ONLY
    pub snapshot_evidence_ref: String,         // data_class: INTERNAL_ONLY
    pub filesystem_id: String,                 // data_class: INTERNAL_ONLY
    pub filesystem_tier: FilesystemTier,       // data_class: PUBLIC
    pub filesystem_mount_policy_ref: String,   // data_class: INTERNAL_ONLY
    pub provider_evidence_refs: Vec<String>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageTenantCellGuardrail {
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub primary_cell_id: Classified<CellId>, // data_class: PUBLIC
    pub bucket_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub object_key_prefix: Classified<ObjectKey>, // data_class: INTERNAL_ONLY
    pub object_versioning_enabled: Classified<bool>, // data_class: INTERNAL_ONLY
    pub object_lock_enabled: Classified<bool>, // data_class: INTERNAL_ONLY
    pub default_object_lock: Classified<ObjectLockPolicy>, // data_class: INTERNAL_ONLY
    pub volume_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub volume_tier: Classified<VolumeTier>, // data_class: PUBLIC
    pub snapshot_required: Classified<bool>, // data_class: INTERNAL_ONLY
    pub snapshot_evidence_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub filesystem_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub filesystem_tier: Classified<FilesystemTier>, // data_class: PUBLIC
    pub filesystem_mount_policy_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub provider_evidence_refs: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}
