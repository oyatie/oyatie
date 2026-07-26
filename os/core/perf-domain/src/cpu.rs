//! CPU statistics: the `CPUStat` resource and its derived utilization.
//!
//! Mirrors `pkg/machinery/resources/perf` (`CPU`) and the machined `perf` CPU
//! controller. The kernel reports cumulative CPU time per state in *USER_HZ*
//! jiffies; Talos surfaces both the raw cumulative counts and, for the
//! dashboard, the per-state utilization computed by diffing two consecutive
//! samples.

use os_kernel::error::{Error, Result};

/// The standard Linux `USER_HZ` clock tick rate. On essentially every Talos
/// target this is 100 Hz, i.e. each jiffy is 10ms.
pub const USER_HZ: u64 = 100;

/// Cumulative CPU time, in jiffies, broken down by scheduler state.
///
/// These map one-to-one to the columns of a `/proc/stat` `cpu`/`cpuN` line, in
/// kernel order. Fields past `steal` (`guest`, `guest_nice`) are already
/// accounted for inside `user`/`nice` by the kernel and so are tracked
/// separately only for completeness.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CpuTimes {
    /// Time in user mode.
    pub user: u64,
    /// Time in user mode with low priority (nice).
    pub nice: u64,
    /// Time in system (kernel) mode.
    pub system: u64,
    /// Idle time.
    pub idle: u64,
    /// Time waiting for I/O to complete.
    pub iowait: u64,
    /// Time servicing hardware interrupts.
    pub irq: u64,
    /// Time servicing soft interrupts.
    pub softirq: u64,
    /// Stolen time (involuntary wait in a virtualized guest).
    pub steal: u64,
    /// Time spent running a virtual CPU for a guest.
    pub guest: u64,
    /// Time spent running a low-priority guest.
    pub guest_nice: u64,
}

impl CpuTimes {
    /// Parse the numeric fields following the `cpu`/`cpuN` token. At least the
    /// first four (user, nice, system, idle) must be present; later fields are
    /// optional and default to zero on older kernels.
    pub fn parse_fields(fields: &[&str]) -> Result<Self> {
        if fields.len() < 4 {
            return Err(Error::parse(alloc::format!(
                "cpu line has {} fields, need at least 4",
                fields.len()
            )));
        }
        let get = |i: usize| -> Result<u64> {
            match fields.get(i) {
                Some(s) => s
                    .parse()
                    .map_err(|_| Error::parse(alloc::format!("bad cpu field '{s}'"))),
                None => Ok(0),
            }
        };
        Ok(CpuTimes {
            user: get(0)?,
            nice: get(1)?,
            system: get(2)?,
            idle: get(3)?,
            iowait: get(4)?,
            irq: get(5)?,
            softirq: get(6)?,
            steal: get(7)?,
            guest: get(8)?,
            guest_nice: get(9)?,
        })
    }

    /// Total of all jiffies across every state (the denominator for
    /// utilization). `guest`/`guest_nice` are excluded because the kernel
    /// already folds them into `user`/`nice`.
    pub fn total(&self) -> u64 {
        self.user
            .wrapping_add(self.nice)
            .wrapping_add(self.system)
            .wrapping_add(self.idle)
            .wrapping_add(self.iowait)
            .wrapping_add(self.irq)
            .wrapping_add(self.softirq)
            .wrapping_add(self.steal)
    }

    /// Jiffies spent doing anything other than idling (excludes `idle` and
    /// `iowait`, matching how most tools define "busy").
    pub fn busy(&self) -> u64 {
        self.total()
            .saturating_sub(self.idle)
            .saturating_sub(self.iowait)
    }

    /// Total CPU time expressed in whole seconds, using [`USER_HZ`].
    pub fn total_secs(&self) -> u64 {
        self.total() / USER_HZ
    }
}

/// Per-state CPU utilization as fractions in `[0.0, 1.0]`, computed by diffing
/// two cumulative [`CpuTimes`] samples. This is what `talosctl dashboard`
/// renders as the CPU bar.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CpuUtilization {
    /// Fraction of the interval spent in user mode (incl. nice).
    pub user: f64,
    /// Fraction in system mode.
    pub system: f64,
    /// Fraction idle.
    pub idle: f64,
    /// Fraction waiting on I/O.
    pub iowait: f64,
    /// Fraction servicing interrupts (hard + soft).
    pub irq: f64,
    /// Fraction stolen by the hypervisor.
    pub steal: f64,
}

impl CpuUtilization {
    /// Compute utilization between an earlier (`prev`) and later (`curr`)
    /// cumulative sample.
    ///
    /// Returns `None` if the total jiffy delta is zero (no time elapsed, or the
    /// counters did not move) since utilization would be undefined.
    pub fn between(prev: &CpuTimes, curr: &CpuTimes) -> Option<Self> {
        let total = curr.total().saturating_sub(prev.total());
        if total == 0 {
            return None;
        }
        let d = total as f64;
        let delta = |c: u64, p: u64| -> f64 { c.saturating_sub(p) as f64 / d };
        Some(CpuUtilization {
            user: delta(curr.user + curr.nice, prev.user + prev.nice),
            system: delta(curr.system, prev.system),
            idle: delta(curr.idle, prev.idle),
            iowait: delta(curr.iowait, prev.iowait),
            irq: delta(curr.irq + curr.softirq, prev.irq + prev.softirq),
            steal: delta(curr.steal, prev.steal),
        })
    }

    /// Overall busy fraction (`1.0 - idle - iowait`).
    pub fn busy(&self) -> f64 {
        (1.0 - self.idle - self.iowait).clamp(0.0, 1.0)
    }
}

/// Auxiliary system-wide counters from `/proc/stat`, surfaced by Talos
/// alongside the CPU times.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SystemCounters {
    /// Total hardware interrupts serviced since boot.
    pub interrupts: u64,
    /// Total context switches since boot.
    pub context_switches: u64,
    /// Boot time as seconds since the Unix epoch.
    pub boot_time: u64,
    /// Number of processes (and threads) created since boot.
    pub processes_created: u64,
    /// Processes currently in the runnable state.
    pub procs_running: u64,
    /// Processes currently blocked on I/O.
    pub procs_blocked: u64,
    /// Total soft interrupts serviced since boot.
    pub soft_interrupts: u64,
}

/// The `CPUStat` resource spec, mirroring `perf.CPU` in Talos.
///
/// Holds the aggregate and per-core cumulative times plus the system counters,
/// and the number of cores. Controllers update this each reconcile tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuStat {
    /// Aggregate (`cpu` line) cumulative times.
    pub total: CpuTimes,
    /// Per-core cumulative times.
    pub per_cpu: alloc::vec::Vec<CpuTimes>,
    /// Auxiliary system counters.
    pub counters: SystemCounters,
}

impl CpuStat {
    /// Construct from parsed parts.
    pub fn new(
        total: CpuTimes,
        per_cpu: alloc::vec::Vec<CpuTimes>,
        counters: SystemCounters,
    ) -> Self {
        CpuStat {
            total,
            per_cpu,
            counters,
        }
    }

    /// Number of CPU cores.
    pub fn num_cpus(&self) -> usize {
        self.per_cpu.len()
    }

    /// Aggregate utilization between this (newer) stat and an `earlier` one.
    pub fn utilization_since(&self, earlier: &CpuStat) -> Option<CpuUtilization> {
        CpuUtilization::between(&earlier.total, &self.total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fields_requires_four() {
        assert!(CpuTimes::parse_fields(&["1", "2", "3"]).is_err());
        let t = CpuTimes::parse_fields(&["10", "0", "5", "85"]).unwrap();
        assert_eq!(t.user, 10);
        assert_eq!(t.idle, 85);
        // Missing later fields default to zero.
        assert_eq!(t.steal, 0);
    }

    #[test]
    fn total_and_busy() {
        let t = CpuTimes {
            user: 100,
            nice: 0,
            system: 50,
            idle: 800,
            iowait: 50,
            ..Default::default()
        };
        assert_eq!(t.total(), 1000);
        // busy excludes idle + iowait
        assert_eq!(t.busy(), 150);
        assert_eq!(t.total_secs(), 10); // 1000 jiffies / 100 Hz
    }

    #[test]
    fn utilization_between_samples() {
        let prev = CpuTimes {
            user: 100,
            system: 50,
            idle: 850,
            ..Default::default()
        };
        let curr = CpuTimes {
            user: 150,
            system: 70,
            idle: 880,
            ..Default::default()
        };
        // deltas: user 50, system 20, idle 30 -> total 100
        let u = CpuUtilization::between(&prev, &curr).unwrap();
        assert!((u.user - 0.5).abs() < 1e-9);
        assert!((u.system - 0.2).abs() < 1e-9);
        assert!((u.idle - 0.3).abs() < 1e-9);
        assert!((u.busy() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn utilization_none_when_no_time_elapsed() {
        let t = CpuTimes {
            user: 1,
            idle: 1,
            system: 1,
            nice: 1,
            ..Default::default()
        };
        assert!(CpuUtilization::between(&t, &t).is_none());
    }

    #[test]
    fn busy_clamped_to_unit_interval() {
        let u = CpuUtilization {
            idle: 0.9,
            iowait: 0.3,
            ..Default::default()
        };
        assert_eq!(u.busy(), 0.0);
    }

    #[test]
    fn cpu_stat_utilization_since() {
        let earlier = CpuStat::new(
            CpuTimes {
                user: 100,
                idle: 900,
                ..Default::default()
            },
            alloc::vec![],
            SystemCounters::default(),
        );
        let newer = CpuStat::new(
            CpuTimes {
                user: 200,
                idle: 1800,
                ..Default::default()
            },
            alloc::vec![],
            SystemCounters::default(),
        );
        let u = newer.utilization_since(&earlier).unwrap();
        // delta user 100, idle 900 -> total 1000
        assert!((u.user - 0.1).abs() < 1e-9);
        assert!((u.idle - 0.9).abs() < 1e-9);
    }
}
