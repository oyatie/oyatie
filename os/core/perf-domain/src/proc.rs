//! `/proc` parsing helpers.
//!
//! Talos's perf controllers read system statistics straight out of `procfs`:
//! `/proc/stat` for CPU times and a handful of system counters, and
//! `/proc/meminfo` for memory and swap usage. This module mirrors the parsers
//! Talos's `gopsutil`-backed collectors rely on, but implemented directly so
//! the rest of the crate can be exercised against an in-memory filesystem.
//!
//! The kernel reports CPU time in *USER_HZ* "jiffies" (typically 100 Hz, i.e.
//! 10ms per tick). We keep the raw tick counts here and let [`crate::cpu`]
//! convert to seconds, matching how Talos surfaces the raw `CPUStat` fields.

use crate::cpu::{CpuTimes, SystemCounters};
use crate::memory::MemInfo;
use os_kernel::error::{Error, Result};
use os_kernel::os::FileSystem;

/// The canonical procfs path for CPU/scheduler statistics.
pub const PROC_STAT: &str = "/proc/stat";

/// The canonical procfs path for memory statistics.
pub const PROC_MEMINFO: &str = "/proc/meminfo";

/// Parse the contents of `/proc/stat`.
///
/// Returns the aggregate (`cpu`) line, every per-core (`cpuN`) line in order,
/// and the auxiliary system counters Talos surfaces (context switches, boot
/// time, processes forked, procs running/blocked).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcStat {
    /// Aggregate CPU times from the `cpu` line.
    pub total: CpuTimes,
    /// Per-core CPU times from the `cpuN` lines, in core-index order.
    pub per_cpu: alloc::vec::Vec<CpuTimes>,
    /// The auxiliary system-wide counters.
    pub counters: SystemCounters,
}

impl ProcStat {
    /// Number of CPU cores reported by the `cpuN` lines.
    pub fn num_cpus(&self) -> usize {
        self.per_cpu.len()
    }

    /// Read and parse `/proc/stat` from a filesystem.
    pub fn read(fs: &dyn FileSystem) -> Result<Self> {
        let text = fs.read_to_string(PROC_STAT)?;
        Self::parse(&text)
    }

    /// Parse raw `/proc/stat` text.
    pub fn parse(text: &str) -> Result<Self> {
        let mut total: Option<CpuTimes> = None;
        let mut per_cpu: alloc::vec::Vec<(usize, CpuTimes)> = alloc::vec::Vec::new();
        let mut counters = SystemCounters::default();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut it = line.split_whitespace();
            let key = match it.next() {
                Some(k) => k,
                None => continue,
            };
            let rest: alloc::vec::Vec<&str> = it.collect();

            if key == "cpu" {
                total = Some(CpuTimes::parse_fields(&rest)?);
            } else if let Some(idx) = key.strip_prefix("cpu") {
                let idx: usize = idx
                    .parse()
                    .map_err(|_| Error::parse(alloc::format!("bad cpu index in '{key}'")))?;
                per_cpu.push((idx, CpuTimes::parse_fields(&rest)?));
            } else {
                match key {
                    "ctxt" => counters.context_switches = parse_first(&rest, "ctxt")?,
                    "btime" => counters.boot_time = parse_first(&rest, "btime")?,
                    "processes" => counters.processes_created = parse_first(&rest, "processes")?,
                    "procs_running" => {
                        counters.procs_running = parse_first(&rest, "procs_running")?
                    }
                    "procs_blocked" => {
                        counters.procs_blocked = parse_first(&rest, "procs_blocked")?
                    }
                    "intr" => counters.interrupts = parse_first(&rest, "intr")?,
                    "softirq" => counters.soft_interrupts = parse_first(&rest, "softirq")?,
                    _ => { /* ignore lines we don't model (page, swap, etc.) */ }
                }
            }
        }

        let total = total.ok_or_else(|| Error::parse("no 'cpu' line in /proc/stat"))?;

        // The kernel emits `cpuN` lines in index order; sort defensively and
        // verify contiguity so a malformed file is rejected rather than
        // silently producing a sparse core list.
        per_cpu.sort_by_key(|(idx, _)| *idx);
        for (expected, (idx, _)) in per_cpu.iter().enumerate() {
            if *idx != expected {
                return Err(Error::parse(alloc::format!(
                    "non-contiguous cpu index: expected cpu{expected}, found cpu{idx}"
                )));
            }
        }
        let per_cpu = per_cpu.into_iter().map(|(_, t)| t).collect();

        Ok(ProcStat {
            total,
            per_cpu,
            counters,
        })
    }
}

/// Parse the contents of `/proc/meminfo` into a [`MemInfo`].
///
/// `/proc/meminfo` reports each field as a key, a value, and (for memory
/// quantities) the `kB` unit. Values are kibibytes; we convert to bytes so
/// downstream consumers work in a single unit, matching Talos's `MemStat`.
pub fn parse_meminfo(text: &str) -> Result<MemInfo> {
    let mut info = MemInfo::default();
    let mut saw_mem_total = false;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| Error::parse(alloc::format!("malformed meminfo line '{line}'")))?;
        let key = key.trim();
        let value = value.trim();

        // Strip the trailing `kB` unit when present and scale to bytes.
        let (num_str, in_kib) = match value.strip_suffix("kB") {
            Some(n) => (n.trim(), true),
            None => (value, false),
        };
        let num: u64 = num_str.parse().map_err(|_| {
            Error::parse(alloc::format!("bad meminfo value for '{key}': '{value}'"))
        })?;
        let bytes = if in_kib {
            num.saturating_mul(1024)
        } else {
            num
        };

        match key {
            "MemTotal" => {
                info.mem_total = bytes;
                saw_mem_total = true;
            }
            "MemFree" => info.mem_free = bytes,
            "MemAvailable" => info.mem_available = bytes,
            "Buffers" => info.buffers = bytes,
            "Cached" => info.cached = bytes,
            "SwapCached" => info.swap_cached = bytes,
            "Active" => info.active = bytes,
            "Inactive" => info.inactive = bytes,
            "SwapTotal" => info.swap_total = bytes,
            "SwapFree" => info.swap_free = bytes,
            "Shmem" => info.shared = bytes,
            "Slab" => info.slab = bytes,
            _ => { /* many fields are not surfaced by Talos MemStat */ }
        }
    }

    if !saw_mem_total {
        return Err(Error::parse("no 'MemTotal' field in /proc/meminfo"));
    }
    info.validate()?;
    Ok(info)
}

/// Read and parse `/proc/meminfo` from a filesystem.
pub fn read_meminfo(fs: &dyn FileSystem) -> Result<MemInfo> {
    let text = fs.read_to_string(PROC_MEMINFO)?;
    parse_meminfo(&text)
}

fn parse_first(fields: &[&str], key: &str) -> Result<u64> {
    let first = fields
        .first()
        .ok_or_else(|| Error::parse(alloc::format!("missing value for '{key}'")))?;
    first
        .parse()
        .map_err(|_| Error::parse(alloc::format!("bad value for '{key}': '{first}'")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_kernel::os::{FileSystem, MemoryFs};

    const SAMPLE_STAT: &str = "\
cpu  100 0 200 700 0 0 0 0 0 0
cpu0 50 0 100 350 0 0 0 0 0 0
cpu1 50 0 100 350 0 0 0 0 0 0
intr 12345 0 0
ctxt 9876543
btime 1700000000
processes 4242
procs_running 2
procs_blocked 0
softirq 555 1 2 3
";

    const SAMPLE_MEMINFO: &str = "\
MemTotal:       16384000 kB
MemFree:         8192000 kB
MemAvailable:   12288000 kB
Buffers:          512000 kB
Cached:          2048000 kB
SwapCached:            0 kB
Active:          3000000 kB
Inactive:        1000000 kB
SwapTotal:       2048000 kB
SwapFree:        2048000 kB
Shmem:            128000 kB
Slab:             256000 kB
";

    #[test]
    fn parses_proc_stat_total_and_per_cpu() {
        let st = ProcStat::parse(SAMPLE_STAT).unwrap();
        assert_eq!(st.num_cpus(), 2);
        assert_eq!(st.total.user, 100);
        assert_eq!(st.total.system, 200);
        assert_eq!(st.total.idle, 700);
        assert_eq!(st.per_cpu[0].user, 50);
        assert_eq!(st.per_cpu[1].idle, 350);
    }

    #[test]
    fn parses_proc_stat_counters() {
        let st = ProcStat::parse(SAMPLE_STAT).unwrap();
        assert_eq!(st.counters.context_switches, 9_876_543);
        assert_eq!(st.counters.boot_time, 1_700_000_000);
        assert_eq!(st.counters.processes_created, 4242);
        assert_eq!(st.counters.procs_running, 2);
        assert_eq!(st.counters.procs_blocked, 0);
        assert_eq!(st.counters.interrupts, 12345);
        assert_eq!(st.counters.soft_interrupts, 555);
    }

    #[test]
    fn proc_stat_requires_cpu_line() {
        assert!(ProcStat::parse("intr 1\nctxt 2\n").is_err());
    }

    #[test]
    fn proc_stat_rejects_noncontiguous_cores() {
        let bad = "cpu 1 0 1 1\ncpu0 1 0 1 1\ncpu2 1 0 1 1\n";
        assert!(ProcStat::parse(bad).is_err());
    }

    #[test]
    fn proc_stat_sorts_out_of_order_cores() {
        let s = "cpu 2 0 2 2\ncpu1 1 0 1 1\ncpu0 1 0 1 1\n";
        let st = ProcStat::parse(s).unwrap();
        assert_eq!(st.num_cpus(), 2);
    }

    #[test]
    fn parses_meminfo_to_bytes() {
        let info = parse_meminfo(SAMPLE_MEMINFO).unwrap();
        assert_eq!(info.mem_total, 16_384_000 * 1024);
        assert_eq!(info.mem_free, 8_192_000 * 1024);
        assert_eq!(info.mem_available, 12_288_000 * 1024);
        assert_eq!(info.swap_total, 2_048_000 * 1024);
        assert_eq!(info.cached, 2_048_000 * 1024);
    }

    #[test]
    fn meminfo_requires_memtotal() {
        assert!(parse_meminfo("MemFree: 100 kB\n").is_err());
    }

    #[test]
    fn meminfo_rejects_malformed_line() {
        assert!(parse_meminfo("MemTotal 100 kB\n").is_err());
        assert!(parse_meminfo("MemTotal: notanumber kB\n").is_err());
    }

    #[test]
    fn reads_from_filesystem() {
        let mut fs = MemoryFs::new();
        fs.write(PROC_STAT, SAMPLE_STAT.as_bytes()).unwrap();
        fs.write(PROC_MEMINFO, SAMPLE_MEMINFO.as_bytes()).unwrap();

        let st = ProcStat::read(&fs).unwrap();
        assert_eq!(st.num_cpus(), 2);
        let info = read_meminfo(&fs).unwrap();
        assert_eq!(info.mem_total, 16_384_000 * 1024);
    }
}
