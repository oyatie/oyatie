//! Device-tree blob (DTB) handling.
//!
//! SBCs boot with a flattened device tree describing their hardware. Talos
//! copies the board's `.dtb` (and any `*.dtbo` overlays) into the boot
//! partition. We model just enough of the FDT header to validate blobs and to
//! place them onto a boot partition abstraction.

use os_kernel::error::{Error, Result};

/// FDT (flattened device tree) header magic, big-endian `0xd00dfeed`.
pub const FDT_MAGIC: [u8; 4] = [0xd0, 0x0d, 0xfe, 0xed];

/// A device-tree blob destined for the board's boot partition.
#[derive(Clone, PartialEq, Eq)]
pub struct DeviceTree {
    /// Target filename on the boot partition (e.g. `bcm2711-rpi-4-b.dtb`).
    pub filename: String,
    /// Raw FDT bytes.
    pub data: Vec<u8>,
    /// Applied device-tree overlays (`.dtbo`), in apply order.
    pub overlays: Vec<DtbOverlay>,
}

impl core::fmt::Debug for DeviceTree {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DeviceTree")
            .field("filename", &self.filename)
            .field("len", &self.data.len())
            .field("overlays", &self.overlays.len())
            .finish()
    }
}

/// A device-tree overlay applied on top of the base DTB.
#[derive(Clone, PartialEq, Eq)]
pub struct DtbOverlay {
    /// Overlay filename (e.g. `disable-bt.dtbo`).
    pub filename: String,
    /// Raw overlay bytes.
    pub data: Vec<u8>,
}

impl core::fmt::Debug for DtbOverlay {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DtbOverlay")
            .field("filename", &self.filename)
            .field("len", &self.data.len())
            .finish()
    }
}

impl DeviceTree {
    /// Build a base device tree from a filename and raw bytes.
    pub fn new(filename: &str, data: &[u8]) -> DeviceTree {
        DeviceTree {
            filename: filename.to_string(),
            data: data.to_vec(),
            overlays: Vec::new(),
        }
    }

    /// Whether the blob begins with the FDT magic.
    pub fn has_magic(&self) -> bool {
        self.data.len() >= 4 && self.data[..4] == FDT_MAGIC
    }

    /// Append an overlay, validating its filename extension.
    pub fn add_overlay(&mut self, filename: &str, data: &[u8]) -> Result<()> {
        if !filename.ends_with(".dtbo") {
            return Err(Error::invalid(format!(
                "overlay must have .dtbo extension: {filename}"
            )));
        }
        self.overlays.push(DtbOverlay {
            filename: filename.to_string(),
            data: data.to_vec(),
        });
        Ok(())
    }

    /// Validate the device tree and all overlays.
    pub fn validate(&self) -> Result<()> {
        if !self.filename.ends_with(".dtb") {
            return Err(Error::invalid(format!(
                "device tree must have .dtb extension: {}",
                self.filename
            )));
        }
        if !self.has_magic() {
            return Err(Error::parse(format!(
                "device tree {} missing FDT magic",
                self.filename
            )));
        }
        for o in &self.overlays {
            if o.data.is_empty() {
                return Err(Error::invalid(format!("overlay {} is empty", o.filename)));
            }
        }
        Ok(())
    }

    /// All files (base + overlays) this device tree contributes to the boot
    /// partition, as `(filename, bytes)` pairs in copy order.
    pub fn boot_files(&self) -> Vec<(String, &[u8])> {
        let mut out: Vec<(String, &[u8])> = Vec::with_capacity(1 + self.overlays.len());
        out.push((self.filename.clone(), self.data.as_slice()));
        for o in &self.overlays {
            out.push((o.filename.clone(), o.data.as_slice()));
        }
        out
    }
}
