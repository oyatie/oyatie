//! A/B boot entry management.
//!
//! Talos uses a dual-boot scheme: two partitions/slots, named `A` and `B`,
//! each holding a kernel + initramfs. Upgrades install onto the *inactive*
//! slot and flip the default so the next boot uses the newly installed slot,
//! keeping the previous slot as a fallback. This mirrors the menu-entry logic
//! in `internal/pkg/bootloader/grub` and the `mountpoints`/label conventions.

use std::fmt;

/// One of the two boot slots in the A/B scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BootEntry {
    /// Slot A — the canonical first slot.
    A,
    /// Slot B — the fallback / second slot.
    B,
}

impl BootEntry {
    /// The other slot. Upgrades always target `current.other()`.
    pub fn other(self) -> BootEntry {
        match self {
            BootEntry::A => BootEntry::B,
            BootEntry::B => BootEntry::A,
        }
    }

    /// Short identifier as used in GRUB menu entry ids and metadata.
    pub fn as_str(self) -> &'static str {
        match self {
            BootEntry::A => "A",
            BootEntry::B => "B",
        }
    }

    /// The GRUB menu-entry label Talos renders (e.g. `"Talos (A)"`).
    pub fn menu_label(self, kernel_version: &str) -> String {
        format!("Talos {kernel_version} ({})", self.as_str())
    }

    /// Parse a slot from its short identifier (case-insensitive).
    pub fn parse(s: &str) -> Option<BootEntry> {
        match s.trim() {
            "A" | "a" => Some(BootEntry::A),
            "B" | "b" => Some(BootEntry::B),
            _ => None,
        }
    }
}

impl fmt::Display for BootEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The contents of one boot slot: which kernel/initramfs it carries and the
/// kernel command line to boot it with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootSlot {
    /// Which slot this is.
    pub entry: BootEntry,
    /// Path to the kernel image relative to the boot partition.
    pub linux: String,
    /// Path to the initramfs image relative to the boot partition.
    pub initrd: String,
    /// Kernel command line for this slot.
    pub cmdline: String,
    /// OS version installed into this slot, if known.
    pub version: Option<String>,
}

impl BootSlot {
    /// Build a slot with the conventional Talos paths
    /// (`/<A|B>/vmlinuz`, `/<A|B>/initramfs.xz`).
    pub fn conventional(entry: BootEntry, version: &str, cmdline: &str) -> BootSlot {
        let p = entry.as_str();
        BootSlot {
            entry,
            linux: format!("/{p}/vmlinuz"),
            initrd: format!("/{p}/initramfs.xz"),
            cmdline: cmdline.to_string(),
            version: Some(version.to_string()),
        }
    }

    /// Whether this slot is provisioned (has a non-empty kernel path).
    pub fn is_populated(&self) -> bool {
        !self.linux.is_empty()
    }
}

/// Boot-time metadata describing both slots and which one is the default.
///
/// Mirrors the GRUB environment block / `default_entry` Talos persists so it
/// knows where to install next and which slot to fall back to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootState {
    /// The slot that boots by default (the "active" slot).
    pub default: BootEntry,
    /// Slot A contents, if provisioned.
    pub slot_a: Option<BootSlot>,
    /// Slot B contents, if provisioned.
    pub slot_b: Option<BootSlot>,
    /// Whether the next boot is a one-shot fallback to the non-default slot
    /// (set after a failed upgrade so a single reboot reverts automatically).
    pub fallback_pending: bool,
}

impl BootState {
    /// A fresh state with `A` as default and nothing provisioned.
    pub fn fresh() -> BootState {
        BootState {
            default: BootEntry::A,
            slot_a: None,
            slot_b: None,
            fallback_pending: false,
        }
    }

    /// The currently active slot (the default unless a one-shot fallback is
    /// pending, in which case the *other* slot boots next).
    pub fn next_boot(&self) -> BootEntry {
        if self.fallback_pending {
            self.default.other()
        } else {
            self.default
        }
    }

    /// The slot an upgrade should install onto (always the inactive one).
    pub fn install_target(&self) -> BootEntry {
        self.default.other()
    }

    /// Immutable view of a slot's contents.
    pub fn slot(&self, entry: BootEntry) -> Option<&BootSlot> {
        match entry {
            BootEntry::A => self.slot_a.as_ref(),
            BootEntry::B => self.slot_b.as_ref(),
        }
    }

    /// Install (or replace) the contents of a slot.
    pub fn set_slot(&mut self, slot: BootSlot) {
        match slot.entry {
            BootEntry::A => self.slot_a = Some(slot),
            BootEntry::B => self.slot_b = Some(slot),
        }
    }

    /// Make `entry` the default. Clears any pending one-shot fallback.
    pub fn switch_default(&mut self, entry: BootEntry) {
        self.default = entry;
        self.fallback_pending = false;
    }

    /// Arm a one-shot fallback to the non-default slot for the next boot.
    /// Only valid when the fallback slot is actually provisioned.
    pub fn arm_fallback(&mut self) -> bool {
        if self.slot(self.default.other()).is_some() {
            self.fallback_pending = true;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn other_is_involutive() {
        assert_eq!(BootEntry::A.other(), BootEntry::B);
        assert_eq!(BootEntry::B.other().other(), BootEntry::B);
    }

    #[test]
    fn parse_and_label() {
        assert_eq!(BootEntry::parse(" a "), Some(BootEntry::A));
        assert_eq!(BootEntry::parse("B"), Some(BootEntry::B));
        assert_eq!(BootEntry::parse("z"), None);
        assert_eq!(BootEntry::A.menu_label("v1.7.0"), "Talos v1.7.0 (A)");
    }

    #[test]
    fn install_target_is_inactive_slot() {
        let mut st = BootState::fresh();
        assert_eq!(st.default, BootEntry::A);
        assert_eq!(st.install_target(), BootEntry::B);
        st.switch_default(BootEntry::B);
        assert_eq!(st.install_target(), BootEntry::A);
    }

    #[test]
    fn conventional_slot_paths() {
        let s = BootSlot::conventional(BootEntry::B, "v1.7.0", "talos.platform=metal");
        assert_eq!(s.linux, "/B/vmlinuz");
        assert_eq!(s.initrd, "/B/initramfs.xz");
        assert!(s.is_populated());
        assert_eq!(s.version.as_deref(), Some("v1.7.0"));
    }

    #[test]
    fn fallback_requires_provisioned_slot() {
        let mut st = BootState::fresh();
        // No B slot yet -> cannot arm.
        assert!(!st.arm_fallback());
        st.set_slot(BootSlot::conventional(BootEntry::B, "v1.6.0", "x"));
        assert!(st.arm_fallback());
        assert_eq!(st.next_boot(), BootEntry::B);
        // switching default clears the one-shot.
        st.switch_default(BootEntry::A);
        assert!(!st.fallback_pending);
        assert_eq!(st.next_boot(), BootEntry::A);
    }
}
