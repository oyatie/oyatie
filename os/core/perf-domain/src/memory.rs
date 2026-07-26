//! Memory statistics: the `MemStat` resource.
//!
//! Mirrors `pkg/machinery/resources/perf` (`Memory`) and the machined `perf`
//! memory controller. All quantities are stored in **bytes** (the procfs source
//! is kibibytes; [`crate::proc::parse_meminfo`] scales on the way in) so the
//! dashboard and `talosctl` see one unit.

use os_kernel::error::{Error, Result};

/// A subset of `/proc/meminfo` fields, the ones Talos's `MemStat` surfaces.
///
/// All fields are byte counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemInfo {
    /// Total usable RAM.
    pub mem_total: u64,
    /// Free RAM not used for anything.
    pub mem_free: u64,
    /// Estimate of RAM available for starting new applications.
    pub mem_available: u64,
    /// Memory in raw disk-block buffers.
    pub buffers: u64,
    /// Page-cache memory.
    pub cached: u64,
    /// Memory that is cached but also present in swap.
    pub swap_cached: u64,
    /// Recently used memory, not reclaimed unless necessary.
    pub active: u64,
    /// Less recently used memory, more reclaimable.
    pub inactive: u64,
    /// Total swap space.
    pub swap_total: u64,
    /// Unused swap space.
    pub swap_free: u64,
    /// Shared memory (tmpfs).
    pub shared: u64,
    /// Kernel slab allocator memory.
    pub slab: u64,
}

impl MemInfo {
    /// Validate internal consistency: free/available cannot exceed total, and
    /// free swap cannot exceed total swap.
    pub fn validate(&self) -> Result<()> {
        if self.mem_total == 0 {
            return Err(Error::invalid("MemTotal is zero"));
        }
        if self.mem_free > self.mem_total {
            return Err(Error::invalid("MemFree exceeds MemTotal"));
        }
        if self.mem_available > self.mem_total {
            return Err(Error::invalid("MemAvailable exceeds MemTotal"));
        }
        if self.swap_free > self.swap_total {
            return Err(Error::invalid("SwapFree exceeds SwapTotal"));
        }
        Ok(())
    }

    /// RAM currently in use, derived as `total - free - buffers - cached`,
    /// matching the classic "used" figure (kernel-reclaimable cache excluded).
    /// Saturating so it can never underflow.
    pub fn used(&self) -> u64 {
        self.mem_total
            .saturating_sub(self.mem_free)
            .saturating_sub(self.buffers)
            .saturating_sub(self.cached)
    }

    /// Swap currently in use.
    pub fn swap_used(&self) -> u64 {
        self.swap_total.saturating_sub(self.swap_free)
    }

    /// Fraction of RAM used in `[0.0, 1.0]`, based on [`used`](Self::used).
    pub fn used_fraction(&self) -> f64 {
        if self.mem_total == 0 {
            return 0.0;
        }
        (self.used() as f64 / self.mem_total as f64).clamp(0.0, 1.0)
    }

    /// Fraction of RAM available (per the kernel's `MemAvailable`) in
    /// `[0.0, 1.0]`.
    pub fn available_fraction(&self) -> f64 {
        if self.mem_total == 0 {
            return 0.0;
        }
        (self.mem_available as f64 / self.mem_total as f64).clamp(0.0, 1.0)
    }

    /// Fraction of swap in use in `[0.0, 1.0]`; `0.0` when there is no swap.
    pub fn swap_used_fraction(&self) -> f64 {
        if self.swap_total == 0 {
            return 0.0;
        }
        (self.swap_used() as f64 / self.swap_total as f64).clamp(0.0, 1.0)
    }

    /// Whether the system is under memory pressure: less than `threshold`
    /// fraction of RAM is available. A threshold of e.g. `0.10` flags when
    /// under 10% remains.
    pub fn under_pressure(&self, threshold: f64) -> bool {
        self.available_fraction() < threshold
    }
}

/// The `MemStat` resource spec, mirroring `perf.Memory` in Talos. A thin
/// newtype over [`MemInfo`] so the controller layer has a distinct resource
/// type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemStat {
    /// The captured memory information.
    pub info: MemInfo,
}

impl MemStat {
    /// Construct from validated info.
    pub fn new(info: MemInfo) -> Result<Self> {
        info.validate()?;
        Ok(MemStat { info })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> MemInfo {
        MemInfo {
            mem_total: 16_000,
            mem_free: 4_000,
            mem_available: 8_000,
            buffers: 1_000,
            cached: 3_000,
            swap_total: 2_000,
            swap_free: 1_500,
            ..Default::default()
        }
    }

    #[test]
    fn used_excludes_cache_and_buffers() {
        let m = sample();
        // 16000 - 4000 - 1000 - 3000 = 8000
        assert_eq!(m.used(), 8_000);
        assert!((m.used_fraction() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn available_fraction() {
        let m = sample();
        assert!((m.available_fraction() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn swap_usage() {
        let m = sample();
        assert_eq!(m.swap_used(), 500);
        assert!((m.swap_used_fraction() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn no_swap_is_zero_fraction() {
        let m = MemInfo {
            mem_total: 100,
            swap_total: 0,
            ..Default::default()
        };
        assert_eq!(m.swap_used_fraction(), 0.0);
    }

    #[test]
    fn pressure_detection() {
        let mut m = sample();
        assert!(!m.under_pressure(0.10));
        m.mem_available = 800; // 5% of 16000
        assert!(m.under_pressure(0.10));
    }

    #[test]
    fn validation_rejects_inconsistent() {
        let mut m = sample();
        assert!(m.validate().is_ok());
        m.mem_free = m.mem_total + 1;
        assert!(m.validate().is_err());

        let mut m2 = sample();
        m2.swap_free = m2.swap_total + 1;
        assert!(m2.validate().is_err());

        let zero = MemInfo::default();
        assert!(zero.validate().is_err());
    }

    #[test]
    fn used_saturates() {
        // Pathological values must never underflow.
        let m = MemInfo {
            mem_total: 100,
            mem_free: 200,
            buffers: 200,
            cached: 200,
            ..Default::default()
        };
        assert_eq!(m.used(), 0);
        assert_eq!(m.used_fraction(), 0.0);
    }

    #[test]
    fn memstat_new_validates() {
        assert!(MemStat::new(sample()).is_ok());
        assert!(MemStat::new(MemInfo::default()).is_err());
    }
}
