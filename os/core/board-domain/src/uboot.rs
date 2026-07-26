//! U-Boot / firmware image modeling and the raw-sector flashing boundary.
//!
//! On SBCs the bootrom expects U-Boot (or SPL+U-Boot) at a fixed raw offset on
//! the boot device, *before* the GPT partitions. Talos's installer `dd`s these
//! blobs into place. We model the disk as a trait so the flashing logic is
//! testable without touching a real device.

use os_kernel::error::{Error, Result};

/// A firmware/U-Boot blob to be written to a raw disk offset.
#[derive(Clone, PartialEq, Eq)]
pub struct UBootImage {
    /// Source filename (for logging / provenance).
    pub name: String,
    /// Raw bytes of the blob.
    pub data: Vec<u8>,
}

impl core::fmt::Debug for UBootImage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UBootImage")
            .field("name", &self.name)
            .field("len", &self.data.len())
            .finish()
    }
}

/// Magic for an FDT/U-Boot legacy image header / DTB (`0xd00dfeed`). We reuse it
/// as a light sanity check on supplied blobs.
pub const FDT_MAGIC: [u8; 4] = [0xd0, 0x0d, 0xfe, 0xed];

impl UBootImage {
    /// Build an image from a name and bytes.
    pub fn new(name: &str, data: &[u8]) -> UBootImage {
        UBootImage {
            name: name.to_string(),
            data: data.to_vec(),
        }
    }

    /// Length in bytes.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the blob is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Whether the blob begins with the FDT/U-Boot magic.
    pub fn has_fdt_magic(&self) -> bool {
        self.data.len() >= 4 && self.data[..4] == FDT_MAGIC
    }

    /// Validate the blob is plausibly a flashable firmware image.
    pub fn validate(&self) -> Result<()> {
        if self.data.is_empty() {
            return Err(Error::invalid("u-boot image is empty"));
        }
        Ok(())
    }
}

/// The raw block-device boundary used when flashing firmware.
///
/// Real implementations write to `/dev/<disk>`; tests use [`InMemoryDisk`].
pub trait RawDisk {
    /// Total size of the device in bytes.
    fn size(&self) -> u64;
    /// Write `data` starting at byte `offset`. Must fail if it would exceed the
    /// device size.
    fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<()>;
    /// Read `len` bytes from `offset`.
    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>>;
}

/// A fixed-size in-memory disk used by tests and the install dry-run path.
#[derive(Debug, Clone)]
pub struct InMemoryDisk {
    bytes: Vec<u8>,
}

impl InMemoryDisk {
    /// A zero-filled disk of `size` bytes.
    pub fn new(size: usize) -> InMemoryDisk {
        InMemoryDisk {
            bytes: vec![0u8; size],
        }
    }
}

impl RawDisk for InMemoryDisk {
    fn size(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn write_at(&mut self, offset: u64, data: &[u8]) -> Result<()> {
        let start = offset as usize;
        let end = start
            .checked_add(data.len())
            .ok_or_else(|| Error::invalid("write offset overflow"))?;
        if end > self.bytes.len() {
            return Err(Error::invalid(format!(
                "write [{start}, {end}) exceeds disk size {}",
                self.bytes.len()
            )));
        }
        self.bytes[start..end].copy_from_slice(data);
        Ok(())
    }

    fn read_at(&self, offset: u64, len: usize) -> Result<Vec<u8>> {
        let start = offset as usize;
        let end = start
            .checked_add(len)
            .ok_or_else(|| Error::invalid("read offset overflow"))?;
        if end > self.bytes.len() {
            return Err(Error::invalid("read exceeds disk size"));
        }
        Ok(self.bytes[start..end].to_vec())
    }
}

/// Flash a single firmware image to a raw offset, validating the blob first and
/// verifying the bytes landed.
pub fn flash_image<D: RawDisk + ?Sized>(
    disk: &mut D,
    offset: u64,
    image: &UBootImage,
) -> Result<()> {
    image.validate()?;
    disk.write_at(offset, &image.data)?;
    let back = disk.read_at(offset, image.data.len())?;
    if back != image.data {
        return Err(Error::invalid_state("firmware verify read-back mismatch"));
    }
    Ok(())
}
