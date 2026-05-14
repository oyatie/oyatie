//! Cloud storage aggregate kernel.
//!
//! This crate owns the Cloud storage product surface named by the Cloud PRD and
//! SPEC (`cloud.storage.object.put` / `.get`, `cloud.storage.block.create`, and
//! the file/archive metadata surfaces). It is intentionally adapter-free: object
//! bodies, block devices, and file shares live behind later infrastructure
//! adapters, while this kernel keeps the typed control/data-plane invariants for
//! resource identity, residency, encryption binding, object-lock, and data-class
//! admission.

use std::collections::{BTreeMap, BTreeSet};

use oya_cloud_kms_domain::{
    CiphertextRef, DestructionProofRef, KmsKeyId, KmsKeyOrigin, KmsPurpose, KmsUseEventId,
    MaterialRef,
};
use oya_cloud_region_domain::{AzCode, CellId, RegionCode};
pub use oya_cloud_resource_domain::{BucketTier, FilesystemTier, VolumeTier};
use oya_cloud_resource_domain::{CloudResourceError, ResourceId, ResourceKind};
use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use oya_residency_domain::{ResidencyClass, residency_class_allows_home_region_label};

const STORAGE_SCHEMA_VERSION: u32 = 1;
const TENANT_ID_PREFIX: &str = "ten_";
const SNAPSHOT_ID_PREFIX: &str = "snap_";
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
pub enum EncryptionMode {
    Sse,
    SseKms,
    Byok,
    Hyok,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VolumePerformance {
    pub iops: u64,            // data_class: PUBLIC
    pub throughput_mbps: u64, // data_class: PUBLIC
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
pub enum CloudStorageError {
    InvalidTenantId,
    InvalidResourceId,
    ResourceTenantMismatch,
    ResourceRegionMismatch,
    ResourceKindMismatch,
    InvalidBucketName,
    InvalidObjectKey,
    InvalidEtag,
    InvalidKmsKeyId,
    InvalidKmsKeyVersion,
    InvalidKmsUseEventId,
    InvalidMaterialRef,
    InvalidCiphertextRef,
    InvalidKmsPurpose,
    InvalidDestructionProofRef,
    MissingKmsKey,
    UnexpectedKmsKey,
    KmsKeyModeMismatch,
    KmsKeyTenantMismatch,
    KmsKeyRegionMismatch,
    InvalidReplicationPolicy,
    DuplicateReplicationRegion,
    ReplicationResidencyDenied,
    EmptyAllowedDataClassSet,
    DuplicateDataClass,
    InvalidDataClass,
    ObjectDataClassDenied,
    InvalidObjectLockPolicy,
    InvalidSize,
    InvalidPerformance,
    InvalidAzCode,
    InvalidCellId,
    AzRegionMismatch,
    CellLocationMismatch,
    InvalidSnapshotId,
    InvalidInitialState,
    InvalidTimeOrder,
    DuplicateBucket,
    UnknownBucket,
    DuplicateObject,
    DuplicateVolume,
    UnknownVolume,
    DuplicateFilesystem,
    DuplicateArchiveVault,
    DuplicateSnapshot,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudStorageCatalog {
    buckets: BTreeMap<ResourceId, Bucket>,
    objects: BTreeMap<(ResourceId, ObjectKey), StoredObject>,
    volumes: BTreeMap<ResourceId, BlockVolume>,
    filesystems: BTreeMap<ResourceId, CloudFilesystem>,
    archive_vaults: BTreeMap<ResourceId, ArchiveVault>,
    snapshots: BTreeMap<SnapshotId, VolumeSnapshot>,
}

pub trait StorageRepo {
    fn create_bucket(&mut self, input: BucketCreate) -> Result<Bucket, CloudStorageError>;
    fn put_object(&mut self, input: ObjectCreate) -> Result<StoredObject, CloudStorageError>;
    fn create_volume(&mut self, input: VolumeCreate) -> Result<BlockVolume, CloudStorageError>;
    fn create_filesystem(
        &mut self,
        input: FilesystemCreate,
    ) -> Result<CloudFilesystem, CloudStorageError>;
    fn create_archive_vault(
        &mut self,
        input: ArchiveVaultCreate,
    ) -> Result<ArchiveVault, CloudStorageError>;
    fn create_snapshot(
        &mut self,
        input: SnapshotCreate,
    ) -> Result<VolumeSnapshot, CloudStorageError>;
}

impl ReplicationPolicy {
    pub const fn mode(&self) -> ReplicationMode {
        match self {
            Self::None => ReplicationMode::None,
            Self::Regional => ReplicationMode::Regional,
            Self::CrossRegion { .. } => ReplicationMode::CrossRegion,
        }
    }
}

impl EncryptionMode {
    const fn required_key_origin(self) -> Option<KmsKeyOrigin> {
        match self {
            Self::Sse => None,
            Self::SseKms => Some(KmsKeyOrigin::OyatieManaged),
            Self::Byok => Some(KmsKeyOrigin::Byok),
            Self::Hyok => Some(KmsKeyOrigin::Hyok),
        }
    }

    const fn object_key_origin(self) -> KmsKeyOrigin {
        match self {
            Self::Sse | Self::SseKms => KmsKeyOrigin::OyatieManaged,
            Self::Byok => KmsKeyOrigin::Byok,
            Self::Hyok => KmsKeyOrigin::Hyok,
        }
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
            != bucket.encryption.value.object_key_origin()
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

fn privacy_class_set(
    data_classes: Vec<DataClass>,
) -> Result<BTreeSet<PrivacyDataClass>, CloudStorageError> {
    if data_classes.is_empty() {
        return Err(CloudStorageError::EmptyAllowedDataClassSet);
    }
    let mut typed = BTreeSet::new();
    for data_class in data_classes {
        let data_class = privacy_class(data_class)?;
        if !typed.insert(data_class) {
            return Err(CloudStorageError::DuplicateDataClass);
        }
    }
    Ok(typed)
}

fn validate_object_lock(policy: Option<ObjectLockPolicy>) -> Result<(), CloudStorageError> {
    if policy.is_some_and(|policy| policy.retain_until_epoch_seconds == 0 && !policy.legal_hold) {
        Err(CloudStorageError::InvalidObjectLockPolicy)
    } else {
        Ok(())
    }
}

fn validate_residency_allows_region(
    residency: &ResidencyClass,
    region: &RegionCode,
) -> Result<(), CloudStorageError> {
    if residency_class_allows_home_region_label(residency, &region.value) {
        Ok(())
    } else {
        Err(CloudStorageError::ReplicationResidencyDenied)
    }
}

fn validate_tenant_id(value: &str) -> Result<(), CloudStorageError> {
    if value.starts_with(TENANT_ID_PREFIX) && value.len() > TENANT_ID_PREFIX.len() {
        Ok(())
    } else {
        Err(CloudStorageError::InvalidTenantId)
    }
}

fn validate_size(value: u64) -> Result<(), CloudStorageError> {
    if value > 0 {
        Ok(())
    } else {
        Err(CloudStorageError::InvalidSize)
    }
}

fn validate_performance(value: VolumePerformance) -> Result<(), CloudStorageError> {
    if value.iops > 0 && value.throughput_mbps > 0 {
        Ok(())
    } else {
        Err(CloudStorageError::InvalidPerformance)
    }
}

fn validate_time_order(start: u64, end: u64) -> Result<(), CloudStorageError> {
    if end >= start {
        Ok(())
    } else {
        Err(CloudStorageError::InvalidTimeOrder)
    }
}

fn validate_az_region(az: &AzCode, region: &RegionCode) -> Result<(), CloudStorageError> {
    if az.value == region.value
        || az
            .value
            .strip_prefix(&region.value)
            .is_some_and(|suffix| suffix.starts_with('-') && suffix.len() > 1)
    {
        Ok(())
    } else {
        Err(CloudStorageError::AzRegionMismatch)
    }
}

fn validate_cell_location(
    cell_id: &CellId,
    region: &RegionCode,
    az: Option<&AzCode>,
) -> Result<(), CloudStorageError> {
    let expected_prefix = match az {
        Some(az) => format!("cell-{}-", az.value),
        None => format!("cell-{}-", region.value),
    };
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudStorageError::CellLocationMismatch)
    }
}

fn validate_dns_name(
    value: &str,
    max_len: usize,
    error: CloudStorageError,
) -> Result<(), CloudStorageError> {
    if value.len() < 3
        || value.len() > max_len
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("..")
        || value.contains("--")
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'.'
        })
    {
        return Err(error);
    }
    Ok(())
}

fn canonical_name(value: String, error: CloudStorageError) -> Result<String, CloudStorageError> {
    validate_canonical_segment(&value, error)?;
    Ok(value)
}

fn validate_canonical_segment(
    value: &str,
    error: CloudStorageError,
) -> Result<(), CloudStorageError> {
    if value.trim().is_empty()
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(error);
    }
    Ok(())
}

fn map_resource_error(error: CloudResourceError) -> CloudStorageError {
    match error {
        CloudResourceError::InvalidResourceId => CloudStorageError::InvalidResourceId,
        CloudResourceError::ResourceIdTenantMismatch => CloudStorageError::ResourceTenantMismatch,
        CloudResourceError::ResourceIdRegionMismatch => CloudStorageError::ResourceRegionMismatch,
        CloudResourceError::ResourceIdKindMismatch => CloudStorageError::ResourceKindMismatch,
        CloudResourceError::InvalidTenantId => CloudStorageError::InvalidTenantId,
        _ => CloudStorageError::InvalidResourceId,
    }
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bucket_create() -> BucketCreate {
        BucketCreate {
            resource_id: "oya:cloud:kr-seoul:ten_kr:bucket:tenant-assets".to_string(),
            tenant_id: "ten_kr".to_string(),
            name: "tenant-assets".to_string(),
            region: "kr-seoul".to_string(),
            residency: ResidencyClass::StrictKr,
            tier: BucketTier::Standard,
            replication: ReplicationPolicyCreate::Regional,
            encryption: EncryptionMode::SseKms,
            kms_key: Some("kms/kr-seoul/ten_kr/object-key".to_string()),
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
            kms_key: "kms/kr-seoul/ten_kr/object-key".to_string(),
            kms_key_version: 1,
            material_ref: "matref/ten_kr/object/report".to_string(),
            ciphertext_ref: "ct/ten_kr/object/report".to_string(),
            kms_encrypt_event_id: "kmsuse_object_report_001".to_string(),
            purpose: KmsPurpose::CloudObjectStorage,
            shred_proof_ref: None,
        }
    }

    fn volume_create() -> VolumeCreate {
        VolumeCreate {
            resource_id: "oya:cloud:kr-seoul:ten_kr:volume:db-primary".to_string(),
            tenant_id: "ten_kr".to_string(),
            name: "db-primary".to_string(),
            region: "kr-seoul".to_string(),
            az: "kr-seoul-a".to_string(),
            cell_id: "cell-kr-seoul-a-001".to_string(),
            residency: ResidencyClass::StrictKr,
            tier: VolumeTier::ProvisionedIopsSsd,
            size_gib: 512,
            performance: VolumePerformance {
                iops: 12_000,
                throughput_mbps: 750,
            },
            encryption: EncryptionMode::Byok,
            kms_key: Some("byok/kr-seoul/ten_kr/db-key".to_string()),
            data_class: DataClass::PiiIdentifying,
            state: VolumeState::Creating,
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    #[test]
    fn creates_bucket_with_resource_residency_encryption_and_data_class_policy() {
        let bucket = Bucket::new(bucket_create()).expect("bucket is valid");

        assert_eq!(bucket.resource_id.value.kind_label().unwrap(), "bucket");
        assert_eq!(bucket.region.value.value, "kr-seoul");
        assert_eq!(bucket.name.value.value, "tenant-assets");
        assert_eq!(bucket.replication.value.mode(), ReplicationMode::Regional);
        assert_eq!(bucket.encryption.value, EncryptionMode::SseKms);
        assert!(bucket.kms_key.value.is_some());
        assert!(
            bucket
                .allowed_data_classes
                .value
                .contains(&PrivacyDataClass::new(DataClass::PiiIdentifying).unwrap())
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
                resource_id: "oya:cloud:kr-seoul:ten_kr:archive-vault:state-test".to_string(),
                tenant_id: "ten_kr".to_string(),
                name: "state-test".to_string(),
                region: "kr-seoul".to_string(),
                residency: ResidencyClass::StrictKr,
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
            resource_id: "oya:cloud:kr-seoul:ten_kr:volume:tenant-assets".to_string(),
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
                destination_regions: vec!["us-east".to_string()],
            },
            ..bucket_create()
        })
        .expect_err("strict KR buckets cannot replicate to US");

        assert_eq!(error, CloudStorageError::ReplicationResidencyDenied);
    }

    #[test]
    fn puts_object_only_when_bucket_allows_the_object_data_class() {
        let bucket = active_bucket();
        let object = StoredObject::new(
            &bucket,
            ObjectCreate {
                bucket_id: bucket.resource_id.value.value.clone(),
                tenant_id: "ten_kr".to_string(),
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
                tenant_id: "ten_kr".to_string(),
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

    #[test]
    fn rejects_object_kms_binding_that_does_not_match_bucket_policy() {
        let bucket = active_bucket();
        let bad_version = StoredObject::new(
            &bucket,
            ObjectCreate {
                bucket_id: bucket.resource_id.value.value.clone(),
                tenant_id: "ten_kr".to_string(),
                key: "workspace/bad-version.pdf".to_string(),
                size_bytes: 42,
                etag: "0123456789abcdef0123456789abcdef".to_string(),
                data_class: DataClass::PiiIdentifying,
                encryption: ObjectEncryptionBindingCreate {
                    kms_key_version: 0,
                    ..object_encryption()
                },
                stored_at_epoch_seconds: 1_700_000_010,
                last_accessed_at_epoch_seconds: None,
            },
        )
        .expect_err("object KMS binding must name a concrete key version");
        assert_eq!(bad_version, CloudStorageError::InvalidKmsKeyVersion);

        let wrong_key = StoredObject::new(
            &bucket,
            ObjectCreate {
                bucket_id: bucket.resource_id.value.value.clone(),
                tenant_id: "ten_kr".to_string(),
                key: "workspace/wrong-key.pdf".to_string(),
                size_bytes: 42,
                etag: "0123456789abcdef0123456789abcdef".to_string(),
                data_class: DataClass::PiiIdentifying,
                encryption: ObjectEncryptionBindingCreate {
                    kms_key: "byok/kr-seoul/ten_kr/object-key".to_string(),
                    kms_encrypt_event_id: "kmsuse_object_wrong_key_001".to_string(),
                    ..object_encryption()
                },
                stored_at_epoch_seconds: 1_700_000_010,
                last_accessed_at_epoch_seconds: None,
            },
        )
        .expect_err("object KMS key origin must match bucket encryption mode");
        assert_eq!(wrong_key, CloudStorageError::KmsKeyModeMismatch);

        let wrong_purpose = StoredObject::new(
            &bucket,
            ObjectCreate {
                bucket_id: bucket.resource_id.value.value.clone(),
                tenant_id: "ten_kr".to_string(),
                key: "workspace/wrong-purpose.pdf".to_string(),
                size_bytes: 42,
                etag: "0123456789abcdef0123456789abcdef".to_string(),
                data_class: DataClass::PiiIdentifying,
                encryption: ObjectEncryptionBindingCreate {
                    purpose: KmsPurpose::CloudBlockStorage,
                    kms_encrypt_event_id: "kmsuse_object_wrong_purpose_001".to_string(),
                    ..object_encryption()
                },
                stored_at_epoch_seconds: 1_700_000_010,
                last_accessed_at_epoch_seconds: None,
            },
        )
        .expect_err("object KMS purpose is storage-object specific");
        assert_eq!(wrong_purpose, CloudStorageError::InvalidKmsPurpose);
    }

    #[test]
    fn rejects_operational_labels_on_storage_payloads_and_class_sets() {
        let class_set_error = Bucket::new(BucketCreate {
            allowed_data_classes: vec![DataClass::Audit],
            ..bucket_create()
        })
        .expect_err("allowed classes are privacy-program classes only");
        assert_eq!(class_set_error, CloudStorageError::InvalidDataClass);

        let volume_error = BlockVolume::new(VolumeCreate {
            data_class: DataClass::Secret,
            ..volume_create()
        })
        .expect_err("volume payload data class rejects operational labels");
        assert_eq!(volume_error, CloudStorageError::InvalidDataClass);
    }

    #[test]
    fn creates_block_volume_with_az_cell_performance_and_byok_binding() {
        let volume = BlockVolume::new(volume_create()).expect("volume is valid");

        assert_eq!(volume.resource_id.value.kind_label().unwrap(), "volume");
        assert_eq!(volume.az.value.value, "kr-seoul-a");
        assert_eq!(volume.cell_id.value.value, "cell-kr-seoul-a-001");
        assert_eq!(volume.performance.value.iops, 12_000);
        assert_eq!(volume.encryption.value, EncryptionMode::Byok);
        assert_eq!(volume.schema_version.value, STORAGE_SCHEMA_VERSION);
    }

    #[test]
    fn rejects_volume_location_and_performance_drift() {
        let az_error = BlockVolume::new(VolumeCreate {
            az: "us-east-a".to_string(),
            cell_id: "cell-us-east-a-001".to_string(),
            ..volume_create()
        })
        .expect_err("volume AZ must belong to region");
        assert_eq!(az_error, CloudStorageError::AzRegionMismatch);

        let cell_error = BlockVolume::new(VolumeCreate {
            cell_id: "cell-kr-seoul-b-001".to_string(),
            ..volume_create()
        })
        .expect_err("volume cell must belong to AZ namespace");
        assert_eq!(cell_error, CloudStorageError::CellLocationMismatch);

        let perf_error = BlockVolume::new(VolumeCreate {
            performance: VolumePerformance {
                iops: 0,
                throughput_mbps: 750,
            },
            ..volume_create()
        })
        .expect_err("performance is positive");
        assert_eq!(perf_error, CloudStorageError::InvalidPerformance);
    }

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
                tenant_id: "ten_kr".to_string(),
                source_volume_id: volume.resource_id.value.value.clone(),
                region: "kr-seoul".to_string(),
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
                tenant_id: "ten_kr".to_string(),
                source_volume_id: volume.resource_id.value.value.clone(),
                region: "kr-seoul".to_string(),
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
            resource_id: "oya:cloud:kr-seoul:ten_kr:filesystem:shared-docs".to_string(),
            tenant_id: "ten_kr".to_string(),
            name: "shared-docs".to_string(),
            region: "kr-seoul".to_string(),
            az: "kr-seoul-a".to_string(),
            cell_id: "cell-kr-seoul-a-001".to_string(),
            residency: ResidencyClass::StrictKr,
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
            resource_id: "oya:cloud:kr-seoul:ten_kr:archive-vault:cold-records".to_string(),
            tenant_id: "ten_kr".to_string(),
            name: "cold-records".to_string(),
            region: "kr-seoul".to_string(),
            residency: ResidencyClass::StrictKr,
            tier: ArchiveTier::DeepCold,
            encryption: EncryptionMode::Hyok,
            kms_key: Some("hyok/kr-seoul/ten_kr/archive-key".to_string()),
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
}
