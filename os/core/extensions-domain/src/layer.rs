//! Extension layer modeling and validation.
//!
//! When Talos builds (or boots) an image, each installed extension contributes a
//! filesystem layer that is merged into the squashfs/overlay rootfs. Talos
//! validates that extensions do not clash: two extensions may not own the same
//! path, and certain paths are reserved for the base OS. This module models the
//! per-extension [`ExtensionLayer`] and a [`LayerSet`] that performs the overlay
//! merge with conflict detection, mirroring the checks in
//! `pkg/machinery/extensions` `Validate` and the installer's layer composition.

use std::collections::BTreeMap;

use os_kernel::error::{Error, Result};

use crate::manifest::{ExtensionKind, ExtensionManifest};

/// Paths the base OS owns; an extension claiming any of these is rejected.
const RESERVED_PREFIXES: &[&str] = &["/etc/os-release", "/sbin/init", "/usr/lib/talos"];

/// A single file or directory an extension installs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerEntry {
    /// Absolute destination path inside the rootfs (e.g. `/usr/local/bin/foo`).
    pub path: String,
    /// Whether the entry is a directory (directories may overlap between layers).
    pub is_dir: bool,
}

impl LayerEntry {
    /// A regular file entry.
    pub fn file(path: impl Into<String>) -> Self {
        LayerEntry {
            path: path.into(),
            is_dir: false,
        }
    }

    /// A directory entry.
    pub fn dir(path: impl Into<String>) -> Self {
        LayerEntry {
            path: path.into(),
            is_dir: true,
        }
    }

    /// Validate that the path is absolute and normalized (no `..`, no trailing
    /// slash except root).
    pub fn validate(&self) -> Result<()> {
        if !self.path.starts_with('/') {
            return Err(Error::invalid(format!(
                "layer path '{}' must be absolute",
                self.path
            )));
        }
        if self.path.split('/').any(|seg| seg == "..") {
            return Err(Error::invalid(format!(
                "layer path '{}' contains '..'",
                self.path
            )));
        }
        Ok(())
    }

    fn is_reserved(&self) -> bool {
        RESERVED_PREFIXES
            .iter()
            .any(|p| self.path == *p || self.path.starts_with(&format!("{p}/")))
    }
}

/// All the filesystem entries one extension contributes, tied to its manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionLayer {
    /// The owning manifest (identifies the extension by name/kind).
    pub manifest: ExtensionManifest,
    /// Files and directories this layer installs.
    pub entries: Vec<LayerEntry>,
}

impl ExtensionLayer {
    /// Create an empty layer for `manifest`.
    pub fn new(manifest: ExtensionManifest) -> Self {
        ExtensionLayer {
            manifest,
            entries: Vec::new(),
        }
    }

    /// Add an entry, returning `self` for chaining.
    pub fn with_entry(mut self, entry: LayerEntry) -> Self {
        self.entries.push(entry);
        self
    }

    /// Validate the layer in isolation: the manifest must be valid, every entry
    /// path must be well-formed, and no file may claim a reserved OS path.
    /// Firmware extensions must install under `/lib/firmware`; kernel modules
    /// under `/lib/modules`.
    pub fn validate(&self) -> Result<()> {
        self.manifest.validate()?;
        for entry in &self.entries {
            entry.validate()?;
            if !entry.is_dir && entry.is_reserved() {
                return Err(Error::invalid(format!(
                    "extension '{}' may not overwrite reserved path '{}'",
                    self.manifest.name, entry.path
                )));
            }
            match self.manifest.kind {
                ExtensionKind::Firmware if !entry.path.starts_with("/lib/firmware") => {
                    return Err(Error::invalid(format!(
                        "firmware extension '{}' installs outside /lib/firmware: '{}'",
                        self.manifest.name, entry.path
                    )));
                }
                ExtensionKind::KernelModule if !entry.path.starts_with("/lib/modules") => {
                    return Err(Error::invalid(format!(
                        "kernel-module extension '{}' installs outside /lib/modules: '{}'",
                        self.manifest.name, entry.path
                    )));
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// File entries only (directories excluded), used for conflict checks.
    pub fn file_paths(&self) -> impl Iterator<Item = &str> {
        self.entries
            .iter()
            .filter(|e| !e.is_dir)
            .map(|e| e.path.as_str())
    }
}

/// An ordered set of extension layers merged into one overlay rootfs.
#[derive(Debug, Default)]
pub struct LayerSet {
    layers: Vec<ExtensionLayer>,
}

impl LayerSet {
    /// An empty layer set.
    pub fn new() -> Self {
        LayerSet { layers: Vec::new() }
    }

    /// Number of layers.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// The layers in insertion order.
    pub fn layers(&self) -> &[ExtensionLayer] {
        &self.layers
    }

    /// Add a layer after validating it in isolation and checking it does not
    /// conflict (claim the same file) with any already-present layer. Also
    /// rejects two extensions with the same name.
    pub fn push(&mut self, layer: ExtensionLayer) -> Result<()> {
        layer.validate()?;

        if self
            .layers
            .iter()
            .any(|l| l.manifest.name == layer.manifest.name)
        {
            return Err(Error::invalid(format!(
                "duplicate extension '{}'",
                layer.manifest.name
            )));
        }

        // Build the set of file paths already claimed.
        let mut owned: BTreeMap<&str, &str> = BTreeMap::new();
        for existing in &self.layers {
            for path in existing.file_paths() {
                owned.insert(path, existing.manifest.name.as_str());
            }
        }
        for path in layer.file_paths() {
            if let Some(owner) = owned.get(path) {
                return Err(Error::invalid(format!(
                    "path conflict on '{}': '{}' vs '{}'",
                    path, owner, layer.manifest.name
                )));
            }
        }

        self.layers.push(layer);
        Ok(())
    }

    /// Validate the whole set against a running Talos version: every layer must
    /// be compatible, and the merged file set must have no conflicts. Returns
    /// the total number of merged file entries on success.
    pub fn validate_for(&self, talos: &os_kernel::version::Version) -> Result<usize> {
        let mut owned: BTreeMap<&str, &str> = BTreeMap::new();
        for layer in &self.layers {
            layer.validate()?;
            if !layer.manifest.is_compatible_with(talos) {
                return Err(Error::invalid(format!(
                    "extension '{}' is not compatible with Talos {}",
                    layer.manifest.name, talos
                )));
            }
            for path in layer.file_paths() {
                if let Some(owner) = owned.insert(path, layer.manifest.name.as_str()) {
                    return Err(Error::invalid(format!(
                        "path conflict on '{}': '{}' vs '{}'",
                        path, owner, layer.manifest.name
                    )));
                }
            }
        }
        Ok(owned.len())
    }

    /// The merged, de-duplicated, sorted list of all file paths in the overlay.
    pub fn merged_paths(&self) -> Vec<String> {
        let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for layer in &self.layers {
            for p in layer.file_paths() {
                set.insert(p.to_string());
            }
        }
        set.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::version::Version;

    fn mk(name: &str, kind: ExtensionKind) -> ExtensionManifest {
        ExtensionManifest::new(name, Version::new(1, 0, 0), kind)
    }

    #[test]
    fn entry_validation() {
        assert!(LayerEntry::file("/usr/bin/x").validate().is_ok());
        assert!(LayerEntry::file("relative/path").validate().is_err());
        assert!(LayerEntry::file("/a/../b").validate().is_err());
    }

    #[test]
    fn reserved_path_rejected() {
        let layer = ExtensionLayer::new(mk("evil", ExtensionKind::Rootfs))
            .with_entry(LayerEntry::file("/sbin/init"));
        assert!(layer.validate().is_err());
    }

    #[test]
    fn firmware_must_live_under_lib_firmware() {
        let ok = ExtensionLayer::new(mk("fw", ExtensionKind::Firmware))
            .with_entry(LayerEntry::file("/lib/firmware/foo.bin"));
        assert!(ok.validate().is_ok());

        let bad = ExtensionLayer::new(mk("fw", ExtensionKind::Firmware))
            .with_entry(LayerEntry::file("/usr/bin/foo"));
        assert!(bad.validate().is_err());
    }

    #[test]
    fn kernel_module_path_check() {
        let bad = ExtensionLayer::new(mk("km", ExtensionKind::KernelModule))
            .with_entry(LayerEntry::file("/usr/lib/mod.ko"));
        assert!(bad.validate().is_err());
        let ok = ExtensionLayer::new(mk("km", ExtensionKind::KernelModule))
            .with_entry(LayerEntry::file("/lib/modules/6.6/mod.ko"));
        assert!(ok.validate().is_ok());
    }

    #[test]
    fn push_detects_path_conflict() {
        let mut set = LayerSet::new();
        let a = ExtensionLayer::new(mk("a", ExtensionKind::Rootfs))
            .with_entry(LayerEntry::file("/usr/local/bin/tool"));
        let b = ExtensionLayer::new(mk("b", ExtensionKind::Rootfs))
            .with_entry(LayerEntry::file("/usr/local/bin/tool"));
        set.push(a).unwrap();
        let err = set.push(b).unwrap_err();
        assert_eq!(err.kind(), "invalid");
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn push_rejects_duplicate_extension() {
        let mut set = LayerSet::new();
        set.push(ExtensionLayer::new(mk("dup", ExtensionKind::Rootfs)))
            .unwrap();
        assert!(
            set.push(ExtensionLayer::new(mk("dup", ExtensionKind::Rootfs)))
                .is_err()
        );
    }

    #[test]
    fn directories_may_overlap() {
        let mut set = LayerSet::new();
        let a = ExtensionLayer::new(mk("a", ExtensionKind::Rootfs))
            .with_entry(LayerEntry::dir("/usr/local/bin"))
            .with_entry(LayerEntry::file("/usr/local/bin/a"));
        let b = ExtensionLayer::new(mk("b", ExtensionKind::Rootfs))
            .with_entry(LayerEntry::dir("/usr/local/bin"))
            .with_entry(LayerEntry::file("/usr/local/bin/b"));
        set.push(a).unwrap();
        set.push(b).unwrap();
        assert_eq!(set.len(), 2);
        assert_eq!(
            set.merged_paths(),
            vec!["/usr/local/bin/a", "/usr/local/bin/b"]
        );
    }

    #[test]
    fn validate_for_checks_compatibility() {
        let mut set = LayerSet::new();
        let mut m = mk("needs-new-talos", ExtensionKind::Rootfs);
        m.compatibility = crate::manifest::Compatibility {
            talos: Some(crate::manifest::VersionConstraint::parse(">= v1.8.0").unwrap()),
        };
        set.push(ExtensionLayer::new(m).with_entry(LayerEntry::file("/usr/bin/new")))
            .unwrap();
        assert!(set.validate_for(&Version::new(1, 7, 0)).is_err());
        assert_eq!(set.validate_for(&Version::new(1, 8, 0)).unwrap(), 1);
    }
}
