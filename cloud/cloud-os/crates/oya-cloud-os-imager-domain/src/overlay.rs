//! Board/overlay integration.
//!
//! Mirrors Talos overlays (`siderolabs/overlays`): a board overlay supplies an
//! architecture, a device-tree / firmware payload, extra kernel args and a disk
//! partition layout used when imaging for a specific single-board computer
//! (Raspberry Pi, Jetson, etc.). The imager looks an overlay up by name from an
//! [`OverlayRegistry`] and applies it to a build.

use crate::profile::Arch;
use std::collections::BTreeMap;

/// A board overlay definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overlay {
    /// Overlay name, e.g. `rpi_generic`.
    pub name: String,
    /// Architecture this overlay applies to.
    pub arch: Arch,
    /// Extra kernel command-line arguments contributed by the board.
    pub extra_cmdline: Vec<String>,
    /// Firmware/u-boot blobs the overlay installs, by destination path.
    pub firmware: BTreeMap<String, u64>,
    /// First-partition (boot) offset in bytes the board requires.
    pub boot_partition_offset: u64,
}

impl Overlay {
    /// Construct an overlay.
    pub fn new(name: impl Into<String>, arch: Arch) -> Overlay {
        Overlay {
            name: name.into(),
            arch,
            extra_cmdline: Vec::new(),
            firmware: BTreeMap::new(),
            boot_partition_offset: 0,
        }
    }

    /// Add an extra cmdline argument (builder).
    pub fn with_cmdline(mut self, arg: impl Into<String>) -> Overlay {
        self.extra_cmdline.push(arg.into());
        self
    }

    /// Register a firmware blob at `path` of `len` bytes (builder).
    pub fn with_firmware(mut self, path: impl Into<String>, len: u64) -> Overlay {
        self.firmware.insert(path.into(), len);
        self
    }

    /// Set the required boot partition offset (builder).
    pub fn with_boot_offset(mut self, offset: u64) -> Overlay {
        self.boot_partition_offset = offset;
        self
    }

    /// Total bytes the overlay's firmware payload occupies.
    pub fn firmware_bytes(&self) -> u64 {
        self.firmware.values().copied().sum()
    }
}

/// A registry of available overlays, looked up by name.
#[derive(Debug, Default, Clone)]
pub struct OverlayRegistry {
    overlays: BTreeMap<String, Overlay>,
}

impl OverlayRegistry {
    /// An empty registry.
    pub fn new() -> OverlayRegistry {
        OverlayRegistry {
            overlays: BTreeMap::new(),
        }
    }

    /// A registry pre-populated with the common Talos boards.
    pub fn with_builtins() -> OverlayRegistry {
        let mut reg = OverlayRegistry::new();
        reg.register(
            Overlay::new("rpi_generic", Arch::Arm64)
                .with_cmdline("console=tty0")
                .with_firmware("config.txt", 2048)
                .with_firmware("u-boot.bin", 512 * 1024)
                .with_boot_offset(2048 * 512),
        );
        reg.register(
            Overlay::new("jetson_nano", Arch::Arm64)
                .with_cmdline("console=ttyTHS0")
                .with_firmware("u-boot.bin", 1024 * 1024)
                .with_boot_offset(2048 * 512),
        );
        reg
    }

    /// Register (or replace) an overlay.
    pub fn register(&mut self, overlay: Overlay) {
        self.overlays.insert(overlay.name.clone(), overlay);
    }

    /// Look an overlay up by name.
    pub fn get(&self, name: &str) -> Option<&Overlay> {
        self.overlays.get(name)
    }

    /// Number of registered overlays.
    pub fn len(&self) -> usize {
        self.overlays.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.overlays.is_empty()
    }
}
