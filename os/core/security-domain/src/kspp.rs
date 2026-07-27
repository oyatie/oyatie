//! KSPP — Kernel Self Protection Project recommended sysctl/sysfs defaults.
//!
//! Mirrors Talos `pkg/kernel/kspp` and the `KernelParamConfigController` that
//! applies the KSPP-recommended hardening sysctls on boot. These are the
//! values Talos enforces so the node passes the upstream `kubeadm` /
//! `kube-bench` style hardening checks.
//!
//! The set here is faithful to the keys Talos ships (see
//! `internal/pkg/kspp/kspp.go`): kexec lockdown, kptr restriction, dmesg
//! restriction, BPF hardening, ptrace scope, and a few more.

use crate::cmdline::Cmdline;
use crate::kernel_param::KernelParamSink;
use crate::kernel_param::{KernelParamSpec, MemoryParamSink};

/// The recommended KSPP kernel parameters, as `(key, value)` pairs. All are
/// sysctls under `/proc/sys`.
const KSPP_SYSCTLS: &[(&str, &str)] = &[
    // Disallow kexec of a new kernel after boot (anti-persistence).
    ("kernel.kexec_load_disabled", "1"),
    // Restrict exposure of kernel pointers in /proc and other interfaces.
    ("kernel.kptr_restrict", "1"),
    // Restrict access to the kernel log buffer (dmesg) to privileged users.
    ("kernel.dmesg_restrict", "1"),
    // Disallow unprivileged loading of BPF programs.
    ("kernel.unprivileged_bpf_disabled", "1"),
    // Harden the BPF JIT against spraying attacks.
    ("net.core.bpf_jit_harden", "1"),
    // Restrict ptrace to child processes only (YAMA).
    ("kernel.yama.ptrace_scope", "1"),
    // Disable the legacy SysRq key combinations.
    ("kernel.sysrq", "0"),
    // Restrict access to performance events to privileged users.
    ("kernel.perf_event_paranoid", "3"),
];

/// A configured set of KSPP hardening parameters.
///
/// Constructed from the built-in recommended set, with the option to drop
/// individual keys that conflict with an operator override (Talos lets machine
/// config win over KSPP defaults).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KsppConfig {
    specs: Vec<KernelParamSpec>,
}

impl KsppConfig {
    /// The full KSPP-recommended set.
    pub fn recommended() -> Self {
        let specs = KSPP_SYSCTLS
            .iter()
            .map(|(k, v)| KernelParamSpec::sysctl(*k, *v).expect("KSPP defaults are valid"))
            .collect();
        KsppConfig { specs }
    }

    /// An empty config (no hardening). Useful as a base to add to.
    pub fn empty() -> Self {
        KsppConfig { specs: Vec::new() }
    }

    /// Borrow the configured specs.
    pub fn specs(&self) -> &[KernelParamSpec] {
        &self.specs
    }

    /// Number of parameters in this config.
    pub fn len(&self) -> usize {
        self.specs.len()
    }

    /// Whether the config is empty.
    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    /// Whether a specific key is present.
    pub fn contains(&self, key: &str) -> bool {
        self.specs.iter().any(|s| s.key == key)
    }

    /// Look up the desired value for a key, if present.
    pub fn value_of(&self, key: &str) -> Option<&str> {
        self.specs
            .iter()
            .find(|s| s.key == key)
            .map(|s| s.value.as_str())
    }

    /// Remove a key from the set (operator override wins). Returns true if a
    /// key was actually removed.
    pub fn override_key(&mut self, key: &str) -> bool {
        let before = self.specs.len();
        self.specs.retain(|s| s.key != key);
        self.specs.len() != before
    }

    /// Produce a [`MemoryParamSink`] pre-seeded with all KSPP keys at a
    /// permissive "0" default, suitable for driving the controller in tests or
    /// dry-runs (the kernel exposes these keys; they just aren't hardened yet).
    pub fn seed_sink(&self) -> MemoryParamSink {
        let mut sink = MemoryParamSink::new();
        for spec in &self.specs {
            sink = sink.with(&spec.key, "0");
        }
        sink
    }

    /// How many KSPP keys would actually change from their current value in
    /// `sink` (i.e. the hardening work still outstanding).
    pub fn pending_changes(&self, sink: &impl KernelParamSink) -> usize {
        self.specs
            .iter()
            .filter(|s| match sink.read(&s.key) {
                Ok(current) => current != s.value,
                Err(_) => true,
            })
            .count()
    }
}

impl Default for KsppConfig {
    fn default() -> Self {
        KsppConfig::recommended()
    }
}

/// KSPP-recommended kernel command-line parameters.
///
/// Talos sets these on the boot cmdline (see `pkg/kernel/kspp` and the install
/// path that builds the kernel args). Each entry is either a bare flag
/// (`slab_nomerge`, `pti=on`) or a `key=value` requirement. The audit below
/// checks a parsed [`Cmdline`] for their presence / correct value.
const KSPP_CMDLINE: &[KsppCmdlineArg] = &[
    // Disable slab merging so use-after-free in one cache can't corrupt another.
    KsppCmdlineArg::flag("slab_nomerge"),
    // Zero freshly allocated and freed pages/heap to blunt info leaks / UAF.
    KsppCmdlineArg::kv("init_on_alloc", "1"),
    KsppCmdlineArg::kv("init_on_free", "1"),
    // Enable page allocator freelist randomization.
    KsppCmdlineArg::kv("page_alloc.shuffle", "1"),
    // Force kernel page-table isolation (Meltdown mitigation).
    KsppCmdlineArg::kv("pti", "on"),
    // Enable SLUB red-zoning and poisoning sanity checks.
    KsppCmdlineArg::kv("slub_debug", "FZ"),
    // Enable the strongest practical vsyscall hardening.
    KsppCmdlineArg::kv("vsyscall", "none"),
    // Disable the legacy 16-bit vDSO / debug interfaces.
    KsppCmdlineArg::kv("debugfs", "off"),
    // Enforce module signature checking.
    KsppCmdlineArg::kv("module.sig_enforce", "1"),
    // Put the kernel in confidentiality lockdown.
    KsppCmdlineArg::kv("lockdown", "confidentiality"),
    // Randomize kernel stack offset on syscall entry.
    KsppCmdlineArg::kv("randomize_kstack_offset", "on"),
];

/// A single KSPP cmdline requirement: either a bare flag or a `key=value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KsppCmdlineArg {
    /// The cmdline key (for a flag like `slab_nomerge`, the whole token).
    pub key: &'static str,
    /// The required value, or `None` for a bare flag.
    pub value: Option<&'static str>,
}

impl KsppCmdlineArg {
    /// A bare flag requirement (just present).
    pub const fn flag(key: &'static str) -> Self {
        KsppCmdlineArg { key, value: None }
    }

    /// A `key=value` requirement.
    pub const fn kv(key: &'static str, value: &'static str) -> Self {
        KsppCmdlineArg {
            key,
            value: Some(value),
        }
    }

    /// Render this arg as it would appear on the cmdline.
    pub fn render(&self) -> String {
        match self.value {
            Some(v) => format!("{}={}", self.key, v),
            None => self.key.to_string(),
        }
    }

    /// Whether `cmdline` satisfies this requirement.
    pub fn satisfied_by(&self, cmdline: &Cmdline) -> bool {
        match self.value {
            None => cmdline.contains(self.key),
            Some(v) => cmdline.get_first(self.key) == Some(v),
        }
    }
}

/// The status of a single KSPP cmdline arg after auditing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KsppCmdlineFinding {
    /// The required arg.
    pub arg: KsppCmdlineArg,
    /// Whether the running cmdline satisfies it.
    pub satisfied: bool,
}

/// The full KSPP cmdline audit result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KsppCmdlineAudit {
    findings: Vec<KsppCmdlineFinding>,
}

impl KsppCmdlineAudit {
    /// Audit a parsed cmdline against the full KSPP cmdline set.
    pub fn audit(cmdline: &Cmdline) -> Self {
        let findings = KSPP_CMDLINE
            .iter()
            .map(|arg| KsppCmdlineFinding {
                arg: *arg,
                satisfied: arg.satisfied_by(cmdline),
            })
            .collect();
        KsppCmdlineAudit { findings }
    }

    /// All findings.
    pub fn findings(&self) -> &[KsppCmdlineFinding] {
        &self.findings
    }

    /// The args that are missing or have the wrong value.
    pub fn missing(&self) -> Vec<KsppCmdlineArg> {
        self.findings
            .iter()
            .filter(|f| !f.satisfied)
            .map(|f| f.arg)
            .collect()
    }

    /// Whether every KSPP cmdline arg is satisfied.
    pub fn is_compliant(&self) -> bool {
        self.findings.iter().all(|f| f.satisfied)
    }

    /// How many of the required args are satisfied.
    pub fn satisfied_count(&self) -> usize {
        self.findings.iter().filter(|f| f.satisfied).count()
    }
}

impl KsppConfig {
    /// The full list of KSPP-recommended cmdline args.
    pub fn cmdline_args() -> &'static [KsppCmdlineArg] {
        KSPP_CMDLINE
    }

    /// Inject every KSPP cmdline arg that is not already present into `cmdline`,
    /// returning the augmented cmdline. Existing keys are left untouched (the
    /// operator's value wins), matching how Talos merges the KSPP defaults under
    /// any explicit machine-config extra args.
    pub fn augment_cmdline(cmdline: Cmdline) -> Cmdline {
        let mut out = cmdline;
        for arg in KSPP_CMDLINE {
            if out.contains(arg.key) {
                continue;
            }
            match arg.value {
                Some(v) => out = out.set(arg.key, v),
                // Re-parse the flag token (it may itself contain `=`, e.g.
                // `slub_debug=FZ`) and merge it in.
                None => {
                    if let Some((k, v)) = arg.key.split_once('=') {
                        if !out.contains(k) {
                            out = out.set(k, v);
                        }
                    } else {
                        out = out.set(arg.key, "");
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommended_set_is_nonempty_and_valid() {
        let cfg = KsppConfig::recommended();
        assert!(!cfg.is_empty());
        assert_eq!(cfg.len(), KSPP_SYSCTLS.len());
        assert!(cfg.specs().iter().all(|s| s.validate().is_ok()));
    }

    #[test]
    fn known_hardening_keys_present() {
        let cfg = KsppConfig::recommended();
        assert!(cfg.contains("kernel.kexec_load_disabled"));
        assert_eq!(cfg.value_of("kernel.kptr_restrict"), Some("1"));
        assert_eq!(cfg.value_of("kernel.sysrq"), Some("0"));
        assert_eq!(cfg.value_of("nonexistent"), None);
    }

    #[test]
    fn override_removes_key() {
        let mut cfg = KsppConfig::recommended();
        assert!(cfg.contains("kernel.dmesg_restrict"));
        assert!(cfg.override_key("kernel.dmesg_restrict"));
        assert!(!cfg.contains("kernel.dmesg_restrict"));
        // Removing again is a no-op.
        assert!(!cfg.override_key("kernel.dmesg_restrict"));
    }

    #[test]
    fn seed_sink_starts_unhardened_then_pending_drops_to_zero() {
        let cfg = KsppConfig::recommended();
        let mut sink = cfg.seed_sink();
        // Everything seeded at "0", so each "1"/"3" target is pending; only the
        // keys whose target is "0" (sysrq) are already satisfied.
        let expected_pending = cfg.specs().iter().filter(|s| s.value != "0").count();
        assert_eq!(cfg.pending_changes(&sink), expected_pending);

        for spec in cfg.specs() {
            sink.write(&spec.key, &spec.value).unwrap();
        }
        assert_eq!(cfg.pending_changes(&sink), 0);
    }

    #[test]
    fn empty_and_default() {
        assert!(KsppConfig::empty().is_empty());
        assert_eq!(KsppConfig::default(), KsppConfig::recommended());
    }

    #[test]
    fn cmdline_args_render_correctly() {
        let flag = KsppCmdlineArg::flag("slab_nomerge");
        assert_eq!(flag.render(), "slab_nomerge");
        let kv = KsppCmdlineArg::kv("init_on_alloc", "1");
        assert_eq!(kv.render(), "init_on_alloc=1");
    }

    #[test]
    fn cmdline_arg_satisfied_by() {
        let cl = Cmdline::parse("slab_nomerge init_on_alloc=1 pti=off").unwrap();
        assert!(KsppCmdlineArg::flag("slab_nomerge").satisfied_by(&cl));
        assert!(KsppCmdlineArg::kv("init_on_alloc", "1").satisfied_by(&cl));
        // Wrong value is not satisfied.
        assert!(!KsppCmdlineArg::kv("pti", "on").satisfied_by(&cl));
        // Absent flag is not satisfied.
        assert!(!KsppCmdlineArg::flag("randomize_kstack_offset").satisfied_by(&cl));
    }

    #[test]
    fn audit_empty_cmdline_is_noncompliant() {
        let audit = KsppCmdlineAudit::audit(&Cmdline::new());
        assert!(!audit.is_compliant());
        assert_eq!(audit.satisfied_count(), 0);
        assert_eq!(audit.missing().len(), KsppConfig::cmdline_args().len());
    }

    #[test]
    fn audit_fully_hardened_cmdline_is_compliant() {
        // Build a cmdline that satisfies every KSPP arg.
        let mut cl = Cmdline::new();
        for arg in KsppConfig::cmdline_args() {
            cl = match arg.value {
                Some(v) => cl.set(arg.key, v),
                None => cl.set(arg.key, ""),
            };
        }
        let audit = KsppCmdlineAudit::audit(&cl);
        assert!(audit.is_compliant(), "missing: {:?}", audit.missing());
        assert_eq!(audit.satisfied_count(), KsppConfig::cmdline_args().len());
        assert!(audit.missing().is_empty());
    }

    #[test]
    fn augment_cmdline_adds_missing_keeps_existing() {
        // Operator pins pti=off; KSPP must not override it but must add the rest.
        let base = Cmdline::parse("pti=off console=ttyS0").unwrap();
        let augmented = KsppConfig::augment_cmdline(base);
        // Existing operator value preserved.
        assert_eq!(augmented.get_first("pti"), Some("off"));
        assert_eq!(augmented.get_first("console"), Some("ttyS0"));
        // A KSPP arg that was missing is now present.
        assert_eq!(augmented.get_first("init_on_alloc"), Some("1"));
        assert!(augmented.contains("slab_nomerge"));
        // After augmenting, every arg except the overridden pti is satisfied.
        let audit = KsppCmdlineAudit::audit(&augmented);
        let missing = audit.missing();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].key, "pti");
    }

    #[test]
    fn augment_is_idempotent() {
        let once = KsppConfig::augment_cmdline(Cmdline::new());
        let twice = KsppConfig::augment_cmdline(once.clone());
        assert_eq!(once, twice);
    }
}
