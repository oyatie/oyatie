//! A concrete systemd-boot (`sd-boot`) [`Bootloader`] implementation.
//!
//! Models `internal/pkg/bootloader/sdboot`. Unlike GRUB, sd-boot does not use a
//! single script; it reads a global `loader.conf` plus one "Type #1"
//! `.conf` file per boot entry under `loader/entries/`. On Talos/SecureBoot
//! installs each entry points at a single Unified Kernel Image (UKI) — a signed
//! PE binary bundling kernel+initrd+cmdline — rather than separate
//! `linux`/`initrd` lines.
//!
//! sd-boot also supports an EFI variable based one-shot override
//! (`LoaderEntryOneShot`) used to boot the fallback exactly once after a failed
//! upgrade; we model that as [`SdBoot::set_oneshot`].

use crate::boot_entry::{BootEntry, BootSlot};
use crate::bootloader::{BootResult, Bootloader, BootloaderError, BootloaderKind};
use std::fmt::Write as _;

/// The on-disk path layout sd-boot expects on the EFI System Partition.
pub mod paths {
    /// Directory holding Type #1 entry files.
    pub const ENTRIES_DIR: &str = "loader/entries";
    /// Global loader configuration file.
    pub const LOADER_CONF: &str = "loader/loader.conf";
    /// Directory holding UKIs on Talos installs.
    pub const UKI_DIR: &str = "EFI/Linux";
}

/// One sd-boot Type #1 boot entry. A Talos entry references a single UKI
/// (`uki`) when SecureBoot/UKI mode is used, or a separate `linux`/`initrd`
/// pair otherwise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SdBootEntry {
    /// Slot this entry boots.
    pub entry: BootEntry,
    /// Human title shown in the menu.
    pub title: String,
    /// Version sort key (sd-boot sorts entries by `version` descending).
    pub version: String,
    /// Path to the UKI on the ESP, if this is a UKI entry.
    pub uki: Option<String>,
    /// Kernel path (used only for non-UKI entries).
    pub linux: Option<String>,
    /// Initrd path (used only for non-UKI entries).
    pub initrd: Option<String>,
    /// Kernel command line (only meaningful for non-UKI entries; for UKIs the
    /// cmdline is baked into the image and this is informational).
    pub options: String,
}

impl SdBootEntry {
    /// Build a UKI-backed entry with the conventional Talos UKI path
    /// (`EFI/Linux/Talos-<A|B>.efi`).
    pub fn uki(entry: BootEntry, version: &str, cmdline: &str) -> SdBootEntry {
        SdBootEntry {
            entry,
            title: entry.menu_label(version),
            version: version.to_string(),
            uki: Some(format!("{}/Talos-{}.efi", paths::UKI_DIR, entry.as_str())),
            linux: None,
            initrd: None,
            options: cmdline.to_string(),
        }
    }

    /// Build a classic (non-UKI) entry from a [`BootSlot`].
    pub fn from_slot(slot: &BootSlot) -> SdBootEntry {
        let version = slot.version.as_deref().unwrap_or("unknown");
        SdBootEntry {
            entry: slot.entry,
            title: slot.entry.menu_label(version),
            version: version.to_string(),
            uki: None,
            linux: Some(slot.linux.clone()),
            initrd: Some(slot.initrd.clone()),
            options: slot.cmdline.clone(),
        }
    }

    /// The conf-file basename sd-boot stores this entry under, also used as the
    /// entry id for the one-shot/default EFI variables (e.g. `Talos-A.conf`).
    pub fn conf_name(&self) -> String {
        format!("Talos-{}.conf", self.entry.as_str())
    }

    /// The entry identifier (conf name without extension) used by
    /// `LoaderEntryDefault` / `LoaderEntryOneShot`.
    pub fn id(&self) -> String {
        format!("Talos-{}", self.entry.as_str())
    }

    /// Whether this is a UKI entry.
    pub fn is_uki(&self) -> bool {
        self.uki.is_some()
    }

    /// Validate the entry. UKI entries must carry a UKI path; classic entries
    /// must carry both a kernel and an initrd. All entries need a title.
    pub fn validate(&self) -> BootResult<()> {
        if self.title.trim().is_empty() {
            return Err(BootloaderError::InvalidConfig("empty entry title".into()));
        }
        match (&self.uki, &self.linux, &self.initrd) {
            (Some(u), None, None) => {
                if u.trim().is_empty() {
                    return Err(BootloaderError::InvalidConfig("empty uki path".into()));
                }
                Ok(())
            }
            (None, Some(l), Some(i)) => {
                if l.trim().is_empty() {
                    return Err(BootloaderError::InvalidConfig("empty linux path".into()));
                }
                if i.trim().is_empty() {
                    return Err(BootloaderError::InvalidConfig("empty initrd path".into()));
                }
                Ok(())
            }
            (Some(_), _, _) => Err(BootloaderError::InvalidConfig(
                "uki entry must not also set linux/initrd".into(),
            )),
            (None, _, _) => Err(BootloaderError::InvalidConfig(
                "non-uki entry needs both linux and initrd".into(),
            )),
        }
    }

    /// Render this entry as a Type #1 `.conf` file body.
    pub fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "title {}", self.title);
        let _ = writeln!(out, "version {}", self.version);
        if let Some(uki) = &self.uki {
            // sd-boot Type #2 / UKI entries live in EFI/Linux and are
            // auto-discovered, but Talos also writes an explicit efi line so the
            // mapping is deterministic in our model.
            let _ = writeln!(out, "efi /{uki}");
        } else {
            if let Some(l) = &self.linux {
                let _ = writeln!(out, "linux {l}");
            }
            if let Some(i) = &self.initrd {
                let _ = writeln!(out, "initrd {i}");
            }
            if !self.options.is_empty() {
                let _ = writeln!(out, "options {}", self.options);
            }
        }
        out
    }
}

/// In-memory systemd-boot bootloader.
#[derive(Debug, Clone, Default)]
pub struct SdBoot {
    entry_a: Option<SdBootEntry>,
    entry_b: Option<SdBootEntry>,
    default: Option<BootEntry>,
    previous: Option<BootEntry>,
    /// One-shot override (`LoaderEntryOneShot`): boots this slot exactly once.
    oneshot: Option<BootEntry>,
    /// Menu timeout in seconds written to `loader.conf`.
    timeout: u32,
    /// Whether this install uses UKIs (and thus requires SecureBoot signing).
    uki_mode: bool,
}

impl SdBoot {
    /// A fresh classic (non-UKI) sd-boot install.
    pub fn new() -> SdBoot {
        SdBoot {
            timeout: 3,
            ..SdBoot::default()
        }
    }

    /// A fresh sd-boot install that uses signed UKIs (SecureBoot path).
    pub fn new_uki() -> SdBoot {
        SdBoot {
            timeout: 3,
            uki_mode: true,
            ..SdBoot::default()
        }
    }

    /// Whether this install is in UKI mode.
    pub fn is_uki_mode(&self) -> bool {
        self.uki_mode
    }

    /// Configure the menu timeout.
    pub fn set_timeout(&mut self, secs: u32) {
        self.timeout = secs;
    }

    fn entry(&self, e: BootEntry) -> Option<&SdBootEntry> {
        match e {
            BootEntry::A => self.entry_a.as_ref(),
            BootEntry::B => self.entry_b.as_ref(),
        }
    }

    fn put(&mut self, entry: SdBootEntry) {
        match entry.entry {
            BootEntry::A => self.entry_a = Some(entry),
            BootEntry::B => self.entry_b = Some(entry),
        }
    }

    /// Install a fully-formed sd-boot entry (allows UKI entries directly).
    pub fn install_entry(&mut self, entry: &SdBootEntry) -> BootResult<()> {
        entry.validate()?;
        if entry.is_uki() != self.uki_mode {
            return Err(BootloaderError::InvalidConfig(format!(
                "entry uki={} does not match loader uki_mode={}",
                entry.is_uki(),
                self.uki_mode
            )));
        }
        self.previous = self.default;
        let target = entry.entry;
        self.put(entry.clone());
        self.default = Some(target);
        self.oneshot = None;
        Ok(())
    }

    /// The fallback slot (other provisioned entry, if any).
    pub fn fallback_entry(&self) -> Option<BootEntry> {
        let def = self.default?;
        let other = def.other();
        self.entry(other).map(|_| other)
    }

    /// Arm a one-shot boot of the given slot (models `LoaderEntryOneShot`).
    /// Fails if that slot is not provisioned.
    pub fn set_oneshot(&mut self, entry: BootEntry) -> BootResult<()> {
        if self.entry(entry).is_none() {
            return Err(BootloaderError::SlotNotPopulated(entry));
        }
        self.oneshot = Some(entry);
        Ok(())
    }

    /// Clear any pending one-shot override.
    pub fn clear_oneshot(&mut self) {
        self.oneshot = None;
    }

    /// The slot that will actually boot next: the one-shot override if set,
    /// otherwise the persisted default.
    pub fn next_boot(&self) -> Option<BootEntry> {
        self.oneshot.or(self.default)
    }

    /// Render the global `loader.conf`.
    pub fn render_loader_conf(&self) -> BootResult<String> {
        let default = self
            .default
            .ok_or_else(|| BootloaderError::InvalidConfig("no default entry set".into()))?;
        if self.entry(default).is_none() {
            return Err(BootloaderError::SlotNotPopulated(default));
        }
        let mut out = String::new();
        let _ = writeln!(out, "default Talos-{}", default.as_str());
        let _ = writeln!(out, "timeout {}", self.timeout);
        // Newer/higher sort entries first; Talos disables editor for security.
        out.push_str("editor no\n");
        if self.uki_mode {
            out.push_str("secure-boot-enroll if-safe\n");
        }
        Ok(out)
    }

    /// Render every provisioned entry as `(conf_name, body)` pairs, A before B.
    pub fn render_entries(&self) -> Vec<(String, String)> {
        let mut v = Vec::new();
        for e in [BootEntry::A, BootEntry::B] {
            if let Some(entry) = self.entry(e) {
                v.push((entry.conf_name(), entry.render()));
            }
        }
        v
    }
}

impl Bootloader for SdBoot {
    fn kind(&self) -> BootloaderKind {
        BootloaderKind::SystemdBoot
    }

    fn install(&mut self, slot: &BootSlot) -> BootResult<()> {
        // The generic Bootloader::install entry point always builds the entry
        // matching this loader's current mode.
        let entry = if self.uki_mode {
            let version = slot.version.as_deref().unwrap_or("unknown");
            SdBootEntry::uki(slot.entry, version, &slot.cmdline)
        } else {
            SdBootEntry::from_slot(slot)
        };
        self.install_entry(&entry)
    }

    fn default_entry(&self) -> Option<BootEntry> {
        self.default
    }

    fn revert(&mut self) -> BootResult<()> {
        match self.previous {
            Some(prev) if self.entry(prev).is_some() => {
                self.default = Some(prev);
                self.previous = None;
                self.oneshot = None;
                Ok(())
            }
            _ => Err(BootloaderError::NoPreviousEntry),
        }
    }

    fn render_config(&self) -> BootResult<String> {
        // For sd-boot the "config" is loader.conf followed by each entry file,
        // separated by a marker so callers can see the full materialized layout.
        let mut out = self.render_loader_conf()?;
        for (name, body) in self.render_entries() {
            let _ = writeln!(out, "\n# {}/{name}", paths::ENTRIES_DIR);
            out.push_str(&body);
        }
        Ok(out)
    }

    fn requires_secureboot(&self) -> bool {
        self.uki_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(e: BootEntry, v: &str) -> BootSlot {
        BootSlot::conventional(e, v, "talos.platform=metal console=ttyS0")
    }

    #[test]
    fn classic_install_renders_loader_and_entry() {
        let mut sd = SdBoot::new();
        sd.install(&slot(BootEntry::A, "v1.7.0")).unwrap();
        assert_eq!(sd.kind(), BootloaderKind::SystemdBoot);
        assert!(!sd.requires_secureboot());
        let conf = sd.render_loader_conf().unwrap();
        assert!(conf.contains("default Talos-A"));
        assert!(conf.contains("timeout 3"));
        assert!(conf.contains("editor no"));
        assert!(!conf.contains("secure-boot-enroll"));
        let entries = sd.render_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "Talos-A.conf");
        assert!(entries[0].1.contains("linux /A/vmlinuz"));
        assert!(entries[0].1.contains("initrd /A/initramfs.xz"));
        assert!(entries[0].1.contains("options talos.platform=metal"));
    }

    #[test]
    fn uki_mode_requires_secureboot_and_renders_efi_line() {
        let mut sd = SdBoot::new_uki();
        assert!(sd.is_uki_mode());
        sd.install(&slot(BootEntry::A, "v1.7.0")).unwrap();
        assert!(sd.requires_secureboot());
        let entries = sd.render_entries();
        assert!(entries[0].1.contains("efi /EFI/Linux/Talos-A.efi"));
        // UKI entries do not emit linux/initrd lines.
        assert!(!entries[0].1.contains("linux "));
        assert!(!entries[0].1.contains("initrd "));
        let conf = sd.render_loader_conf().unwrap();
        assert!(conf.contains("secure-boot-enroll if-safe"));
    }

    #[test]
    fn second_install_flips_default_and_sets_fallback() {
        let mut sd = SdBoot::new();
        sd.install(&slot(BootEntry::A, "v1.6.0")).unwrap();
        sd.install(&slot(BootEntry::B, "v1.7.0")).unwrap();
        assert_eq!(sd.default_entry(), Some(BootEntry::B));
        assert_eq!(sd.fallback_entry(), Some(BootEntry::A));
        let full = sd.render_config().unwrap();
        let a = full.find("Talos-A.conf").unwrap();
        let b = full.find("Talos-B.conf").unwrap();
        assert!(a < b, "A entry must render before B");
        assert!(full.contains("default Talos-B"));
    }

    #[test]
    fn oneshot_overrides_next_boot_without_changing_default() {
        let mut sd = SdBoot::new();
        sd.install(&slot(BootEntry::A, "v1.6.0")).unwrap();
        sd.install(&slot(BootEntry::B, "v1.7.0")).unwrap();
        assert_eq!(sd.next_boot(), Some(BootEntry::B));
        sd.set_oneshot(BootEntry::A).unwrap();
        assert_eq!(sd.next_boot(), Some(BootEntry::A));
        // Default unchanged.
        assert_eq!(sd.default_entry(), Some(BootEntry::B));
        sd.clear_oneshot();
        assert_eq!(sd.next_boot(), Some(BootEntry::B));
    }

    #[test]
    fn oneshot_on_unprovisioned_slot_errors() {
        let mut sd = SdBoot::new();
        sd.install(&slot(BootEntry::A, "v1.6.0")).unwrap();
        assert_eq!(
            sd.set_oneshot(BootEntry::B),
            Err(BootloaderError::SlotNotPopulated(BootEntry::B))
        );
    }

    #[test]
    fn install_clears_pending_oneshot() {
        let mut sd = SdBoot::new();
        sd.install(&slot(BootEntry::A, "v1.6.0")).unwrap();
        sd.install(&slot(BootEntry::B, "v1.7.0")).unwrap();
        sd.set_oneshot(BootEntry::A).unwrap();
        sd.install(&slot(BootEntry::A, "v1.8.0")).unwrap();
        assert!(sd.next_boot().is_some());
        // After install the default is A and there is no pending one-shot.
        assert_eq!(sd.default_entry(), Some(BootEntry::A));
        assert_eq!(sd.next_boot(), Some(BootEntry::A));
    }

    #[test]
    fn revert_restores_previous_and_clears_oneshot() {
        let mut sd = SdBoot::new();
        sd.install(&slot(BootEntry::A, "v1.6.0")).unwrap();
        sd.install(&slot(BootEntry::B, "v1.7.0")).unwrap();
        sd.set_oneshot(BootEntry::B).unwrap();
        sd.revert().unwrap();
        assert_eq!(sd.default_entry(), Some(BootEntry::A));
        assert_eq!(sd.next_boot(), Some(BootEntry::A));
        assert_eq!(sd.revert(), Err(BootloaderError::NoPreviousEntry));
    }

    #[test]
    fn mode_mismatch_rejected() {
        let mut sd = SdBoot::new_uki();
        // A classic entry cannot be installed into a UKI loader.
        let classic = SdBootEntry::from_slot(&slot(BootEntry::A, "v1.7.0"));
        assert!(matches!(
            sd.install_entry(&classic),
            Err(BootloaderError::InvalidConfig(_))
        ));
    }

    #[test]
    fn entry_validation_rules() {
        let mut bad = SdBootEntry::uki(BootEntry::A, "v1", "cmd");
        bad.uki = Some("  ".into());
        assert!(bad.validate().is_err());

        let mut mixed = SdBootEntry::uki(BootEntry::A, "v1", "cmd");
        mixed.linux = Some("/A/vmlinuz".into());
        assert!(mixed.validate().is_err());

        let mut classic = SdBootEntry::from_slot(&slot(BootEntry::B, "v1"));
        classic.initrd = None;
        assert!(classic.validate().is_err());
    }

    #[test]
    fn ids_and_conf_names() {
        let e = SdBootEntry::uki(BootEntry::B, "v1.7.0", "cmd");
        assert_eq!(e.id(), "Talos-B");
        assert_eq!(e.conf_name(), "Talos-B.conf");
        assert!(e.is_uki());
    }

    #[test]
    fn render_config_without_default_errors() {
        let sd = SdBoot::new();
        assert!(matches!(
            sd.render_config(),
            Err(BootloaderError::InvalidConfig(_))
        ));
    }

    #[test]
    fn timeout_is_configurable() {
        let mut sd = SdBoot::new();
        sd.set_timeout(10);
        sd.install(&slot(BootEntry::A, "v1.7.0")).unwrap();
        assert!(sd.render_loader_conf().unwrap().contains("timeout 10"));
    }
}
