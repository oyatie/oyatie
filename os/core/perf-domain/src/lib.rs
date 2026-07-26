//! # talos-perf
//!
//! System performance statistics, mirroring Talos's
//! `internal/app/machined/pkg/controllers/perf` and
//! `pkg/machinery/resources/perf`.
//!
//! Talos surfaces CPU and memory usage to `talosctl dashboard` and the
//! `MachineService` stats APIs by running a small controller that periodically
//! reads `procfs` and publishes two singleton COSI resources in the `runtime`
//! namespace: `CPU` (cumulative CPU times + system counters) and `Memory`
//! (RAM/swap usage).
//!
//! This crate models that faithfully:
//!
//! * [`proc`] — parsers for `/proc/stat` and `/proc/meminfo`, reading through
//!   the [`os_kernel::os::FileSystem`] boundary so they can be tested against
//!   an in-memory filesystem.
//! * [`cpu`] — the [`cpu::CpuStat`] resource, cumulative [`cpu::CpuTimes`], and
//!   the derived per-state [`cpu::CpuUtilization`] computed by diffing samples.
//! * [`memory`] — the [`memory::MemStat`] resource and [`memory::MemInfo`] with
//!   used/available/swap accounting and pressure detection.
//! * [`controller`] — the [`controller::StatsController`], a `reconcile`-style
//!   poll loop driven by a [`os_kernel::os::Clock`] and `FileSystem` that
//!   publishes and versions the CPU/Memory resources.
//!
//! The crate links `std` on the host (for the test harness) but is written
//! against `alloc` so it stays aligned with the rest of the workspace.

extern crate alloc;

pub mod controller;
pub mod cpu;
pub mod memory;
pub mod proc;

pub use controller::{
    CONTROLLER_NAME, CPU_RESOURCE_ID, CPU_RESOURCE_KIND, CpuResource, DEFAULT_INTERVAL_NANOS,
    MEM_RESOURCE_ID, MEM_RESOURCE_KIND, MemResource, ReconcileOutput, StatsController,
};
pub use cpu::{CpuStat, CpuTimes, CpuUtilization, SystemCounters, USER_HZ};
pub use memory::{MemInfo, MemStat};
pub use proc::{PROC_MEMINFO, PROC_STAT, ProcStat, parse_meminfo, read_meminfo};

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::os::{FileSystem, ManualClock, MemoryFs};

    // An end-to-end smoke test exercising the full stack: write procfs, run the
    // controller, and confirm the published resources reflect the input.
    #[test]
    fn end_to_end_collection() {
        let clock = ManualClock::new(1_700_000_000_000_000_000);
        let mut fs = MemoryFs::new();
        fs.write(
            PROC_STAT,
            b"cpu  1000 0 500 8500 0 0 0 0 0 0\ncpu0 1000 0 500 8500 0 0 0 0 0 0\nctxt 42\nbtime 1700000000\nprocesses 7\nprocs_running 1\nprocs_blocked 0\n",
        )
        .unwrap();
        fs.write(
            PROC_MEMINFO,
            b"MemTotal: 16000 kB\nMemFree: 4000 kB\nMemAvailable: 8000 kB\nBuffers: 1000 kB\nCached: 3000 kB\nSwapTotal: 2000 kB\nSwapFree: 1500 kB\n",
        )
        .unwrap();

        let mut ctrl = StatsController::default();
        let out = ctrl.reconcile(&clock, &fs).unwrap();

        assert_eq!(out.cpu.stat.num_cpus(), 1);
        assert_eq!(out.cpu.stat.total.total(), 10_000);
        assert_eq!(out.cpu.stat.counters.boot_time, 1_700_000_000);

        // 16000k - 4000k - 1000k - 3000k = 8000k bytes used.
        assert_eq!(out.memory.stat.info.used(), 8_000 * 1024);
        assert!((out.memory.stat.info.swap_used_fraction() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn reexports_are_wired() {
        // Touch the re-exported constants/types to ensure the public surface
        // compiles and is reachable.
        assert_eq!(CPU_RESOURCE_ID, "cpu");
        assert_eq!(MEM_RESOURCE_ID, "memory");
        assert_eq!(USER_HZ, 100);
        let _ = CpuTimes::default();
        let _ = MemInfo::default();
        assert_eq!(CONTROLLER_NAME, "perf.StatsController");
        assert_eq!(DEFAULT_INTERVAL_NANOS, 1_000_000_000);
        assert!(CPU_RESOURCE_KIND.contains("perf.talos.dev"));
        assert!(MEM_RESOURCE_KIND.contains("perf.talos.dev"));
    }
}
