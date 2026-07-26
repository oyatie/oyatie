//! The `perf` stats controller.
//!
//! Mirrors `internal/app/machined/pkg/controllers/perf`. Talos runs a single
//! controller that, on a fixed poll interval, reads `/proc/stat` and
//! `/proc/meminfo` and writes/updates the singleton `CPU` and `Memory`
//! resources in the `runtime` namespace (ids `cpu` and `memory`). The dashboard
//! and `talosctl` then read those resources.
//!
//! Here the controller is modeled as a `reconcile`-style struct driven by a
//! [`Clock`] and a [`FileSystem`], producing typed [`CpuStat`]/[`MemStat`]
//! outputs and tracking the previous CPU sample so it can compute live
//! utilization.

use crate::cpu::{CpuStat, CpuUtilization};
use crate::memory::MemStat;
use crate::proc::{ProcStat, read_meminfo};
use os_kernel::address::ResourceId;
use os_kernel::error::Result;
use os_kernel::os::{Clock, FileSystem};
use os_kernel::resource::{Metadata, Namespace, ResourceKind};

/// The runtime-namespace resource id of the singleton CPU stats resource.
pub const CPU_RESOURCE_ID: &str = "cpu";

/// The runtime-namespace resource id of the singleton memory stats resource.
pub const MEM_RESOURCE_ID: &str = "memory";

/// The COSI resource kind string for CPU stats.
pub const CPU_RESOURCE_KIND: &str = "CPUStats.perf.talos.dev";

/// The COSI resource kind string for memory stats.
pub const MEM_RESOURCE_KIND: &str = "MemoryStats.perf.talos.dev";

/// The owner string this controller stamps onto resources it manages.
pub const CONTROLLER_NAME: &str = "perf.StatsController";

/// The default poll interval Talos uses for the perf controller (1 second),
/// expressed in nanoseconds.
pub const DEFAULT_INTERVAL_NANOS: u64 = 1_000_000_000;

/// A managed CPU stats resource: metadata plus the latest [`CpuStat`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuResource {
    /// COSI metadata.
    pub metadata: Metadata,
    /// The latest captured CPU stats.
    pub stat: CpuStat,
}

/// A managed memory stats resource: metadata plus the latest [`MemStat`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemResource {
    /// COSI metadata.
    pub metadata: Metadata,
    /// The latest captured memory stats.
    pub stat: MemStat,
}

/// The result of a single successful [`StatsController::reconcile`] tick.
#[derive(Debug, Clone)]
pub struct ReconcileOutput {
    /// The updated CPU resource.
    pub cpu: CpuResource,
    /// The updated memory resource.
    pub memory: MemResource,
    /// CPU utilization since the previous tick, if a previous sample existed
    /// and time advanced. `None` on the very first reconcile.
    pub utilization: Option<CpuUtilization>,
}

/// The perf stats controller.
///
/// Holds the published CPU/memory resources (if any), the last CPU sample for
/// delta computation, and the wall-clock timestamp of the last reconcile so it
/// can enforce the poll interval.
#[derive(Debug)]
pub struct StatsController {
    interval_nanos: u64,
    last_reconcile_nanos: Option<u64>,
    last_cpu: Option<CpuStat>,
    cpu: Option<CpuResource>,
    memory: Option<MemResource>,
    reconcile_count: u64,
}

impl Default for StatsController {
    fn default() -> Self {
        Self::new(DEFAULT_INTERVAL_NANOS)
    }
}

impl StatsController {
    /// Create a controller with the given poll interval (nanoseconds).
    pub fn new(interval_nanos: u64) -> Self {
        StatsController {
            interval_nanos: interval_nanos.max(1),
            last_reconcile_nanos: None,
            last_cpu: None,
            cpu: None,
            memory: None,
            reconcile_count: 0,
        }
    }

    /// The configured poll interval in nanoseconds.
    pub fn interval_nanos(&self) -> u64 {
        self.interval_nanos
    }

    /// Number of successful reconciles performed.
    pub fn reconcile_count(&self) -> u64 {
        self.reconcile_count
    }

    /// The currently published CPU resource, if any.
    pub fn cpu(&self) -> Option<&CpuResource> {
        self.cpu.as_ref()
    }

    /// The currently published memory resource, if any.
    pub fn memory(&self) -> Option<&MemResource> {
        self.memory.as_ref()
    }

    /// Whether enough time has elapsed since the last reconcile to run again.
    /// Always true before the first reconcile.
    pub fn should_reconcile(&self, now_nanos: u64) -> bool {
        match self.last_reconcile_nanos {
            None => true,
            Some(last) => now_nanos.saturating_sub(last) >= self.interval_nanos,
        }
    }

    /// Run one reconcile tick: read procfs, update both resources, and compute
    /// CPU utilization against the previous sample.
    ///
    /// This always performs the work; use [`should_reconcile`](Self::should_reconcile)
    /// or [`tick`](Self::tick) to honor the poll interval.
    pub fn reconcile(&mut self, clock: &dyn Clock, fs: &dyn FileSystem) -> Result<ReconcileOutput> {
        let now = clock.now_unix_nanos();

        // --- CPU ---
        let proc_stat = ProcStat::read(fs)?;
        let cpu_stat = CpuStat::new(proc_stat.total, proc_stat.per_cpu, proc_stat.counters);

        let utilization = self
            .last_cpu
            .as_ref()
            .and_then(|prev| CpuUtilization::between(&prev.total, &cpu_stat.total));

        let cpu_res = self.upsert_cpu(cpu_stat.clone())?;
        self.last_cpu = Some(cpu_stat);

        // --- Memory ---
        let mem_info = read_meminfo(fs)?;
        let mem_stat = MemStat::new(mem_info)?;
        let mem_res = self.upsert_mem(mem_stat)?;

        self.last_reconcile_nanos = Some(now);
        self.reconcile_count += 1;

        Ok(ReconcileOutput {
            cpu: cpu_res,
            memory: mem_res,
            utilization,
        })
    }

    /// Reconcile only if the poll interval has elapsed. Returns `Ok(None)` when
    /// it is too soon, otherwise the reconcile output.
    pub fn tick(
        &mut self,
        clock: &dyn Clock,
        fs: &dyn FileSystem,
    ) -> Result<Option<ReconcileOutput>> {
        if !self.should_reconcile(clock.now_unix_nanos()) {
            return Ok(None);
        }
        self.reconcile(clock, fs).map(Some)
    }

    fn upsert_cpu(&mut self, stat: CpuStat) -> Result<CpuResource> {
        match &mut self.cpu {
            Some(existing) => {
                existing.stat = stat;
                existing.metadata.bump_version();
            }
            None => {
                let mut metadata = Metadata::new(
                    Namespace::runtime(),
                    ResourceKind::new(CPU_RESOURCE_KIND)?,
                    ResourceId::new(CPU_RESOURCE_ID)?,
                );
                metadata.set_owner(CONTROLLER_NAME)?;
                self.cpu = Some(CpuResource { metadata, stat });
            }
        }
        Ok(self.cpu.clone().expect("cpu just set"))
    }

    fn upsert_mem(&mut self, stat: MemStat) -> Result<MemResource> {
        match &mut self.memory {
            Some(existing) => {
                existing.stat = stat;
                existing.metadata.bump_version();
            }
            None => {
                let mut metadata = Metadata::new(
                    Namespace::runtime(),
                    ResourceKind::new(MEM_RESOURCE_KIND)?,
                    ResourceId::new(MEM_RESOURCE_ID)?,
                );
                metadata.set_owner(CONTROLLER_NAME)?;
                self.memory = Some(MemResource { metadata, stat });
            }
        }
        Ok(self.memory.clone().expect("memory just set"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proc::{PROC_MEMINFO, PROC_STAT};
    use os_kernel::os::{FileSystem, ManualClock, MemoryFs};

    fn write_procfs(fs: &mut MemoryFs, cpu_user: u64, idle: u64, mem_free: u64) {
        let stat = alloc::format!(
            "cpu  {cpu_user} 0 50 {idle} 0 0 0 0 0 0\ncpu0 {cpu_user} 0 50 {idle} 0 0 0 0 0 0\nctxt 100\nbtime 1700000000\nprocesses 10\nprocs_running 1\nprocs_blocked 0\n"
        );
        let mem = alloc::format!(
            "MemTotal: 16000 kB\nMemFree: {mem_free} kB\nMemAvailable: 12000 kB\nBuffers: 100 kB\nCached: 200 kB\nSwapTotal: 0 kB\nSwapFree: 0 kB\n"
        );
        fs.write(PROC_STAT, stat.as_bytes()).unwrap();
        fs.write(PROC_MEMINFO, mem.as_bytes()).unwrap();
    }

    #[test]
    fn first_reconcile_creates_resources_with_owner() {
        let clock = ManualClock::new(0);
        let mut fs = MemoryFs::new();
        write_procfs(&mut fs, 100, 900, 8000);

        let mut ctrl = StatsController::default();
        assert!(ctrl.cpu().is_none());

        let out = ctrl.reconcile(&clock, &fs).unwrap();
        assert!(out.utilization.is_none()); // no previous sample
        assert_eq!(out.cpu.metadata.owner(), Some(CONTROLLER_NAME));
        assert_eq!(out.cpu.metadata.version(), 1);
        assert_eq!(out.cpu.metadata.pointer().id.as_str(), CPU_RESOURCE_ID);
        assert_eq!(out.memory.metadata.owner(), Some(CONTROLLER_NAME));
        assert_eq!(ctrl.reconcile_count(), 1);
    }

    #[test]
    fn second_reconcile_bumps_version_and_computes_utilization() {
        let clock = ManualClock::new(0);
        let mut fs = MemoryFs::new();
        write_procfs(&mut fs, 100, 900, 8000);

        let mut ctrl = StatsController::default();
        ctrl.reconcile(&clock, &fs).unwrap();

        // Advance counters: user +100, idle +900 -> total delta 1000.
        write_procfs(&mut fs, 200, 1800, 7000);
        let out = ctrl.reconcile(&clock, &fs).unwrap();

        assert_eq!(out.cpu.metadata.version(), 2);
        assert_eq!(out.memory.metadata.version(), 2);
        let u = out.utilization.expect("utilization on second tick");
        assert!((u.user - 0.1).abs() < 1e-9);
        assert!((u.idle - 0.9).abs() < 1e-9);
    }

    #[test]
    fn tick_respects_interval() {
        let clock = ManualClock::new(0);
        let mut fs = MemoryFs::new();
        write_procfs(&mut fs, 100, 900, 8000);

        let mut ctrl = StatsController::new(DEFAULT_INTERVAL_NANOS);
        assert!(ctrl.should_reconcile(0));
        assert!(ctrl.tick(&clock, &fs).unwrap().is_some());

        // Too soon: no time advanced.
        assert!(!ctrl.should_reconcile(clock.now_unix_nanos()));
        assert!(ctrl.tick(&clock, &fs).unwrap().is_none());
        assert_eq!(ctrl.reconcile_count(), 1);

        // After the interval elapses, it runs again.
        clock.advance_nanos(DEFAULT_INTERVAL_NANOS);
        assert!(ctrl.should_reconcile(clock.now_unix_nanos()));
        assert!(ctrl.tick(&clock, &fs).unwrap().is_some());
        assert_eq!(ctrl.reconcile_count(), 2);
    }

    #[test]
    fn reconcile_propagates_parse_errors() {
        let clock = ManualClock::new(0);
        let mut fs = MemoryFs::new();
        // /proc/stat missing the required cpu line.
        fs.write(PROC_STAT, b"ctxt 1\n").unwrap();
        fs.write(PROC_MEMINFO, b"MemTotal: 16000 kB\n").unwrap();

        let mut ctrl = StatsController::default();
        assert!(ctrl.reconcile(&clock, &fs).is_err());
        // Nothing should have been published on failure.
        assert!(ctrl.cpu().is_none());
    }

    #[test]
    fn missing_procfs_is_not_found() {
        let clock = ManualClock::new(0);
        let fs = MemoryFs::new();
        let mut ctrl = StatsController::default();
        let err = ctrl.reconcile(&clock, &fs).unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn interval_floored_to_one() {
        let ctrl = StatsController::new(0);
        assert_eq!(ctrl.interval_nanos(), 1);
    }
}
