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
