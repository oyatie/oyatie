//! `talos-init` — early-userspace / PID 1 (init) for the operating-system Talos-in-Rust
//! port.
//!
//! This library models Talos' `cmd/init` faithfully and host-testably. The
//! binary (`src/main.rs`) is the thin Linux-only PID 1 entrypoint that plugs
//! real `mount(2)`/`waitpid(2)`/`reboot(2)` implementations into the pure logic
//! defined here.
//!
//! Modules:
//! * [`mount`]      — the essential pseudo-filesystem mount table + `Mounter` trait.
//! * [`reaper`]     — child reaping (the classic PID 1 duty) + `ChildWaiter` trait.
//! * [`switch_root`]— pivot from initramfs to the real root and exec `machined`.
//! * [`cmdline`]    — `/proc/cmdline` parsing with Talos-specific accessors.
//! * [`config`]     — early machine-config extraction (hostname, type, disk).
//! * [`kmsg`]       — `/dev/kmsg` + console logging.
//! * [`signals`]    — PID 1 signal disposition model.
//! * [`boot`]       — the early-boot sequence state machine tying it together.
//!
//! Everything kernel-facing is expressed as a trait with an in-memory fake used
//! by the unit tests, so the whole crate builds and tests on a non-Linux host.

pub mod boot;
pub mod cmdline;
pub mod config;
pub mod kmsg;
pub mod mount;
pub mod platform_config;
pub mod reaper;
pub mod signals;
pub mod switch_root;

/// Default hostname applied when neither the machine config nor the kernel
/// command line provides one.
pub const DEFAULT_HOSTNAME: &str = "talos-rust";

/// Path to the machine config baked into the initramfs root.
pub const MACHINE_CONFIG_PATH: &str = "/machine-config.yaml";

/// Path to the kernel command line.
pub const PROC_CMDLINE_PATH: &str = "/proc/cmdline";

/// Conventional path of the real root mount in the initramfs before pivot.
pub const NEW_ROOT_PATH: &str = "/root";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_talos_conventions() {
        assert_eq!(DEFAULT_HOSTNAME, "talos-rust");
        assert_eq!(MACHINE_CONFIG_PATH, "/machine-config.yaml");
        assert_eq!(PROC_CMDLINE_PATH, "/proc/cmdline");
        assert_eq!(NEW_ROOT_PATH, "/root");
    }

    #[test]
    fn end_to_end_minimal_boot_smoke() {
        // A small integration over the public API: parse cmdline, extract config,
        // build a switch-root plan, and confirm validation passes for a healthy
        // fake rootfs.
        use crate::switch_root::{FakeRootFs, SwitchRootPlan, validate};

        let cl = cmdline::CmdLine::parse("console=ttyS0 talos.platform=metal");
        assert_eq!(cl.platform(), Some("metal"));

        let ec = config::early_config(
            "version: v1alpha1\nmachine:\n  type: controlplane\n  network:\n    hostname: cp1\n",
        );
        assert_eq!(ec.hostname.as_deref(), Some("cp1"));
        assert!(ec.machine_type.unwrap().is_control_plane());

        let plan = SwitchRootPlan::to_machined(NEW_ROOT_PATH);
        let fs = FakeRootFs::healthy(NEW_ROOT_PATH, "/root/sbin/machined", 10);
        assert!(validate(&plan, &fs).is_ok());
    }
}
