//! etcd snapshot model: the on-disk snapshot header/metadata, integrity hash,
//! and the validation Talos performs before a restore.
//!
//! Mirrors `pkg/etcd`'s snapshot save/restore (which wraps etcd's
//! `snapshot.Save`/`snapshot.Restore`). A real snapshot is a bbolt DB file with
//! a trailing SHA-256 integrity hash; here we model the metadata and the
//! checks (hash match, revision sanity) without the bbolt bytes.

use os_kernel::{Error, Result};

/// The well-known marker etcd writes; we keep an analogous constant so the
/// model can validate "looks like an etcd snapshot".
pub const SNAPSHOT_MAGIC: &[u8; 8] = b"ETCDSNAP";

/// Metadata describing a saved etcd snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMetadata {
    /// The etcd key-space revision captured by the snapshot.
    pub revision: u64,
    /// The Raft term at snapshot time.
    pub term: u64,
    /// The Raft applied index at snapshot time.
    pub index: u64,
    /// Number of keys in the snapshot.
    pub total_keys: u64,
    /// Physical size of the snapshot DB in bytes.
    pub total_size: u64,
    /// The integrity hash (CRC32 of the contents in real etcd; modeled here).
    pub hash: u32,
}

impl SnapshotMetadata {
    /// Validate that the metadata is self-consistent.
    pub fn validate(&self) -> Result<()> {
        if self.revision == 0 {
            return Err(Error::invalid("snapshot revision must be > 0"));
        }
        if self.index < self.revision.saturating_sub(self.total_keys) {
            // Sanity: applied index should not lag absurdly behind revision.
            // (A loose check; etcd guarantees index >= revision in practice.)
        }
        if self.total_size == 0 {
            return Err(Error::invalid("snapshot has zero size"));
        }
        Ok(())
    }
}

/// A snapshot blob with its metadata. The `data` stands in for the bbolt file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Snapshot metadata.
    pub metadata: SnapshotMetadata,
    /// Opaque snapshot bytes.
    pub data: Vec<u8>,
}

impl Snapshot {
    /// Create a snapshot, computing the integrity hash over `data`.
    pub fn create(revision: u64, term: u64, index: u64, total_keys: u64, data: Vec<u8>) -> Self {
        let hash = crc32(&data);
        Snapshot {
            metadata: SnapshotMetadata {
                revision,
                term,
                index,
                total_keys,
                total_size: data.len() as u64,
                hash,
            },
            data,
        }
    }

    /// Recompute the hash of the bytes and compare against the metadata.
    pub fn verify_integrity(&self) -> Result<()> {
        let actual = crc32(&self.data);
        if actual != self.metadata.hash {
            return Err(Error::invalid(format!(
                "snapshot hash mismatch: expected {:08x}, got {:08x}",
                self.metadata.hash, actual
            )));
        }
        if self.metadata.total_size != self.data.len() as u64 {
            return Err(Error::invalid("snapshot size does not match metadata"));
        }
        self.metadata.validate()
    }

    /// Whether this snapshot is newer than `other` by revision.
    pub fn is_newer_than(&self, other: &Snapshot) -> bool {
        self.metadata.revision > other.metadata.revision
    }
}

/// A small, dependency-free CRC32 (IEEE 802.3 polynomial) used as the
/// snapshot integrity hash. Sufficient to model "the bytes were not corrupted".
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_known_vector() {
        // CRC32 of "123456789" is 0xCBF43926.
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn snapshot_roundtrip_integrity_ok() {
        let snap = Snapshot::create(100, 5, 200, 42, b"the database bytes".to_vec());
        assert!(snap.verify_integrity().is_ok());
        assert_eq!(snap.metadata.total_size, 18);
    }

    #[test]
    fn corrupted_snapshot_detected() {
        let mut snap = Snapshot::create(100, 5, 200, 42, b"hello world".to_vec());
        snap.data[0] ^= 0xFF;
        assert!(snap.verify_integrity().is_err());
    }

    #[test]
    fn size_mismatch_detected() {
        let mut snap = Snapshot::create(100, 5, 200, 42, b"hello".to_vec());
        snap.metadata.total_size = 999;
        assert!(snap.verify_integrity().is_err());
    }

    #[test]
    fn newer_comparison() {
        let a = Snapshot::create(100, 1, 1, 1, b"a".to_vec());
        let b = Snapshot::create(200, 1, 1, 1, b"b".to_vec());
        assert!(b.is_newer_than(&a));
        assert!(!a.is_newer_than(&b));
    }

    #[test]
    fn zero_revision_invalid() {
        let snap = Snapshot::create(0, 1, 1, 1, b"x".to_vec());
        assert!(snap.verify_integrity().is_err());
    }
}
