//! Linux capability sets and the Talos capabilities policy.
//!
//! Mirrors Talos `pkg/machinery/config` capability handling and the
//! `securityContext`/bounding-set logic in machined: Talos drops a default set
//! of dangerous capabilities and lets the machine config grant additional ones
//! (`machine.kubelet.extraConfig` / pod security). Here we model the capability
//! enum, a bounding set, and the policy that computes the effective set.

use std::collections::BTreeSet;
use std::fmt;

use crate::kernel_param::KernelParamError;

/// A Linux capability (subset of `<linux/capability.h>` relevant to Talos /
/// Kubernetes node hardening). The discriminant matches the kernel `CAP_*`
/// constant so the bounding-set bitmask is faithful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Capability {
    Chown = 0,
    DacOverride = 1,
    Fowner = 3,
    Kill = 5,
    Setgid = 6,
    Setuid = 7,
    SetPcap = 8,
    NetBindService = 10,
    NetRaw = 13,
    NetAdmin = 12,
    SysModule = 16,
    SysRawio = 17,
    SysChroot = 18,
    SysPtrace = 19,
    SysAdmin = 21,
    SysBoot = 22,
    SysNice = 23,
    SysTime = 25,
    MkNod = 27,
    AuditWrite = 29,
    Setfcap = 31,
    BPF = 39,
    Perfmon = 38,
}

impl Capability {
    /// All modeled capabilities.
    pub fn all() -> &'static [Capability] {
        use Capability::{
            AuditWrite, BPF, Chown, DacOverride, Fowner, Kill, MkNod, NetAdmin, NetBindService,
            NetRaw, Perfmon, SetPcap, Setfcap, Setgid, Setuid, SysAdmin, SysBoot, SysChroot,
            SysModule, SysNice, SysPtrace, SysRawio, SysTime,
        };
        &[
            Chown,
            DacOverride,
            Fowner,
            Kill,
            Setgid,
            Setuid,
            SetPcap,
            NetBindService,
            NetRaw,
            NetAdmin,
            SysModule,
            SysRawio,
            SysChroot,
            SysPtrace,
            SysAdmin,
            SysBoot,
            SysNice,
            SysTime,
            MkNod,
            AuditWrite,
            Setfcap,
            BPF,
            Perfmon,
        ]
    }

    /// The canonical `CAP_*` name (without the `CAP_` prefix, lower-cased), as
    /// used in machine config (`add: ["net_admin"]`).
    pub fn name(self) -> &'static str {
        use Capability::{
            AuditWrite, BPF, Chown, DacOverride, Fowner, Kill, MkNod, NetAdmin, NetBindService,
            NetRaw, Perfmon, SetPcap, Setfcap, Setgid, Setuid, SysAdmin, SysBoot, SysChroot,
            SysModule, SysNice, SysPtrace, SysRawio, SysTime,
        };
        match self {
            Chown => "chown",
            DacOverride => "dac_override",
            Fowner => "fowner",
            Kill => "kill",
            Setgid => "setgid",
            Setuid => "setuid",
            SetPcap => "setpcap",
            NetBindService => "net_bind_service",
            NetRaw => "net_raw",
            NetAdmin => "net_admin",
            SysModule => "sys_module",
            SysRawio => "sys_rawio",
            SysChroot => "sys_chroot",
            SysPtrace => "sys_ptrace",
            SysAdmin => "sys_admin",
            SysBoot => "sys_boot",
            SysNice => "sys_nice",
            SysTime => "sys_time",
            MkNod => "mknod",
            AuditWrite => "audit_write",
            Setfcap => "setfcap",
            BPF => "bpf",
            Perfmon => "perfmon",
        }
    }

    /// The kernel capability number.
    pub fn number(self) -> u8 {
        self as u8
    }

    /// Parse a capability from its config name, accepting an optional `cap_`
    /// prefix and arbitrary case (`NET_ADMIN`, `cap_net_admin`, `net_admin`).
    pub fn parse(name: &str) -> Result<Self, KernelParamError> {
        let lower = name.trim().to_ascii_lowercase();
        let bare = lower.strip_prefix("cap_").unwrap_or(&lower);
        Capability::all()
            .iter()
            .copied()
            .find(|c| c.name() == bare)
            .ok_or_else(|| KernelParamError::InvalidValue(format!("unknown capability: {name}")))
    }

    /// Whether this capability is considered dangerous enough that Talos drops
    /// it from the default bounding set (it must be explicitly re-added).
    pub fn is_dangerous(self) -> bool {
        use Capability::{BPF, SysAdmin, SysBoot, SysModule, SysPtrace, SysRawio, SysTime};
        matches!(
            self,
            SysModule | SysRawio | SysAdmin | SysBoot | SysPtrace | SysTime | BPF
        )
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The Talos capabilities policy: a bounding set computed from a hardened
/// default with operator-requested additions and drops applied on top.
///
/// Talos starts from the full set minus the dangerous ones, then applies
/// `add`/`drop` lists from machine config. `drop: ["all"]` clears the set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitiesConfig {
    bounding: BTreeSet<Capability>,
}

impl CapabilitiesConfig {
    /// The hardened Talos default: every modeled capability except the
    /// dangerous ones.
    pub fn talos_default() -> Self {
        let bounding = Capability::all()
            .iter()
            .copied()
            .filter(|c| !c.is_dangerous())
            .collect();
        CapabilitiesConfig { bounding }
    }

    /// The full set (every modeled capability).
    pub fn full() -> Self {
        CapabilitiesConfig {
            bounding: Capability::all().iter().copied().collect(),
        }
    }

    /// An empty set.
    pub fn empty() -> Self {
        CapabilitiesConfig {
            bounding: BTreeSet::new(),
        }
    }

    /// Whether a capability is in the bounding set.
    pub fn has(&self, cap: Capability) -> bool {
        self.bounding.contains(&cap)
    }

    /// Number of capabilities retained.
    pub fn len(&self) -> usize {
        self.bounding.len()
    }

    /// Whether the bounding set is empty.
    pub fn is_empty(&self) -> bool {
        self.bounding.is_empty()
    }

    /// Add a capability to the bounding set.
    pub fn add(&mut self, cap: Capability) -> &mut Self {
        self.bounding.insert(cap);
        self
    }

    /// Drop a capability from the bounding set.
    pub fn drop(&mut self, cap: Capability) -> &mut Self {
        self.bounding.remove(&cap);
        self
    }

    /// Apply machine-config `add`/`drop` lists (capability names). The special
    /// token `all` in `drop` clears the set; `all` in `add` grants every
    /// modeled capability. Drops are applied after adds, matching Kubernetes
    /// `securityContext.capabilities` precedence... actually Talos applies
    /// drops first then adds, which we follow here.
    pub fn apply(&mut self, add: &[&str], drop: &[&str]) -> Result<&mut Self, KernelParamError> {
        // Drops first.
        for name in drop {
            if name.eq_ignore_ascii_case("all") {
                self.bounding.clear();
            } else {
                self.bounding.remove(&Capability::parse(name)?);
            }
        }
        // Then adds.
        for name in add {
            if name.eq_ignore_ascii_case("all") {
                self.bounding = Capability::all().iter().copied().collect();
            } else {
                self.bounding.insert(Capability::parse(name)?);
            }
        }
        Ok(self)
    }

    /// The capability numbers as a 64-bit bounding-set bitmask (as the kernel
    /// represents `CapBnd`).
    pub fn bitmask(&self) -> u64 {
        self.bounding
            .iter()
            .fold(0u64, |acc, c| acc | (1u64 << c.number()))
    }

    /// The sorted list of capability names retained.
    pub fn names(&self) -> Vec<&'static str> {
        self.bounding.iter().map(|c| c.name()).collect()
    }

    /// The dangerous capabilities still present in this set (those Talos drops
    /// by default but that an `add` may have re-granted). Empty for a hardened
    /// config.
    pub fn retained_dangerous(&self) -> Vec<Capability> {
        self.bounding
            .iter()
            .copied()
            .filter(|c| c.is_dangerous())
            .collect()
    }

    /// Whether this set holds no dangerous capabilities (passes the Talos
    /// hardening audit).
    pub fn is_hardened(&self) -> bool {
        self.retained_dangerous().is_empty()
    }

    /// Capabilities present in `self` but absent from `other` (what `self`
    /// grants beyond `other`).
    pub fn added_over(&self, other: &CapabilitiesConfig) -> Vec<Capability> {
        self.bounding.difference(&other.bounding).copied().collect()
    }

    /// Capabilities present in `other` but absent from `self` (what `self` drops
    /// relative to `other`).
    pub fn dropped_from(&self, other: &CapabilitiesConfig) -> Vec<Capability> {
        other.bounding.difference(&self.bounding).copied().collect()
    }

    /// Build a config directly from a list of capability names (parsing each).
    pub fn from_names(names: &[&str]) -> Result<Self, KernelParamError> {
        let mut bounding = BTreeSet::new();
        for n in names {
            bounding.insert(Capability::parse(n)?);
        }
        Ok(CapabilitiesConfig { bounding })
    }

    /// Iterate the retained capabilities in number order.
    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.bounding.iter().copied()
    }
}

impl Capability {
    /// The set of capabilities Talos drops from the default bounding set.
    pub fn dangerous_set() -> Vec<Capability> {
        Capability::all()
            .iter()
            .copied()
            .filter(|c| c.is_dangerous())
            .collect()
    }
}

impl Default for CapabilitiesConfig {
    fn default() -> Self {
        CapabilitiesConfig::talos_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_prefixes_and_case() {
        assert_eq!(
            Capability::parse("net_admin").unwrap(),
            Capability::NetAdmin
        );
        assert_eq!(
            Capability::parse("CAP_NET_ADMIN").unwrap(),
            Capability::NetAdmin
        );
        assert_eq!(
            Capability::parse("  Sys_Admin ").unwrap(),
            Capability::SysAdmin
        );
        assert!(matches!(
            Capability::parse("bogus"),
            Err(KernelParamError::InvalidValue(_))
        ));
    }

    #[test]
    fn talos_default_drops_dangerous() {
        let cfg = CapabilitiesConfig::talos_default();
        assert!(!cfg.has(Capability::SysAdmin));
        assert!(!cfg.has(Capability::BPF));
        assert!(cfg.has(Capability::NetBindService));
        assert!(cfg.has(Capability::Chown));
    }

    #[test]
    fn apply_add_and_drop() {
        let mut cfg = CapabilitiesConfig::talos_default();
        assert!(!cfg.has(Capability::SysAdmin));
        cfg.apply(&["sys_admin"], &["chown"]).unwrap();
        assert!(cfg.has(Capability::SysAdmin));
        assert!(!cfg.has(Capability::Chown));
    }

    #[test]
    fn drop_all_then_add_specific() {
        let mut cfg = CapabilitiesConfig::full();
        cfg.apply(&["net_raw", "net_bind_service"], &["all"])
            .unwrap();
        assert_eq!(cfg.len(), 2);
        assert!(cfg.has(Capability::NetRaw));
        assert!(cfg.has(Capability::NetBindService));
    }

    #[test]
    fn bitmask_reflects_capability_numbers() {
        let mut cfg = CapabilitiesConfig::empty();
        cfg.add(Capability::Chown).add(Capability::NetAdmin);
        // CAP_CHOWN = 0, CAP_NET_ADMIN = 12.
        assert_eq!(cfg.bitmask(), (1 << 0) | (1 << 12));
    }

    #[test]
    fn names_sorted_and_unknown_in_apply_errors() {
        let mut cfg = CapabilitiesConfig::empty();
        cfg.add(Capability::NetRaw).add(Capability::Chown);
        // BTreeSet ordering: Chown(0) before NetRaw(13).
        assert_eq!(cfg.names(), vec!["chown", "net_raw"]);
        assert!(cfg.apply(&["nope"], &[]).is_err());
    }

    #[test]
    fn default_is_hardened_full_is_not() {
        assert!(CapabilitiesConfig::talos_default().is_hardened());
        assert!(
            CapabilitiesConfig::talos_default()
                .retained_dangerous()
                .is_empty()
        );
        let full = CapabilitiesConfig::full();
        assert!(!full.is_hardened());
        assert!(!full.retained_dangerous().is_empty());
    }

    #[test]
    fn re_adding_dangerous_cap_fails_audit() {
        let mut cfg = CapabilitiesConfig::talos_default();
        cfg.apply(&["sys_admin"], &[]).unwrap();
        assert!(!cfg.is_hardened());
        assert_eq!(cfg.retained_dangerous(), vec![Capability::SysAdmin]);
    }

    #[test]
    fn added_over_and_dropped_from() {
        let base = CapabilitiesConfig::from_names(&["chown", "kill"]).unwrap();
        let other = CapabilitiesConfig::from_names(&["chown", "net_raw"]).unwrap();
        // base grants `kill` beyond other.
        assert_eq!(base.added_over(&other), vec![Capability::Kill]);
        // base drops `net_raw` relative to other.
        assert_eq!(base.dropped_from(&other), vec![Capability::NetRaw]);
    }

    #[test]
    fn from_names_and_iter() {
        let cfg = CapabilitiesConfig::from_names(&["CAP_NET_ADMIN", "chown"]).unwrap();
        let caps: Vec<_> = cfg.iter().collect();
        assert_eq!(caps, vec![Capability::Chown, Capability::NetAdmin]);
        assert!(CapabilitiesConfig::from_names(&["bogus"]).is_err());
    }

    #[test]
    fn dangerous_set_matches_default_drops() {
        let dangerous = Capability::dangerous_set();
        let default = CapabilitiesConfig::talos_default();
        for cap in dangerous {
            assert!(!default.has(cap), "{cap} should be dropped by default");
        }
    }
}
