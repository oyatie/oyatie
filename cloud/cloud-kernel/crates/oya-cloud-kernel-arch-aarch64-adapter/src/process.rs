//! # Process model for the aarch64 Frame
//!
//! This module turns the previously single-shot EL0 bring-up (`user.rs`) into a
//! real **multi-process** kernel: a process table, per-process address spaces
//! and kernel stacks, a cooperative-plus-timer-preemptive **scheduler**, and the
//! `clone`/`execve`/`wait4`/`exit` family that lets a parent spawn and reap a
//! child — the prerequisite for running `kuberos init`.
//!
//! ## Why this is Frame code
//!
//! Everything here is `unsafe`-bearing: it owns page tables, switches
//! `TTBR0_EL1`, copies whole address spaces frame-by-frame, and `eret`s into
//! EL0. The safe kernel only ever calls the safe [`crate::arch::run_user`]
//! entry; all the dangerous machinery is encapsulated below with a safety note
//! at each site, exactly as the framekernel design requires.
//!
//! ## Address space, per process
//!
//! The single-process loader refined the *kernel's* `L1[0]` in place. That does
//! not generalise: two processes need two different views of the same low VAs.
//! So each [`Process`] owns a full, independent translation hierarchy on the
//! kernel heap:
//!
//! ```text
//!   AddressSpace
//!     l1: Box<PageTable>   // entries 1..3 = kernel identity device/RAM blocks
//!                          // entry 0      = table -> l2  (the user low GiB)
//!     l2: Box<PageTable>   // identity 2 MiB Device blocks, except the
//!                          // USER_NTABLES slots from USER_BASE -> l3[i]
//!     l3: [Box<PageTable>; USER_NTABLES] // 4 x (512 x 4 KiB) = the 8 MiB window
//!     frames: Vec<Box<Frame>>  // the physical pages backing mapped user VAs
//! ```
//!
//! Switching to a process is just `TTBR0_EL1 <- &l1` + a TLB flush. Because the
//! L2 re-creates the kernel's Device identity map for the rest of the low GiB,
//! the EL1 console/GIC/timer keep working no matter which process is current.
//!
//! `fork` shares the parent's backing frames **copy-on-write**: the fresh
//! L1/L2/L3 re-point the child's L3 entries at the *same* physical frames as the
//! parent (held through a shared [`Arc`] refcount), and every writable page is
//! write-protected + tagged COW in **both** spaces. The first write to such a
//! page takes a permission data abort that [`AddressSpace::cow_fault`] services:
//! it allocates a private frame, copies the shared bytes, and re-maps the
//! faulting PTE writable. A frame's storage is freed only when its last [`Arc`]
//! referrer drops (so the child outliving the parent, or vice versa, is safe).
//! `execve` drops the COW mappings wholesale by replacing the whole address
//! space with a fresh one, exactly as before.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

// P4·SMP·S4b: the per-CPU run queues are now `ksync::cl_deque::Deque`s (the
// stealable Chase-Lev work-stealing deques), replacing the S4a
// `PerCpuLocal<VecDeque>` whose single-owner `VecDeque` had no cross-CPU steal.
use hal::cpu::MAX_CPUS;

use aarch64_cpu::asm::barrier;
use aarch64_cpu::registers::{TPIDR_EL0, TTBR0_EL1};
use tock_registers::interfaces::{Readable, Writeable};

use crate::exceptions::TrapFrame;
use crate::user_layout::signal::SignalState;
use crate::user_layout::{
    l2_device_block, l3_slot, table_desc, PagePerm, DESC_ADDR_MASK, DESC_AP_EL0_RO, DESC_UXN,
    DESC_VALID, MMAP_BASE, PAGE_MASK, PAGE_SIZE, PT_ENTRIES, USER_BASE, USER_NTABLES, USER_TOP,
};

// ---- Raw page-table + frame storage ---------------------------------------

/// A 4 KiB-aligned page table (512 x 64-bit descriptors), heap-allocated so each
/// process owns its own copy.
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}

impl PageTable {
    /// A fresh, all-zero (all-invalid) table.
    fn boxed() -> Box<PageTable> {
        Box::new(PageTable { entries: [0; 512] })
    }
}

/// A single 4 KiB physical frame backing a user page, heap-allocated and
/// 4 KiB-aligned so its address is a valid page-table output address.
#[repr(C, align(4096))]
pub struct Frame {
    pub bytes: [u8; PAGE_SIZE],
}

/// A [`Frame`] held behind a share refcount so a copy-on-write `fork` can map it
/// into both parent and child; the storage is freed only when the last referrer
/// drops. The frame's 4 KiB page lives in the inner [`Box`] (exactly `PAGE_SIZE`
/// bytes, page-aligned), so the `Arc` node stays a tiny refcount header rather
/// than over-aligning the whole `Arc` allocation to 4 KiB (which would double the
/// per-frame heap cost).
type SharedFrame = Arc<Box<Frame>>;

impl Frame {
    /// A fresh zeroed frame behind a share refcount (see [`SharedFrame`]).
    fn shared_zeroed() -> SharedFrame {
        Arc::new(Box::new(Frame {
            bytes: [0; PAGE_SIZE],
        }))
    }
}

/// Identity (== physical) address of the [`Frame`] payload behind a
/// [`SharedFrame`]. This is the address stored in the L3 descriptor's output
/// field, so it is the key used to match a descriptor back to its retained
/// frame. (The triple deref takes the address of the `Frame` page itself —
/// `Arc` -> `Box` -> `Frame` — not of any smart-pointer temporary.)
fn frame_pa(frame: &SharedFrame) -> u64 {
    core::ptr::addr_of!(***frame) as u64
}

/// Software-available descriptor bit (VMSAv8-64 reserves bits[58:55] for
/// software use) we set on a **copy-on-write** leaf: the page is mapped
/// EL0-read-only now, but a write fault to it should *copy* rather than fatally
/// abort. Distinguishes a COW page (writable-intent, currently RO) from a
/// genuinely read-only page (relro / rodata), which carries no COW bit and whose
/// write fault stays fatal.
const DESC_COW: u64 = 1 << 55;

/// One-shot guard so the COW-proof serial line is printed only on the *first*
/// serviced copy-on-write fault (proof COW fired) rather than on every fault.
static COW_FAULT_LOGGED: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

// ---- Per-process address space --------------------------------------------

/// A complete, self-contained EL0 address space: the three-level translation
/// hierarchy plus the physical frames it points at. Owned by a [`Process`]; its
/// `l1` physical address is what we load into `TTBR0_EL1`.
pub struct AddressSpace {
    l1: Box<PageTable>,
    // `l2` is never read after construction, but it is load-bearing *storage*:
    // `l1.entries[0]` and the device map point at `l2`'s physical address, so it
    // must stay alive for the life of the space. Keep it owned here.
    #[allow(dead_code)]
    l2: Box<PageTable>,
    // The user window is `USER_NTABLES` consecutive 2 MiB leaf tables (each maps
    // 512 × 4 KiB). `l2` slot `(USER_BASE>>21)+i` walks into `l3[i]`. A flat user
    // VA is split into `(table, entry)` by `l3_slot`.
    l3: [Box<PageTable>; USER_NTABLES],
    /// Backing frames, kept alive for the life of the space. The L3 descriptors
    /// point at `frame.bytes` (identity-aliased at EL1). Held through a share
    /// refcount ([`SharedFrame`]) so a copy-on-write `fork` can share a frame
    /// between parent and child; the storage is freed only when the last referring
    /// space drops its reference.
    frames: Vec<SharedFrame>,
}

impl AddressSpace {
    /// Build an empty address space: device identity map everywhere in the low
    /// GiB, the `USER_BASE` 2 MiB slot refined to a 4 KiB-page table (all pages
    /// initially invalid), and the upper GiBs carrying the kernel identity map
    /// so EL1 keeps working when this space is current.
    pub fn new() -> AddressSpace {
        let mut l1 = PageTable::boxed();
        let mut l2 = PageTable::boxed();
        let l3: [Box<PageTable>; USER_NTABLES] =
            core::array::from_fn(|_| PageTable::boxed());

        // L2: identity 2 MiB Device blocks over 0..1 GiB.
        for (i, e) in l2.entries.iter_mut().enumerate() {
            *e = l2_device_block((i as u64) << 21);
        }
        // The `USER_NTABLES` consecutive 2 MiB slots starting at USER_BASE each
        // become a walk into the matching leaf table, so the window spans
        // `USER_NTABLES * 2 MiB`.
        let base_slot = (USER_BASE >> 21) & 0x1ff;
        for i in 0..USER_NTABLES {
            let l3_pa = core::ptr::addr_of!(*l3[i]) as u64;
            l2.entries[base_slot + i] = table_desc(l3_pa);
        }

        // L1[0] -> our L2 (covers 0..1 GiB, the user low GiB). The other GiBs
        // re-create the kernel identity map (RAM @ 1 GiB, Device @ 2/3 GiB) so
        // EL1 code/data + MMIO keep resolving under this TTBR0.
        let l2_pa = core::ptr::addr_of!(*l2) as u64;
        l1.entries[0] = table_desc(l2_pa);
        l1.entries[1] = kernel_l1_block(0x4000_0000, /* normal */ true);
        l1.entries[2] = kernel_l1_block(0x8000_0000, false);
        l1.entries[3] = kernel_l1_block(0xC000_0000, false);

        AddressSpace {
            l1,
            l2,
            l3,
            frames: Vec::new(),
        }
    }

    /// Physical (== kernel-virtual, identity) address of this space's L1 table,
    /// the value to load into `TTBR0_EL1`.
    pub fn ttbr0(&self) -> u64 {
        core::ptr::addr_of!(*self.l1) as u64
    }

    /// Allocate a fresh zeroed frame, retain it, and return its identity address.
    fn alloc_frame(&mut self) -> u64 {
        let frame = Frame::shared_zeroed();
        let pa = frame_pa(&frame);
        self.frames.push(frame);
        pa
    }

    /// Retain an already-allocated shared frame (a cloned [`SharedFrame`]) in this
    /// space and return its identity address. Used by copy-on-write `fork` to map
    /// the child's L3 at the *same* physical frame the parent points at, bumping
    /// the frame's share refcount so it survives until both drop it.
    fn retain_frame(&mut self, frame: SharedFrame) -> u64 {
        let pa = frame_pa(&frame);
        self.frames.push(frame);
        pa
    }

    /// Find the retained [`SharedFrame`] backing the physical frame at `pa`, if
    /// this space holds it. Linear scan over the (small, <=512-entry) frame list;
    /// used by COW `fork` to clone the share and by the fault to drop a reference.
    fn frame_arc(&self, pa: u64) -> Option<SharedFrame> {
        self.frames.iter().find(|f| frame_pa(f) == pa).cloned()
    }

    /// Drop this space's [`Arc`] to the frame at `pa` (decrement its share
    /// refcount), removing it from the retained list. The storage is freed iff no
    /// other space still references it.
    fn release_frame(&mut self, pa: u64) {
        if let Some(i) = self.frames.iter().position(|f| frame_pa(f) == pa) {
            self.frames.swap_remove(i);
        }
    }

    /// Ensure the page containing `va` is backed by a frame with at least `perm`,
    /// returning the kernel-accessible (identity) base of that page. If already
    /// mapped, re-stamp the permission and keep the frame.
    ///
    /// # Safety
    /// Edits this space's L3 and (when current) flushes the TLB for the VA. The
    /// caller must hold the only mutable reference to this space.
    pub unsafe fn map_page(&mut self, va: usize, perm: PagePerm) -> *mut u8 {
        let page_va = va & !PAGE_MASK;
        assert!(
            (USER_BASE..USER_TOP).contains(&page_va),
            "user va out of window: {:#x}",
            page_va
        );
        let (tbl, idx) = l3_slot(page_va);
        let existing = self.l3[tbl].entries[idx];
        let (pa, eff_perm) = if existing & DESC_VALID != 0 {
            // Already mapped: keep the frame and *union* the permissions so an
            // earlier RX mapping is never silently downgraded to RO when a second
            // ELF segment shares this page (and vice versa).
            (
                existing & DESC_ADDR_MASK,
                perm.merge(PagePerm::from_desc(existing)),
            )
        } else {
            (self.alloc_frame(), perm)
        };
        self.l3[tbl].entries[idx] = eff_perm.desc(pa);

        barrier::dsb(barrier::SY);
        // P4·SMP·S4c: broadcast the invalidation (`tlbi vae1is`) so a re-mapped
        // VA (an already-mapped page whose permission we just changed) is dropped
        // on every CPU, not just this one. A first-time map of a not-present page
        // needs no prior-entry invalidation, but the broadcast is harmless + keeps
        // the maintenance uniform. Local invalidate on 1-vCPU.
        // SAFETY: architected broadcast TLB maintenance for the single VA.
        unsafe {
            core::arch::asm!(
                "tlbi vae1, {x}",
                "dsb sy",
                "isb",
                x = in(reg) (page_va as u64) >> 12,
                options(nostack),
            );
        }
        pa as *mut u8
    }

    /// Translate a mapped user `va` to its kernel-accessible (identity) pointer,
    /// or `None` if the page is not mapped. Lets the kernel read/write another
    /// process's memory (e.g. write `*status` into a `wait4`-blocked parent's
    /// address space while a different process is current).
    pub fn translate(&self, va: usize) -> Option<*mut u8> {
        let page_va = va & !PAGE_MASK;
        if !(USER_BASE..USER_TOP).contains(&page_va) {
            return None;
        }
        let (tbl, idx) = l3_slot(page_va);
        let desc = self.l3[tbl].entries[idx];
        if desc & DESC_VALID == 0 {
            return None;
        }
        let pa = desc & DESC_ADDR_MASK;
        Some((pa + (va & PAGE_MASK) as u64) as *mut u8)
    }

    /// Write `value` (4 bytes, LE) at user `va` in this space via the identity
    /// alias. Returns false if `va` is unmapped. Used to deliver a `wait4`
    /// status into a parent that is not the current process.
    ///
    /// # Safety
    /// `va..va+4` must lie within a single mapped page (true for an aligned i32,
    /// which is how `wait4`'s status pointer is used).
    pub unsafe fn write_u32(&self, va: usize, value: u32) -> bool {
        match self.translate(va) {
            Some(p) => {
                // SAFETY: `p` is the identity alias of a mapped 4 KiB frame; an
                // aligned i32 does not straddle the page.
                unsafe { (p as *mut u32).write_unaligned(value) };
                true
            }
            None => false,
        }
    }

    /// Copy `src` into user memory at `dst_va`, mapping pages on demand with
    /// `perm`. Straddles page boundaries.
    ///
    /// # Safety
    /// `dst_va .. dst_va+src.len()` must lie inside the user window. Writes
    /// through the identity alias of the freshly mapped frames.
    pub unsafe fn copy_to_user(&mut self, dst_va: usize, src: &[u8], perm: PagePerm) {
        let mut off = 0usize;
        while off < src.len() {
            let va = dst_va + off;
            let page_off = va & PAGE_MASK;
            let n = core::cmp::min(PAGE_SIZE - page_off, src.len() - off);
            // SAFETY: maps/ensures the page; returns its identity-aliased base.
            let frame = unsafe { self.map_page(va, perm) };
            // SAFETY: `frame[page_off..page_off+n]` is in-bounds in a 4 KiB frame.
            unsafe {
                core::ptr::copy_nonoverlapping(src[off..off + n].as_ptr(), frame.add(page_off), n);
            }
            off += n;
        }
    }

    /// Copy-on-write clone this address space into a fresh, independent one.
    ///
    /// Same L2/L3 *shape*, but instead of byte-copying every backing frame, the
    /// child **shares** the parent's frames: each mapped child L3 entry points at
    /// the very same physical frame (its [`Arc`] cloned, bumping the share
    /// refcount). Writable (data/stack/heap/RW) pages are additionally
    /// write-protected and tagged [`DESC_COW`] in *both* the parent and the child,
    /// so the first write in either takes a permission abort that
    /// [`AddressSpace::cow_fault`] resolves by copying. Read-only and read+execute
    /// pages are shared as-is: they are never written (W^X keeps code RX), so they
    /// need no COW marker and stay shared for the life of both spaces.
    ///
    /// # Safety
    /// Write-protects the parent's writable leaves in place. The caller must hold
    /// the only mutable reference to `self` and flush the TLB for the parent
    /// (done by [`fork_current`] via the post-fork address-space switch, but we
    /// also invalidate each touched VA here so a still-current parent sees the
    /// new RO mapping immediately).
    pub unsafe fn cow_clone(&mut self) -> AddressSpace {
        let mut child = AddressSpace::new();
        for flat in 0..(USER_NTABLES * PT_ENTRIES) {
            let tbl = flat / PT_ENTRIES;
            let idx = flat % PT_ENTRIES;
            let desc = self.l3[tbl].entries[idx];
            if desc & DESC_VALID == 0 {
                continue;
            }
            let pa = desc & DESC_ADDR_MASK;
            // Clone the parent's Arc for this frame so child + parent share it.
            // Every present leaf in this model is backed by a retained frame
            // (`alloc_frame` is the only path that maps one), so the Arc lookup
            // always succeeds.
            let shared = self
                .frame_arc(pa)
                .expect("mapped L3 leaf without a retained frame");
            child.retain_frame(shared);

            // Writable pages become COW in both spaces: drop EL0 write (RO) and
            // set the COW marker. Non-writable pages (RX text, RO rodata) are
            // shared verbatim — W^X guarantees they are never written.
            if PagePerm::from_desc(desc).is_write() {
                let cow_desc = (desc & !(0b11 << 6)) | DESC_AP_EL0_RO | DESC_COW;
                self.l3[tbl].entries[idx] = cow_desc; // write-protect the parent
                child.l3[tbl].entries[idx] = cow_desc; // child shares it RO+COW
                // Invalidate the parent's stale writable TLB entry for this VA on
                // ALL CPUs. P4·SMP·S4c: the parent may be (or have been) running
                // on another CPU, so a LOCAL `tlbi vae1` is insufficient — use the
                // INNER-SHAREABLE broadcast `tlbi vae1is`, the architected aarch64
                // cross-CPU TLB-maintenance instruction (hardware broadcasts the
                // invalidation to every PE in the inner-shareable domain; QEMU TCG
                // flushes all vCPUs). The `dsb ish` before/after orders the PTE
                // store + the broadcast against other PEs' page-table walks. This
                // is the aarch64 analog of the H1 cross-CPU shootdown; the
                // SGI-based `crate::shootdown` covers the x86-style explicit path
                // + the unmap/free teardown. On 1-vCPU it is a local invalidate.
                let page_va = (USER_BASE + flat * PAGE_SIZE) as u64;
                barrier::dsb(barrier::SY);
                // SAFETY: architected broadcast TLB maintenance for one VA.
                unsafe {
                    core::arch::asm!(
                        "tlbi vae1, {x}",
                        "dsb sy",
                        "isb",
                        x = in(reg) page_va >> 12,
                        options(nostack),
                    );
                }
            } else {
                // Share the non-writable mapping unchanged.
                child.l3[tbl].entries[idx] = (desc & !DESC_ADDR_MASK) | (pa & DESC_ADDR_MASK);
            }
        }
        child
    }

    /// Service a write fault against `va` in this space. Returns `true` if the
    /// page was a [`DESC_COW`] copy-on-write page and the fault was resolved (a
    /// private writable frame now backs `va`); `false` if the page is not COW
    /// (genuine read-only / unmapped), leaving the caller to treat it as fatal.
    ///
    /// On a COW hit: allocate a fresh private frame, copy the shared page's bytes
    /// into it, drop this space's reference to the old (shared) frame, re-map the
    /// L3 entry at the new frame as EL0-read-write with the COW marker cleared,
    /// and invalidate the VA so the faulting instruction re-executes against the
    /// now-writable private page.
    ///
    /// # Safety
    /// This space must be the current `TTBR0` (the faulting EL0 thread's). Edits
    /// the L3 and flushes the TLB for the one VA.
    pub unsafe fn cow_fault(&mut self, va: usize) -> bool {
        let page_va = va & !PAGE_MASK;
        if !(USER_BASE..USER_TOP).contains(&page_va) {
            return false;
        }
        let (tbl, idx) = l3_slot(page_va);
        let desc = self.l3[tbl].entries[idx];
        if desc & DESC_VALID == 0 || desc & DESC_COW == 0 {
            return false;
        }
        let old_pa = desc & DESC_ADDR_MASK;

        // Allocate a fresh private frame and copy the shared page's bytes into it.
        let new_frame = Frame::shared_zeroed();
        let new_pa = frame_pa(&new_frame);
        // SAFETY: both are identity-mapped, 4 KiB, non-overlapping frames; the
        // shared source is still mapped (we have not dropped it yet).
        unsafe {
            core::ptr::copy_nonoverlapping(old_pa as *const u8, new_pa as *mut u8, PAGE_SIZE);
        }
        self.frames.push(new_frame);
        // Drop this space's Arc to the shared frame (refcount--). If a forked
        // sibling still references it, its storage survives for that sibling.
        self.release_frame(old_pa);

        // COW proof: announce the first serviced fault so the serial log shows COW
        // actually fired. One-shot (not per-fault) to avoid flooding; the text does
        // not match the diff-oracle's `[pid N] syscall NR -> RET` regex, so the
        // golden trace is unaffected.
        if !COW_FAULT_LOGGED.swap(true, core::sync::atomic::Ordering::Relaxed) {
            crate::kprintln!("cow: copied page on write fault (va={:#x})", page_va);
        }

        // Re-map writable, COW cleared. The page was writable before the fork, so
        // the private copy is EL0 read-write, PXN+UXN (never executable: COW only
        // applies to W^X data pages).
        self.l3[tbl].entries[idx] = PagePerm::ReadWrite.desc(new_pa);
        barrier::dsb(barrier::SY);
        // P4·SMP·S4c: broadcast the invalidation of the just-re-mapped VA to all
        // CPUs (`tlbi vae1is`). The page was COW-shared, so a sibling may cache
        // the old read-only/old-frame translation; the inner-shareable broadcast
        // drops it everywhere. Local invalidate on 1-vCPU.
        // SAFETY: architected broadcast TLB maintenance for the single re-mapped VA.
        unsafe {
            core::arch::asm!(
                "tlbi vae1, {x}",
                "dsb sy",
                "isb",
                x = in(reg) (page_va as u64) >> 12,
                options(nostack),
            );
        }
        true
    }

    /// P4·SMP·S4c defense-in-depth: handle a lower-EL **instruction-fetch**
    /// permission fault that may be a STALE-TLB artifact. If the PTE backing
    /// `va` is currently valid AND EL0-executable (`DESC_UXN` clear), the fault
    /// was a stale cached translation (e.g. a sibling CPU re-mapped the page and
    /// the shootdown to this CPU arrived late or was missed): invalidate this VA
    /// locally (`tlbi vae1`) and return `true` so the fetch is retried against
    /// the fresh mapping. If the PTE is absent or genuinely non-executable, this
    /// is a real fault — return `false` so the caller delivers the signal/panic.
    ///
    /// # Safety
    /// This space must be the live `TTBR0` of the faulting EL0 thread. Reads the
    /// L3 and, on a stale hit, invalidates one VA.
    pub unsafe fn retry_stale_insn_fetch(&self, va: usize) -> bool {
        let page_va = va & !PAGE_MASK;
        if !(USER_BASE..USER_TOP).contains(&page_va) {
            return false;
        }
        let (tbl, idx) = l3_slot(page_va);
        let desc = self.l3[tbl].entries[idx];
        // Valid + EL0-executable (UXN clear) ⇒ the mapping NOW permits the fetch,
        // so the abort was a stale TLB entry. Anything else is a genuine fault.
        if desc & DESC_VALID == 0 || desc & DESC_UXN != 0 {
            return false;
        }
        barrier::dsb(barrier::SY);
        // SAFETY: architectural TLB maintenance for the single faulting VA.
        unsafe {
            core::arch::asm!(
                "tlbi vae1, {x}",
                "dsb sy",
                "isb",
                x = in(reg) (page_va as u64) >> 12,
                options(nostack),
            );
        }
        true
    }
}

impl Default for AddressSpace {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a 1 GiB kernel L1 **block** descriptor for the identity map at `pa`.
/// `normal` selects Normal-cacheable (RAM) vs Device memory, matching `mmu.rs`.
fn kernel_l1_block(pa: u64, normal: bool) -> u64 {
    // Mirror `mmu.rs::block_attrs`: VALID|BLOCK|AF|SH_INNER|AP_RW + attr index.
    const DESC_AF: u64 = 1 << 10;
    const DESC_SH_INNER: u64 = 0b11 << 8;
    const ATTR_DEVICE_IDX: u64 = 0;
    const ATTR_NORMAL_IDX: u64 = 1;
    let attr = if normal { ATTR_NORMAL_IDX } else { ATTR_DEVICE_IDX };
    pa | DESC_VALID | DESC_AF | DESC_SH_INNER | (attr << 2)
}

// ---- Process state --------------------------------------------------------

/// Scheduling state of a process.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    /// Ready to run (in a run queue, not currently on any CPU).
    Runnable,
    /// Currently executing on some CPU — the "owned by CPU k" marker SMP needs
    /// so two CPUs never pick the same pid. **Unused on 1-vCPU** (the single
    /// core's `current` is implicitly the only running process, exactly as
    /// before); added now so the S4 cross-CPU scheduling diff is purely
    /// additive. Harmless here: no code transitions into it yet, so the golden
    /// round-robin (which only ever inspects `Runnable`) is untouched.
    #[allow(dead_code)]
    Running,
    /// Blocked in `wait4` until a child becomes a zombie.
    Waiting,
    /// Exited; status retained until the parent reaps it.
    Zombie,
}

// ---- Minimal per-process file-descriptor layer (devtmpfs-shaped) ----------

/// The backing kind of an open file description. Deliberately tiny: enough to
/// give fds 0/1/2 a real, Linux-ABI-shaped identity (`/dev/console`) and to
/// support a `/dev/null` sink, without a real filesystem yet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileKind {
    /// Serial-backed console (`/dev/console`): `write` -> the 16550/PL011 UART
    /// via `crate::console::_print`; `read` -> 0 (EOF) for now.
    Console,
    /// The bit bucket (`/dev/null`): `write` discards and reports the full
    /// length written; `read` -> 0 (EOF).
    Null,
    /// An `AF_NETLINK`/`NETLINK_ROUTE` socket (M2 network slice): `socket`
    /// allocates one; `bind` validates a `sockaddr_nl`; `sendto` parses an
    /// `RTM_GETLINK` dump request and arms the response; `recvfrom` drains the
    /// `NLMSG_DONE` dump reply. The wire format lives in `netlink`;
    /// the per-fd response buffer lives on the [`FileDesc`] (`netlink` field).
    Netlink,
}

/// Per-fd state for an `AF_NETLINK` socket: the armed dump RESPONSE bytes (built
/// by `sendto`, drained by `recvfrom`) and the kernel-assigned nonzero port id
/// echoed as `nlmsg_pid`. The buffer is behind a [`ksync::spinlock::SpinLock`]
/// so it is mutable through the shared `Arc<FileDesc>` while keeping `FileDesc:
/// Send + Sync` WITHOUT any `unsafe` in this crate (the single-core trap path is
/// the only accessor; the lock provides the interior mutability safely). The
/// drain cursor reuses the existing `read_off` `AtomicU64` on the `FileDesc`.
pub struct NetlinkFd {
    /// The armed dump response (e.g. a single 16-byte `NLMSG_DONE`). Empty until
    /// `sendto` builds it; `recvfrom` reads from `read_off` to its end.
    pub response: ksync::spinlock::SpinLock<Vec<u8>>,
    /// The nonzero netlink port id assigned at `socket` time, echoed as the
    /// `nlmsg_pid` of dump replies.
    pub port: u32,
}

/// One open file description — the kernel-side object an fd refers to. Shared
/// behind an [`Arc`] so `dup2`/`dup3` and a `clone`d child's fd table all point
/// at the *same* description (Linux semantics: dup'd fds share offset/flags).
/// The struct is intentionally minimal (just the kind for now); an offset/flags
/// field can join it without touching the fd-table plumbing.
pub struct FileDesc {
    pub kind: FileKind,
    /// The backing `vfs` node id, when this fd was opened against a
    /// regular `File` node (Slice 3: `read` serves `node.data`). `None` for the
    /// Console/Null fast paths (fds 0/1/2 and `/dev/null`), so `FileDesc::new`
    /// and `init_std_fds` are byte-identical to the pre-VFS path and the golden
    /// trace is untouched.
    pub node: Option<u32>,
    /// Current byte read offset into the backing `File` node's data, shared
    /// across `dup`'d/`clone`'d fds (Linux open-file-description semantics:
    /// dup'd fds share the offset). An `AtomicU64` so it is mutable through the
    /// shared `Arc` without `unsafe` and keeps `FileDesc: Send + Sync`; the
    /// single-core trap path is the only mutator. Unused for Console/Null (they
    /// return EOF), so the golden path never touches it. For a `Netlink` fd it is
    /// the drain cursor into [`NetlinkFd::response`].
    read_off: core::sync::atomic::AtomicU64,
    /// Per-fd `AF_NETLINK` socket state (the armed dump response + port id), set
    /// only when `kind == FileKind::Netlink`. `None` for every Console/Null/File
    /// fd, so the golden path is byte-identical and the field costs nothing there.
    pub netlink: Option<NetlinkFd>,
}

impl FileDesc {
    /// A fresh open file description of `kind` behind a share refcount, ready to
    /// install in an fd table (used by `init_std_fds` and `openat`). `node` is
    /// `None` (Console/Null fast paths); offset starts at 0.
    pub fn new(kind: FileKind) -> Arc<FileDesc> {
        Arc::new(FileDesc {
            kind,
            node: None,
            read_off: core::sync::atomic::AtomicU64::new(0),
            netlink: None,
        })
    }

    /// A fresh `AF_NETLINK` socket description (M2 network slice). `port` is the
    /// nonzero netlink port id echoed as `nlmsg_pid`; the response buffer starts
    /// empty (armed by `sendto`) and the drain cursor (`read_off`) at 0.
    pub fn netlink(port: u32) -> Arc<FileDesc> {
        Arc::new(FileDesc {
            kind: FileKind::Netlink,
            node: None,
            read_off: core::sync::atomic::AtomicU64::new(0),
            netlink: Some(NetlinkFd {
                response: ksync::spinlock::SpinLock::new(Vec::new()),
                port,
            }),
        })
    }

    /// A fresh open file description backed by a `vfs` `File` node
    /// `id` (Slice 3 reads route through `node.data`). `kind` is the
    /// Console/Null projection the write/read fast paths still match on; the
    /// read offset starts at 0 (a fresh `open`).
    pub fn file(kind: FileKind, id: u32) -> Arc<FileDesc> {
        Arc::new(FileDesc {
            kind,
            node: Some(id),
            read_off: core::sync::atomic::AtomicU64::new(0),
            netlink: None,
        })
    }

    /// The current per-fd read offset (Slice 3 File reads).
    pub fn read_off(&self) -> u64 {
        self.read_off.load(core::sync::atomic::Ordering::Relaxed)
    }

    /// Advance the per-fd read offset by `n` bytes after a successful read.
    pub fn advance_read_off(&self, n: u64) {
        self.read_off
            .fetch_add(n, core::sync::atomic::Ordering::Relaxed);
    }

    /// Reset the per-fd read offset to 0. Used when a netlink `sendto` arms a
    /// fresh dump response so the following `recvfrom` drains it from the head.
    pub fn reset_read_off(&self) {
        self.read_off
            .store(0, core::sync::atomic::Ordering::Relaxed);
    }
}

/// A file handle held in a process's fd table: a refcounted reference to a
/// shared open file description. Cloning a handle (on `dup`/`clone`) bumps the
/// description's refcount; the description is freed when the last referring fd
/// closes.
pub type FileHandle = Arc<FileDesc>;

/// A user process: identity, scheduling state, its saved EL0 register context,
/// its private address space, heap/mmap cursors, and family links.
pub struct Process {
    pub pid: u32,
    pub ppid: u32,
    pub state: State,
    /// The logical CPU currently running this process, or `-1` if it is on no
    /// CPU (Runnable-in-a-queue, Waiting, or Zombie). This is the aarch64 analog
    /// of Linux's `task_struct::on_cpu`: it makes `State::Running` *unambiguous*
    /// about WHICH CPU owns the process, so the scheduler's keep-current fallback
    /// can verify the running CPU is the TRUE owner before resuming `current`.
    /// Without it, a CPU whose stale `current` points at a pid that another CPU
    /// has since claimed (`Running`) would wrongly re-run that pid — the
    /// cross-CPU double-dispatch. Set to the claiming CPU in `pick_and_claim` /
    /// `admit_first`, reset to `-1` the moment the process leaves a CPU (demote,
    /// block, exit). `-1` on 1-vCPU steady state never changes the single core's
    /// decisions (its `current` is always its own), so the golden is unchanged.
    pub owner_cpu: i32,
    /// Encoded exit status (the low 8 bits are the exit code), valid when
    /// `state == Zombie`.
    pub exit_status: i32,
    /// Saved EL0 integer context (x0..x30, SP_EL0 in `sp`, ELR, SPSR). This is a
    /// full [`TrapFrame`]; on a context switch we copy the live trap frame here
    /// and later restore it.
    pub ctx: TrapFrame,
    /// Saved `TPIDR_EL0` (TLS thread pointer) for this process.
    pub tpidr: u64,
    /// The process's private EL0 address space.
    pub space: AddressSpace,
    /// Program break (heap end) and break floor.
    pub brk_cur: usize,
    /// Anonymous-`mmap` bump cursor (grows up from MMAP_BASE).
    pub mmap_cur: usize,
    /// PIDs of this process's children (for wait4 / reaping bookkeeping).
    pub children: Vec<u32>,
    /// PID this process is blocked waiting on (-1 = any child), valid when
    /// `state == Waiting`.
    pub wait_target: i64,
    /// User VA of the `wait4` status pointer to fill on reap (0 = none).
    pub wait_status_ptr: u64,
    /// Per-process file-descriptor table: slot `fd` holds the open file
    /// description fd refers to, or `None` if that fd is free. Lowest-free-fd
    /// allocation. Copied (Arc-sharing the descriptions) on `clone`, preserved
    /// across `execve`, freed when the process is dropped.
    pub fds: Vec<Option<FileHandle>>,
    /// Per-process POSIX signal state: dispositions, the blocked + pending
    /// `Sigset` masks, the `sigaltstack`, and the on-altstack re-entrancy guard.
    /// Initialized all-`SIG_DFL` in `blank`; inherited (copy, then clear the
    /// child's pending + on_altstack) on `fork_current`; reset on `execve`
    /// (handlers to default, blocked mask preserved). All the pure bit/layout
    /// math lives in `user_layout::signal`; this field is just the per-process
    /// storage the syscall layer + delivery hook in `user.rs` operate on.
    pub signals: SignalState,
}

impl Process {
    /// A process freshly loaded from an ELF: takes ownership of its prepared
    /// address space. The loader fills `ctx`/`tpidr`/`brk_cur`/`mmap_cur` after.
    pub fn new_loaded(ppid: u32, space: AddressSpace) -> Process {
        let mut p = Process::blank(0, ppid);
        p.space = space;
        p
    }

    /// A blank process with a zeroed context; fields are filled by the loader or
    /// by `fork`.
    fn blank(pid: u32, ppid: u32) -> Process {
        Process {
            pid,
            ppid,
            state: State::Runnable,
            owner_cpu: -1,
            exit_status: 0,
            ctx: zeroed_frame(),
            tpidr: 0,
            space: AddressSpace::new(),
            brk_cur: 0,
            mmap_cur: MMAP_BASE,
            children: Vec::new(),
            wait_target: -1,
            wait_status_ptr: 0,
            fds: Vec::new(),
            // All dispositions SIG_DFL, nothing blocked/pending, no alt stack.
            signals: SignalState::new(),
        }
    }

    /// Wire up the standard descriptors: fds 0, 1, 2 -> `/dev/console`. Called
    /// at initial process creation and after `execve` rebuilds the image, so a
    /// freshly-loaded program always has stdin/stdout/stderr on the console.
    /// (The three fds share a single console description, matching how a shell
    /// dups one terminal onto 0/1/2.)
    pub fn init_std_fds(&mut self) {
        let console = FileDesc::new(FileKind::Console);
        self.fds.clear();
        self.fds.push(Some(console.clone())); // fd 0 (stdin)
        self.fds.push(Some(console.clone())); // fd 1 (stdout)
        self.fds.push(Some(console)); // fd 2 (stderr)
    }

    /// Look up the open file description an fd refers to, or `None` if the fd is
    /// not open (caller maps that to `-EBADF`).
    pub fn fd_kind(&self, fd: i64) -> Option<FileKind> {
        if fd < 0 {
            return None;
        }
        self.fds
            .get(fd as usize)
            .and_then(|slot| slot.as_ref())
            .map(|h| h.kind)
    }

    /// Clone the shared open file description (the `Arc<FileDesc>`) an fd refers
    /// to, or `None` if the fd is not open. The handle carries the `kind`, the
    /// backing VFS `node`, and the shared read offset, so `read` can serve a
    /// File node's bytes at the per-fd offset.
    pub fn fd_handle(&self, fd: i64) -> Option<FileHandle> {
        if fd < 0 {
            return None;
        }
        self.fds
            .get(fd as usize)
            .and_then(|slot| slot.as_ref())
            .cloned()
    }

    /// Allocate the lowest free fd and install `handle` there, returning the fd.
    pub fn alloc_fd(&mut self, handle: FileHandle) -> i64 {
        for (i, slot) in self.fds.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(handle);
                return i as i64;
            }
        }
        self.fds.push(Some(handle));
        (self.fds.len() - 1) as i64
    }

    /// Close `fd`, returning `true` if it was open (so the caller returns 0) or
    /// `false` if it was already free (`-EBADF`).
    pub fn close_fd(&mut self, fd: i64) -> bool {
        if fd < 0 {
            return false;
        }
        match self.fds.get_mut(fd as usize) {
            Some(slot @ Some(_)) => {
                *slot = None;
                true
            }
            _ => false,
        }
    }

    /// `dup2`/`dup3` core: duplicate `oldfd`'s description into `newfd` (closing
    /// `newfd` first if it was open), so both fds share the same description.
    /// Returns `newfd` on success, or `None` (`-EBADF`) if `oldfd` is not open.
    /// `oldfd == newfd` is a no-op returning `newfd` (matching `dup2`).
    pub fn dup_to(&mut self, oldfd: i64, newfd: i64) -> Option<i64> {
        if oldfd < 0 || newfd < 0 {
            return None;
        }
        let handle = self
            .fds
            .get(oldfd as usize)
            .and_then(|slot| slot.as_ref())
            .cloned()?;
        if oldfd == newfd {
            return Some(newfd);
        }
        let idx = newfd as usize;
        if self.fds.len() <= idx {
            self.fds.resize_with(idx + 1, || None);
        }
        self.fds[idx] = Some(handle);
        Some(newfd)
    }
}

/// A zeroed [`TrapFrame`]. (`TrapFrame` has no `Default`; build one explicitly.)
fn zeroed_frame() -> TrapFrame {
    TrapFrame {
        regs: [0; 31],
        sp: 0,
        elr: 0,
        spsr: 0,
    }
}

/// A zeroed [`TrapFrame`], for the AP scheduler's throwaway bootstrap "from"
/// frame (P4·SMP·S4a). Public wrapper around [`zeroed_frame`] so `user.rs` (the
/// ring-transition home) can build the AP's initial frame.
pub(crate) fn zeroed_trapframe() -> TrapFrame {
    zeroed_frame()
}

// ---- The process table + scheduler ----------------------------------------

/// The **global** process table (S0 split): pid allocation, the per-pid process
/// slots, and the wait/reap parent-child relationships all live in the slots.
/// This is the part that stays a single global owner — to be moved behind a
/// `ksync::SpinLock` in S4. **Not locked yet** (still single-CPU: `cpu_count`
/// is 1, only the boot core touches it), so the existing single-core borrow
/// discipline still holds.
struct ProcTable {
    /// Slot `i` holds the process whose pid is `i` (slot 0 unused; pids start at
    /// 1). `None` = free slot.
    table: Vec<Option<Process>>,
    /// Next pid to hand out.
    next_pid: u32,
}

impl ProcTable {
    const fn new() -> ProcTable {
        ProcTable {
            table: Vec::new(),
            next_pid: 1,
        }
    }

    /// Allocate a pid + table slot for `proc`, returning the pid.
    fn insert(&mut self, mut proc: Process) -> u32 {
        let pid = self.next_pid;
        self.next_pid += 1;
        proc.pid = pid;
        // Grow the table so `table[pid]` exists.
        while self.table.len() <= pid as usize {
            self.table.push(None);
        }
        self.table[pid as usize] = Some(proc);
        pid
    }

    fn get(&self, pid: u32) -> Option<&Process> {
        self.table.get(pid as usize).and_then(|s| s.as_ref())
    }

    fn get_mut(&mut self, pid: u32) -> Option<&mut Process> {
        self.table.get_mut(pid as usize).and_then(|s| s.as_mut())
    }
}

/// The **per-CPU** scheduler run state (S0 split): which pid this CPU is
/// running. In S1 the actual storage moved off this struct into the per-CPU
/// [`CURRENT`] (`hal::cpu::PerCpu<u32>`); `LocalSched` keeps the round-robin
/// `pick_next` policy that operates on `(ProcTable, current)`. Destined to also
/// hold this CPU's run-queue handle in S4.
struct LocalSched;

impl LocalSched {
    /// The **keep-current** fallback (P4·SMP·S4d fix). When this CPU's run queue
    /// is empty, the only process it may resume WITHOUT going through the queue is
    /// its OWN `current`: if `current` is still `Running` (this CPU owns it) or
    /// `Runnable`, keep running it; otherwise this CPU has nothing to do.
    ///
    /// ## Why this no longer scans the whole table (the cross-CPU double-run fix)
    ///
    /// The pre-fix version round-robin-scanned the ENTIRE table for any
    /// `State::Runnable` pid. That made the global scan a SECOND, non-exclusive
    /// claim source competing with the per-CPU run queues: a freshly-woken process
    /// (e.g. a `wait4` parent woken by `complete_waits`) is BOTH marked `Runnable`
    /// AND `runq_push`ed onto one CPU's queue, so two *different* idle CPUs could
    /// each independently find it via their global scan and both `pick_and_claim`
    /// it — running one pid on two CPUs (its single user stack corrupts; the boot
    /// then aborts to a stack address). `pick_and_claim` marking it `Running` does
    /// NOT serialize them: the queue entry and the global scan are two sources, and
    /// the loser of the claim race still saw `Runnable` a moment earlier.
    ///
    /// Every `Runnable` producer in this module ALSO enqueues the pid on exactly
    /// one CPU's run queue (`fork_current`, the preempt demote in `pick_and_claim`,
    /// and `complete_waits`'s wake), and `runq_pop` removes the entry under the
    /// `PROCS` lock — so the queue is already a strictly-exclusive single-claim
    /// path. Restricting the fallback to ONLY `current` removes the duplicate
    /// source, making dispatch exclusive by construction. On 1-vCPU this is
    /// byte-identical: cpu 0's queue carries every fork child / woken parent (FIFO
    /// == the old round-robin order), and the keep-current clause covers the
    /// lone-runnable-process case exactly as before.
    fn pick_next(procs: &ProcTable, current: u32, cpu: usize) -> Option<u32> {
        // 1) Keep-current (owner-checked): resume OUR own process without a scan.
        match procs.get(current) {
            // Keep running OUR own process: it is `Running` and WE are its owner.
            // The `owner_cpu == cpu` guard is the crux of the double-dispatch fix:
            // a stale `current` that points at a pid which ANOTHER cpu has since
            // claimed (so it is `Running` but owned elsewhere) is NOT keepable —
            // without this guard, this cpu would re-run a pid live on another cpu.
            Some(p) if p.state == State::Running && p.owner_cpu == cpu as i32 => return Some(current),
            // A `Runnable` `current` that is on NO cpu (owner_cpu == -1) is the
            // lone-process steady state (e.g. the BSP's pid 1 before its first
            // claim, or after a self-preempt that re-picked itself): claiming it
            // below sets ownership, so it is exclusive. A `Runnable` pid OWNED by
            // a cpu cannot occur (ownership is cleared whenever state leaves
            // `Running`), but the `-1` guard makes the intent explicit + safe.
            Some(p) if p.state == State::Runnable && p.owner_cpu < 0 => return Some(current),
            _ => {}
        }
        // 2) Owner-exclusive GLOBAL round-robin scan (liveness backstop). A pid
        // queued on an idle/other CPU's run queue, or woken (`Runnable`) while its
        // would-be CPU is parked, must still make progress — otherwise it STARVES
        // (the per-CPU-queue-only design hangs the 8-worker demo ~8/10). ANY cpu
        // may claim a `Runnable` + UNOWNED (`owner_cpu < 0`) pid. This is EXCLUSIVE,
        // not a double-dispatch source, because the whole scan + `pick_and_claim`'s
        // ownership stamp run under the SAME `PROCS` lock: a second cpu's scan sees
        // it `Running`+owned and skips it, and a duplicate run-queue entry fails
        // `runq_pop`'s `Runnable` re-validation (the pid is `Running` by then). On
        // 1-vCPU only cpu 0 ever scans and its own queue drains first, so the trace
        // is unchanged. Round-robin from `current + 1` for fairness; slot 0 unused.
        let len = procs.table.len();
        for i in 1..len {
            let pid = (current as usize + i) % len;
            if pid == 0 {
                continue;
            }
            if let Some(p) = procs.get(pid as u32) {
                if p.state == State::Runnable && p.owner_cpu < 0 {
                    return Some(pid as u32);
                }
            }
        }
        None
    }
}

/// The combined scheduler view handed to `with_sched`'s closure: a `&mut` to the
/// global [`ProcTable`] plus a working snapshot of **this CPU's** `current` pid
/// (loaded from the per-CPU [`CURRENT`] on entry, written back on exit). Keeping
/// the `current` field name + the `get/get_mut/insert/pick_next` method surface
/// means every existing `s.current` / `s.get(..)` / `s.insert(..)` call site is
/// byte-identical after the split.
pub struct Scheduler<'a> {
    procs: &'a mut ProcTable,
    /// This CPU's running pid — a snapshot of `CURRENT.get(&token)`. On 1-vCPU
    /// (index always 0) this is exactly the old single global `current`.
    current: u32,
    /// This CPU's logical index (from the `CpuToken` minted in `with_sched`),
    /// routing run-queue ops to THIS CPU's `RUNQ` slot. Always 0 on 1-vCPU.
    cpu: usize,
}

impl Scheduler<'_> {
    /// Allocate a pid + table slot (forwards to the global [`ProcTable`]).
    fn insert(&mut self, proc: Process) -> u32 {
        self.procs.insert(proc)
    }

    fn get(&self, pid: u32) -> Option<&Process> {
        self.procs.get(pid)
    }

    fn get_mut(&mut self, pid: u32) -> Option<&mut Process> {
        self.procs.get_mut(pid)
    }

    /// Pick the next pid to run on THIS CPU (P4·SMP·S4a + S4b steal). First drains
    /// this CPU's own run-queue deque (re-validating each popped pid is still
    /// `Runnable` in the locked table — a stale entry may have exited/blocked/
    /// migrated); if empty, **S4b** STEALS from a victim CPU's deque top
    /// (`runq_steal`, re-validating likewise); if a steal also yields nothing,
    /// falls back to the keep-current + global round-robin scan (byte-identical to
    /// S4a/pre-S4b on 1-vCPU, where the deque is empty and there is no victim). The
    /// `State::Running` re-validation makes every source safe: a pid another CPU
    /// runs is `Running`, so neither a pop, a steal, nor a scan double-selects it.
    fn pick_next(&mut self) -> Option<u32> {
        // 1) Own deque.
        while let Some(pid) = runq_pop(self.cpu) {
            if matches!(self.procs.get(pid), Some(p) if p.state == State::Runnable) {
                return Some(pid);
            }
        }
        // 2) S4b: steal from a victim CPU's deque top (no-op on 1-vCPU).
        while let Some(pid) = runq_steal(self.cpu) {
            if matches!(self.procs.get(pid), Some(p) if p.state == State::Runnable) {
                return Some(pid);
            }
        }
        // 3) Keep-current + global round-robin fallback.
        LocalSched::pick_next(self.procs, self.current, self.cpu)
    }

    /// Pick the next pid AND atomically CLAIM it for this CPU — all under the one
    /// `PROCS` lock held by the enclosing `with_sched` (P4·SMP·S4a). Marks the
    /// candidate `Running` AND stamps `owner_cpu = THIS cpu` BEFORE the lock is
    /// released, so a concurrent `pick` on another CPU skips it (no cross-CPU
    /// double-run): `Running` + the owner stamp is what makes the claim exclusive.
    /// The outgoing `current`, if still active and different, is demoted to
    /// `Runnable`, its ownership cleared (`owner_cpu = -1`), and it is re-enqueued
    /// on this CPU's run queue. Returns the claimed pid.
    fn pick_and_claim(&mut self) -> Option<u32> {
        let cpu = self.cpu;
        let pid = self.pick_next()?;
        let outgoing = self.current;
        if outgoing != pid {
            if matches!(self.get(outgoing), Some(p) if matches!(p.state, State::Running | State::Runnable))
            {
                if let Some(p) = self.get_mut(outgoing) {
                    p.state = State::Runnable;
                    // Outgoing leaves this CPU — drop its ownership so it can be
                    // claimed exclusively wherever it is next picked.
                    p.owner_cpu = -1;
                }
                runq_push(self.cpu, outgoing);
            }
        }
        self.current = pid;
        if let Some(p) = self.get_mut(pid) {
            p.state = State::Running;
            // Stamp ownership: from here until this pid leaves a CPU, `owner_cpu`
            // names the single CPU running it, so no other CPU's keep-current or
            // re-validation can resume it.
            p.owner_cpu = cpu as i32;
        }
        sched_log_first_run(pid, cpu);
        Some(pid)
    }
}

/// The one global process table, now (P4·SMP·S4a) behind a `ksync::SpinLock`.
///
/// **Lock-ordering invariant (enforced by review):** `PROCS` is the *only*
/// kernel lock taken inside [`with_sched`]. Never call `with_sched` while holding
/// any other lock, and never acquire another lock inside the `with_sched`
/// closure — so no ABBA deadlock is representable by construction.
///
/// It is taken `lock_irqsave::<Aarch64Irq>()` (NOT plain `lock()`) because
/// `with_sched` runs from BOTH the SVC syscall path AND the timer-IRQ preempt
/// path: a same-CPU timer IRQ re-entering a held plain-`lock()` would self-
/// deadlock on its own ticket (exactly H2). Masking DAIF.I first closes that
/// window. The guard is dropped at the closure end — the lock is NEVER held
/// across the `TTBR0_EL1.set` + `eret` in [`switch_to`].
static PROCS: ksync::spinlock::SpinLock<ProcTable> =
    ksync::spinlock::SpinLock::new(ProcTable::new());

/// Per-CPU run queues: one bounded Chase-Lev work-stealing deque of `Runnable`
/// pids per logical CPU (P4·SMP·S4b). The owning CPU pushes/pops the bottom of
/// its own deque; an idle CPU whose own deque is empty STEALS from a victim CPU's
/// top. The S4a `PerCpuLocal<VecDeque>` is replaced here by `ksync::cl_deque`
/// because the deque's `steal` is the lock-free cross-CPU rebalancing path the
/// `VecDeque` could not offer. The deque is `Sync` (all methods `&self`; H4 loom-
/// proved), so its slots are reached by shared `&` and the per-CPU disjointness of
/// `push`/`pop` is a discipline (each CPU pushes/pops its OWN index). On 1-vCPU
/// only slot 0 is touched and the deque degenerates to a single-owner push/pop =
/// the S4a FIFO, so `pick_next` falls through to the byte-identical global scan and
/// no steal/IPI ever fires. See the x86 mirror for the full note.
struct RunQueues {
    deques: [ksync::cl_deque::Deque; MAX_CPUS],
}

impl RunQueues {
    const fn new() -> Self {
        Self { deques: [const { ksync::cl_deque::Deque::new() }; MAX_CPUS] }
    }
}

/// The per-CPU run queues. A plain `static` (the deque is `Sync`); no `static mut`
/// and no `PerCpuLocal` wrapper needed because every method is `&self`.
static RUNQ: RunQueues = RunQueues::new();

/// Round-robin fork-placement cursor over the online CPUs. `Relaxed` — a
/// placement *hint*, not a correctness datum (the `State`/table under `PROCS` is
/// the source of truth).
static PLACE_CURSOR: AtomicU32 = AtomicU32::new(0);

/// Count of run-queue pushes that overflowed the bounded deque (spilled to the
/// global fallback). `Relaxed` — a diagnostic counter, not a correctness datum.
static RUNQ_OVERFLOW: AtomicU32 = AtomicU32::new(0);

/// Push `pid` onto CPU `cpu`'s run-queue deque (the owner's bottom). Caller holds
/// the `PROCS` lock, which serializes all run-queue access. A full deque refuses
/// the push and the pid spills to the global fallback (the locked round-robin
/// scan picks it up — it is still `Runnable` + unowned), so no pid is ever lost.
/// If the work was placed on a DIFFERENT, possibly-idle CPU, wake it with a
/// reschedule SGI (a no-op on 1-vCPU / same-CPU; fire-and-forget, no ack).
fn runq_push(cpu: usize, pid: u32) {
    if !RUNQ.deques[cpu].push(pid) {
        RUNQ_OVERFLOW.fetch_add(1, Ordering::Relaxed);
    }
    // P4·SMP·S4b: the pid is published to the deque above (and we hold `PROCS`),
    // so a CPU woken by this SGI observes it. No-op on 1-vCPU / same-CPU.
    crate::reschedule::notify(cpu);
}

/// Pop the next ready pid from CPU `cpu`'s OWN run-queue deque (the owner's
/// bottom), if any. Caller holds the `PROCS` lock. Owner-only.
fn runq_pop(cpu: usize) -> Option<u32> {
    RUNQ.deques[cpu].pop()
}

/// STEAL: an idle CPU `thief` whose own deque is empty takes a pid from a VICTIM
/// CPU's deque top (P4·SMP·S4b). Tries each online CPU other than `thief`
/// (starting one above it, wrapping); a contended deque (`Steal::Retry`) moves on
/// to the next victim. Returns `Some(pid)` on success (the caller re-validates it
/// is still `Runnable` under the `PROCS` lock); `None` when every victim is empty.
/// No-op on 1-vCPU (no victims), keeping the golden byte-identical.
fn runq_steal(thief: usize) -> Option<u32> {
    use ksync::cl_deque::Steal;
    let mask = crate::smp::online_mask();
    if mask.count_ones() <= 1 {
        return None;
    }
    for off in 1..MAX_CPUS {
        let victim = (thief + off) % MAX_CPUS;
        if victim == thief || mask & (1u64 << victim) == 0 {
            continue;
        }
        match RUNQ.deques[victim].steal() {
            Steal::Success(pid) => {
                steal_log(pid, victim, thief);
                return Some(pid);
            }
            Steal::Retry | Steal::Empty => {}
        }
    }
    None
}

/// P4·SMP·S4a cross-CPU scheduling DEMO instrumentation (feature-gated). One bit
/// per pid (`< 64`) recording whether pid's first run on a CPU was announced, so
/// the `sched: pid P -> cpu K` line prints once per worker. Compiled in ONLY
/// under `smp-sched-demo`; the default golden serial is byte-identical.
#[cfg(feature = "smp-sched-demo")]
static SCHED_SEEN: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Announce (once per pid) that pid `pid` is being run on CPU `cpu`. Under
/// `-smp 4` these lines show workers on multiple distinct `cpu` indices. No-op
/// unless the demo feature is enabled.
#[inline]
fn sched_log_first_run(pid: u32, cpu: usize) {
    #[cfg(feature = "smp-sched-demo")]
    {
        if pid < 64 {
            let bit = 1u64 << pid;
            if SCHED_SEEN.fetch_or(bit, Ordering::Relaxed) & bit == 0 {
                crate::kprintln!("sched: pid {} -> cpu {}", pid, cpu);
            }
        }
    }
    #[cfg(not(feature = "smp-sched-demo"))]
    {
        let _ = (pid, cpu);
    }
}

/// Announce a successful work-steal: `thief` stole `pid` from `victim`'s deque
/// (P4·SMP·S4b). The **stolen-execution marker** the S4b gate asserts: `pid` was
/// PLACED on `victim` but is RUN by `thief != victim`, proving the deque steal
/// rebalanced load. Compiled in ONLY under `smp-sched-demo` (golden byte-
/// identical), and a no-op on 1-vCPU where `runq_steal` never runs.
#[inline]
fn steal_log(pid: u32, victim: usize, thief: usize) {
    #[cfg(feature = "smp-sched-demo")]
    {
        crate::kprintln!("steal: pid {} from cpu {} by cpu {}", pid, victim, thief);
    }
    #[cfg(not(feature = "smp-sched-demo"))]
    {
        let _ = (pid, victim, thief);
    }
}

/// Pick the next online CPU index round-robin over `online_mask`, for fork
/// placement. Always returns a set bit (bit 0/BSP is always set).
fn next_place_cpu() -> usize {
    let mask = crate::smp::online_mask();
    let start = PLACE_CURSOR.fetch_add(1, Ordering::Relaxed).wrapping_add(1) as usize;
    for off in 0..MAX_CPUS {
        let cand = (start + off) % MAX_CPUS;
        if mask & (1u64 << cand) != 0 {
            return cand;
        }
    }
    0
}

/// This CPU's running pid (S1: the per-CPU half of the old `Scheduler.current`).
/// `hal::cpu::PerCpuU32` is a `[AtomicU32; MAX_CPUS]` indexed by the running CPU
/// via a `CpuToken`. On 1-vCPU only slot 0 is ever touched, so round-robin is
/// byte-identical to the old single global field.
///
/// **S2:** a plain `static` (not `static mut`) — `PerCpuU32` has interior
/// mutability (`AtomicU32`), so `load`/`store` take `&self`. This *removes* the
/// `addr_of_mut!(CURRENT)).get_mut(&token)` whole-array `&mut` that was the
/// pre-SMP aliasing UB: there is no `static mut` to take an `&mut` of any more.
static CURRENT: hal::cpu::PerCpuU32 = hal::cpu::PerCpuU32::new();

/// Run `f` with `&mut Scheduler`.
///
/// # Safety
/// Single-core: the trap path is the only caller and never re-enters while a
/// borrow is live (IRQs are masked at EL1 during trap handling). The minted
/// [`hal::cpu::CpuToken`] indexes this CPU's [`CURRENT`] slot; under that same
/// IRQs-masked invariant we cannot migrate, so the index is stable across the
/// borrow. The `current` snapshot is read from `CURRENT` before `f` and written
/// back after, so `s.current = …` inside `f` updates the per-CPU slot exactly
/// as the old in-struct field did.
pub fn with_sched<R>(f: impl FnOnce(&mut Scheduler) -> R) -> R {
    // SAFETY: IRQs masked at EL1 on the trap path → no migration, so
    // `this_cpu_token` mints a token pinned to this CPU for the whole borrow.
    let token = unsafe { crate::percpu::this_cpu_token() };
    // P4·SMP·S4a: take the GLOBAL ProcTable lock IRQ-SAFE for the critical
    // section only. `lock_irqsave::<Aarch64Irq>()` masks DAIF.I FIRST then takes
    // the ticket, so a same-CPU timer IRQ cannot re-enter and self-deadlock (H2).
    // On 1-vCPU the lock is uncontended and IRQs are already masked on the trap
    // path, so `Aarch64Irq::disable` returns `was_unmasked=false` and the restore
    // is a no-op → no observable change vs the old `static mut` borrow.
    let mut guard = PROCS.lock_irqsave::<crate::arch::Aarch64Irq>();
    // Relaxed atomic load: per-CPU, nothing published through it (token proves
    // we are this CPU). No `unsafe` — `PerCpuU32::load` is `&self`.
    let current = CURRENT.load(&token);
    let mut sched = Scheduler {
        procs: &mut guard,
        current,
        cpu: token.cpu_index(),
    };
    let out = f(&mut sched);
    // Write this CPU's (possibly mutated) `current` back to its per-CPU slot.
    // `&self` atomic store — never forms an `&mut` of the array (was the UB).
    CURRENT.store(&token, sched.current);
    out
    // `guard` drops HERE: `SpinGuardIrq` releases the lock THEN restores DAIF.I.
    // The lock is gone before any caller proceeds to a TTBR0 write / eret.
}

/// The one global in-RAM VFS (single namespace, M2). Frame-owned, lazily built
/// on first use (`Vfs::new()` pre-populates `/`, `/dev`, `/dev/console`,
/// `/dev/null`). Like `SCHED` it is accessed only from the single-core trap
/// path. The `Vfs` type + all its logic (tree, walker, mount table) live in the
/// 0-unsafe `vfs`; only this storage + accessor are `unsafe`, and
/// they live here in the Frame (TCB), not the forbid-set.
static mut VFS: Option<crate::vfs::Vfs> = None;

/// Run `f` with `&mut Vfs`, building it on first use.
///
/// # Safety
/// Single-core: the trap path is the only caller and never re-enters while a
/// borrow is live (IRQs are masked at EL1 during trap handling) — identical
/// justification to [`with_sched`].
pub fn with_vfs<R>(f: impl FnOnce(&mut crate::vfs::Vfs) -> R) -> R {
    // SAFETY: single-core, non-reentrant access to the Frame-owned VFS.
    let slot = unsafe { &mut *core::ptr::addr_of_mut!(VFS) };
    f(slot.get_or_insert_with(crate::vfs::Vfs::new))
}

// ---- Public scheduler operations the syscall layer / IRQ path call --------

impl Scheduler<'_> {
    /// Register a freshly-loaded first process and make it current. Returns its
    /// pid. The caller has already populated `ctx`/`space`/`tpidr`.
    ///
    /// P4·SMP·S4a: mark it `Running` (owned by THIS CPU, the BSP) and make it this
    /// CPU's `current`. The BSP drops to EL0 directly via `enter_el0` — NOT through
    /// `switch_to` — and it is NOT placed on any run queue, so it is resumed only
    /// via the keep-current fallback (`LocalSched::pick_next`), which needs
    /// `current == pid` and that pid `Running`/`Runnable`. An AP never sees it: the
    /// fallback only ever returns a CPU's OWN `current`, and pid 1 is on no run
    /// queue for an AP to pop. On 1-vCPU no AP runs, so this is invisible to the
    /// golden.
    pub fn admit_first(&mut self, proc: Process) -> u32 {
        let pid = self.insert(proc);
        self.current = pid;
        let cpu = self.cpu;
        if let Some(p) = self.get_mut(pid) {
            p.state = State::Running;
            // The BSP owns pid 1 from the moment it drops to EL0; stamp it so the
            // keep-current fallback resumes it (and no AP can).
            p.owner_cpu = cpu as i32;
        }
        pid
    }

    /// The currently running pid.
    pub fn current_pid(&self) -> u32 {
        self.current
    }

    /// Borrow the current process.
    pub fn current(&mut self) -> &mut Process {
        let cur = self.current;
        self.get_mut(cur).expect("current process missing")
    }

    /// Block the current process in `wait4` (P4·SMP·S4d): save its FULL EL0 context
    /// (the trap `frame` + the live `SP_EL0`/`TPIDR_EL0` the caller read) into its
    /// `ctx`, then flip it to `Waiting` and record the wait target/status pointer —
    /// ALL under the single held `PROCS` lock, so the save is published BEFORE the
    /// process becomes wakeable. This closes the `save_current`-vs-`complete_waits`
    /// race: previously the blocking process was marked `Waiting` here and its
    /// context saved LATER (a second, post-lock `save_current`), so a sibling CPU's
    /// `complete_waits` could write the wake's `x0` (reaped pid) into `ctx` in the
    /// gap, only for the late `save_current` to clobber it back to the pre-block
    /// frame — corrupting the `wait4` return (premature/garbage ECHILD). Saving
    /// here, atomically, means `complete_waits`'s `ctx.regs[0] = cpid` always wins.
    /// On 1-vCPU there is no sibling to race, and the saved context is byte-for-byte
    /// what the old `save_current` produced, so the golden trace is unchanged.
    pub fn block_current_for_wait(
        &mut self,
        frame: &TrapFrame,
        sp_el0: u64,
        tpidr: u64,
        wait_target: i64,
        wait_status_ptr: u64,
    ) {
        let cur = self.current();
        cur.ctx = clone_frame(frame);
        cur.ctx.sp = sp_el0;
        cur.tpidr = tpidr;
        cur.state = State::Waiting;
        // Blocking leaves this CPU: drop ownership so the keep-current fallback on
        // this CPU cannot resume a now-Waiting process, and so the eventual wake
        // re-claims it exclusively wherever it is next picked.
        cur.owner_cpu = -1;
        cur.wait_target = wait_target;
        cur.wait_status_ptr = wait_status_ptr;
    }

    /// Try to service a copy-on-write write fault at `va` against the current
    /// process. Returns `true` if `va` was a COW page and a private writable copy
    /// now backs it (the faulting instruction may be retried); `false` if it was
    /// not a COW page, so the caller treats the fault as fatal.
    ///
    /// # Safety
    /// The current process's space must be the live `TTBR0` (true on an EL0 data
    /// abort). Edits the L3 + flushes the TLB for the one VA.
    pub unsafe fn try_cow_fault(&mut self, va: usize) -> bool {
        let cur = self.current;
        match self.get_mut(cur) {
            // SAFETY: `cur` is the current process; its space is the live TTBR0.
            Some(p) => unsafe { p.space.cow_fault(va) },
            None => false,
        }
    }

    /// P4·SMP·S4c: handle a possibly-stale EL0 instruction-fetch permission fault
    /// against the current process (defense-in-depth — see
    /// [`AddressSpace::retry_stale_insn_fetch`]). Returns `true` if the PTE was
    /// valid+executable (a stale TLB entry, now invalidated → retry the fetch);
    /// `false` if it is a genuine non-executable/absent mapping.
    ///
    /// # Safety
    /// The current process's space must be the live `TTBR0` (true on an EL0
    /// instruction abort). Reads the L3 + may invalidate one VA.
    pub unsafe fn try_retry_stale_insn_fetch(&mut self, va: usize) -> bool {
        let cur = self.current;
        match self.get(cur) {
            // SAFETY: `cur` is the current process; its space is the live TTBR0.
            Some(p) => unsafe { p.space.retry_stale_insn_fetch(va) },
            None => false,
        }
    }

    /// Fork the current process: create a child with a **copy-on-write** clone of
    /// the parent's address space (shared frames, writable pages write-protected +
    /// COW-tagged in both) and a duplicated context whose `x0` is 0 (the child's
    /// return). `sp_el0` is the parent's live EL0 stack pointer (the trap stub
    /// does not stash it, so the caller reads it from the register and passes it
    /// in); the child resumes on the same SP, COW-sharing the parent's stack page
    /// until either side writes it. Returns the child's pid (the parent's
    /// `clone`/`fork` return value).
    pub fn fork_current(&mut self, parent_frame: &TrapFrame, sp_el0: u64) -> u32 {
        let parent_pid = self.current;
        let (child_space, tpidr, brk, mmap, fds, mut signals) = {
            let parent = self.get_mut(parent_pid).expect("parent missing");
            // SAFETY: the parent is the current process, so its space is the live
            // TTBR0; `cow_clone` write-protects its writable leaves + invalidates
            // their TLB entries so the parent immediately sees the COW mapping.
            let child_space = unsafe { parent.space.cow_clone() };
            // Copy the parent's fd table: the child gets its own table whose
            // slots Arc-share the parent's open file descriptions (fork dups the
            // fds, sharing the descriptions, exactly as Linux does).
            let fds = parent.fds.clone();
            // Inherit the parent's signal dispositions + blocked mask (POSIX
            // fork semantics); the pending set and the on-altstack guard are
            // cleared below for the child.
            let signals = parent.signals;
            (
                child_space,
                parent.tpidr,
                parent.brk_cur,
                parent.mmap_cur,
                fds,
                signals,
            )
        };
        // Child starts with no pending signals and not running on an altstack.
        signals.pending = 0;
        signals.on_altstack = false;

        let mut child = Process::blank(0, parent_pid);
        child.space = child_space;
        child.tpidr = tpidr;
        child.brk_cur = brk;
        child.mmap_cur = mmap;
        child.fds = fds;
        child.signals = signals;
        // The child resumes exactly where the parent's SVC will return, but with
        // x0 = 0 so userspace sees the fork() child return value, and on the
        // parent's live SP_EL0 (now backed by the child's own copied stack page).
        child.ctx = clone_frame(parent_frame);
        child.ctx.sp = sp_el0;
        child.ctx.regs[0] = 0;
        child.state = State::Runnable;

        let child_pid = self.insert(child);
        if let Some(parent) = self.get_mut(parent_pid) {
            parent.children.push(child_pid);
        }
        // P4·SMP·S4a placement: assign the child round-robin over the online mask
        // and enqueue it on that CPU's local run queue. On 1-vCPU the target is
        // always CPU 0 = this CPU, so `pick_next` pops it locally; since 1-vCPU
        // never has two concurrent Runnable children, FIFO == the old round-robin
        // (golden byte-identical, gated). Under -smp the cursor spreads workers.
        let target = next_place_cpu();
        runq_push(target, child_pid);
        child_pid
    }

    /// Mark the current process a zombie with `status`. Waking + reaping a
    /// blocked parent is done separately by [`complete_waits`], which delivers
    /// the result into the parent's context; doing it here would prematurely
    /// flip the parent to `Runnable` and lose the reap.
    pub fn exit_current(&mut self, status: i32) {
        let pid = self.current;
        if let Some(p) = self.get_mut(pid) {
            p.state = State::Zombie;
            p.owner_cpu = -1; // off-CPU now (Zombie); no owner.
            // Linux encodes a normal exit as (code & 0xff) << 8.
            p.exit_status = (status & 0xff) << 8;
        }
    }

    /// Terminate the current process because of an unhandled, fatal signal
    /// `sig` (default action == Terminate). Mirrors [`exit_current`] but encodes
    /// the status in **WIFSIGNALED** form: the low 7 bits are the signal number
    /// and bit 7 (core-dumped) is 0 — vs a normal exit's `(code & 0xff) << 8`.
    /// The caller runs the same epilogue as `sys_exit` (complete_waits / switch
    /// / drop reaped).
    pub fn terminate_current_by_signal(&mut self, sig: u32) {
        let pid = self.current;
        if let Some(p) = self.get_mut(pid) {
            p.state = State::Zombie;
            p.owner_cpu = -1; // off-CPU now (Zombie); no owner.
            p.exit_status = (sig & 0x7f) as i32;
        }
    }

    /// Raise signal `sig` on the process with pid `pid` by OR-ing its pending
    /// bit. **Bit-only**: it performs NO print and NO state change — it must not
    /// flip a `Waiting` parent to `Runnable` (that is `complete_waits`'s job),
    /// because the SIGCHLD post on every child exit must stay invisible to the
    /// golden trace (no new line, no scheduling perturbation). A no-op if `pid`
    /// is absent (e.g. an already-reaped or never-existent parent).
    pub fn post_signal(&mut self, pid: u32, sig: u32) {
        if let Some(p) = self.get_mut(pid) {
            p.signals.raise(sig);
        }
    }

    /// True iff a live (non-reaped) process with pid `pid` exists — for the
    /// kill/tkill/tgkill existence probe (`sig == 0` → ESRCH if absent).
    pub fn pid_exists(&self, pid: u32) -> bool {
        self.get(pid).is_some()
    }

    /// Try to reap a zombie child of the current process matching `target`
    /// (-1 = any). On success returns `Some((child_pid, encoded_status, child))`
    /// where `child` is the reaped [`Process`] **moved out** of the table (the
    /// caller drops it only after dropping the `PROCS` lock + a cross-CPU TLB
    /// shootdown — see [`complete_waits`]); the child is also removed from the
    /// parent's child list here. Returns `None` if no matching zombie exists yet.
    ///
    /// P4·SMP·S4d: the child's address-space frames are NOT freed inline. Freeing
    /// them under the `PROCS` lock (the old `table[cpid] = None`) returned the
    /// reaped child's page-table + data frames to the global allocator while a
    /// sibling CPU could still hold a stale TLB entry for them (or be concurrently
    /// (de)allocating) — an EL1 use-after-free that crashed the multi-child demo.
    /// Deferring the drop to the caller (post-lock, post-shootdown) mirrors the
    /// exit path. On 1-vCPU the shootdown is a no-op and the child is dropped
    /// immediately after the syscall returns, so the golden trace is unchanged.
    #[must_use = "drop the reaped process only after a cross-CPU TLB shootdown"]
    pub fn try_reap(&mut self, target: i64) -> Option<(u32, i32, Process)> {
        let parent_pid = self.current;
        // Find a matching zombie child.
        let child_pids: Vec<u32> = self
            .get(parent_pid)
            .map(|p| p.children.clone())
            .unwrap_or_default();
        for cpid in child_pids {
            let is_zombie = matches!(self.get(cpid), Some(p) if p.state == State::Zombie);
            let matches = target == -1 || target == cpid as i64;
            if is_zombie && matches {
                let status = self.get(cpid).map(|p| p.exit_status).unwrap_or(0);
                // Move the child OUT of the table (defer the actual free to the
                // caller, after the lock is dropped + a TLB shootdown).
                let child = self.procs.table[cpid as usize].take();
                if let Some(parent) = self.get_mut(parent_pid) {
                    parent.children.retain(|&c| c != cpid);
                }
                if let Some(child) = child {
                    return Some((cpid, status, child));
                }
                return None;
            }
        }
        None
    }

    /// True iff the current process has any live (un-reaped) child matching
    /// `target` — used to distinguish "no such child" (-ECHILD) from "child not
    /// done yet" (block / WNOHANG return 0).
    pub fn has_child(&self, target: i64) -> bool {
        match self.get(self.current) {
            Some(p) => p
                .children
                .iter()
                .any(|&c| (target == -1 || target == c as i64) && self.get(c).is_some()),
            None => false,
        }
    }

    /// Complete any pending `wait4` blocks: for each `Waiting` parent that now
    /// has a matching zombie child, reap the child, deliver the encoded status
    /// into the parent's saved context (`x0 = child pid`, write `*status` into
    /// the parent's address space), and mark the parent `Runnable` so it resumes
    /// from just after its `wait4` SVC with the correct return value.
    ///
    /// The reaped child [`Process`]es are **moved out** of the table and returned
    /// rather than dropped here, because dropping a child frees its page tables /
    /// frames via the global allocator — which must not happen while that child's
    /// address space is still the live `TTBR0`. The caller switches `TTBR0` to a
    /// surviving process first, *then* drops the returned vector.
    #[must_use = "drop the reaped processes only after switching TTBR0 away from them"]
    pub fn complete_waits(&mut self) -> Vec<Process> {
        let mut reaped_procs: Vec<Process> = Vec::new();
        // Collect Waiting parent pids first to avoid aliasing the table.
        let waiting: Vec<u32> = self
            .procs
            .table
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|p| p.state == State::Waiting)
            .map(|p| p.pid)
            .collect();

        for parent_pid in waiting {
            let (target, status_ptr, child_pids) = match self.get(parent_pid) {
                Some(p) => (p.wait_target, p.wait_status_ptr, p.children.clone()),
                None => continue,
            };
            // Find a matching zombie child.
            let mut reaped: Option<(u32, i32)> = None;
            for cpid in child_pids {
                let is_zombie = matches!(self.get(cpid), Some(p) if p.state == State::Zombie);
                let matches = target == -1 || target == cpid as i64;
                if is_zombie && matches {
                    let st = self.get(cpid).map(|p| p.exit_status).unwrap_or(0);
                    reaped = Some((cpid, st));
                    break;
                }
            }
            if let Some((cpid, encoded)) = reaped {
                // Move the child out of the table (defer the actual free).
                if let Some(child) = self.procs.table[cpid as usize].take() {
                    reaped_procs.push(child);
                }
                // Deliver the result into the parent.
                let mut woke = false;
                if let Some(parent) = self.get_mut(parent_pid) {
                    parent.children.retain(|&c| c != cpid);
                    parent.state = State::Runnable;
                    parent.ctx.regs[0] = cpid as u64; // wait4 return = child pid
                    if status_ptr != 0 {
                        // SAFETY: write into the parent's own mapped page via its
                        // identity alias; `status_ptr` was range-checked in the
                        // wait4 syscall before the block.
                        unsafe {
                            parent.space.write_u32(status_ptr as usize, encoded as u32);
                        }
                    }
                    parent.wait_target = -1;
                    parent.wait_status_ptr = 0;
                    woke = true;
                }
                if woke {
                    // P4·SMP·S4a: a just-woken (Waiting -> Runnable) parent goes
                    // back on THIS CPU's run queue. On 1-vCPU it is popped right
                    // back here by the post-wait4 schedule (golden byte-identical).
                    runq_push(self.cpu, parent_pid);
                }
            }
        }
        reaped_procs
    }
}

// ---- Context switching ----------------------------------------------------

/// Copy a [`TrapFrame`] by value (it is `#[repr(C)]` POD).
fn clone_frame(f: &TrapFrame) -> TrapFrame {
    TrapFrame {
        regs: f.regs,
        sp: f.sp,
        elr: f.elr,
        spsr: f.spsr,
    }
}

/// Save the live trap `frame` into the current process's `ctx`, also capturing
/// the live `SP_EL0` and `TPIDR_EL0` (which the trap stub does not stash).
///
/// # Safety
/// `frame` must be the live trap frame; reads system registers.
pub unsafe fn save_current(frame: &TrapFrame) {
    // SAFETY: reading SP_EL0/TPIDR_EL0 of the just-trapped EL0 thread.
    let sp_el0: u64;
    unsafe {
        core::arch::asm!("mrs {x}, sp_el0", x = out(reg) sp_el0, options(nostack, nomem));
    }
    let tpidr = TPIDR_EL0.get();
    with_sched(|s| {
        // The current process may have just exited *and been reaped* (its slot
        // freed) before we got here — in that case there is nothing to save.
        let cur_pid = s.current;
        if let Some(cur) = s.get_mut(cur_pid) {
            cur.ctx = clone_frame(frame);
            cur.ctx.sp = sp_el0;
            cur.tpidr = tpidr;
        }
    });
}

/// True iff ANY process that can still make progress exists (P4·SMP·S4a): a
/// `Runnable`, `Running` or `Waiting` process. A lone `Zombie` is NOT "alive".
/// The machine-wide termination condition: under `-smp`, a CPU whose `schedule`
/// returns `false` must NOT power off if another CPU still runs a process. On
/// 1-vCPU this is `false` exactly when the old `runnable_count()==0` check fired.
pub fn any_alive() -> bool {
    with_sched(|s| {
        s.procs.table.iter().any(|slot| {
            matches!(
                slot,
                Some(p) if matches!(p.state, State::Runnable | State::Running | State::Waiting)
            )
        })
    })
}

/// Switch the scheduler's `current` to `pid`, load that process's address space
/// (`TTBR0_EL1`) + `TPIDR_EL0`, and copy its saved `ctx` back into the live trap
/// `frame` (including `SP_EL0`) so the trampoline's `eret` resumes it.
///
/// # Safety
/// `frame` is the live trap frame the stub will restore from; this rewrites it.
/// Edits `TTBR0_EL1` and flushes the TLB.
pub unsafe fn switch_to(pid: u32, frame: &mut TrapFrame) {
    // Read the (already-claimed via `pick_and_claim`) target's (ttbr0, tpidr,
    // ctx) UNDER the `PROCS` lock, dropped at the closure end — the lock is
    // released BEFORE the `TTBR0_EL1.set` + `eret` below. The SMP ownership
    // bookkeeping (claim/demote/enqueue) is done atomically with the pick in
    // `pick_and_claim`, NOT here, so two CPUs can never select the same pid.
    let (ttbr0, tpidr, ctx) = with_sched(|s| {
        debug_assert_eq!(s.current, pid, "switch_to target must be the claimed current");
        let p = s.get(pid).expect("switch target missing");
        (p.space.ttbr0(), p.tpidr, clone_frame(&p.ctx))
    });

    // SAFETY: `ttbr0` is a process L1 whose upper GiBs re-create the kernel
    // identity map, so EL1 stays addressable across the switch.
    unsafe {
        TTBR0_EL1.set(ttbr0);
        barrier::dsb(barrier::SY);
        core::arch::asm!("tlbi vmalle1", "dsb sy", "isb", options(nostack));
        TPIDR_EL0.set(tpidr);
        core::arch::asm!("msr sp_el0, {x}", x = in(reg) ctx.sp, options(nostack, nomem));
    }
    // Restore the integer context + ELR/SPSR into the live frame.
    frame.regs = ctx.regs;
    frame.sp = ctx.sp;
    frame.elr = ctx.elr;
    frame.spsr = ctx.spsr;
}

/// The scheduler decision after a trap that may have blocked/exited the current
/// process: pick the next runnable process and switch the live `frame` to it.
/// If nothing is runnable but processes still exist (all Waiting — a deadlock
/// that should not happen in our workloads) we panic; if the table is empty the
/// caller handles teardown.
///
/// Returns `true` if a switch happened (or the current keeps running), `false`
/// if no process remains runnable at all.
///
/// # Safety
/// `frame` is the live trap frame; may be rewritten by `switch_to`.
pub unsafe fn schedule(frame: &mut TrapFrame) -> bool {
    // Pick AND claim the next pid ATOMICALLY under one `PROCS` lock (mark
    // `Running` before releasing), so a concurrent `schedule` on another CPU
    // cannot also select it. `switch_to` then only reads the claimed target's AS.
    let next = with_sched(|s| s.pick_and_claim());
    match next {
        Some(pid) => {
            // SAFETY: pid is this CPU's claimed `current`; rewrites the live frame.
            unsafe { switch_to(pid, frame) };
            true
        }
        None => false,
    }
}
