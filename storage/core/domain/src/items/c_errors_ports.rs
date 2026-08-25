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
    InvalidStorageNamespacePolicy,
    InvalidEvidenceRef,
    DuplicateBucket,
    UnknownBucket,
    DuplicateObject,
    DuplicateVolume,
    UnknownVolume,
    DuplicateFilesystem,
    DuplicateArchiveVault,
    DuplicateSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageProviderObjectError {
    InvalidProviderBucketRef,
    InvalidProviderRequestId,
    InvalidProviderEvidenceRef,
    InvalidObjectBodyRef,
    InvalidIdempotencyKey,
    InvalidActorRef,
    InvalidRequestShape(CloudStorageError),
    ProviderRejected {
        provider: StorageProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
    ProviderUnavailable {
        provider: StorageProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageProviderBlockError {
    InvalidProviderVolumeRef,
    InvalidProviderRequestId,
    InvalidProviderEvidenceRef,
    InvalidIdempotencyKey,
    InvalidActorRef,
    InvalidRequestShape(CloudStorageError),
    ProviderRejected {
        provider: StorageProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
    ProviderUnavailable {
        provider: StorageProviderKind, // data_class: PUBLIC
        reason: String,                // data_class: INTERNAL_ONLY
    },
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

pub trait StorageProviderObjectPort {
    fn provider_kind(&self) -> StorageProviderKind;

    fn put_object(
        &self,
        input: StorageProviderObjectPutRequest,
    ) -> Result<StorageProviderObjectReceipt, StorageProviderObjectError>;

    fn get_object(
        &self,
        input: StorageProviderObjectGetRequest,
    ) -> Result<StorageProviderObjectReceipt, StorageProviderObjectError>;
}

pub trait StorageProviderBlockPort {
    fn provider_kind(&self) -> StorageProviderKind;

    fn create_volume(
        &self,
        input: StorageProviderBlockCreateVolumeRequest,
    ) -> Result<StorageProviderBlockReceipt, StorageProviderBlockError>;
}
