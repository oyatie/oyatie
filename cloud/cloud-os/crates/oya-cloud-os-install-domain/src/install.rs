//! The upgrade / install state machine.
//!
//! Models the high-level flow in `cmd/installer` + `internal/pkg/install`:
//! probe the disk, extract the new image onto the *inactive* A/B slot, point
//! the bootloader at it, and reboot. If the post-upgrade health check fails,
//! the previous slot is still intact and the bootloader reverts to it.

use crate::boot_entry::{BootEntry, BootSlot};
use crate::bootloader::{BootResult, Bootloader, BootloaderError};

/// The disk image to extract, identified by version and an opaque content
/// digest. Mirrors the `imager`/`extract` boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallImage {
    /// OS version this image carries (e.g. `"v1.7.0"`).
    pub version: String,
    /// Content digest (e.g. sha256 hex) used to detect corruption.
    pub digest: String,
    /// Kernel command line to bake into the slot.
    pub cmdline: String,
}

impl InstallImage {
    /// Build an install image descriptor.
    pub fn new(version: &str, digest: &str, cmdline: &str) -> InstallImage {
        InstallImage {
            version: version.to_string(),
            digest: digest.to_string(),
            cmdline: cmdline.to_string(),
        }
    }
}

/// The phases an upgrade moves through. Drives ordering and prevents illegal
/// transitions (e.g. you cannot `commit` before `extract`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradePhase {
    /// Nothing started yet.
    Idle,
    /// Disk probed, target slot chosen.
    Probed,
    /// Image extracted onto the target slot.
    Extracted,
    /// Bootloader updated to point at the new slot.
    Committed,
    /// Post-upgrade health check confirmed the new slot booted.
    Confirmed,
    /// Upgrade failed and the previous slot was restored.
    RolledBack,
}

/// Errors specific to the upgrade flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeError {
    /// A phase transition was attempted out of order.
    WrongPhase {
        expected: UpgradePhase,
        actual: UpgradePhase,
    },
    /// The extracted image digest did not match what was advertised.
    DigestMismatch { expected: String, actual: String },
    /// An underlying bootloader operation failed.
    Bootloader(BootloaderError),
    /// The disk is too small to hold the required layout.
    DiskTooSmall { needed: u64, available: u64 },
    /// The target version is not newer than the current one (and `force` unset).
    NotNewer { current: String, target: String },
}

impl From<BootloaderError> for UpgradeError {
    fn from(e: BootloaderError) -> Self {
        UpgradeError::Bootloader(e)
    }
}

/// The boundary for materializing an image onto a slot. The real installer
/// writes partitions/extracts a squashfs; tests supply an in-memory fake.
pub trait ImageExtractor {
    /// Extract `image` onto `target`, returning the digest actually written.
    /// A faithful implementation streams bytes and hashes them.
    fn extract(&mut self, image: &InstallImage, target: BootEntry) -> String;
}

/// Drives an upgrade across a bootloader and an image extractor.
pub struct Upgrade<'a, B: Bootloader, X: ImageExtractor> {
    bootloader: &'a mut B,
    extractor: &'a mut X,
    phase: UpgradePhase,
    /// Slot the in-progress upgrade targets (the inactive one).
    target: Option<BootEntry>,
    /// The slot that was active before this upgrade started.
    previous_default: Option<BootEntry>,
}

impl<'a, B: Bootloader, X: ImageExtractor> Upgrade<'a, B, X> {
    /// Begin an upgrade against the given bootloader and extractor.
    pub fn new(bootloader: &'a mut B, extractor: &'a mut X) -> Self {
        Upgrade {
            bootloader,
            extractor,
            phase: UpgradePhase::Idle,
            target: None,
            previous_default: None,
        }
    }

    /// Current phase.
    pub fn phase(&self) -> UpgradePhase {
        self.phase
    }

    /// Slot this upgrade targets, once probed.
    pub fn target(&self) -> Option<BootEntry> {
        self.target
    }

    fn expect(&self, expected: UpgradePhase) -> Result<(), UpgradeError> {
        if self.phase == expected {
            Ok(())
        } else {
            Err(UpgradeError::WrongPhase {
                expected,
                actual: self.phase,
            })
        }
    }

    /// Probe the disk: record the current default and choose the inactive slot
    /// as the install target. If nothing is installed yet, default to A.
    pub fn probe(&mut self) -> Result<BootEntry, UpgradeError> {
        self.expect(UpgradePhase::Idle)?;
        let current = self.bootloader.default_entry();
        let target = match current {
            Some(c) => c.other(),
            None => BootEntry::A,
        };
        self.previous_default = current;
        self.target = Some(target);
        self.phase = UpgradePhase::Probed;
        Ok(target)
    }

    /// Extract the image onto the target slot, verifying the resulting digest.
    pub fn extract(&mut self, image: &InstallImage) -> Result<(), UpgradeError> {
        self.expect(UpgradePhase::Probed)?;
        let target = self.target.expect("probed implies target");
        let written = self.extractor.extract(image, target);
        if written != image.digest {
            return Err(UpgradeError::DigestMismatch {
                expected: image.digest.clone(),
                actual: written,
            });
        }
        self.phase = UpgradePhase::Extracted;
        Ok(())
    }

    /// Point the bootloader at the freshly-extracted slot.
    pub fn commit(&mut self, image: &InstallImage) -> Result<(), UpgradeError> {
        self.expect(UpgradePhase::Extracted)?;
        let target = self.target.expect("extracted implies target");
        let slot = BootSlot::conventional(target, &image.version, &image.cmdline);
        self.bootloader.install(&slot)?;
        self.phase = UpgradePhase::Committed;
        Ok(())
    }

    /// Mark the upgrade confirmed (the new slot booted and passed health checks).
    pub fn confirm(&mut self) -> Result<(), UpgradeError> {
        self.expect(UpgradePhase::Committed)?;
        self.phase = UpgradePhase::Confirmed;
        Ok(())
    }

    /// Roll back to the previous default after a failed upgrade. Valid once the
    /// bootloader has been committed (so there is a previous entry to restore).
    pub fn rollback(&mut self) -> Result<(), UpgradeError> {
        if self.phase != UpgradePhase::Committed {
            return Err(UpgradeError::WrongPhase {
                expected: UpgradePhase::Committed,
                actual: self.phase,
            });
        }
        self.bootloader.revert()?;
        self.phase = UpgradePhase::RolledBack;
        Ok(())
    }
}

/// Run a full happy-path upgrade end to end, returning the slot now active.
pub fn run_upgrade<B: Bootloader, X: ImageExtractor>(
    bootloader: &mut B,
    extractor: &mut X,
    image: &InstallImage,
) -> Result<BootEntry, UpgradeError> {
    let mut up = Upgrade::new(bootloader, extractor);
    let target = up.probe()?;
    up.extract(image)?;
    up.commit(image)?;
    up.confirm()?;
    Ok(target)
}

/// Convenience: a [`BootResult`] flavored wrapper for callers working purely in
/// bootloader error terms.
pub fn install_fresh<B: Bootloader>(bootloader: &mut B, slot: &BootSlot) -> BootResult<()> {
    bootloader.install(slot)
}

/// The well-known GPT partitions Talos lays down on the install disk, in the
/// order they are created. Mirrors `internal/pkg/partition` labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Partition {
    /// EFI System Partition (FAT32), holds the bootloader + UKIs.
    Efi,
    /// BIOS boot partition (GRUB legacy embedding).
    Bios,
    /// `BOOT`: kernels/initramfs for the A/B slots.
    Boot,
    /// `META`: small partition for install/upgrade metadata.
    Meta,
    /// `STATE`: machine config + persisted node state.
    State,
    /// `EPHEMERAL`: container/ephemeral data, grown to fill the disk.
    Ephemeral,
}

impl Partition {
    /// The GPT partition label Talos assigns.
    pub fn label(self) -> &'static str {
        match self {
            Partition::Efi => "EFI",
            Partition::Bios => "BIOS",
            Partition::Boot => "BOOT",
            Partition::Meta => "META",
            Partition::State => "STATE",
            Partition::Ephemeral => "EPHEMERAL",
        }
    }

    /// The minimum size in MiB Talos requires for this partition. `Ephemeral`
    /// has no fixed size (it grows to fill the disk) and returns 0.
    #[allow(clippy::match_same_arms)] // distinct partitions that happen to share a size
    pub fn min_size_mib(self) -> u64 {
        match self {
            Partition::Efi => 100,
            Partition::Bios => 1,
            Partition::Boot => 1000,
            Partition::Meta => 1,
            Partition::State => 100,
            Partition::Ephemeral => 0,
        }
    }

    /// Whether this partition's size is fixed (vs. grown to fill remaining space).
    pub fn is_fixed_size(self) -> bool {
        !matches!(self, Partition::Ephemeral)
    }
}

/// The disk layout an installer produces, parameterized by firmware mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskLayout {
    /// Total disk capacity in MiB.
    pub disk_mib: u64,
    /// Whether the system boots via UEFI (ESP) or legacy BIOS (BIOS boot part).
    pub uefi: bool,
}

impl DiskLayout {
    /// Build a layout descriptor.
    pub fn new(disk_mib: u64, uefi: bool) -> DiskLayout {
        DiskLayout { disk_mib, uefi }
    }

    /// The ordered partitions for this layout. UEFI installs get an ESP;
    /// legacy installs get a BIOS boot partition instead.
    pub fn partitions(&self) -> Vec<Partition> {
        let mut p = Vec::new();
        if self.uefi {
            p.push(Partition::Efi);
        } else {
            p.push(Partition::Bios);
        }
        p.push(Partition::Boot);
        p.push(Partition::Meta);
        p.push(Partition::State);
        p.push(Partition::Ephemeral);
        p
    }

    /// Sum of the fixed-size partitions' minimums (everything except EPHEMERAL).
    pub fn fixed_mib(&self) -> u64 {
        self.partitions()
            .iter()
            .filter(|p| p.is_fixed_size())
            .map(|p| p.min_size_mib())
            .sum()
    }

    /// The space EPHEMERAL would receive (disk minus fixed partitions). Returns
    /// `None` if the disk is too small to hold even the fixed partitions.
    pub fn ephemeral_mib(&self) -> Option<u64> {
        self.disk_mib.checked_sub(self.fixed_mib())
    }

    /// Validate the disk can hold the layout with at least `min_ephemeral_mib`
    /// left for EPHEMERAL.
    pub fn validate(&self, min_ephemeral_mib: u64) -> Result<(), UpgradeError> {
        match self.ephemeral_mib() {
            Some(e) if e >= min_ephemeral_mib => Ok(()),
            _ => Err(UpgradeError::DiskTooSmall {
                needed: self.fixed_mib() + min_ephemeral_mib,
                available: self.disk_mib,
            }),
        }
    }
}

/// A streaming, hashing [`ImageExtractor`] that records what it wrote to each
/// slot and computes a content digest the same way the real installer verifies
/// the unpacked image. Pre-seeded with the bytes for an image version.
pub struct HashingExtractor {
    /// version -> raw image bytes the extractor will "write".
    images: std::collections::BTreeMap<String, Vec<u8>>,
    /// Record of (version, slot) extractions performed, in order.
    pub writes: Vec<(String, BootEntry)>,
    /// If set, corrupt the next write by flipping a byte (simulates bad media).
    pub corrupt_next: bool,
}

impl HashingExtractor {
    /// Create an empty extractor.
    pub fn new() -> HashingExtractor {
        HashingExtractor {
            images: std::collections::BTreeMap::new(),
            writes: Vec::new(),
            corrupt_next: false,
        }
    }

    /// Register the raw bytes backing an image version.
    pub fn add_image(&mut self, version: &str, bytes: impl Into<Vec<u8>>) {
        self.images.insert(version.to_string(), bytes.into());
    }

    /// The digest the extractor would compute for a version's registered bytes,
    /// in the `"sha-fnv:<hex>"` form. Callers use this to set
    /// [`InstallImage::digest`] so a clean write verifies.
    pub fn digest_for(&self, version: &str) -> Option<String> {
        self.images.get(version).map(|b| digest_bytes(b))
    }
}

impl Default for HashingExtractor {
    fn default() -> Self {
        HashingExtractor::new()
    }
}

/// Compute the content digest of raw image bytes (FNV-1a based, hex tagged).
pub fn digest_bytes(bytes: &[u8]) -> String {
    format!(
        "sha-fnv:{}",
        crate::secureboot::hex64(crate::secureboot::fnv1a(bytes))
    )
}

impl ImageExtractor for HashingExtractor {
    fn extract(&mut self, image: &InstallImage, target: BootEntry) -> String {
        self.writes.push((image.version.clone(), target));
        let mut bytes = self.images.get(&image.version).cloned().unwrap_or_default();
        if self.corrupt_next && !bytes.is_empty() {
            bytes[0] ^= 0xff;
            self.corrupt_next = false;
        }
        digest_bytes(&bytes)
    }
}

/// Options controlling an upgrade, mirroring `machine.UpgradeRequest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeOptions {
    /// Skip wiping EPHEMERAL (preserve user data across the upgrade).
    pub preserve: bool,
    /// Force the upgrade even if the target version is the same or older.
    pub force: bool,
    /// Reboot into the new slot immediately rather than staging it.
    pub stage: bool,
}

impl Default for UpgradeOptions {
    fn default() -> Self {
        UpgradeOptions {
            preserve: true,
            force: false,
            stage: false,
        }
    }
}

/// Compare two `vX.Y.Z` version strings. Returns `Ordering` of `a` vs `b`,
/// or `None` if either is not parseable. A leading `v` is tolerated and any
/// `-suffix` (pre-release/build) is ignored for the numeric comparison.
pub fn compare_versions(a: &str, b: &str) -> Option<std::cmp::Ordering> {
    fn parse(s: &str) -> Option<(u64, u64, u64)> {
        let s = s.trim().trim_start_matches('v');
        let core = s.split(['-', '+']).next().unwrap_or(s);
        let mut it = core.split('.');
        let major = it.next()?.parse().ok()?;
        let minor = it.next().unwrap_or("0").parse().ok()?;
        let patch = it.next().unwrap_or("0").parse().ok()?;
        Some((major, minor, patch))
    }
    Some(parse(a)?.cmp(&parse(b)?))
}

/// Decide whether an upgrade from `current` to `target` is permitted under the
/// given options. Without `force`, downgrades and same-version installs are
/// rejected.
pub fn upgrade_permitted(
    current: &str,
    target: &str,
    opts: &UpgradeOptions,
) -> Result<(), UpgradeError> {
    if opts.force {
        return Ok(());
    }
    match compare_versions(target, current) {
        Some(std::cmp::Ordering::Greater) => Ok(()),
        _ => Err(UpgradeError::NotNewer {
            current: current.to_string(),
            target: target.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grub::Grub;

    /// Honest in-memory extractor: echoes the advertised digest.
    struct GoodExtractor {
        last: Option<(String, BootEntry)>,
    }
    impl ImageExtractor for GoodExtractor {
        fn extract(&mut self, image: &InstallImage, target: BootEntry) -> String {
            self.last = Some((image.version.clone(), target));
            image.digest.clone()
        }
    }

    /// Corrupting extractor: returns the wrong digest.
    struct CorruptExtractor;
    impl ImageExtractor for CorruptExtractor {
        fn extract(&mut self, _image: &InstallImage, _target: BootEntry) -> String {
            "corrupt-digest".to_string()
        }
    }

    fn image(v: &str) -> InstallImage {
        InstallImage::new(v, "sha256:abcd", "talos.platform=metal")
    }

    #[test]
    fn fresh_install_targets_a() {
        let mut g = Grub::new();
        let mut x = GoodExtractor { last: None };
        let active = run_upgrade(&mut g, &mut x, &image("v1.7.0")).unwrap();
        assert_eq!(active, BootEntry::A);
        assert_eq!(g.default_entry(), Some(BootEntry::A));
        assert_eq!(x.last, Some(("v1.7.0".to_string(), BootEntry::A)));
    }

    #[test]
    fn upgrade_targets_inactive_slot() {
        let mut g = Grub::new();
        let mut x = GoodExtractor { last: None };
        run_upgrade(&mut g, &mut x, &image("v1.6.0")).unwrap();
        assert_eq!(g.default_entry(), Some(BootEntry::A));
        // Second upgrade flips to B.
        let active = run_upgrade(&mut g, &mut x, &image("v1.7.0")).unwrap();
        assert_eq!(active, BootEntry::B);
        assert_eq!(g.default_entry(), Some(BootEntry::B));
        let cfg = g.render_config().unwrap();
        assert!(cfg.contains("set default=B"));
        assert!(cfg.contains("set fallback=A"));
    }

    #[test]
    fn digest_mismatch_aborts_before_commit() {
        let mut g = Grub::new();
        let mut x = CorruptExtractor;
        let mut up = Upgrade::new(&mut g, &mut x);
        up.probe().unwrap();
        let err = up.extract(&image("v1.7.0")).unwrap_err();
        assert!(matches!(err, UpgradeError::DigestMismatch { .. }));
        // Bootloader was never touched.
        assert_eq!(g.default_entry(), None);
    }

    #[test]
    fn out_of_order_transition_rejected() {
        let mut g = Grub::new();
        let mut x = GoodExtractor { last: None };
        let mut up = Upgrade::new(&mut g, &mut x);
        // commit before probe/extract is illegal.
        let err = up.commit(&image("v1.7.0")).unwrap_err();
        assert_eq!(
            err,
            UpgradeError::WrongPhase {
                expected: UpgradePhase::Extracted,
                actual: UpgradePhase::Idle
            }
        );
    }

    #[test]
    fn rollback_restores_previous_default() {
        let mut g = Grub::new();
        let mut x = GoodExtractor { last: None };
        // First good install -> A.
        run_upgrade(&mut g, &mut x, &image("v1.6.0")).unwrap();
        // Begin a second upgrade to B but fail health check -> rollback.
        {
            let mut up = Upgrade::new(&mut g, &mut x);
            up.probe().unwrap();
            up.extract(&image("v1.7.0")).unwrap();
            up.commit(&image("v1.7.0")).unwrap();
            assert_eq!(up.phase(), UpgradePhase::Committed);
            up.rollback().unwrap();
            assert_eq!(up.phase(), UpgradePhase::RolledBack);
        }
        // After rollback the bootloader points back at the original slot.
        assert_eq!(g.default_entry(), Some(BootEntry::A));
    }

    #[test]
    fn rollback_before_commit_is_wrong_phase() {
        let mut g = Grub::new();
        let mut x = GoodExtractor { last: None };
        let mut up = Upgrade::new(&mut g, &mut x);
        up.probe().unwrap();
        assert!(matches!(
            up.rollback(),
            Err(UpgradeError::WrongPhase { .. })
        ));
    }

    #[test]
    fn uefi_layout_has_esp_legacy_has_bios() {
        let uefi = DiskLayout::new(20_000, true).partitions();
        assert_eq!(uefi[0], Partition::Efi);
        assert!(uefi.contains(&Partition::Ephemeral));
        let legacy = DiskLayout::new(20_000, false).partitions();
        assert_eq!(legacy[0], Partition::Bios);
        assert!(!legacy.contains(&Partition::Efi));
    }

    #[test]
    fn partition_labels_and_sizes() {
        assert_eq!(Partition::State.label(), "STATE");
        assert!(Partition::Ephemeral.min_size_mib() == 0);
        assert!(!Partition::Ephemeral.is_fixed_size());
        assert!(Partition::Efi.is_fixed_size());
        assert!(Partition::Boot.min_size_mib() >= 1000);
    }

    #[test]
    fn ephemeral_gets_remaining_space() {
        let layout = DiskLayout::new(10_000, true);
        let fixed = layout.fixed_mib();
        assert_eq!(layout.ephemeral_mib(), Some(10_000 - fixed));
        layout.validate(1000).unwrap();
    }

    #[test]
    fn too_small_disk_rejected() {
        let layout = DiskLayout::new(50, true);
        assert!(layout.ephemeral_mib().is_none() || layout.ephemeral_mib() == Some(0));
        let err = layout.validate(500).unwrap_err();
        assert!(matches!(err, UpgradeError::DiskTooSmall { .. }));
    }

    #[test]
    fn hashing_extractor_digest_roundtrips() {
        let mut x = HashingExtractor::new();
        x.add_image("v1.7.0", b"talos-image-bytes".to_vec());
        let digest = x.digest_for("v1.7.0").unwrap();
        assert!(digest.starts_with("sha-fnv:"));
        let img = InstallImage::new("v1.7.0", &digest, "talos.platform=metal");
        let mut g = Grub::new();
        let active = run_upgrade(&mut g, &mut x, &img).unwrap();
        assert_eq!(active, BootEntry::A);
        assert_eq!(x.writes, vec![("v1.7.0".to_string(), BootEntry::A)]);
    }

    #[test]
    fn hashing_extractor_detects_corruption() {
        let mut x = HashingExtractor::new();
        x.add_image("v1.7.0", b"talos-image-bytes".to_vec());
        let digest = x.digest_for("v1.7.0").unwrap();
        x.corrupt_next = true;
        let img = InstallImage::new("v1.7.0", &digest, "talos.platform=metal");
        let mut g = Grub::new();
        let err = run_upgrade(&mut g, &mut x, &img).unwrap_err();
        assert!(matches!(err, UpgradeError::DigestMismatch { .. }));
        // Bootloader untouched because the digest check failed before commit.
        assert_eq!(g.default_entry(), None);
    }

    #[test]
    fn version_comparison() {
        use std::cmp::Ordering;
        assert_eq!(
            compare_versions("v1.7.0", "v1.6.0"),
            Some(Ordering::Greater)
        );
        assert_eq!(compare_versions("1.6.0", "v1.6.0"), Some(Ordering::Equal));
        assert_eq!(
            compare_versions("v1.7.0-beta.1", "v1.7.0"),
            Some(Ordering::Equal)
        );
        assert_eq!(
            compare_versions("v1.10.0", "v1.9.0"),
            Some(Ordering::Greater)
        );
        assert_eq!(compare_versions("garbage", "v1.0.0"), None);
    }

    #[test]
    fn upgrade_permission_rules() {
        let opts = UpgradeOptions::default();
        assert!(!opts.force);
        assert!(opts.preserve);
        upgrade_permitted("v1.6.0", "v1.7.0", &opts).unwrap();
        // Downgrade rejected without force.
        assert!(matches!(
            upgrade_permitted("v1.7.0", "v1.6.0", &opts),
            Err(UpgradeError::NotNewer { .. })
        ));
        // Same version rejected without force.
        assert!(upgrade_permitted("v1.7.0", "v1.7.0", &opts).is_err());
        // Force allows anything, even unparseable.
        let forced = UpgradeOptions {
            force: true,
            ..UpgradeOptions::default()
        };
        upgrade_permitted("v1.7.0", "v1.6.0", &forced).unwrap();
        upgrade_permitted("weird", "alsoweird", &forced).unwrap();
    }
}
