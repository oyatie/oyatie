//! Output kinds and per-output options.
//!
//! Mirrors Talos `pkg/imager` output handling: the imager can produce an ISO,
//! a raw/cloud disk image, an `installer` container, a bare kernel, a bare
//! initramfs, a metal artifact, or a SecureBoot UKI. Each output kind has
//! different capabilities (overlay support, SecureBoot support) and a different
//! set of options (disk size, disk format, compression).

use std::fmt;

/// The kind of artifact the imager produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputKind {
    /// A bootable `.iso` (ISO9660, UEFI + BIOS).
    Iso,
    /// A raw disk image (`disk.raw`), optionally converted to a cloud format.
    DiskImage,
    /// An `installer` container image used by `talosctl upgrade`.
    Installer,
    /// A bare kernel image.
    Kernel,
    /// A bare initramfs image.
    Initramfs,
    /// A metal image (raw disk for bare-metal PXE/boot).
    Metal,
    /// A SecureBoot Unified Kernel Image (`.efi`).
    SecureBootUki,
}

impl OutputKind {
    /// Canonical Talos name for this output kind.
    pub fn as_str(self) -> &'static str {
        match self {
            OutputKind::Iso => "iso",
            OutputKind::DiskImage => "disk-image",
            OutputKind::Installer => "installer",
            OutputKind::Kernel => "kernel",
            OutputKind::Initramfs => "initramfs",
            OutputKind::Metal => "metal",
            OutputKind::SecureBootUki => "secureboot-uki",
        }
    }

    /// Parse an output kind from its name.
    pub fn parse(s: &str) -> Option<OutputKind> {
        match s.trim().to_ascii_lowercase().as_str() {
            "iso" => Some(OutputKind::Iso),
            "disk-image" | "image" | "disk" => Some(OutputKind::DiskImage),
            "installer" => Some(OutputKind::Installer),
            "kernel" => Some(OutputKind::Kernel),
            "initramfs" => Some(OutputKind::Initramfs),
            "metal" => Some(OutputKind::Metal),
            "secureboot-uki" | "uki" => Some(OutputKind::SecureBootUki),
            _ => None,
        }
    }

    /// The on-disk filename produced for this kind, given an architecture name.
    pub fn artifact_filename(self, arch: &str) -> String {
        match self {
            OutputKind::Iso => format!("talos-{arch}.iso"),
            OutputKind::DiskImage | OutputKind::Metal => format!("talos-{arch}.raw"),
            OutputKind::Installer => format!("installer-{arch}.tar"),
            OutputKind::Kernel => format!("vmlinuz-{arch}"),
            OutputKind::Initramfs => format!("initramfs-{arch}.xz"),
            OutputKind::SecureBootUki => format!("talos-{arch}-uki.efi"),
        }
    }

    /// Whether this output kind can be built for SecureBoot.
    pub fn supports_secureboot(self) -> bool {
        matches!(
            self,
            OutputKind::Iso | OutputKind::DiskImage | OutputKind::Metal | OutputKind::SecureBootUki
        )
    }

    /// Whether this output kind uses board overlays.
    pub fn supports_overlay(self) -> bool {
        matches!(self, OutputKind::DiskImage | OutputKind::Metal)
    }

    /// Whether this output kind is a disk-shaped artifact (has a size/format).
    pub fn is_disk(self) -> bool {
        matches!(self, OutputKind::DiskImage | OutputKind::Metal)
    }

    /// Whether the output can be compressed in place.
    pub fn compressible(self) -> bool {
        !matches!(self, OutputKind::Installer)
    }
}

impl fmt::Display for OutputKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The disk image container format a disk output is converted to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskFormat {
    /// Raw sector image (`.raw`).
    Raw,
    /// QEMU copy-on-write v2 (`.qcow2`).
    Qcow2,
    /// VMware disk (`.vmdk`).
    Vmdk,
    /// VirtualBox / OVF disk (`.vhd`).
    Vhd,
    /// Amazon machine image stream-optimized (`.raw` uploaded as AMI).
    Ova,
}

impl DiskFormat {
    /// File extension for this format.
    pub fn extension(self) -> &'static str {
        match self {
            DiskFormat::Raw => "raw",
            DiskFormat::Qcow2 => "qcow2",
            DiskFormat::Vmdk => "vmdk",
            DiskFormat::Vhd => "vhd",
            DiskFormat::Ova => "ova",
        }
    }

    /// Parse from extension/name.
    pub fn parse(s: &str) -> Option<DiskFormat> {
        match s.trim().to_ascii_lowercase().as_str() {
            "raw" => Some(DiskFormat::Raw),
            "qcow2" | "qcow" => Some(DiskFormat::Qcow2),
            "vmdk" => Some(DiskFormat::Vmdk),
            "vhd" | "vpc" => Some(DiskFormat::Vhd),
            "ova" | "ovf" => Some(DiskFormat::Ova),
            _ => None,
        }
    }
}

/// The compression applied to the final artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    /// No compression.
    None,
    /// gzip (`.gz`).
    Gzip,
    /// xz (`.xz`) — Talos's default for initramfs and disk images.
    Xz,
    /// zstd (`.zst`).
    Zstd,
}

impl Compression {
    /// File extension suffix added when compressed (empty for `None`).
    pub fn suffix(self) -> &'static str {
        match self {
            Compression::None => "",
            Compression::Gzip => ".gz",
            Compression::Xz => ".xz",
            Compression::Zstd => ".zst",
        }
    }

    /// Parse from name.
    pub fn parse(s: &str) -> Option<Compression> {
        match s.trim().to_ascii_lowercase().as_str() {
            "none" | "" => Some(Compression::None),
            "gzip" | "gz" => Some(Compression::Gzip),
            "xz" => Some(Compression::Xz),
            "zstd" | "zst" => Some(Compression::Zstd),
            _ => None,
        }
    }

    /// Apply a (modeled) compression ratio to a byte length. Real codecs are an
    /// OS boundary; we model deterministic ratios so builds are testable.
    pub fn apply(self, raw_len: u64) -> u64 {
        let ratio_num: u64 = match self {
            Compression::None => 100,
            Compression::Gzip => 45,
            Compression::Xz => 30,
            Compression::Zstd => 35,
        };
        // ceil to avoid a 0-length compressed artifact for tiny inputs.
        raw_len.saturating_mul(ratio_num).div_ceil(100)
    }
}

/// The minimum disk image size Talos enforces (in bytes): 1 GiB.
pub const MIN_DISK_SIZE: u64 = 1024 * 1024 * 1024;

/// Per-output options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputOptions {
    /// Disk size in bytes (only meaningful for disk outputs).
    pub disk_size: u64,
    /// Disk container format (only meaningful for disk outputs).
    pub disk_format: DiskFormat,
    /// Compression applied to the final artifact.
    pub compression: Compression,
}

impl OutputOptions {
    /// Sensible defaults for the given output kind.
    pub fn defaults_for(kind: OutputKind) -> OutputOptions {
        let compression = match kind {
            OutputKind::Initramfs | OutputKind::DiskImage | OutputKind::Metal => Compression::Xz,
            _ => Compression::None,
        };
        OutputOptions {
            disk_size: MIN_DISK_SIZE,
            disk_format: DiskFormat::Raw,
            compression,
        }
    }

    /// Validate the options against the output kind.
    pub fn validate(&self, kind: OutputKind) -> Vec<OptionError> {
        let mut errs = Vec::new();

        if kind.is_disk() {
            if self.disk_size < MIN_DISK_SIZE {
                errs.push(OptionError::DiskTooSmall {
                    requested: self.disk_size,
                    minimum: MIN_DISK_SIZE,
                });
            }
        } else if self.disk_format != DiskFormat::Raw {
            // Non-disk outputs must not carry a non-raw disk format.
            errs.push(OptionError::FormatOnNonDisk(kind));
        }

        if self.compression != Compression::None && !kind.compressible() {
            errs.push(OptionError::CompressionUnsupported(kind));
        }

        errs
    }
}

/// An output-option validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionError {
    /// Requested disk size below the enforced minimum.
    DiskTooSmall {
        /// Requested size in bytes.
        requested: u64,
        /// Minimum allowed size in bytes.
        minimum: u64,
    },
    /// A non-raw disk format was set on a non-disk output.
    FormatOnNonDisk(OutputKind),
    /// Compression requested for an output that cannot be compressed.
    CompressionUnsupported(OutputKind),
}

impl fmt::Display for OptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionError::DiskTooSmall { requested, minimum } => {
                write!(f, "disk size {requested} below minimum {minimum}")
            }
            OptionError::FormatOnNonDisk(k) => {
                write!(f, "disk format is meaningless for output '{}'", k.as_str())
            }
            OptionError::CompressionUnsupported(k) => {
                write!(f, "compression not supported for output '{}'", k.as_str())
            }
        }
    }
}

impl std::error::Error for OptionError {}
