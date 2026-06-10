//! # talos-security
//!
//! Security and kernel parameter management for the operating-system Talos migration.
//!
//! This crate ports the Talos subsystems that live under
//! `internal/app/machined/pkg/controllers/perf` + `runtime` and `pkg/kernel`:
//!
//! * **sysctl / sysfs** kernel parameter controllers (`/proc/sys`, `/sys`),
//! * **KSPP** (Kernel Self Protection Project) recommended defaults,
//! * **kernel cmdline** parsing (`/proc/cmdline`),
//! * **capability** and **seccomp** policy, and
//! * the **COSI** resource specs/status for the above
//!   (`KernelParamSpec`, `KernelParamStatus`).
//!
//! Where the real subsystem touches the kernel (writing to `/proc/sys` or
//! `/sys`) the boundary is modeled as the [`kernel_param::KernelParamSink`]
//! trait, with an in-memory implementation used by the controller and tests.
//!
//! The crate uses only the standard library (zero external dependencies) plus
//! an internal path dependency on `talos-core` for the shared error type.

// Pedantic-doc / must-use annotation lints are intentionally relaxed crate-wide:
// this is an internal port crate whose error and self-returning shapes are
// uniform, so annotating every getter/builder/`Result` method with `#[must_use]`
// or boilerplate `# Errors`/`# Panics` doc sections adds noise without improving
// the API's usability.
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

pub mod capabilities;
pub mod cmdline;
pub mod controller;
pub mod kernel_param;
pub mod kspp;
pub mod lockdown;
pub mod seccomp;
pub mod sysctl;
pub mod sysfs;

pub use capabilities::{CapabilitiesConfig, Capability};
pub use cmdline::{Cmdline, CmdlineParam};
pub use controller::{KernelParamController, ReconcilePlan, ReconcileReport};
pub use kernel_param::{
    KernelParamError, KernelParamKind, KernelParamSink, KernelParamSpec, KernelParamStatus,
    MemoryParamSink,
};
pub use kspp::{KsppCmdlineArg, KsppCmdlineAudit, KsppConfig};
pub use lockdown::{LockdownAudit, LockdownMode};
pub use seccomp::{SeccompAction, SeccompProfile, SeccompRule};
pub use sysctl::SysctlSpec;
pub use sysfs::SysfsSpec;

/// Convenience result alias for this crate.
pub type Result<T> = core::result::Result<T, KernelParamError>;

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn crate_result_alias_works() {
        let r: Result<u32> = Ok(7);
        assert!(matches!(r, Ok(7)));
        let e: Result<u32> = Err(KernelParamError::NotFound("x".into()));
        assert!(e.is_err());
    }

    #[test]
    fn reexports_are_reachable() {
        let s = KernelParamSpec::sysctl("net.ipv4.ip_forward", "1").unwrap();
        assert_eq!(s.kind, KernelParamKind::Sysctl);
        let _sink = MemoryParamSink::new();
        let defaults = kspp::KsppConfig::recommended();
        assert!(!defaults.specs().is_empty());
    }

    #[test]
    fn new_reexports_are_reachable() {
        let _: LockdownMode = LockdownMode::Confidentiality;
        let audit = KsppCmdlineAudit::audit(&Cmdline::new());
        assert!(!audit.is_compliant());
        let _: KsppCmdlineArg = KsppConfig::cmdline_args()[0];
        let sink = MemoryParamSink::new().with("a.b", "0");
        let plan =
            ReconcilePlan::compute(&[KernelParamSpec::sysctl("a.b", "1").unwrap()], &sink).unwrap();
        assert_eq!(plan.change_count(), 1);
    }

    #[test]
    fn end_to_end_kspp_sysctl_reconcile() {
        // Drive the KSPP sysctls through the controller against a seeded sink and
        // assert the node ends up fully hardened.
        let cfg = KsppConfig::recommended();
        let mut sink = cfg.seed_sink();
        let mut ctrl = KernelParamController::new();
        let report = ctrl.reconcile(cfg.specs(), &mut sink).unwrap();
        assert_eq!(cfg.pending_changes(&sink), 0);
        // Every changed key recorded a default so teardown can restore it.
        assert!(report.changed_count() > 0);
        let restored = ctrl.teardown(&mut sink).unwrap();
        assert_eq!(restored, report.changed_count());
    }

    #[test]
    fn lockdown_audit_against_cmdline() {
        let cl = Cmdline::parse("lockdown=integrity").unwrap();
        let active = LockdownMode::parse(cl.lockdown_level().unwrap()).unwrap();
        let audit = LockdownAudit::new(active, LockdownMode::Integrity);
        assert!(audit.is_compliant());
        let stricter = LockdownAudit::new(active, LockdownMode::Confidentiality);
        assert!(!stricter.is_compliant());
    }
}
