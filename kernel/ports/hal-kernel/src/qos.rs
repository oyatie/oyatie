//! Quality-of-Service group newtypes for per-pod cache/bandwidth partitioning
//! (x86 RDT/CAT/MBM via CLOSID+RMID, aarch64 MPAM via PARTID).
//!
//! Pure newtypes — no logic. They give the context-switch fast path (roadmap
//! P6) a typed handle to write into PQR_ASSOC (x86) / MPAM system registers
//! (aarch64) without confusing the three distinct identifier spaces. The
//! [`QosGroup`] bundle is what a pod cgroup maps to.

/// x86 RDT Class-of-Service ID — selects the LLC/MB allocation mask (CAT/MBA).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ClosId(pub u32);

/// x86 RDT Resource-Monitoring ID — the counter bucket for LLC occupancy /
/// memory-bandwidth monitoring (CMT/MBM).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct RmId(pub u32);

/// aarch64 MPAM Partition ID — the per-request partition tag for cache/bandwidth
/// portioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct PartId(pub u32);

/// The QoS identifiers a pod is assigned, written together on context switch.
///
/// One arch uses `closid`+`rmid` (x86), the other `partid` (aarch64); the
/// bundle is arch-neutral so the scheduler carries a single field. Unused
/// members on a given arch stay at their `Default` zero (the "shared/default"
/// partition).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QosGroup {
    /// x86 allocation class.
    pub closid: ClosId,
    /// x86 monitoring bucket.
    pub rmid: RmId,
    /// aarch64 partition tag.
    pub partid: PartId,
}
