//! Supervisor-mode hardware protections in `CR4` (SMEP / SMAP / UMIP).
//!
//! This slice lays the ring-3 scaffolding; nothing executes in user mode yet.
//! Enabling these `CR4` bits now is safe precisely *because* there is no user
//! mapping or user access on the boot/timer path for them to break:
//!
//!   * **SMEP** (`CR4.SMEP`, bit 20) — the CPU faults if ring 0 *executes* from
//!     a page whose PTE has the user (US) bit set. The kernel never executes
//!     user pages, so this is inert until user code exists.
//!   * **SMAP** (`CR4.SMAP`, bit 21) — the CPU faults if ring 0 *reads/writes*
//!     a user page outside a `STAC`/`CLAC` window. There is no user mapping in
//!     this slice, so the kernel never touches one; once user memory exists the
//!     copy-to/from-user helpers must bracket their accesses with `STAC`/`CLAC`
//!     (a later slice — this module deliberately adds no user access).
//!   * **UMIP** (`CR4.UMIP`, bit 11) — the CPU faults ring 3 attempts to run
//!     `sgdt`/`sidt`/`sldt`/`smsw`/`str` (which would leak descriptor-table
//!     layout). Harmless in ring 0; only constrains ring 3, which does not run
//!     yet.
//!
//! Each bit is **CpuCaps-gated**: we set it only if [`probe_cpu_caps`] reports
//! the feature present. Setting an unsupported `CR4` bit is a `#GP`, which on
//! the boot path with no handler would escalate to a triple fault and reset —
//! so the gate is what keeps boot reaching the OK marker on CPUs (e.g. QEMU's
//! default `qemu64`) that lack these features. The bits are *pinned* by being
//! set once here in bring-up; this module exposes no way to clear them, and the
//! boot/timer path never rewrites `CR4` wholesale, so they stay set.
//!
//! [`probe_cpu_caps`]: crate::hal_caps::probe_cpu_caps

use x86_64::registers::control::{Cr4, Cr4Flags};

use crate::hal_caps::probe_cpu_caps;

/// Which supervisor-protection `CR4` bits we actually enabled, for the bring-up
/// log / report. A bit is `true` only when CPUID reported it present *and* we
/// set it (the two are equivalent here: a present bit is always set).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnabledProtections {
    /// `CR4.SMEP` (bit 20) was set (CPUID.7.0:EBX[7] present).
    pub smep: bool,
    /// `CR4.SMAP` (bit 21) was set (CPUID.7.0:EBX[20] present).
    pub smap: bool,
    /// `CR4.UMIP` (bit 11) was set (CPUID.7.0:ECX[2] present).
    pub umip: bool,
}

/// Enable the CPUID-supported subset of {SMEP, SMAP, UMIP} in `CR4`, leaving
/// every other `CR4` bit untouched, and report which were turned on.
///
/// Safe-fallback by construction: a feature CPUID does not report is skipped,
/// so we never write a reserved/unsupported `CR4` bit (which would `#GP` →
/// triple-fault on the boot path). Idempotent for already-set bits.
///
/// # Safety
/// Call once during bring-up on the boot core, after the GDT/IDT are installed
/// (so a `#GP` — e.g. from a CPU that reports a feature but rejects the bit —
/// would at least reach the exception handler rather than triple-faulting
/// immediately). Reconfigures supervisor-mode access/execute protections; only
/// the named bits are touched via a read-modify-write that preserves the rest.
pub unsafe fn enable_supervisor_protections() -> EnabledProtections {
    let caps = probe_cpu_caps();

    let mut to_set = Cr4Flags::empty();
    if caps.smep {
        to_set |= Cr4Flags::SUPERVISOR_MODE_EXECUTION_PROTECTION;
    }
    if caps.smap {
        to_set |= Cr4Flags::SUPERVISOR_MODE_ACCESS_PREVENTION;
    }
    if caps.umip {
        to_set |= Cr4Flags::USER_MODE_INSTRUCTION_PREVENTION;
    }

    if !to_set.is_empty() {
        // SAFETY: read-modify-write of CR4 that only *adds* the CpuCaps-gated
        // supervisor-protection bits and preserves every other bit (PAE, PGE,
        // etc. the boot trampoline set). Each added bit is backed by a positive
        // CPUID report, so the write cannot set a reserved/unsupported bit; the
        // protections are inert in this slice (no user mode/mapping yet).
        unsafe {
            Cr4::update(|flags| *flags |= to_set);
        }
    }

    EnabledProtections {
        smep: caps.smep,
        smap: caps.smap,
        umip: caps.umip,
    }
}
