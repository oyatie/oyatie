const STORAGE_SCHEMA_VERSION: u32 = 1;
const TENANT_ID_PREFIX: &str = "ten_";
const SNAPSHOT_ID_PREFIX: &str = "snap_";
const REF_EVIDENCE_PREFIX: &str = "evidence/";
const REF_SNAPSHOT_EVIDENCE_PREFIX: &str = "snapshot-evidence/";
const REF_MOUNT_POLICY_PREFIX: &str = "mount-policy/";
const MAX_BUCKET_NAME_LEN: usize = 63;
const MAX_OBJECT_KEY_LEN: usize = 1024;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BucketName {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObjectKey {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ETag {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VolumeName {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct FilesystemName {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ArchiveVaultName {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SnapshotId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ReplicationMode {
    None,
    Regional,
    CrossRegion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplicationPolicyCreate {
    None,
    Regional,
    CrossRegion { destination_regions: Vec<String> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplicationPolicy {
    None,
    Regional,
    CrossRegion {
        destination_regions: Vec<RegionCode>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ObjectLockMode {
    Governance,
    Compliance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObjectLockPolicy {
    pub mode: ObjectLockMode,            // data_class: PUBLIC
    pub retain_until_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub legal_hold: bool,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BucketState {
    Creating,
    Active,
    Suspended,
    Deleting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VolumeState {
    Creating,
    Available,
    Attached,
    Deleting,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FilesystemState {
    Creating,
    Available,
    Mounted,
    Deleting,
    Error,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SnapshotState {
    Creating,
    Complete,
    Deleting,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BucketCreate {
    pub resource_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub name: String,                          // data_class: INTERNAL_ONLY
    pub region: String,                        // data_class: PUBLIC
    pub residency: ResidencyClass,             // data_class: INTERNAL_ONLY
    pub tier: BucketTier,                      // data_class: PUBLIC
    pub replication: ReplicationPolicyCreate,  // data_class: INTERNAL_ONLY
    pub encryption: EncryptionMode,            // data_class: PUBLIC
    pub kms_key: Option<String>,               // data_class: INTERNAL_ONLY
    pub object_lock: Option<ObjectLockPolicy>, // data_class: INTERNAL_ONLY
    pub allowed_data_classes: Vec<DataClass>,  // data_class: INTERNAL_ONLY
    pub state: BucketState,                    // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bucket {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub name: Classified<BucketName>,        // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub tier: Classified<BucketTier>,        // data_class: PUBLIC
    pub replication: Classified<ReplicationPolicy>, // data_class: INTERNAL_ONLY
    pub encryption: Classified<EncryptionMode>, // data_class: PUBLIC
    pub kms_key: Classified<Option<KmsKeyId>>, // data_class: INTERNAL_ONLY
    pub object_lock: Classified<Option<ObjectLockPolicy>>, // data_class: INTERNAL_ONLY
    pub allowed_data_classes: Classified<BTreeSet<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub state: Classified<BucketState>,      // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectCreate {
    pub bucket_id: String,                           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                           // data_class: INTERNAL_ONLY
    pub key: String,                                 // data_class: INTERNAL_ONLY
    pub size_bytes: u64,                             // data_class: INTERNAL_ONLY
    pub etag: String,                                // data_class: INTERNAL_ONLY
    pub data_class: DataClass,                       // data_class: INTERNAL_ONLY
    pub encryption: ObjectEncryptionBindingCreate,   // data_class: INTERNAL_ONLY
    pub stored_at_epoch_seconds: u64,                // data_class: INTERNAL_ONLY
    pub last_accessed_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectEncryptionBindingCreate {
    pub kms_key: String,                 // data_class: INTERNAL_ONLY
    pub kms_key_version: u32,            // data_class: INTERNAL_ONLY
    pub material_ref: String,            // data_class: INTERNAL_ONLY
    pub ciphertext_ref: String,          // data_class: INTERNAL_ONLY
    pub kms_encrypt_event_id: String,    // data_class: INTERNAL_ONLY
    pub purpose: KmsPurpose,             // data_class: INTERNAL_ONLY
    pub shred_proof_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectEncryptionBinding {
    pub kms_key: KmsKeyId,                            // data_class: INTERNAL_ONLY
    pub kms_key_version: u32,                         // data_class: INTERNAL_ONLY
    pub material_ref: MaterialRef,                    // data_class: INTERNAL_ONLY
    pub ciphertext_ref: CiphertextRef,                // data_class: INTERNAL_ONLY
    pub kms_encrypt_event_id: KmsUseEventId,          // data_class: INTERNAL_ONLY
    pub purpose: KmsPurpose,                          // data_class: INTERNAL_ONLY
    pub shred_proof_ref: Option<DestructionProofRef>, // data_class: INTERNAL_ONLY
}
