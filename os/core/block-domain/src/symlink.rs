//! Stable device-symlink resolution (`/dev/disk/by-*`).
//!
//! Talos addresses disks not just by their volatile kernel name (`sda`, which
//! can change across boots) but by the stable udev-maintained symlinks under
//! `/dev/disk/by-id`, `/dev/disk/by-path`, `/dev/disk/by-uuid`,
//! `/dev/disk/by-partlabel` and `/dev/disk/by-partuuid`. This module models the
//! symlink farm and the resolution machined performs when a machine config
//! references a disk by one of these stable identifiers.

use std::collections::BTreeMap;

use crate::disk::Disk;
use crate::partition::Partition;
use crate::{BlockError, Result};

/// The category of a `/dev/disk/by-*` symlink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymlinkKind {
    /// `/dev/disk/by-id` — model + serial.
    ById,
    /// `/dev/disk/by-path` — PCI/bus topology path.
    ByPath,
    /// `/dev/disk/by-uuid` — filesystem UUID.
    ByUuid,
    /// `/dev/disk/by-partlabel` — GPT partition name.
    ByPartLabel,
    /// `/dev/disk/by-partuuid` — GPT partition GUID.
    ByPartUuid,
    /// `/dev/disk/by-label` — filesystem label.
    ByLabel,
}

impl SymlinkKind {
    /// The directory prefix under `/dev/disk` for this kind.
    pub fn dir(self) -> &'static str {
        match self {
            SymlinkKind::ById => "/dev/disk/by-id",
            SymlinkKind::ByPath => "/dev/disk/by-path",
            SymlinkKind::ByUuid => "/dev/disk/by-uuid",
            SymlinkKind::ByPartLabel => "/dev/disk/by-partlabel",
            SymlinkKind::ByPartUuid => "/dev/disk/by-partuuid",
            SymlinkKind::ByLabel => "/dev/disk/by-label",
        }
    }

    /// Parse a `/dev/disk/by-*` path into its kind and the link name, e.g.
    /// `/dev/disk/by-uuid/abc` -> `(ByUuid, "abc")`.
    pub fn parse_path(path: &str) -> Option<(SymlinkKind, &str)> {
        for kind in [
            SymlinkKind::ById,
            SymlinkKind::ByPath,
            SymlinkKind::ByUuid,
            SymlinkKind::ByPartLabel,
            SymlinkKind::ByPartUuid,
            SymlinkKind::ByLabel,
        ] {
            let prefix = kind.dir();
            if let Some(rest) = path.strip_prefix(prefix)
                && let Some(name) = rest.strip_prefix('/')
                && !name.is_empty()
            {
                return Some((kind, name));
            }
        }
        None
    }
}

/// An in-memory model of the `/dev/disk/by-*` symlink farm.
///
/// Maps a fully-qualified symlink path (e.g. `/dev/disk/by-uuid/1234`) to the
/// canonical `/dev` target (e.g. `/dev/sda2`).
#[derive(Debug, Default, Clone)]
pub struct SymlinkTable {
    links: BTreeMap<String, String>,
}

impl SymlinkTable {
    /// A fresh, empty table.
    pub fn new() -> Self {
        SymlinkTable::default()
    }

    /// Number of registered symlinks.
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    /// Register a single symlink of `kind` named `name` pointing at `target`.
    ///
    /// `target` must be an absolute `/dev` path. Re-registering the same link
    /// path with a different target is rejected (udev would never do this).
    pub fn add(&mut self, kind: SymlinkKind, name: &str, target: &str) -> Result<()> {
        if name.is_empty() {
            return Err(BlockError::InvalidDevice("empty symlink name".to_string()));
        }
        if !target.starts_with("/dev/") {
            return Err(BlockError::InvalidDevice(format!(
                "symlink target {target:?} is not under /dev"
            )));
        }
        let path = format!("{}/{}", kind.dir(), name);
        if let Some(existing) = self.links.get(&path)
            && existing != target
        {
            return Err(BlockError::InvalidDevice(format!(
                "symlink {path} already points at {existing}"
            )));
        }
        self.links.insert(path, target.to_string());
        Ok(())
    }

    /// Resolve a `/dev/disk/by-*` path (or a plain `/dev/...` path) to a
    /// canonical device path. Plain paths resolve to themselves.
    pub fn resolve(&self, path: &str) -> Result<String> {
        if SymlinkKind::parse_path(path).is_some() {
            return self
                .links
                .get(path)
                .cloned()
                .ok_or_else(|| BlockError::NotFound(format!("no such symlink {path}")));
        }
        if path.starts_with("/dev/") {
            return Ok(path.to_string());
        }
        Err(BlockError::InvalidDevice(format!(
            "not a device path: {path}"
        )))
    }

    /// All symlink paths that resolve to `target`, sorted.
    pub fn links_to(&self, target: &str) -> Vec<String> {
        self.links
            .iter()
            .filter(|(_, t)| t.as_str() == target)
            .map(|(p, _)| p.clone())
            .collect()
    }

    /// Populate the standard `by-id` / `by-path` links a disk would expose.
    pub fn register_disk(&mut self, disk: &Disk) -> Result<()> {
        let target = disk.dev_path();
        if let (Some(model), Some(serial)) = (&disk.model, &disk.serial) {
            let id = format!("{}_{}", sanitize(model), sanitize(serial));
            self.add(SymlinkKind::ById, &id, &target)?;
        } else if let Some(serial) = &disk.serial {
            self.add(SymlinkKind::ById, &sanitize(serial), &target)?;
        }
        for link in &disk.symlinks {
            // Each pre-seeded symlink is treated as a by-id alias.
            if let Some((kind, name)) = SymlinkKind::parse_path(link) {
                self.add(kind, name, &target)?;
            }
        }
        Ok(())
    }

    /// Populate the partition-level links (`by-partlabel`, `by-partuuid`,
    /// `by-uuid`, `by-label`) a formatted partition exposes.
    pub fn register_partition(&mut self, part: &Partition) -> Result<()> {
        let target = format!("/dev/{}", part.dev_name);
        if let Some(label) = &part.label
            && !label.is_empty()
        {
            self.add(SymlinkKind::ByPartLabel, label, &target)?;
            self.add(SymlinkKind::ByLabel, label, &target)?;
        }
        if let Some(uuid) = &part.uuid
            && !uuid.is_empty()
        {
            self.add(SymlinkKind::ByPartUuid, uuid, &target)?;
            self.add(SymlinkKind::ByUuid, uuid, &target)?;
        }
        Ok(())
    }
}

/// Replace characters udev escapes in `by-id` names (spaces, slashes) with `_`.
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::DiskBus;
    use crate::partition::PartitionRole;

    #[test]
    fn parse_by_paths() {
        let (k, n) = SymlinkKind::parse_path("/dev/disk/by-uuid/1234-ABCD").unwrap();
        assert_eq!(k, SymlinkKind::ByUuid);
        assert_eq!(n, "1234-ABCD");
        assert!(SymlinkKind::parse_path("/dev/sda1").is_none());
        assert!(SymlinkKind::parse_path("/dev/disk/by-uuid/").is_none());
    }

    #[test]
    fn add_and_resolve() {
        let mut t = SymlinkTable::new();
        t.add(SymlinkKind::ByUuid, "abc", "/dev/sda2").unwrap();
        assert_eq!(t.resolve("/dev/disk/by-uuid/abc").unwrap(), "/dev/sda2");
        // Plain dev path resolves to itself.
        assert_eq!(t.resolve("/dev/sda2").unwrap(), "/dev/sda2");
        // Missing symlink.
        assert!(matches!(
            t.resolve("/dev/disk/by-uuid/nope"),
            Err(BlockError::NotFound(_))
        ));
    }

    #[test]
    fn rejects_non_dev_target_and_conflicts() {
        let mut t = SymlinkTable::new();
        assert!(t.add(SymlinkKind::ById, "x", "sda").is_err());
        t.add(SymlinkKind::ById, "x", "/dev/sda").unwrap();
        // Same target is idempotent.
        t.add(SymlinkKind::ById, "x", "/dev/sda").unwrap();
        // Conflicting target rejected.
        assert!(t.add(SymlinkKind::ById, "x", "/dev/sdb").is_err());
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn register_disk_builds_by_id() {
        let mut d = Disk::new("sda", 1 << 30, DiskBus::Nvme);
        d.model = Some("Samsung SSD 990".to_string());
        d.serial = Some("S/N 42".to_string());
        let mut t = SymlinkTable::new();
        t.register_disk(&d).unwrap();
        let links = t.links_to("/dev/sda");
        assert_eq!(links.len(), 1);
        // Spaces and slashes sanitized to underscores.
        assert!(links[0].contains("Samsung_SSD_990_S_N_42"));
    }

    #[test]
    fn register_partition_builds_label_and_uuid_links() {
        let mut p = Partition::new("sda2", 2, 4096, 8192, PartitionRole::State);
        p.uuid = Some("uuid-state".to_string());
        let mut t = SymlinkTable::new();
        t.register_partition(&p).unwrap();
        assert_eq!(
            t.resolve("/dev/disk/by-partlabel/STATE").unwrap(),
            "/dev/sda2"
        );
        assert_eq!(
            t.resolve("/dev/disk/by-partuuid/uuid-state").unwrap(),
            "/dev/sda2"
        );
        assert_eq!(
            t.resolve("/dev/disk/by-uuid/uuid-state").unwrap(),
            "/dev/sda2"
        );
        // by-partlabel and by-label both registered.
        assert_eq!(t.links_to("/dev/sda2").len(), 4);
    }

    #[test]
    fn invalid_paths_rejected() {
        let t = SymlinkTable::new();
        assert!(t.resolve("relative/path").is_err());
        assert!(t.resolve("by-uuid/abc").is_err());
    }
}
