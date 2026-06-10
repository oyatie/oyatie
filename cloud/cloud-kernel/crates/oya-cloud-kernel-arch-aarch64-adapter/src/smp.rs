//! aarch64 SMP / AP bring-up (P4·SMP·S3). Frame code (TCB).
//!
//! Brings secondary CPUs (APs) online on the QEMU `virt` machine via PSCI
//! `CPU_ON`, then leaves them idling at `wfe`. The dangerous work lives here in
//! the Frame: the `hvc` PSCI call, the read-only DTB `/cpus` walk, and the AP's
//! per-CPU MMU/GICR/ICC bring-up. The safe kernel sees only the
//! `#![forbid(unsafe_code)]` [`hal::smp::Smp`] seam this module implements on
//! [`crate::Aarch64`].
//!
//! ## End to end
//!
//! 1. `_start` preserved the DTB base QEMU passed in `x0` (the bss-clobber bug
//!    is fixed in `boot.rs`); [`store_dtb`] recorded it. [`enumerate`] does a
//!    minimal FDT `/cpus` walk: it counts `cpu@*` nodes (`cpu_count`) and reads
//!    each node's `reg` cell (the MPIDR affinity for PSCI `CPU_ON`).
//! 2. [`Aarch64::start_secondaries`] issues PSCI `CPU_ON(target_mpidr, &_ap_start,
//!    context_id = cpu_index)` for each AP, then bounded-spins on [`ONLINE_MASK`]
//!    until that AP publishes its online bit, printing `cpu k online` as it does.
//! 3. PSCI hands each AP to `_ap_start` (in `boot.rs`) with `x0 = cpu_index`; the
//!    asm sets the AP's stack from [`AP_STACKS`] and calls [`ap_rust_entry`],
//!    which enables translation against the SHARED L1 table (TTBR0 only), sets
//!    `TPIDR_EL1`, wakes its own GICR + ICC + timer PPI, `Release`-publishes its
//!    online bit, and `wfe`-idles forever.
//!
//! The online-mask handshake is the H1-proven publish/observe discipline: the
//! AP `fetch_or(Release)` after FULL init, the BSP `load(Acquire)`, so a set bit
//! carries that AP's completed per-CPU writes.

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use hal::cpu::MAX_CPUS;

use crate::gicv3::GicVersion;

// ---------------------------------------------------------------------------
// Online mask (the startup sync; bit 0 = BSP, pre-set)
// ---------------------------------------------------------------------------

/// Bitmask of CPUs that have published their online bit. Bit 0 (BSP) is pre-set;
/// each AP `fetch_or(Release)`-s its bit after full init, and the BSP
/// `load(Acquire)`-s here (the H1 publish/observe discipline). `MAX_CPUS ≤ 64`.
static ONLINE_MASK: AtomicU64 = AtomicU64::new(1);

/// Read the online mask with `Acquire` so an observed bit `k` carries CPU `k`'s
/// completed per-CPU init.
#[inline]
pub fn online_mask() -> u64 {
    ONLINE_MASK.load(Ordering::Acquire)
}

// ---------------------------------------------------------------------------
// Per-AP stacks (static, .bss; 64 KiB each to match the BSP boot stack)
// ---------------------------------------------------------------------------

/// Per-AP kernel stack size (64 KiB, matching the BSP boot stack in `linker.ld`).
/// An idle AP uses almost none of this; it is generous and bounded.
pub const AP_STACK_SIZE: usize = 64 * 1024;

/// One 16-byte-aligned stack per logical CPU. Slot 0 (BSP) is unused (the BSP
/// runs on the linker `__stack_top`). Lives in the dedicated `.ap_stacks` NOLOAD
/// section (placed AFTER the boot stack by `linker.ld`), NOT in `.bss` — so its
/// 8×64 KiB does not grow `.bss` and shift `__stack_top` / the heap, keeping the
/// default 1-vCPU layout (and the byte-identical golden) untouched. `_ap_start`
/// references it by the `__ap_stacks` symbol and indexes `idx * AP_STACK_SIZE`.
#[repr(C, align(16))]
struct ApStacks([[u8; AP_STACK_SIZE]; MAX_CPUS]);

#[no_mangle]
#[link_section = ".ap_stacks"]
static mut __ap_stacks: ApStacks = ApStacks([[0; AP_STACK_SIZE]; MAX_CPUS]);

// ---------------------------------------------------------------------------
// DTB pointer + /cpus enumeration
// ---------------------------------------------------------------------------

/// The DTB base QEMU passed in `x0`, preserved by `_start` and recorded by
/// `rust_start`. `usize::MAX` means "not yet stored / unavailable".
static DTB_PTR: AtomicUsize = AtomicUsize::new(usize::MAX);

/// Record the preserved DTB base (called from `rust_start`). Plain pointer store.
pub fn store_dtb(dtb: *const u8) {
    DTB_PTR.store(dtb as usize, Ordering::Relaxed);
}

/// The result of the `/cpus` walk: how many CPUs, and each one's MPIDR affinity.
#[derive(Clone, Copy)]
struct CpuTopology {
    count: usize,
    /// `mpidr[i]` = the affinity value from `cpu@i`'s `reg` property, used as the
    /// PSCI `CPU_ON` target. Slot 0 is the BSP.
    mpidr: [u64; MAX_CPUS],
}

// --- minimal big-endian FDT reads ------------------------------------------

///
/// # Safety
/// `p` must point at 4 readable bytes.
#[inline]
unsafe fn be32(p: *const u8) -> u32 {
    // SAFETY (delegated to caller): `p` points at 4 readable bytes. FDT is
    // big-endian; assemble the u32 from individual byte reads (no alignment
    // assumption).
    unsafe {
        ((*p as u32) << 24)
            | ((*p.add(1) as u32) << 16)
            | ((*p.add(2) as u32) << 8)
            | (*p.add(3) as u32)
    }
}

// FDT structure-block tokens (big-endian u32 at each 4-byte step).
const FDT_BEGIN_NODE: u32 = 0x0000_0001;
const FDT_END_NODE: u32 = 0x0000_0002;
const FDT_PROP: u32 = 0x0000_0003;
const FDT_NOP: u32 = 0x0000_0004;
const FDT_END: u32 = 0x0000_0009;
const FDT_MAGIC: u32 = 0xd00d_feed;

/// Walk the flattened device tree's `/cpus` node and return the CPU topology:
/// the number of `cpu@*` children and each child's `reg` (MPIDR affinity).
///
/// This is a single-purpose, bounded FDT structure-block walk — NOT a general
/// parser. It finds the `cpus` node, then for each child node reads its first
/// `reg` property cell as the MPIDR. If the DTB is missing/malformed it returns
/// `None`, and the caller falls back to the documented contiguous-MPIDR floor.
///
/// # Safety
/// `dtb` must be a valid FDT base (the pointer QEMU passed in `x0`, identity
/// mapped). Read-only.
unsafe fn walk_cpus(dtb: *const u8) -> Option<CpuTopology> {
    // SAFETY (delegated): bounded read-only walk of a valid FDT blob.
    unsafe {
        if be32(dtb) != FDT_MAGIC {
            return None;
        }
        let off_struct = be32(dtb.add(8)) as usize;
        let off_strings = be32(dtb.add(12)) as usize;
        let strings = dtb.add(off_strings);

        let mut p = dtb.add(off_struct);
        // Depth tracking so we only treat DIRECT children of `cpus` as CPUs.
        let mut depth: i32 = 0;
        let mut in_cpus_depth: i32 = -1; // depth at which `cpus` opened, else -1
        let mut topo = CpuTopology {
            count: 0,
            mpidr: [0; MAX_CPUS],
        };
        // Bound the walk so a malformed blob cannot spin forever.
        let mut guard: u32 = 1_000_000;

        loop {
            guard -= 1;
            if guard == 0 {
                return None;
            }
            let tok = be32(p);
            p = p.add(4);
            match tok {
                FDT_BEGIN_NODE => {
                    // Node name (NUL-terminated, padded to 4 bytes).
                    let name = p;
                    let mut len = 0usize;
                    while *name.add(len) != 0 {
                        len += 1;
                    }
                    depth += 1;
                    // Is this node `cpus` (a direct child of root, depth 1)?
                    if in_cpus_depth < 0 && name_is(name, len, b"cpus") {
                        in_cpus_depth = depth;
                    } else if in_cpus_depth >= 0
                        && depth == in_cpus_depth + 1
                        && name_starts_with(name, len, b"cpu@")
                    {
                        // A `cpu@N` node: its index slot is the count so far.
                        if topo.count < MAX_CPUS {
                            // MPIDR filled when we hit its `reg` prop below.
                            topo.mpidr[topo.count] = topo.count as u64;
                            topo.count += 1;
                        }
                    }
                    // Advance past the name (NUL + pad to 4).
                    p = p.add((len + 1 + 3) & !3);
                }
                FDT_END_NODE => {
                    if in_cpus_depth >= 0 && depth == in_cpus_depth {
                        // Left the `cpus` subtree — done.
                        return Some(topo);
                    }
                    depth -= 1;
                }
                FDT_PROP => {
                    let prop_len = be32(p) as usize;
                    let nameoff = be32(p.add(4)) as usize;
                    let data = p.add(8);
                    // If this prop is `reg` on the cpu node we just opened, read
                    // its first cell as the MPIDR. The cpu node is the most
                    // recent child counted; depth == in_cpus_depth + 1.
                    if in_cpus_depth >= 0
                        && depth == in_cpus_depth + 1
                        && topo.count > 0
                        && name_is_str(strings.add(nameoff), b"reg")
                        && prop_len >= 4
                    {
                        let idx = topo.count - 1;
                        // `reg` may be 1 or 2 cells; the low cell is Aff0..2.
                        let mpidr = if prop_len >= 8 {
                            ((be32(data) as u64) << 32) | be32(data.add(4)) as u64
                        } else {
                            be32(data) as u64
                        };
                        topo.mpidr[idx] = mpidr;
                    }
                    // Advance past the prop value (padded to 4).
                    p = p.add(8 + ((prop_len + 3) & !3));
                }
                FDT_NOP => {}
                FDT_END => return Some(topo),
                _ => return None, // malformed token
            }
        }
    }
}

/// Compare a (ptr,len) node name against an exact byte string.
///
/// # Safety
/// `name` points at `len` readable bytes.
unsafe fn name_is(name: *const u8, len: usize, want: &[u8]) -> bool {
    if len != want.len() {
        return false;
    }
    // SAFETY (delegated): `len` bytes readable.
    unsafe {
        for (i, &w) in want.iter().enumerate() {
            if *name.add(i) != w {
                return false;
            }
        }
    }
    true
}

/// Whether a (ptr,len) node name starts with `prefix`.
///
/// # Safety
/// `name` points at `len` readable bytes.
unsafe fn name_starts_with(name: *const u8, len: usize, prefix: &[u8]) -> bool {
    if len < prefix.len() {
        return false;
    }
    // SAFETY (delegated): `len` bytes readable.
    unsafe {
        for (i, &w) in prefix.iter().enumerate() {
            if *name.add(i) != w {
                return false;
            }
        }
    }
    true
}

/// Whether the NUL-terminated string at `s` equals `want`.
///
/// # Safety
/// `s` is a readable NUL-terminated string (the FDT strings block).
unsafe fn name_is_str(s: *const u8, want: &[u8]) -> bool {
    // SAFETY (delegated): NUL-terminated readable string.
    unsafe {
        let mut i = 0usize;
        while i < want.len() {
            if *s.add(i) != want[i] {
                return false;
            }
            i += 1;
        }
        *s.add(i) == 0
    }
}

/// Locate the FDT base from the preserved boot-register pointer, validated by
/// the FDT magic. Returns a validated FDT base, or `None`.
///
/// NOTE: QEMU `virt` only hands the DTB base in `x0` for the Linux `Image`/
/// `zImage` boot protocol; for a raw-ELF `-kernel` boot (our case under TCG) it
/// passes `x0 = 0` and does not place a DTB at a fixed low address. So this
/// returns `None` under the ELF boot, and [`enumerate`] falls back to the
/// PSCI `AFFINITY_INFO` probe below — the documented grounded floor (contiguous
/// MPIDR `Aff0 = 0..N-1`, dumpdtb-confirmed for QEMU flat `-smp N`). On a real
/// firmware/`Image` boot the FDT pointer IS valid and the `/cpus` walk is used.
fn find_fdt() -> Option<*const u8> {
    let p = DTB_PTR.load(Ordering::Relaxed);
    if p != usize::MAX && p != 0 {
        // SAFETY: validate the magic before trusting the pointer; a single
        // 4-byte read at an identity-mapped address. (Only reached on a real
        // firmware/`Image` boot where x0 carried a valid FDT base.)
        if unsafe { be32(p as *const u8) } == FDT_MAGIC {
            return Some(p as *const u8);
        }
    }
    None
}

/// PSCI `AFFINITY_INFO` function ID (SMC64), DTB-confirmed conduit (`hvc`).
/// Returns `0` (ON), `1` (OFF), `2` (ON_PENDING) for a CPU that EXISTS, or a
/// negative error (`-2` INVALID_PARAMS) for a non-existent affinity — a clean,
/// read-only "does CPU k exist?" probe that needs no DTB.
const PSCI_AFFINITY_INFO: u64 = 0xC400_0004;

/// Read-only PSCI `AFFINITY_INFO(target_affinity, lowest_affinity_level)` over
/// the same `hvc` conduit. Does NOT start the CPU — it only queries state.
///
/// # Safety
/// A side-effect-free firmware query (no CPU is powered on). Clobbers x0..x2.
unsafe fn psci_affinity_info(target_affinity: u64, lowest_affinity_level: u64) -> i64 {
    let r: i64;
    // SAFETY (delegated): architected PSCI AFFINITY_INFO query; x0 returns the
    // status, x0..x2 are clobbered by the SMC convention.
    unsafe {
        core::arch::asm!(
            "hvc #0",
            inout("x0") PSCI_AFFINITY_INFO => r,
            in("x1") target_affinity,
            in("x2") lowest_affinity_level,
            options(nostack, nomem),
        );
    }
    r
}

/// Enumerate CPUs, clamped to `MAX_CPUS`. Prefers the DTB `/cpus` walk when a
/// valid FDT pointer is available (real firmware/`Image` boot); otherwise uses
/// the PSCI `AFFINITY_INFO` probe with contiguous MPIDRs (the QEMU raw-ELF boot
/// floor). Always returns ≥ 1.
fn enumerate() -> CpuTopology {
    if let Some(fdt) = find_fdt() {
        // SAFETY: `fdt` was validated to carry the FDT magic and lies in the
        // RAM/device identity map; the walk is read-only + bounded.
        if let Some(mut topo) = unsafe { walk_cpus(fdt) } {
            if topo.count >= 1 {
                topo.count = topo.count.clamp(1, MAX_CPUS);
                return topo;
            }
        }
    }

    // Floor: probe contiguous affinities 0..MAX_CPUS via PSCI AFFINITY_INFO.
    // A CPU exists iff the query returns a non-negative status (ON/OFF/PENDING);
    // a negative status (INVALID_PARAMS) means no such CPU — stop counting.
    let mut topo = CpuTopology {
        count: 0,
        mpidr: [0; MAX_CPUS],
    };
    for aff in 0..MAX_CPUS {
        // SAFETY: read-only PSCI query for affinity `aff` at level 0.
        let info = unsafe { psci_affinity_info(aff as u64, 0) };
        if info < 0 {
            break; // no CPU at this affinity (contiguous ⇒ none beyond).
        }
        topo.mpidr[topo.count] = aff as u64; // Aff0 = aff (flat QEMU topology).
        topo.count += 1;
    }
    if topo.count == 0 {
        // PSCI itself unavailable ⇒ assume the lone boot CPU.
        topo.count = 1;
        topo.mpidr[0] = 0;
    }
    topo
}

/// The enumerated CPU count (≥ 1), for [`hal::smp::Smp::cpu_count`].
pub fn cpu_count() -> u32 {
    enumerate().count as u32
}

// ---------------------------------------------------------------------------
// PSCI CPU_ON (rides the exact hvc/smccc conduit power_off() uses)
// ---------------------------------------------------------------------------

/// PSCI `CPU_ON` function ID (SMC64), DTB-confirmed (`psci { cpu_on = 0xc4000003 }`).
const PSCI_CPU_ON: u64 = 0xC400_0003;

/// Issue PSCI `CPU_ON(target_mpidr, entry, context_id)` over `hvc` — the same
/// conduit `power_off()` uses. Returns the PSCI status (0 = SUCCESS, negative =
/// error e.g. -4 ALREADY_ON, -2 INVALID_PARAMS).
///
/// # Safety
/// Issues a firmware/hypervisor call that powers on another CPU at `entry`
/// (which must be a valid physical entry point — here `_ap_start`). Cross-CPU
/// memory effects (NOT `nomem`): the AP begins executing against shared memory.
unsafe fn psci_cpu_on(target_mpidr: u64, entry: u64, context_id: u64) -> i64 {
    let r: i64;
    // SAFETY (delegated): the architected PSCI CPU_ON SMC64 call; x0..x3 carry
    // the function id + args, x0 returns the status, and x0..x3 are clobbered by
    // the SMC return convention.
    unsafe {
        core::arch::asm!(
            "hvc #0",
            inout("x0") PSCI_CPU_ON => r,
            in("x1") target_mpidr,
            in("x2") entry,
            in("x3") context_id,
            lateout("x2") _,
            lateout("x3") _,
            options(nostack),
        );
    }
    r
}

// ---------------------------------------------------------------------------
// AP Rust entry (called by `_ap_start` with x0 = cpu_index)
// ---------------------------------------------------------------------------

/// First Rust code on an AP. The asm prologue (`_ap_start`) set this AP's stack
/// and passed its `cpu_index` (the PSCI `context_id`). Here we bring up this
/// AP's per-CPU state against the SHARED kernel structures, publish the online
/// bit, then idle forever. NEVER returns.
///
/// # Safety
/// Entered once per AP from `_ap_start` with the MMU off, IRQs masked, on this
/// AP's own stack, `idx` = its unique logical index `< MAX_CPUS`.
pub(crate) extern "C" fn ap_rust_entry(idx: usize) -> ! {
    // 1. Enable translation on THIS CPU against the already-built shared L1
    //    table (TTBR0 only). The image is identity-mapped, so the PC stays valid
    //    across the SCTLR write.
    // SAFETY: AP, MMU off; the BSP already built the shared L1 table.
    unsafe {
        crate::mmu::enable_translation();
    }

    // 2. Per-CPU anchor: TPIDR_EL1 = idx, so `percpu::this_cpu_token()` indexes
    //    this AP's slot.
    // SAFETY: AP, first per-CPU anchor install; `idx < MAX_CPUS`.
    unsafe {
        crate::percpu::init_cpu(idx);
    }

    // 3. Wake THIS AP's local interrupt chip (per-CPU GICR + ICC on v3, or the
    //    per-CPU banked GICC on v2) and enable the timer PPI. The version was
    //    probed + stored by the BSP. The AP's timer itself stays masked (no
    //    scheduling in S3).
    // SAFETY: AP, IRQs masked, MMU up so the GIC device frames are mapped.
    unsafe {
        match crate::gicv3::active_gic() {
            GicVersion::V3 => {
                crate::gicv3::init_ap(idx, crate::gic::TIMER_INTID);
                // P4·SMP·S4c: enable the cross-CPU TLB-shootdown SGI on THIS AP's
                // redistributor so it can receive shootdown IPIs.
                crate::gicv3::enable_ppi_ap(idx, crate::shootdown::SHOOTDOWN_SGI);
                // P4·SMP·S4b: enable the reschedule SGI (INTID 1) on THIS AP's
                // redistributor so a placement/steal can wake it from `wfe`.
                crate::gicv3::enable_ppi_ap(idx, crate::reschedule::RESCHED_SGI);
            }
            GicVersion::V2 => {
                crate::gic::init_ap(crate::gic::TIMER_INTID);
                // P4·SMP·S4c: enable the shootdown SGI in this AP's banked
                // distributor SGI registers.
                crate::gic::init_ap(crate::shootdown::SHOOTDOWN_SGI);
                // P4·SMP·S4b: enable the reschedule SGI (INTID 1) in this AP's
                // banked distributor SGI registers.
                crate::gic::init_ap(crate::reschedule::RESCHED_SGI);
            }
        }
    }

    // 4. Publish the online bit (Release) — this publishes all the AP's per-CPU
    //    init writes to the BSP's Acquire load.
    ONLINE_MASK.fetch_or(1u64 << idx, Ordering::Release);

    // 5. P4·SMP·S4a: enter the AP scheduler. `ap_bootstrap_and_run` waits for the
    //    process model to go live, completes this AP's per-CPU scheduler state
    //    (FP/SIMD + its preemption timer + IRQ unmask), then runs the
    //    idle→schedule loop — picking processes off this CPU's run queue (or the
    //    global Runnable set), dropping to EL0 to run them, and `wfe`-idling on
    //    its periodic tick when it has no work. NEVER returns.
    // SAFETY: this AP's MMU/anchor/GIC/timer-PPI are up and its online bit is
    //    published; `idx` is this CPU's unique logical index.
    unsafe { crate::user::ap_bootstrap_and_run(idx) }
}

// ---------------------------------------------------------------------------
// The hal::smp::Smp seam (implemented on Aarch64; all unsafe is here in Frame)
// ---------------------------------------------------------------------------

/// Bounded spin cap waiting for an AP's online bit (mirrors `gicv3.rs`'s waker
/// guard). Generous for TCG round-robin scheduling; prevents a dead AP from
/// hanging boot.
const AP_ONLINE_GUARD: u64 = 50_000_000;

// `Smp` is sealed (only this workspace's Frame backends may implement it). The
// `Aarch64` struct does not otherwise name `Sealed`, so seal it here next to its
// `Smp` impl.
impl hal::sealed::Sealed for crate::Aarch64 {}

impl hal::smp::Smp for crate::Aarch64 {
    fn cpu_count(&self) -> u32 {
        cpu_count()
    }

    fn start_secondaries(&mut self, _entry: hal::smp::ApEntry) -> Result<u32, hal::ArchError> {
        let topo = enumerate();
        if topo.count <= 1 {
            // 1-vCPU: no AP code, no output (golden byte-identical).
            return Ok(1);
        }

        let entry = ap_start_pa();

        // Bring up each AP in turn (CPU 0 is the BSP, already online).
        for k in 1..topo.count {
            // SAFETY: PSCI CPU_ON to MPIDR `topo.mpidr[k]`, entering `_ap_start`
            // with context_id = k (delivered to the AP in x0).
            let status = unsafe { psci_cpu_on(topo.mpidr[k], entry, k as u64) };
            if status != 0 {
                crate::kprintln!("smp: AP {} CPU_ON failed (psci {})", k, status);
                continue;
            }
            // Bounded-spin until the AP publishes its online bit (single console
            // writer = the BSP, so `cpu k online` cannot interleave with AP UART).
            let mut guard = AP_ONLINE_GUARD;
            while (online_mask() & (1u64 << k)) == 0 && guard > 0 {
                guard -= 1;
                core::hint::spin_loop();
            }
            if (online_mask() & (1u64 << k)) != 0 {
                crate::kprintln!("cpu {} online", k);
            } else {
                crate::kprintln!("smp: AP {} failed to come online", k);
            }
        }

        Ok(online_mask().count_ones())
    }

    fn online_mask(&self) -> u64 {
        online_mask()
    }
}

/// Physical entry address of the AP asm prologue `_ap_start` (identity-mapped,
/// so VA == PA). Used as the PSCI `CPU_ON` entry.
fn ap_start_pa() -> u64 {
    extern "C" {
        fn _ap_start() -> !;
    }
    // Cast through a fn pointer then a thin raw pointer to avoid the
    // `fn_to_numeric_cast` lint (direct `fn item as integer`).
    (_ap_start as unsafe extern "C" fn() -> !) as *const () as u64
}
