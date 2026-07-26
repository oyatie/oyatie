//! The persistence boundary for the META partition.
//!
//! The real Talos `meta` package reads and writes the ADV blob through a block
//! device. That syscall/IO boundary is modelled here as the [`MetaStore`]
//! trait: a small load/commit interface over a [`MetaPartition`]. Production
//! code would implement it against `/dev/...`; tests use [`InMemoryMetaStore`],
//! which keeps the partition image in a `Vec<u8>`.

use crate::adv::{Adv, AdvFormat};
use crate::meta::Meta;
use crate::partition::{MetaPartition, Slot};
use os_kernel::Result;

/// Abstracts reading and writing the META partition image.
///
/// Implementors are responsible only for moving raw partition-sized byte
/// images in and out of durable storage; all ADV encoding/validation is done
/// by the default methods using the associated [`MetaPartition`].
pub trait MetaStore {
    /// The partition layout this store operates on.
    fn partition(&self) -> &MetaPartition;

    /// Reads the full partition image from the backing device.
    fn read_raw(&self) -> Result<Vec<u8>>;

    /// Writes the full partition image to the backing device.
    fn write_raw(&mut self, image: &[u8]) -> Result<()>;

    /// Loads and decodes the META document, selecting the freshest valid ADV
    /// copy. The returned [`Meta`] is clean (matches what is on disk).
    fn load(&self) -> Result<Meta> {
        let image = self.read_raw()?;
        let (_slot, adv) = self.partition().read_image(&image)?;
        let bytes = adv.encode()?;
        Meta::decode(&bytes)
    }

    /// Encodes `meta` and writes both ADV copies, then marks it clean.
    fn commit(&mut self, meta: &mut Meta) -> Result<()> {
        self.commit_as(meta, AdvFormat::V1)
    }

    /// Like [`MetaStore::commit`] but encodes in the given on-disk `format`.
    ///
    /// This lets tooling rewrite a legacy-format partition without silently
    /// upgrading it to ADV1.
    fn commit_as(&mut self, meta: &mut Meta, format: AdvFormat) -> Result<()> {
        let image = self.partition().write_image_as(meta.adv(), format)?;
        self.write_raw(&image)?;
        meta.mark_clean();
        Ok(())
    }

    /// Loads, decodes and returns which slot the freshest copy lives in.
    fn load_with_slot(&self) -> Result<(Slot, Adv)> {
        let image = self.read_raw()?;
        self.partition().read_image(&image)
    }

    /// Loads, decoding the freshest copy and reporting the on-disk format it
    /// used. Returns `(slot, meta, format)`.
    fn load_detect(&self) -> Result<(Slot, Meta, AdvFormat)> {
        let image = self.read_raw()?;
        let (slot, adv, format) = self.partition().read_image_detect(&image)?;
        let bytes = adv.encode()?;
        Ok((slot, Meta::decode(&bytes)?, format))
    }

    /// Loads the META document, returning a fresh empty [`Meta`] when no valid
    /// ADV copy exists yet (e.g. a brand-new, zeroed partition). This mirrors
    /// Talos treating an uninitialized META partition as "empty" rather than an
    /// error during first boot.
    fn load_or_default(&self) -> Result<Meta> {
        match self.load() {
            Ok(meta) => Ok(meta),
            Err(_) => Ok(Meta::new()),
        }
    }
}

/// An in-memory [`MetaStore`] backed by a `Vec<u8>` partition image.
///
/// Suitable for tests and host tooling. The image is initialized to all zeros
/// (which decodes as "no valid ADV") until the first [`MetaStore::commit`].
#[derive(Debug, Clone)]
pub struct InMemoryMetaStore {
    partition: MetaPartition,
    image: Vec<u8>,
}

impl InMemoryMetaStore {
    /// Creates a store with a zeroed image of the given partition's size.
    pub fn new(partition: MetaPartition) -> Self {
        let image = vec![0u8; partition.size()];
        Self { partition, image }
    }

    /// Creates a store over the default 1 MiB META partition.
    pub fn with_default_partition() -> Self {
        Self::new(MetaPartition::default())
    }

    /// Borrows the raw backing image (for inspection in tests).
    pub fn image(&self) -> &[u8] {
        &self.image
    }
}

impl Default for InMemoryMetaStore {
    fn default() -> Self {
        Self::with_default_partition()
    }
}

impl MetaStore for InMemoryMetaStore {
    fn partition(&self) -> &MetaPartition {
        &self.partition
    }

    fn read_raw(&self) -> Result<Vec<u8>> {
        Ok(self.image.clone())
    }

    fn write_raw(&mut self, image: &[u8]) -> Result<()> {
        if image.len() != self.partition.size() {
            return Err(os_kernel::Error::invalid(
                "image size does not match partition",
            ));
        }
        self.image.clear();
        self.image.extend_from_slice(image);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::MetaKey;
    use crate::partition::MetaPartition;

    #[test]
    fn fresh_store_has_no_valid_meta() {
        let store = InMemoryMetaStore::default();
        // All-zero image: no valid ADV copy.
        assert!(store.load().is_err());
    }

    #[test]
    fn commit_then_load_round_trips() {
        let mut store = InMemoryMetaStore::new(MetaPartition::new(8192).unwrap());
        let mut meta = Meta::new();
        meta.set_unique_machine_token("tok-persist").unwrap();
        meta.set_staged_upgrade_image_ref("installer:v1.7.0")
            .unwrap();
        assert!(meta.is_dirty());

        store.commit(&mut meta).unwrap();
        assert!(!meta.is_dirty(), "commit should mark clean");

        let loaded = store.load().unwrap();
        assert_eq!(
            loaded.unique_machine_token().unwrap().unwrap(),
            "tok-persist"
        );
        assert_eq!(
            loaded.staged_upgrade_image_ref().unwrap().unwrap(),
            "installer:v1.7.0"
        );
        assert!(!loaded.is_dirty());
    }

    #[test]
    fn commit_writes_both_slots() {
        let mut store = InMemoryMetaStore::new(MetaPartition::new(8192).unwrap());
        let mut meta = Meta::new();
        meta.set_unique_machine_token("tok").unwrap();
        store.commit(&mut meta).unwrap();

        let (slot, _) = store.load_with_slot().unwrap();
        assert_eq!(slot, Slot::Primary);

        // Corrupt primary in the backing image; load must still succeed via backup.
        let mut img = store.read_raw().unwrap();
        img[0] = b'X';
        store.write_raw(&img).unwrap();
        let (slot, adv) = store.load_with_slot().unwrap();
        assert_eq!(slot, Slot::Backup);
        assert!(adv.get(MetaKey::UniqueMachineToken).is_some());
    }

    #[test]
    fn write_raw_rejects_wrong_size() {
        let mut store = InMemoryMetaStore::new(MetaPartition::new(8192).unwrap());
        assert!(store.write_raw(&[0u8; 10]).is_err());
    }

    #[test]
    fn second_commit_overwrites() {
        let mut store = InMemoryMetaStore::new(MetaPartition::new(8192).unwrap());
        let mut meta = Meta::new();
        meta.set_unique_machine_token("first").unwrap();
        store.commit(&mut meta).unwrap();

        meta.set_unique_machine_token("second").unwrap();
        store.commit(&mut meta).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.unique_machine_token().unwrap().unwrap(), "second");
    }

    #[test]
    fn load_or_default_on_blank_partition() {
        let store = InMemoryMetaStore::new(MetaPartition::new(8192).unwrap());
        let meta = store.load_or_default().unwrap();
        assert!(meta.is_empty());
        assert!(!meta.is_dirty());
    }

    #[test]
    fn commit_as_legacy_and_detect() {
        let mut store = InMemoryMetaStore::new(MetaPartition::new(8192).unwrap());
        let mut meta = Meta::new();
        meta.set_unique_machine_token("tok-legacy").unwrap();
        store.commit_as(&mut meta, AdvFormat::Legacy).unwrap();
        assert!(!meta.is_dirty());

        let (slot, loaded, fmt) = store.load_detect().unwrap();
        assert_eq!(slot, Slot::Primary);
        assert_eq!(fmt, AdvFormat::Legacy);
        assert_eq!(
            loaded.unique_machine_token().unwrap().unwrap(),
            "tok-legacy"
        );
    }

    #[test]
    fn v1_commit_detected_as_v1() {
        let mut store = InMemoryMetaStore::new(MetaPartition::new(8192).unwrap());
        let mut meta = Meta::new();
        meta.set_upgrade("v1.6.0").unwrap();
        store.commit(&mut meta).unwrap();
        let (_, _, fmt) = store.load_detect().unwrap();
        assert_eq!(fmt, AdvFormat::V1);
    }
}
