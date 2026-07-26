//! Filesystem creation (`mkfs`), mirroring Talos's `pkg/makefs`.
//!
//! Talos formats partitions by shelling out to `mkfs.xfs`, `mkfs.vfat`,
//! `mkfs.ext4`, and `mkswap` with a fixed set of options (label, force, block
//! size, etc.).
//! This module models the *command surface* — the option set and the argument
//! vector each `mkfs` invocation would receive — plus the in-memory effect of
//! the format (an on-disk superblock signature). The syscall/exec boundary is
//! the [`FsMaker`] trait, with an in-memory [`MemFsMaker`] used by tests.

use crate::filesystem::FilesystemType;
use crate::probe::MemReader;
use crate::{BlockError, Result};

/// Options controlling how a filesystem is created.
///
/// Mirrors the knobs Talos exposes through `makefs.WithLabel`,
/// `makefs.WithForce`, `makefs.WithReproducible` and friends.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MakeFsOptions {
    /// Filesystem label to set (length-validated against the fs type).
    pub label: Option<String>,
    /// Force creation even if a filesystem already exists on the device.
    pub force: bool,
    /// Reproducible output: pin the UUID / timestamps so images are
    /// bit-for-bit reproducible (Talos uses this for the boot assets).
    pub reproducible: bool,
    /// Explicit UUID to stamp into the superblock, if any.
    pub uuid: Option<String>,
    /// Override the filesystem block size in bytes (0 = mkfs default).
    pub block_size: u32,
}

impl MakeFsOptions {
    /// Fresh default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set the label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Builder: enable `force`.
    pub fn forced(mut self) -> Self {
        self.force = true;
        self
    }

    /// Builder: enable reproducible output.
    pub fn reproducible(mut self) -> Self {
        self.reproducible = true;
        self
    }

    /// Builder: pin an explicit UUID.
    pub fn with_uuid(mut self, uuid: impl Into<String>) -> Self {
        self.uuid = Some(uuid.into());
        self
    }

    /// Builder: set an explicit block size.
    pub fn with_block_size(mut self, size: u32) -> Self {
        self.block_size = size;
        self
    }

    /// Validate the options against `fs`: label length and block-size sanity.
    pub fn validate(&self, fs: FilesystemType) -> Result<()> {
        if let Some(label) = &self.label
            && !fs.label_fits(label.len())
        {
            return Err(BlockError::BadTable(format!(
                "label {label:?} too long for {}",
                fs.as_str()
            )));
        }
        if self.block_size != 0 && !self.block_size.is_power_of_two() {
            return Err(BlockError::Geometry(
                "block size must be a power of two".to_string(),
            ));
        }
        // vfat block sizes are constrained to 512..=4096.
        if fs == FilesystemType::Vfat
            && self.block_size != 0
            && (self.block_size < 512 || self.block_size > 4096)
        {
            return Err(BlockError::Geometry(
                "vfat block size out of range".to_string(),
            ));
        }
        if fs == FilesystemType::Swap && self.block_size != 0 {
            return Err(BlockError::Geometry(
                "swap block size override is not supported".to_string(),
            ));
        }
        Ok(())
    }
}

/// Build the `mkfs` argument vector Talos would exec for `fs` on `device`.
///
/// The argv mirrors the real flags:
/// * xfs: `mkfs.xfs -f [-L label] [-m uuid=..] [-b size=..] device`
/// * ext4: `mkfs.ext4 -F [-L label] [-U uuid] [-b size] device`
/// * vfat: `mkfs.vfat [-n LABEL] [-i uuid] [-S size] device`
/// * swap: `mkswap [-f] [-L label] [-U uuid] device`
///
/// iso9660 is read-only and virtiofs is an external mount source; this returns
/// an error for both.
pub fn mkfs_argv(fs: FilesystemType, device: &str, opts: &MakeFsOptions) -> Result<Vec<String>> {
    opts.validate(fs)?;
    if device.is_empty() {
        return Err(BlockError::InvalidDevice("empty device".to_string()));
    }
    if fs.is_read_only() {
        return Err(BlockError::BadTable(format!(
            "cannot create read-only filesystem {}",
            fs.as_str()
        )));
    }
    if fs == FilesystemType::Virtiofs {
        return Err(BlockError::BadTable(
            "cannot create external filesystem virtiofs".to_string(),
        ));
    }
    let mut argv = Vec::new();
    match fs {
        FilesystemType::Xfs => {
            argv.push("mkfs.xfs".to_string());
            if opts.force {
                argv.push("-f".to_string());
            }
            if let Some(label) = &opts.label {
                argv.push("-L".to_string());
                argv.push(label.clone());
            }
            if let Some(uuid) = &opts.uuid {
                argv.push("-m".to_string());
                argv.push(format!("uuid={uuid}"));
            }
            if opts.block_size != 0 {
                argv.push("-b".to_string());
                argv.push(format!("size={}", opts.block_size));
            }
            // Reproducible xfs only constrains the metadata UUID, which is
            // already emitted above via `-m uuid=...`; no extra flag is needed.
        }
        FilesystemType::Ext4 => {
            argv.push("mkfs.ext4".to_string());
            if opts.force {
                argv.push("-F".to_string());
            }
            if let Some(label) = &opts.label {
                argv.push("-L".to_string());
                argv.push(label.clone());
            }
            if let Some(uuid) = &opts.uuid {
                argv.push("-U".to_string());
                argv.push(uuid.clone());
            }
            if opts.block_size != 0 {
                argv.push("-b".to_string());
                argv.push(opts.block_size.to_string());
            }
        }
        FilesystemType::Vfat => {
            argv.push("mkfs.vfat".to_string());
            if let Some(label) = &opts.label {
                argv.push("-n".to_string());
                argv.push(label.to_ascii_uppercase());
            }
            if let Some(uuid) = &opts.uuid {
                // vfat volume id is a 32-bit hex serial passed via -i.
                argv.push("-i".to_string());
                argv.push(uuid.clone());
            }
            if opts.block_size != 0 {
                argv.push("-S".to_string());
                argv.push(opts.block_size.to_string());
            }
        }
        FilesystemType::Swap => {
            argv.push("mkswap".to_string());
            if opts.force {
                argv.push("-f".to_string());
            }
            if let Some(label) = &opts.label {
                argv.push("-L".to_string());
                argv.push(label.clone());
            }
            if let Some(uuid) = &opts.uuid {
                argv.push("-U".to_string());
                argv.push(uuid.clone());
            }
        }
        FilesystemType::Iso9660 | FilesystemType::Virtiofs => unreachable!("guarded above"),
    }
    argv.push(device.to_string());
    Ok(argv)
}

/// The boundary that actually creates a filesystem on a device.
pub trait FsMaker {
    /// Create `fs` on `device` with `opts`, writing the appropriate superblock
    /// signature so a subsequent probe recognises it.
    fn make(&mut self, device: &str, fs: FilesystemType, opts: &MakeFsOptions) -> Result<()>;
}

/// An in-memory `mkfs` that stamps the right magic bytes into a [`MemReader`].
///
/// Lets tests round-trip `make` -> [`crate::filesystem::FilesystemType::detect`]
/// without touching a real device.
#[derive(Debug, Default)]
pub struct MemFsMaker {
    /// Records of every (device, fs) pair formatted, in order.
    pub log: Vec<(String, FilesystemType)>,
}

impl MemFsMaker {
    /// A fresh maker with an empty log.
    pub fn new() -> Self {
        MemFsMaker::default()
    }

    /// Format a backing [`MemReader`] in place, stamping the fs signature so it
    /// can be re-probed. Returns the argv that would have been exec'd.
    pub fn format_reader(
        &mut self,
        device: &str,
        reader: &mut MemReader,
        fs: FilesystemType,
        opts: &MakeFsOptions,
    ) -> Result<Vec<String>> {
        let argv = mkfs_argv(fs, device, opts)?;
        stamp_signature(reader, fs)?;
        self.log.push((device.to_string(), fs));
        Ok(argv)
    }
}

/// Write the canonical superblock magic for `fs` into `reader`'s bytes so
/// [`FilesystemType::detect`] will recognise it.
pub fn stamp_signature(reader: &mut MemReader, fs: FilesystemType) -> Result<()> {
    let need = match fs {
        FilesystemType::Iso9660 => 0x8006,
        FilesystemType::Ext4 => 0x440,
        FilesystemType::Xfs | FilesystemType::Vfat => 512,
        FilesystemType::Swap => 4096,
        FilesystemType::Virtiofs => {
            return Err(BlockError::BadTable(
                "cannot stamp external filesystem virtiofs".to_string(),
            ));
        }
    };
    let bytes = reader.bytes_mut();
    if bytes.len() < need {
        bytes.resize(need, 0);
    }
    match fs {
        FilesystemType::Xfs => bytes[..4].copy_from_slice(b"XFSB"),
        FilesystemType::Ext4 => {
            bytes[0x438] = 0x53;
            bytes[0x439] = 0xEF;
        }
        FilesystemType::Vfat => {
            bytes[510] = 0x55;
            bytes[511] = 0xAA;
        }
        FilesystemType::Swap => bytes[4086..4096].copy_from_slice(b"SWAPSPACE2"),
        FilesystemType::Iso9660 => bytes[0x8001..0x8006].copy_from_slice(b"CD001"),
        FilesystemType::Virtiofs => unreachable!("guarded above"),
    }
    Ok(())
}

impl FsMaker for MemFsMaker {
    fn make(&mut self, device: &str, fs: FilesystemType, opts: &MakeFsOptions) -> Result<()> {
        mkfs_argv(fs, device, opts)?;
        self.log.push((device.to_string(), fs));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::probe;

    #[test]
    fn xfs_argv_has_force_and_label() {
        let opts = MakeFsOptions::new().forced().with_label("STATE");
        let argv = mkfs_argv(FilesystemType::Xfs, "/dev/sda2", &opts).unwrap();
        assert_eq!(argv[0], "mkfs.xfs");
        assert!(argv.contains(&"-f".to_string()));
        assert!(argv.contains(&"-L".to_string()));
        assert!(argv.contains(&"STATE".to_string()));
        assert_eq!(argv.last().unwrap(), "/dev/sda2");
    }

    #[test]
    fn ext4_uuid_and_blocksize() {
        let opts = MakeFsOptions::new().with_uuid("1234").with_block_size(4096);
        let argv = mkfs_argv(FilesystemType::Ext4, "/dev/sdb1", &opts).unwrap();
        assert_eq!(argv[0], "mkfs.ext4");
        let i = argv.iter().position(|a| a == "-U").unwrap();
        assert_eq!(argv[i + 1], "1234");
        let b = argv.iter().position(|a| a == "-b").unwrap();
        assert_eq!(argv[b + 1], "4096");
    }

    #[test]
    fn vfat_label_uppercased() {
        let opts = MakeFsOptions::new().with_label("efi");
        let argv = mkfs_argv(FilesystemType::Vfat, "/dev/sda1", &opts).unwrap();
        assert_eq!(argv[0], "mkfs.vfat");
        let n = argv.iter().position(|a| a == "-n").unwrap();
        assert_eq!(argv[n + 1], "EFI");
    }

    #[test]
    fn swap_command_surface() {
        let opts = MakeFsOptions::new()
            .with_label("SWAP")
            .with_uuid("11111111-2222-3333-4444-555555555555")
            .forced();
        let argv = mkfs_argv(FilesystemType::Swap, "/dev/sda3", &opts).unwrap();
        assert_eq!(
            argv,
            [
                "mkswap",
                "-f",
                "-L",
                "SWAP",
                "-U",
                "11111111-2222-3333-4444-555555555555",
                "/dev/sda3",
            ]
        );
    }

    #[test]
    fn iso9660_cannot_be_created() {
        let err = mkfs_argv(FilesystemType::Iso9660, "/dev/sr0", &MakeFsOptions::new());
        assert!(matches!(err, Err(BlockError::BadTable(_))));
    }

    #[test]
    fn rejects_overlong_label_and_bad_blocksize() {
        let too_long = MakeFsOptions::new().with_label("THIS_IS_WAY_TOO_LONG");
        assert!(mkfs_argv(FilesystemType::Vfat, "/dev/sda1", &too_long).is_err());
        let odd_block = MakeFsOptions::new().with_block_size(1000);
        assert!(mkfs_argv(FilesystemType::Ext4, "/dev/sda1", &odd_block).is_err());
        let big_vfat = MakeFsOptions::new().with_block_size(8192);
        assert!(mkfs_argv(FilesystemType::Vfat, "/dev/sda1", &big_vfat).is_err());
        let swap_block = MakeFsOptions::new().with_block_size(4096);
        assert!(mkfs_argv(FilesystemType::Swap, "/dev/sda3", &swap_block).is_err());
    }

    #[test]
    fn empty_device_rejected() {
        assert!(mkfs_argv(FilesystemType::Xfs, "", &MakeFsOptions::new()).is_err());
    }

    #[test]
    fn stamp_then_probe_round_trips() {
        for fs in [
            FilesystemType::Xfs,
            FilesystemType::Ext4,
            FilesystemType::Vfat,
            FilesystemType::Swap,
        ] {
            let mut reader = MemReader::zeroed(1 << 20);
            let mut maker = MemFsMaker::new();
            maker
                .format_reader("/dev/sda1", &mut reader, fs, &MakeFsOptions::new().forced())
                .unwrap();
            let result = probe(&reader).unwrap();
            // xfs/ext4 are whole-device filesystems; vfat sets the MBR sig so it
            // is detected as an MBR table by the probe ordering.
            if fs == FilesystemType::Vfat {
                assert!(result.is_partitioned());
            } else {
                assert_eq!(result.filesystem, Some(fs));
            }
        }
    }

    #[test]
    fn maker_logs_every_format() {
        let mut maker = MemFsMaker::new();
        maker
            .make(
                "/dev/sda1",
                FilesystemType::Xfs,
                &MakeFsOptions::new().forced(),
            )
            .unwrap();
        maker
            .make("/dev/sda2", FilesystemType::Ext4, &MakeFsOptions::new())
            .unwrap();
        assert_eq!(maker.log.len(), 2);
        assert_eq!(maker.log[0], ("/dev/sda1".to_string(), FilesystemType::Xfs));
    }

    #[test]
    fn reproducible_xfs_argv_is_clean() {
        let opts = MakeFsOptions::new()
            .forced()
            .reproducible()
            .with_uuid("00000000");
        let argv = mkfs_argv(FilesystemType::Xfs, "/dev/sda2", &opts).unwrap();
        // The reproducible flag must not leave dangling args.
        assert!(!argv.contains(&"-N".to_string()));
        assert!(
            argv.windows(2)
                .any(|w| w[0] == "-m" && w[1] == "uuid=00000000")
        );
    }
}
