//! A concrete GRUB [`Bootloader`] implementation.
//!
//! Models `internal/pkg/bootloader/grub`: GRUB keeps an A/B menu with a
//! persisted default and a "fallback" pointer. Talos writes a `grub.cfg`
//! containing one `menuentry` per provisioned slot plus a `set default=`/
//! `set fallback=` preamble derived from the saved environment block.

use crate::boot_entry::{BootEntry, BootSlot};
use crate::bootloader::{BootResult, Bootloader, BootloaderError, BootloaderKind};
use std::fmt::Write as _;

/// In-memory GRUB bootloader holding both slots and the persisted default.
#[derive(Debug, Clone, Default)]
pub struct Grub {
    slot_a: Option<BootSlot>,
    slot_b: Option<BootSlot>,
    default: Option<BootEntry>,
    /// The previously-active default, used by [`Bootloader::revert`].
    previous: Option<BootEntry>,
}

impl Grub {
    /// A fresh, empty GRUB install.
    pub fn new() -> Grub {
        Grub::default()
    }

    fn slot(&self, e: BootEntry) -> Option<&BootSlot> {
        match e {
            BootEntry::A => self.slot_a.as_ref(),
            BootEntry::B => self.slot_b.as_ref(),
        }
    }

    fn put(&mut self, slot: BootSlot) {
        match slot.entry {
            BootEntry::A => self.slot_a = Some(slot),
            BootEntry::B => self.slot_b = Some(slot),
        }
    }

    /// The slot GRUB falls back to if the default fails to boot (the other
    /// provisioned slot, if any).
    pub fn fallback_entry(&self) -> Option<BootEntry> {
        let def = self.default?;
        let other = def.other();
        self.slot(other).map(|_| other)
    }

    fn render_menuentry(slot: &BootSlot) -> String {
        let id = slot.entry.as_str();
        let version = slot.version.as_deref().unwrap_or("unknown");
        let label = slot.entry.menu_label(version);
        format!(
            "menuentry '{label}' --id {id} {{\n  \
             linux {linux} {cmdline}\n  \
             initrd {initrd}\n}}\n",
            linux = slot.linux,
            cmdline = slot.cmdline,
            initrd = slot.initrd,
        )
    }

    fn validate(slot: &BootSlot) -> BootResult<()> {
        if slot.linux.trim().is_empty() {
            return Err(BootloaderError::InvalidConfig("empty linux path".into()));
        }
        if slot.initrd.trim().is_empty() {
            return Err(BootloaderError::InvalidConfig("empty initrd path".into()));
        }
        if slot.cmdline.trim().is_empty() {
            return Err(BootloaderError::InvalidConfig("empty cmdline".into()));
        }
        Ok(())
    }
}

impl Bootloader for Grub {
    fn kind(&self) -> BootloaderKind {
        BootloaderKind::Grub
    }

    fn install(&mut self, slot: &BootSlot) -> BootResult<()> {
        Grub::validate(slot)?;
        self.previous = self.default;
        let target = slot.entry;
        self.put(slot.clone());
        self.default = Some(target);
        Ok(())
    }

    fn default_entry(&self) -> Option<BootEntry> {
        self.default
    }

    fn revert(&mut self) -> BootResult<()> {
        match self.previous {
            Some(prev) if self.slot(prev).is_some() => {
                self.default = Some(prev);
                self.previous = None;
                Ok(())
            }
            _ => Err(BootloaderError::NoPreviousEntry),
        }
    }

    fn render_config(&self) -> BootResult<String> {
        let default = self
            .default
            .ok_or_else(|| BootloaderError::InvalidConfig("no default entry set".into()))?;
        if self.slot(default).is_none() {
            return Err(BootloaderError::SlotNotPopulated(default));
        }

        let mut out = String::new();
        let _ = writeln!(out, "set default={}", default.as_str());
        if let Some(fb) = self.fallback_entry() {
            let _ = writeln!(out, "set fallback={}", fb.as_str());
        }
        out.push_str("set timeout=3\n\n");

        // Deterministic ordering: A before B.
        for e in [BootEntry::A, BootEntry::B] {
            if let Some(slot) = self.slot(e) {
                out.push_str(&Self::render_menuentry(slot));
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(e: BootEntry, v: &str) -> BootSlot {
        BootSlot::conventional(e, v, "talos.platform=metal console=ttyS0")
    }

    #[test]
    fn install_sets_default_and_renders_menuentry() {
        let mut g = Grub::new();
        g.install(&slot(BootEntry::A, "v1.7.0")).unwrap();
        assert_eq!(g.kind(), BootloaderKind::Grub);
        assert_eq!(g.default_entry(), Some(BootEntry::A));
        let cfg = g.render_config().unwrap();
        assert!(cfg.contains("set default=A"));
        assert!(cfg.contains("--id A"));
        assert!(cfg.contains("linux /A/vmlinuz talos.platform=metal"));
        assert!(cfg.contains("initrd /A/initramfs.xz"));
        // Only one slot provisioned -> no fallback line.
        assert!(!cfg.contains("set fallback"));
    }

    #[test]
    fn second_install_targets_other_slot_and_arms_fallback() {
        let mut g = Grub::new();
        g.install(&slot(BootEntry::A, "v1.6.0")).unwrap();
        g.install(&slot(BootEntry::B, "v1.7.0")).unwrap();
        assert_eq!(g.default_entry(), Some(BootEntry::B));
        assert_eq!(g.fallback_entry(), Some(BootEntry::A));
        let cfg = g.render_config().unwrap();
        assert!(cfg.contains("set default=B"));
        assert!(cfg.contains("set fallback=A"));
        // Both menu entries rendered, A before B.
        let a = cfg.find("--id A").unwrap();
        let b = cfg.find("--id B").unwrap();
        assert!(a < b);
    }

    #[test]
    fn revert_returns_to_previous_default() {
        let mut g = Grub::new();
        g.install(&slot(BootEntry::A, "v1.6.0")).unwrap();
        g.install(&slot(BootEntry::B, "v1.7.0")).unwrap();
        assert_eq!(g.default_entry(), Some(BootEntry::B));
        g.revert().unwrap();
        assert_eq!(g.default_entry(), Some(BootEntry::A));
        // A second revert has nothing to revert to.
        assert_eq!(g.revert(), Err(BootloaderError::NoPreviousEntry));
    }

    #[test]
    fn install_rejects_empty_cmdline() {
        let mut g = Grub::new();
        let mut s = slot(BootEntry::A, "v1.7.0");
        s.cmdline = String::new();
        assert_eq!(
            g.install(&s),
            Err(BootloaderError::InvalidConfig("empty cmdline".into()))
        );
    }

    #[test]
    fn render_without_default_errors() {
        let g = Grub::new();
        assert_eq!(
            g.render_config(),
            Err(BootloaderError::InvalidConfig(
                "no default entry set".into()
            ))
        );
    }
}
