//! talos-imager
//!
//! Models Talos's imager (`cmd/imager` + `pkg/imager`): assembling boot assets
//! (kernel, initramfs, ISO, disk images, secureboot UKI) from a [`Profile`]
//! describing platform, architecture, [`OutputKind`], compression and bundled
//! system extensions / board overlays.
//!
//! The build pipeline is modeled as a small state machine ([`Imager`]) that
//! validates the profile, gathers boot assets from an [`AssetSource`] (an OS
//! boundary with an in-memory implementation), assembles the requested output,
//! applies compression and produces an [`Artifact`]. All filesystem, signing
//! and codec work is behind traits so the whole pipeline is testable offline.

pub mod disk_image;
pub mod iso;
pub mod output;
pub mod overlay;
pub mod profile;
pub mod uki;

pub use disk_image::{DiskImage, DiskImageError, Partition};
pub use iso::{IsoEntry, IsoImage};
pub use output::{Compression, DiskFormat, MIN_DISK_SIZE, OptionError, OutputKind, OutputOptions};
pub use overlay::{Overlay, OverlayRegistry};
pub use profile::{Arch, Input, Profile, ProfileError, SecureBootMode, SystemExtension};
pub use uki::{InMemoryUkiSigner, SignedUki, UkiError, UkiLayout, UkiSection, UkiSigner};

use std::collections::BTreeMap;

/// The boot assets the imager consumes, identified by logical name. In Talos
/// these are extracted from the base installer image's `/usr/install` tree.
///
/// Modeled as a trait (an OS boundary) so builds can run against an in-memory
/// asset store in tests.
pub trait AssetSource {
    /// Return the byte length of the named asset, or `None` if absent.
    fn len_of(&self, name: &str) -> Option<u64>;

    /// Whether the named asset exists.
    fn has(&self, name: &str) -> bool {
        self.len_of(name).is_some()
    }
}

/// An in-memory [`AssetSource`] mapping asset names to byte lengths.
#[derive(Debug, Default, Clone)]
pub struct MemoryAssets {
    assets: BTreeMap<String, u64>,
}

impl MemoryAssets {
    /// An empty asset store.
    pub fn new() -> MemoryAssets {
        MemoryAssets {
            assets: BTreeMap::new(),
        }
    }

    /// A store pre-populated with a kernel and initramfs for `arch`.
    pub fn with_kernel_initramfs(arch: Arch, kernel_len: u64, initramfs_len: u64) -> MemoryAssets {
        let mut a = MemoryAssets::new();
        a.insert(arch.kernel_image_name(), kernel_len);
        a.insert("initramfs.xz", initramfs_len);
        a
    }

    /// Insert/replace an asset.
    pub fn insert(&mut self, name: impl Into<String>, len: u64) {
        self.assets.insert(name.into(), len);
    }
}

impl AssetSource for MemoryAssets {
    fn len_of(&self, name: &str) -> Option<u64> {
        self.assets.get(name).copied()
    }
}

/// The product of a successful build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    /// Output kind that was built.
    pub kind: OutputKind,
    /// Final filename including compression suffix.
    pub filename: String,
    /// Uncompressed size in bytes.
    pub raw_size: u64,
    /// Final size after compression (equals `raw_size` if uncompressed).
    pub final_size: u64,
    /// Compression applied.
    pub compression: Compression,
    /// If a UKI was signed, the signing certificate fingerprint.
    pub signed_by: Option<String>,
}

impl Artifact {
    /// The compression ratio achieved as a percentage (final/raw * 100).
    pub fn ratio_percent(&self) -> u64 {
        if self.raw_size == 0 {
            return 100;
        }
        self.final_size.saturating_mul(100) / self.raw_size
    }
}

/// The phases the imager moves through while building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildPhase {
    /// Validating the profile.
    Validate,
    /// Gathering boot assets from the [`AssetSource`].
    GatherAssets,
    /// Assembling the requested output.
    Assemble,
    /// Compressing the assembled artifact.
    Compress,
    /// Build complete.
    Done,
}

/// A build failure with the phase it occurred in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// The profile failed validation.
    InvalidProfile(Vec<ProfileError>),
    /// A required boot asset was missing.
    MissingAsset(String),
    /// Disk image assembly failed.
    DiskImage(DiskImageError),
    /// UKI assembly/signing failed.
    Uki(UkiError),
    /// SecureBoot was requested but no signer was supplied.
    NoSigner,
    /// An overlay was named in the profile but not found in the registry.
    OverlayNotFound(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::InvalidProfile(es) => {
                write!(f, "invalid profile: ")?;
                for (i, e) in es.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{e}")?;
                }
                Ok(())
            }
            BuildError::MissingAsset(n) => write!(f, "missing boot asset '{n}'"),
            BuildError::DiskImage(e) => write!(f, "disk image: {e}"),
            BuildError::Uki(e) => write!(f, "uki: {e}"),
            BuildError::NoSigner => write!(f, "secureboot requested but no signer supplied"),
            BuildError::OverlayNotFound(n) => write!(f, "overlay '{n}' not found"),
        }
    }
}

impl std::error::Error for BuildError {}

/// The imager build engine. Holds the asset source, overlay registry and an
/// optional UKI signer; [`Imager::build`] drives the [`BuildPhase`] pipeline.
pub struct Imager<'a, A: AssetSource> {
    assets: &'a A,
    overlays: &'a OverlayRegistry,
    signer: Option<&'a dyn UkiSigner>,
}

impl<'a, A: AssetSource> Imager<'a, A> {
    /// Construct an imager over the given asset source and overlay registry.
    pub fn new(assets: &'a A, overlays: &'a OverlayRegistry) -> Imager<'a, A> {
        Imager {
            assets,
            overlays,
            signer: None,
        }
    }

    /// Attach a UKI signer (required for SecureBoot outputs).
    pub fn with_signer(mut self, signer: &'a dyn UkiSigner) -> Imager<'a, A> {
        self.signer = Some(signer);
        self
    }

    /// Resolve the kernel and initramfs lengths for a profile, erroring if an
    /// asset is missing.
    fn boot_assets(&self, profile: &Profile) -> Result<(u64, u64), BuildError> {
        let kernel = self
            .assets
            .len_of(profile.arch.kernel_image_name())
            .ok_or_else(|| {
                BuildError::MissingAsset(profile.arch.kernel_image_name().to_string())
            })?;
        let initramfs = self
            .assets
            .len_of("initramfs.xz")
            .ok_or_else(|| BuildError::MissingAsset("initramfs.xz".to_string()))?;
        Ok((kernel, initramfs))
    }

    /// Resolve and validate the overlay named in the profile, if any.
    fn resolve_overlay(&self, profile: &Profile) -> Result<Option<&'a Overlay>, BuildError> {
        match &profile.overlay {
            None => Ok(None),
            Some(name) => self
                .overlays
                .get(name)
                .map(Some)
                .ok_or_else(|| BuildError::OverlayNotFound(name.clone())),
        }
    }

    /// Build a signed UKI for the profile.
    fn build_uki(
        &self,
        profile: &Profile,
        kernel: u64,
        initramfs: u64,
    ) -> Result<SignedUki, BuildError> {
        let signer = self.signer.ok_or(BuildError::NoSigner)?;
        let cmdline = profile.cmdline();
        let os_release = format!("NAME=Talos\nVERSION={}\n", profile.input.version);
        let uname = format!("{}-talos", profile.input.version);
        let layout = UkiLayout::build(
            profile.arch,
            kernel,
            initramfs,
            &cmdline,
            &os_release,
            &uname,
        );
        signer.sign(&layout).map_err(BuildError::Uki)
    }

    /// Run the full build pipeline for `profile`, returning the [`Artifact`].
    pub fn build(&self, profile: &Profile) -> Result<Artifact, BuildError> {
        // Phase: Validate.
        let errs = profile.validate();
        if !errs.is_empty() {
            return Err(BuildError::InvalidProfile(errs));
        }

        // Phase: GatherAssets.
        let (kernel, initramfs) = self.boot_assets(profile)?;
        let overlay = self.resolve_overlay(profile)?;

        // Phase: Assemble.
        let (raw_size, signed_by) = self.assemble(profile, kernel, initramfs, overlay)?;

        // Phase: Compress.
        let compression = profile.options.compression;
        let final_size = compression.apply(raw_size);

        // Phase: Done.
        let base = profile.output.artifact_filename(profile.arch.as_str());
        let filename = format!("{base}{}", compression.suffix());

        Ok(Artifact {
            kind: profile.output,
            filename,
            raw_size,
            final_size,
            compression,
            signed_by,
        })
    }

    /// Assemble the requested output, returning its raw size and (for UKI/iso
    /// in secureboot mode) the signing fingerprint.
    fn assemble(
        &self,
        profile: &Profile,
        kernel: u64,
        initramfs: u64,
        overlay: Option<&Overlay>,
    ) -> Result<(u64, Option<String>), BuildError> {
        match profile.output {
            OutputKind::Kernel => Ok((kernel, None)),
            OutputKind::Initramfs => Ok((initramfs, None)),
            OutputKind::Installer => {
                // Installer tar bundles kernel + initramfs + extensions metadata.
                let ext_bytes: u64 = profile.extensions.len() as u64 * 4096;
                Ok((kernel + initramfs + ext_bytes, None))
            }
            OutputKind::SecureBootUki => {
                let uki = self.build_uki(profile, kernel, initramfs)?;
                Ok((uki.signed_len, Some(uki.signer_fingerprint)))
            }
            OutputKind::Iso => {
                let (uki_len, signed_by) = if profile.secureboot.is_enabled() {
                    let uki = self.build_uki(profile, kernel, initramfs)?;
                    (Some(uki.signed_len), Some(uki.signer_fingerprint))
                } else {
                    (None, None)
                };
                let iso = IsoImage::assemble(
                    profile.arch,
                    profile.secureboot,
                    kernel,
                    initramfs,
                    uki_len,
                );
                Ok((iso.image_size(), signed_by))
            }
            OutputKind::DiskImage | OutputKind::Metal => {
                let disk = DiskImage::assemble(
                    profile.options.disk_size,
                    profile.options.disk_format,
                    overlay,
                )
                .map_err(BuildError::DiskImage)?;
                disk.check_layout().map_err(BuildError::DiskImage)?;
                let signed_by = if profile.secureboot.is_enabled() {
                    let uki = self.build_uki(profile, kernel, initramfs)?;
                    Some(uki.signer_fingerprint)
                } else {
                    None
                };
                Ok((disk.size, signed_by))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metal_assets() -> MemoryAssets {
        MemoryAssets::with_kernel_initramfs(Arch::Amd64, 8_000_000, 40_000_000)
    }

    #[test]
    fn arch_parse_roundtrip() {
        assert_eq!(Arch::parse("x86_64"), Some(Arch::Amd64));
        assert_eq!(Arch::parse("aarch64"), Some(Arch::Arm64));
        assert_eq!(Arch::Amd64.as_str(), "amd64");
        assert!(Arch::parse("riscv").is_none());
    }

    #[test]
    fn output_kind_capabilities() {
        assert!(OutputKind::Iso.supports_secureboot());
        assert!(!OutputKind::Kernel.supports_secureboot());
        assert!(OutputKind::DiskImage.supports_overlay());
        assert!(!OutputKind::Iso.supports_overlay());
        assert!(!OutputKind::Installer.compressible());
    }

    #[test]
    fn compression_models_ratio_and_suffix() {
        assert_eq!(Compression::None.apply(1000), 1000);
        assert!(Compression::Xz.apply(1000) < Compression::Gzip.apply(1000));
        assert_eq!(Compression::Xz.suffix(), ".xz");
        // tiny input never compresses to zero.
        assert!(Compression::Xz.apply(1) >= 1);
    }

    #[test]
    fn profile_validates_secureboot_only_on_uefi_outputs() {
        let p = Profile::new(
            Arch::Amd64,
            os_kernel::Platform::Metal,
            OutputKind::Kernel,
            "v1.7.0",
        )
        .with_secureboot();
        let errs = p.validate();
        assert!(errs.contains(&ProfileError::SecureBootUnsupported(OutputKind::Kernel)));
    }

    #[test]
    fn profile_rejects_empty_version_and_dup_extension() {
        let mut p = Profile::new(Arch::Amd64, os_kernel::Platform::Metal, OutputKind::Iso, "");
        p = p
            .with_extension(SystemExtension::new("ghcr.io/x/gvisor:1"))
            .with_extension(SystemExtension::new("ghcr.io/x/gvisor:1"));
        let errs = p.validate();
        assert!(errs.contains(&ProfileError::EmptyVersion));
        assert!(
            errs.iter()
                .any(|e| matches!(e, ProfileError::DuplicateExtension(_)))
        );
    }

    #[test]
    fn arm64_disk_image_requires_overlay() {
        let p = Profile::new(
            Arch::Arm64,
            os_kernel::Platform::Metal,
            OutputKind::DiskImage,
            "v1.7.0",
        );
        assert!(p.validate().contains(&ProfileError::Arm64RequiresOverlay));
        let p2 = p.with_overlay("rpi_generic");
        assert!(!p2.validate().contains(&ProfileError::Arm64RequiresOverlay));
    }

    #[test]
    fn cmdline_includes_platform_and_secureboot_marker() {
        let p = Profile::new(
            Arch::Amd64,
            os_kernel::Platform::Aws,
            OutputKind::Iso,
            "v1.7.0",
        )
        .with_secureboot();
        let cmd = p.cmdline();
        assert!(cmd.contains("talos.platform=aws"));
        assert!(cmd.contains("talos.secureboot=1"));
    }

    #[test]
    fn build_kernel_output_is_passthrough() {
        let assets = metal_assets();
        let reg = OverlayRegistry::new();
        let imager = Imager::new(&assets, &reg);
        let p = Profile::new(
            Arch::Amd64,
            os_kernel::Platform::Metal,
            OutputKind::Kernel,
            "v1.7.0",
        );
        let art = imager.build(&p).expect("build");
        assert_eq!(art.raw_size, 8_000_000);
        assert_eq!(art.final_size, 8_000_000); // kernel uncompressed by default
        assert_eq!(art.filename, "vmlinuz-amd64");
    }

    #[test]
    fn build_missing_asset_errors() {
        let assets = MemoryAssets::new();
        let reg = OverlayRegistry::new();
        let imager = Imager::new(&assets, &reg);
        let p = Profile::new(
            Arch::Amd64,
            os_kernel::Platform::Metal,
            OutputKind::Initramfs,
            "v1.7.0",
        );
        match imager.build(&p) {
            Err(BuildError::MissingAsset(_)) => {}
            other => panic!("expected MissingAsset, got {other:?}"),
        }
    }

    #[test]
    fn build_iso_compressed_is_smaller_than_raw() {
        let assets = metal_assets();
        let reg = OverlayRegistry::new();
        let imager = Imager::new(&assets, &reg);
        let mut p = Profile::new(
            Arch::Amd64,
            os_kernel::Platform::Metal,
            OutputKind::Iso,
            "v1.7.0",
        );
        p.options.compression = Compression::Zstd;
        let art = imager.build(&p).expect("build");
        assert!(art.final_size < art.raw_size);
        assert!(art.filename.ends_with(".iso.zst"));
        assert!(art.ratio_percent() < 100);
    }

    #[test]
    fn build_secureboot_uki_requires_and_records_signer() {
        let assets = metal_assets();
        let reg = OverlayRegistry::new();
        let p = Profile::new(
            Arch::Amd64,
            os_kernel::Platform::Metal,
            OutputKind::SecureBootUki,
            "v1.7.0",
        )
        .with_secureboot();

        // Without a signer it fails.
        let no_signer = Imager::new(&assets, &reg);
        assert_eq!(no_signer.build(&p), Err(BuildError::NoSigner));

        // With a signer the fingerprint is recorded.
        let signer = InMemoryUkiSigner::new("ab12cd34");
        let imager = Imager::new(&assets, &reg).with_signer(&signer);
        let art = imager.build(&p).expect("build");
        assert_eq!(art.signed_by.as_deref(), Some("ab12cd34"));
        assert!(art.raw_size > 8_000_000 + 40_000_000);
    }

    #[test]
    fn build_disk_image_with_overlay_uses_board_offset() {
        let assets = MemoryAssets::with_kernel_initramfs(Arch::Arm64, 8_000_000, 40_000_000);
        let reg = OverlayRegistry::with_builtins();
        let imager = Imager::new(&assets, &reg);
        let mut p = Profile::new(
            Arch::Arm64,
            os_kernel::Platform::Metal,
            OutputKind::DiskImage,
            "v1.7.0",
        )
        .with_overlay("rpi_generic");
        p.options.disk_size = 4 * 1024 * 1024 * 1024;
        let art = imager.build(&p).expect("build");
        assert_eq!(art.kind, OutputKind::DiskImage);
        assert_eq!(art.raw_size, 4 * 1024 * 1024 * 1024);

        // Build the disk directly to inspect the offset.
        let board = reg.get("rpi_generic").unwrap();
        let disk = DiskImage::assemble(p.options.disk_size, DiskFormat::Raw, Some(board)).unwrap();
        assert_eq!(
            disk.partition("EFI").unwrap().start,
            board.boot_partition_offset
        );
        disk.check_layout().unwrap();
    }

    #[test]
    fn build_unknown_overlay_errors() {
        let assets = MemoryAssets::with_kernel_initramfs(Arch::Arm64, 1, 1);
        let reg = OverlayRegistry::new();
        let imager = Imager::new(&assets, &reg);
        let p = Profile::new(
            Arch::Arm64,
            os_kernel::Platform::Metal,
            OutputKind::DiskImage,
            "v1.7.0",
        )
        .with_overlay("nonexistent");
        match imager.build(&p) {
            Err(BuildError::OverlayNotFound(n)) => assert_eq!(n, "nonexistent"),
            other => panic!("expected OverlayNotFound, got {other:?}"),
        }
    }

    #[test]
    fn disk_image_too_small_errors() {
        let err = DiskImage::assemble(MIN_DISK_SIZE / 2, DiskFormat::Raw, None);
        assert!(matches!(err, Err(DiskImageError::TooSmall { .. })));
    }

    #[test]
    fn uki_layout_validation_and_total_len() {
        let layout = UkiLayout::build(
            Arch::Amd64,
            100,
            200,
            "console=ttyS0",
            "NAME=Talos",
            "v1-talos",
        );
        layout.validate().unwrap();
        assert!(layout.total_len() > 300);
        let empty = UkiLayout::build(Arch::Amd64, 0, 200, "x", "y", "z");
        assert_eq!(empty.validate(), Err(UkiError::EmptySection(".linux")));
    }

    #[test]
    fn iso_secureboot_drops_bios_boot() {
        let sb = IsoImage::assemble(Arch::Amd64, SecureBootMode::Enabled, 1, 1, Some(5000));
        assert!(!sb.bios_boot);
        assert!(sb.uefi_boot);
        assert!(sb.is_bootable());
        let legacy = IsoImage::assemble(Arch::Amd64, SecureBootMode::Disabled, 1, 1, None);
        assert!(legacy.bios_boot);
        assert!(legacy.contains("/boot/vmlinuz"));
    }

    #[test]
    fn options_reject_format_on_non_disk() {
        let mut opts = OutputOptions::defaults_for(OutputKind::Iso);
        opts.disk_format = DiskFormat::Qcow2;
        let errs = opts.validate(OutputKind::Iso);
        assert!(errs.contains(&OptionError::FormatOnNonDisk(OutputKind::Iso)));
    }

    #[test]
    fn extension_name_strips_registry_and_tag() {
        let ext = SystemExtension::new("ghcr.io/siderolabs/gvisor:20240305.0");
        assert_eq!(ext.name(), "gvisor");
    }
}
