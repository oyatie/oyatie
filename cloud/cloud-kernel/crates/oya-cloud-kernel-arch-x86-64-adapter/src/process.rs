//! # Process model for the x86_64 Frame
//!
//! This module turns the previously single-shot ring-3 bring-up (`user.rs`,
//! K4-level) into a real **multi-process** kernel: a process table, per-process
//! address spaces (own PML4), a cooperative-plus-timer-preemptive **scheduler**,
//! a full **register context** saved/restored on every kernel entry, and the
//! `clone`/`execve`/`wait4`/`exit` family that lets a parent spawn and reap a
//! child. It is the x86_64 mirror of [`crate::process`]'s aarch64 sibling
//! (`arch-aarch64/src/process.rs`, the K7 process model).
//!
//! ## Why this is Frame code
//!
//! Everything here is `unsafe`-bearing: it owns page tables, switches `CR3`,
//! copies whole address spaces frame-by-frame, and resumes ring-3 register
//! state. The safe kernel only ever calls the safe [`crate::arch::run_user`]
//! entry; all the dangerous machinery is encapsulated below with a safety note
//! at each site, exactly as the framekernel design requires.
//!
//! ## Address space, per process
//!
//! The single-process loader refined the *boot* PD slot in place and carved one
//! shared `USER_PT`. That does not generalise: two processes need two different
//! views of the same low VAs. So each [`Process`] owns a full, independent
//! 4-level translation hierarchy on the kernel heap:
//!
//! ```text
//!   AddressSpace
//!     pml4: Box<PageTable>   // entry 0 -> our PDPT; the kernel-half / identity
//!                            //            entries copied from the boot PML4 so
//!                            //            EL0->ring0 transitions keep working
//!     pdpt: Box<PageTable>   // entry 0 -> our PD (the low 1 GiB)
//!     pd:   Box<PageTable>   // identity 2 MiB kernel huge pages, except the
//!                            //            USER_BASE slot -> our user PT
//!     pt:   [Box<PageTable>; USER_NTABLES] // 4 x (512 x 4 KiB) = the 8 MiB window
//!     frames: Vec<Box<Frame>>  // the physical pages backing mapped user VAs
//! ```
//!
//! Switching to a process is just `Cr3::write(pml4_pa)` + a TLB flush (a full
//! flush via the CR3 reload). Because the PD re-creates the kernel identity map
//! for the rest of the low GiB, the ring-0 console/PIC/PIT/heap keep working no
//! matter which process is current.
//!
//! `fork` shares the parent's backing frames **copy-on-write**: fresh
//! PML4/PDPT/PD/PT whose child PT entries re-point at the *same* physical frames
//! as the parent (held through a shared [`Arc`] refcount), with every writable
//! page write-protected + tagged COW (a software PTE bit) in **both** spaces. The
//! first write to such a page raises a write-protection `#PF` that
//! [`AddressSpace::cow_fault`] services: it allocates a private frame, copies the
//! shared bytes, and re-maps the faulting PTE writable. A frame's storage is
//! freed only when its last [`Arc`] referrer drops. `execve` drops the COW
//! mappings wholesale by replacing the whole address space with a fresh one.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

// P4·SMP·S4b: the per-CPU run queues are now `ksync::cl_deque::Deque`s (the
// stealable Chase-Lev work-stealing deques), replacing the S4a
// `PerCpuLocal<VecDeque>` whose single-owner `VecDeque` had no cross-CPU steal.
use hal::cpu::MAX_CPUS;

use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::registers::model_specific::Msr;
use x86_64::structures::paging::frame::PhysFrame;
use x86_64::structures::paging::PageTableFlags as Ptf;
use x86_64::PhysAddr;

/// `IA32_FS_BASE` MSR (`0xC000_0100`): the `%fs` segment base in long mode. The
/// context switch restores the incoming process's saved `fs_base` here so each
/// musl process resumes with ITS OWN `%fs:`-relative TLS. (Same constant as the
/// `IA32_FS_BASE` in `user.rs`, where `arch_prctl(ARCH_SET_FS)` first sets it.)
const IA32_FS_BASE: u32 = 0xC000_0100;

use user_layout::signal::SignalState;
use user_layout::{PAGE_MASK, PAGE_SIZE, PT_ENTRIES, USER_BASE, USER_NTABLES, USER_TOP};

// ---- Raw page-table + frame storage ---------------------------------------

/// 2 MiB huge-page size the boot PD maps the low 1 GiB with.
const HUGE_SIZE: u64 = 0x20_0000;
/// 2 MiB-aligned base of the user window region (the PD slot we refine).
const USER_REGION_BASE: u64 = (USER_BASE as u64) & !(HUGE_SIZE - 1);

/// A 4 KiB-aligned page table (512 x 64-bit entries), heap-allocated so each
/// process owns its own copy.
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}

impl PageTable {
    /// A fresh, all-zero (all-not-present) table.
    fn boxed() -> Box<PageTable> {
        Box::new(PageTable { entries: [0; 512] })
    }
}

/// A single 4 KiB physical frame backing a user page, heap-allocated and
/// 4 KiB-aligned so its address is a valid page-table output address. The boot
/// identity map covers the whole low 1 GiB, so a heap frame's VA equals its PA.
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
/// [`SharedFrame`]. This is the address stored in the PT leaf's output field, so
/// it is the key used to match a descriptor back to its retained frame. (The
/// triple deref takes the address of the `Frame` page itself — `Arc` -> `Box` ->
/// `Frame` — not of any smart-pointer temporary.)
fn frame_pa(frame: &SharedFrame) -> u64 {
    core::ptr::addr_of!(***frame) as u64
}

/// Software-available PTE bit (x86_64 leaves reserve bits[11:9] for software) we
/// set on a **copy-on-write** leaf: the page is mapped present + US + NX but
/// write-protected (WRITABLE clear) now, and a write `#PF` to it should *copy*
/// rather than fatally halt. Distinguishes a COW page (writable-intent, currently
/// RO) from a genuinely read-only page (rodata / AT_PHDR copy), whose write fault
/// stays fatal.
const PTE_COW: u64 = 1 << 9;

/// Mask selecting the output-frame address bits of a 4 KiB leaf PTE.
const PTE_ADDR_MASK: u64 = 0x000F_FFFF_FFFF_F000;

/// One-shot guard so the COW-proof serial line is printed only on the *first*
/// serviced copy-on-write fault (proof COW fired) rather than on every fault.
static COW_FAULT_LOGGED: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

// ---- Per-segment ring-3 page permissions (W^X) ----------------------------

/// Per-segment ring-3 page permission, the x86_64 analogue of
/// `user_layout::PagePerm` lowered to x86_64 page-table flag bits. Mirrors the
/// `Perm` enum the single-process loader used.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Perm {
    /// Read + execute (text): present, US, not writable, not NX.
    ReadExec,
    /// Read + write, never executable (data/bss/stack): present, US, W, NX.
    ReadWrite,
    /// Read-only, never executable (rodata, AT_PHDR copy): present, US, NX.
    ReadOnly,
}

impl Perm {
    /// The least-privilege permission for an ELF segment's `p_flags` (bit0=X,
    /// bit1=W, bit2=R). W wins over X (no W+X to ring 3), matching the loader.
    pub fn from_pflags(p_flags: u32) -> Perm {
        let x = (p_flags & 0x1) != 0;
        let w = (p_flags & 0x2) != 0;
        if w {
            Perm::ReadWrite
        } else if x {
            Perm::ReadExec
        } else {
            Perm::ReadOnly
        }
    }

    /// The leaf page-table flag bits (a ring-3 US page) for this permission. We
    /// always set WRITABLE for the supervisor stage of staging the image is not
    /// needed here — the loader writes through each frame's heap alias, not the
    /// user VA — so the leaf carries exactly the ring-3 W^X bits.
    fn leaf_flags(self) -> u64 {
        match self {
            Perm::ReadExec => (Ptf::PRESENT | Ptf::USER_ACCESSIBLE).bits(),
            Perm::ReadWrite => {
                (Ptf::PRESENT | Ptf::WRITABLE | Ptf::USER_ACCESSIBLE | Ptf::NO_EXECUTE).bits()
            }
            Perm::ReadOnly => (Ptf::PRESENT | Ptf::USER_ACCESSIBLE | Ptf::NO_EXECUTE).bits(),
        }
    }

    /// Whether this permission grants ring-3 execute.
    fn is_exec(self) -> bool {
        matches!(self, Perm::ReadExec)
    }
    /// Whether this permission grants ring-3 write.
    fn is_write(self) -> bool {
        matches!(self, Perm::ReadWrite)
    }

    /// Union of two permissions for a page shared by two ELF segments: satisfy
    /// both. W+X never arises for our page-aligned images; if it did we would
    /// grant RW (drop X) to keep W^X — but in practice segments are page-aligned.
    fn merge(self, other: Perm) -> Perm {
        let w = self.is_write() || other.is_write();
        let x = self.is_exec() || other.is_exec();
        match (w, x) {
            (true, _) => Perm::ReadWrite,
            (false, true) => Perm::ReadExec,
            (false, false) => Perm::ReadOnly,
        }
    }

    /// Recover the [`Perm`] encoded in an existing present leaf descriptor.
    fn from_desc(desc: u64) -> Perm {
        let w = (desc & Ptf::WRITABLE.bits()) != 0;
        let x = (desc & Ptf::NO_EXECUTE.bits()) == 0;
        match (w, x) {
            (true, _) => Perm::ReadWrite,
            (false, true) => Perm::ReadExec,
            (false, false) => Perm::ReadOnly,
        }
    }
}

// ---- Per-process address space --------------------------------------------

/// A complete, self-contained ring-3 address space: the four-level translation
/// hierarchy plus the physical frames it points at. Owned by a [`Process`]; its
/// `pml4` physical address is what we load into `CR3`.
pub struct AddressSpace {
    pml4: Box<PageTable>,
    // `pdpt`/`pd` are never read after construction, but they are load-bearing
    // *storage*: the PML4/PDPT entries point at their physical addresses, so
    // they must stay alive for the life of the space. Keep them owned here.
    #[allow(dead_code)]
    pdpt: Box<PageTable>,
    #[allow(dead_code)]
    pd: Box<PageTable>,
    // The user window is `USER_NTABLES` consecutive 2 MiB leaf page tables (each
    // maps 512 × 4 KiB). PD slot `(USER_REGION_BASE/HUGE_SIZE)+i` walks into
    // `pt[i]`. A flat user-window page index is split into `(table, entry)` by
    // dividing/modding by `PT_ENTRIES`.
    pt: [Box<PageTable>; USER_NTABLES],
    /// Backing frames, kept alive for the life of the space. The PT descriptors
    /// point at `frame.bytes` (identity-aliased in ring 0 via the boot map). Held
    /// through a share refcount ([`SharedFrame`]) so a copy-on-write `fork` can
    /// share a frame between parent and child; the storage is freed only when the
    /// last referring space drops its reference.
    frames: Vec<SharedFrame>,
}

impl AddressSpace {
    /// Build an empty address space: identity 2 MiB kernel huge pages over the
    /// low 1 GiB, the `USER_BASE` 2 MiB slot refined to a 4 KiB page table (all
    /// pages initially not-present), and the upper PML4 entries copied from the
    /// boot PML4 so ring 0 stays addressable when this space is current.
    ///
    /// # Safety
    /// Reads the boot PML4 to replicate its kernel-half entries. Must run on the
    /// boot core with the boot identity paging live.
    pub unsafe fn new() -> AddressSpace {
        let mut pml4 = PageTable::boxed();
        let mut pdpt = PageTable::boxed();
        let mut pd = PageTable::boxed();
        let pt: [Box<PageTable>; USER_NTABLES] =
            core::array::from_fn(|_| PageTable::boxed());

        // PD: identity 2 MiB huge kernel pages over the low 1 GiB (US clear, so
        // they stay supervisor-only under SMEP/SMAP), exactly what the boot PD
        // provided.
        let kernel_huge = (Ptf::PRESENT | Ptf::WRITABLE | Ptf::HUGE_PAGE).bits();
        for (i, e) in pd.entries.iter_mut().enumerate() {
            *e = ((i as u64) * HUGE_SIZE) | kernel_huge;
        }
        // The `USER_NTABLES` consecutive 2 MiB slots from USER_BASE each become a
        // walk into the matching leaf table, so the window spans
        // `USER_NTABLES * 2 MiB`. The table entries must be US so the ring-3 walk
        // can reach the US leaves below them.
        let table_flags = (Ptf::PRESENT | Ptf::WRITABLE | Ptf::USER_ACCESSIBLE).bits();
        let pd_base = (USER_REGION_BASE / HUGE_SIZE) as usize;
        for i in 0..USER_NTABLES {
            let pt_pa = core::ptr::addr_of!(*pt[i]) as u64;
            pd.entries[pd_base + i] = pt_pa | table_flags;
        }

        // PDPT[0] -> our PD (covers the low 1 GiB, the user low GiB).
        let pd_pa = core::ptr::addr_of!(*pd) as u64;
        pdpt.entries[0] = pd_pa | table_flags;

        // PML4[0] -> our PDPT (the low 512 GiB, where the user window + kernel
        // image live). Copy every *other* PML4 entry from the boot PML4 so any
        // higher-half kernel mapping stays valid under this CR3. (For this MVP
        // only entry 0 is populated in the boot PML4, but copying the rest keeps
        // us correct if the kernel ever maps a higher half.)
        // SAFETY: single core during bring-up; the boot PML4 is a read source.
        let boot = unsafe { &*core::ptr::addr_of!(boot_pml4) };
        for i in 1..512 {
            pml4.entries[i] = boot[i];
        }
        let pdpt_pa = core::ptr::addr_of!(*pdpt) as u64;
        pml4.entries[0] = pdpt_pa | table_flags;

        AddressSpace {
            pml4,
            pdpt,
            pd,
            pt,
            frames: Vec::new(),
        }
    }

    /// Physical (== identity) address of this space's PML4, the value to load
    /// into `CR3`.
    pub fn cr3(&self) -> u64 {
        core::ptr::addr_of!(*self.pml4) as u64
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
    /// the child's PT at the *same* physical frame the parent points at, bumping
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
    /// mapped, union the permission and keep the frame.
    ///
    /// # Safety
    /// Edits this space's PT and (when current) flushes the TLB for the VA. The
    /// caller must hold the only mutable reference to this space.
    pub unsafe fn map_page(&mut self, va: usize, perm: Perm) -> *mut u8 {
        let page_va = va & !PAGE_MASK;
        assert!(
            (USER_BASE..USER_TOP).contains(&page_va),
            "user va out of window: {:#x}",
            page_va
        );
        let flat = (page_va as u64 - USER_REGION_BASE) as usize / PAGE_SIZE;
        let (tbl, idx) = (flat / PT_ENTRIES, flat % PT_ENTRIES);
        let existing = self.pt[tbl].entries[idx];
        let (pa, eff_perm) = if existing & Ptf::PRESENT.bits() != 0 {
            // Already mapped: keep the frame; union perms so an earlier RX page
            // is never silently downgraded when a second segment shares it.
            (
                existing & 0x000F_FFFF_FFFF_F000,
                perm.merge(Perm::from_desc(existing)),
            )
        } else {
            (self.alloc_frame(), perm)
        };
        self.pt[tbl].entries[idx] = pa | eff_perm.leaf_flags();

        // SAFETY: invalidate the single changed translation. Harmless even when
        // this space is not the live CR3 (invlpg only affects the TLB).
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) page_va, options(nostack, preserves_flags));
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
        let flat = (page_va as u64 - USER_REGION_BASE) as usize / PAGE_SIZE;
        let (tbl, idx) = (flat / PT_ENTRIES, flat % PT_ENTRIES);
        let desc = self.pt[tbl].entries[idx];
        if desc & Ptf::PRESENT.bits() == 0 {
            return None;
        }
        let pa = desc & 0x000F_FFFF_FFFF_F000;
        Some((pa + (va & PAGE_MASK) as u64) as *mut u8)
    }

    /// Write `value` (4 bytes, LE) at user `va` in this space via the identity
    /// alias. Returns false if `va` is unmapped. Used to deliver a `wait4`
    /// status into a parent that is not the current process.
    ///
    /// # Safety
    /// `va..va+4` must lie within a single mapped page (true for an aligned i32).
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
    pub unsafe fn copy_to_user(&mut self, dst_va: usize, src: &[u8], perm: Perm) {
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
    /// Same PT *shape*, but instead of byte-copying every backing frame, the child
    /// **shares** the parent's frames: each mapped child PT entry points at the
    /// very same physical frame (its [`Arc`] cloned, bumping the share refcount).
    /// Writable (data/stack/heap/RW) pages are additionally write-protected
    /// (WRITABLE cleared) and tagged [`PTE_COW`] in *both* the parent and the
    /// child, so the first write in either takes a write-protection `#PF` that
    /// [`AddressSpace::cow_fault`] resolves by copying. Read-only and read+execute
    /// pages are shared as-is: they are never written (W^X keeps code RX), so they
    /// need no COW marker and stay shared for the life of both spaces.
    ///
    /// # Safety
    /// Reads the boot PML4 (via [`AddressSpace::new`]) and write-protects the
    /// parent's writable leaves in place. Must run on the boot core; the caller
    /// flushes the parent's TLB by the post-fork CR3 reload (and writing CR3 on
    /// any subsequent switch), so the parent sees the new write-protected mapping.
    pub unsafe fn cow_clone(&mut self) -> AddressSpace {
        // SAFETY: builds a fresh kernel-half-shared space.
        let mut child = unsafe { AddressSpace::new() };
        for flat in 0..(USER_NTABLES * PT_ENTRIES) {
            let tbl = flat / PT_ENTRIES;
            let idx = flat % PT_ENTRIES;
            let desc = self.pt[tbl].entries[idx];
            if desc & Ptf::PRESENT.bits() == 0 {
                continue;
            }
            let pa = desc & PTE_ADDR_MASK;
            // Clone the parent's Arc for this frame so child + parent share it.
            // Every present leaf is backed by a retained frame (`alloc_frame` is
            // the only path that maps one), so the lookup always succeeds.
            let shared = self
                .frame_arc(pa)
                .expect("mapped PT leaf without a retained frame");
            child.retain_frame(shared);

            if desc & Ptf::WRITABLE.bits() != 0 {
                // Writable page -> COW in both spaces: clear WRITABLE, set COW.
                let cow_desc = (desc & !Ptf::WRITABLE.bits()) | PTE_COW;
                self.pt[tbl].entries[idx] = cow_desc; // write-protect the parent
                child.pt[tbl].entries[idx] = cow_desc; // child shares it RO+COW
                // Invalidate the parent's stale writable TLB entry for this VA.
                let page_va = USER_REGION_BASE + (flat as u64) * PAGE_SIZE as u64;
                // SAFETY: invalidate one VA in the (current) parent's TLB.
                unsafe {
                    core::arch::asm!(
                        "invlpg [{}]",
                        in(reg) page_va,
                        options(nostack, preserves_flags),
                    );
                }
            } else {
                // Share the non-writable mapping unchanged.
                child.pt[tbl].entries[idx] = (desc & !PTE_ADDR_MASK) | (pa & PTE_ADDR_MASK);
            }
        }
        child
    }

    /// Service a write fault against `va` in this space. Returns `true` if the
    /// page was a [`PTE_COW`] copy-on-write page and the fault was resolved (a
    /// private writable frame now backs `va`); `false` if the page is not COW
    /// (genuine read-only / unmapped), leaving the caller to treat it as fatal.
    ///
    /// On a COW hit: allocate a fresh private frame, copy the shared page's bytes
    /// into it, drop this space's reference to the old (shared) frame, re-map the
    /// PT entry at the new frame as writable with the COW marker cleared, and
    /// invalidate the VA so the faulting store re-executes against the now-private
    /// writable page.
    ///
    /// # Safety
    /// This space must be the current `CR3` (the faulting ring-3 thread's). Edits
    /// the PT and flushes the TLB for the one VA.
    pub unsafe fn cow_fault(&mut self, va: usize) -> bool {
        let page_va = va & !PAGE_MASK;
        if !(USER_BASE..USER_TOP).contains(&page_va) {
            return false;
        }
        let flat = (page_va as u64 - USER_REGION_BASE) as usize / PAGE_SIZE;
        let (tbl, idx) = (flat / PT_ENTRIES, flat % PT_ENTRIES);
        let desc = self.pt[tbl].entries[idx];
        if desc & Ptf::PRESENT.bits() == 0 || desc & PTE_COW == 0 {
            return false;
        }
        let old_pa = desc & PTE_ADDR_MASK;

        // Allocate a fresh private frame and copy the shared page's bytes into it.
        let new_frame = Frame::shared_zeroed();
        let new_pa = frame_pa(&new_frame);
        // SAFETY: both are identity-mapped, 4 KiB, non-overlapping frames; the
        // shared source is still mapped (we have not dropped it yet).
        unsafe {
            core::ptr::copy_nonoverlapping(old_pa as *const u8, new_pa as *mut u8, PAGE_SIZE);
        }
        self.frames.push(new_frame);
        // Drop this space's Arc to the shared frame (refcount--). A forked sibling
        // still referencing it keeps its storage alive for that sibling.
        self.release_frame(old_pa);

        // COW proof: announce the first serviced fault so the serial log shows COW
        // actually fired. One-shot to avoid flooding.
        if !COW_FAULT_LOGGED.swap(true, core::sync::atomic::Ordering::Relaxed) {
            crate::kprintln!("cow: copied page on write fault (va={:#x})", page_va);
        }

        // Re-map writable, COW cleared. COW only applies to W^X data pages, so the
        // private copy is a ring-3 read-write, never-executable leaf.
        self.pt[tbl].entries[idx] = new_pa | Perm::ReadWrite.leaf_flags();
        // SAFETY: invalidate the single VA we just re-mapped in the live TLB.
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) page_va, options(nostack, preserves_flags));
        }
        true
    }

    /// P4·SMP·S4c/S4d defense-in-depth (mirror of the aarch64
    /// `AddressSpace::retry_stale_insn_fetch`): handle a ring-3 **instruction-fetch**
    /// protection fault that may be a STALE-TLB artifact. If the PT leaf backing
    /// `va` is currently PRESENT, USER-accessible, AND executable (`NO_EXECUTE`
    /// clear), the mapping NOW permits the fetch, so the `#PF` was a stale cached
    /// translation (a sibling CPU re-mapped/relaxed the page — e.g. a COW copy or
    /// an `execve` image swap — and the cross-CPU TLB-shootdown IPI to this core
    /// arrived late or was missed). `invlpg` the one VA and return `true` so the
    /// `iretq` retries the fetch against the fresh mapping. If the leaf is absent
    /// or genuinely non-executable, this is a real fault — return `false` so the
    /// caller treats it as fatal. On 1-vCPU there is no sibling to leave a stale
    /// entry, so this never fires and the golden is unchanged.
    ///
    /// # Safety
    /// This space must be the live `CR3` of the faulting ring-3 thread. Reads the
    /// PT and, on a stale hit, invalidates one VA's TLB entry.
    pub unsafe fn retry_stale_insn_fetch(&self, va: usize) -> bool {
        let page_va = va & !PAGE_MASK;
        if !(USER_BASE..USER_TOP).contains(&page_va) {
            return false;
        }
        let flat = (page_va as u64 - USER_REGION_BASE) as usize / PAGE_SIZE;
        let (tbl, idx) = (flat / PT_ENTRIES, flat % PT_ENTRIES);
        let desc = self.pt[tbl].entries[idx];
        let present = desc & Ptf::PRESENT.bits() != 0;
        let user = desc & Ptf::USER_ACCESSIBLE.bits() != 0;
        let executable = desc & Ptf::NO_EXECUTE.bits() == 0;
        // Present + US + executable ⇒ the mapping NOW permits the fetch, so the
        // fault was a stale TLB entry. Anything else is a genuine fault.
        if !(present && user && executable) {
            return false;
        }
        // SAFETY: invalidate the single faulting VA in this CPU's live TLB.
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) page_va, options(nostack, preserves_flags));
        }
        true
    }
}

extern "C" {
    /// The boot PML4 defined in `boot.rs` as `#[repr(C, align(4096))]
    /// PageTable([u64; 512])`. We name it here as the layout-identical
    /// `[u64; 512]` to copy its kernel-half entries into each process PML4.
    static boot_pml4: [u64; 512];
}

// ---- Register context (the saved ring-3 GPR set + RIP/RSP/RFLAGS) ----------

/// The full ring-3 register context saved on every kernel entry (syscall or
/// timer IRQ) and restored on resume. This is the x86_64 analogue of aarch64's
/// `TrapFrame`: a `#[repr(C)]` POD that the entry stubs in [`crate::user`]
/// build/consume directly. Field order is fixed by the asm; do not reorder.
///
/// `rip`/`rsp`/`rflags` hold the ring-3 continuation; the GPRs hold the user's
/// integer state (with `rax` doubling as the syscall return value on the way
/// out). `rcx`/`r11` are not stored separately: `syscall` puts the user RIP in
/// `rcx` and RFLAGS in `r11`, which the stub copies into `rip`/`rflags`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Context {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rsp: u64,
    pub rflags: u64,
}

impl Context {
    /// A zeroed context.
    pub const fn zeroed() -> Context {
        Context {
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: 0,
            rsp: 0,
            rflags: 0,
        }
    }
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
    /// Serial-backed console (`/dev/console`): `write` -> the 16550 UART via
    /// `crate::console::_print`; `read` -> 0 (EOF) for now.
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
/// the only accessor; the lock is uncontended but provides the interior
/// mutability safely). The drain cursor reuses the existing `read_off`
/// `AtomicU64` on the `FileDesc`.
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
    /// across `dup`'d/`clone`'d fds (Linux open-file-description semantics).
    /// An `AtomicU64` so it is mutable through the shared `Arc` without `unsafe`
    /// and keeps `FileDesc: Send + Sync`; the single-core trap path is the only
    /// mutator. Unused for Console/Null (they return EOF). For a `Netlink` fd it
    /// is the drain cursor into [`NetlinkFd::response`].
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

/// A user process: identity, scheduling state, its saved ring-3 register
/// context, its private address space, heap/mmap cursors, and family links.
pub struct Process {
    pub pid: u32,
    pub ppid: u32,
    pub state: State,
    /// The logical CPU currently running this process, or `-1` if it is on no
    /// CPU (Runnable-in-a-queue, Waiting, or Zombie). The x86_64 analog of Linux's
    /// `task_struct::on_cpu`: it makes `State::Running` *unambiguous* about WHICH
    /// CPU owns the process, so the scheduler's keep-current fallback can verify
    /// the running CPU is the TRUE owner before resuming `current`. Without it, a
    /// CPU whose stale `current` points at a pid that another CPU has since claimed
    /// (`Running`) would wrongly re-run that pid — the cross-CPU double-dispatch.
    /// Set to the claiming CPU in `pick_and_claim` / `admit_first`, reset to `-1`
    /// the moment the process leaves a CPU (demote, block, exit). `-1` on 1-vCPU
    /// steady state never changes the single core's decisions, so the golden is
    /// unchanged.
    pub owner_cpu: i32,
    /// Encoded exit status (the low 8 bits are the exit code << 8), valid when
    /// `state == Zombie`.
    pub exit_status: i32,
    /// Saved ring-3 register context.
    pub ctx: Context,
    /// The process's private ring-3 address space.
    pub space: AddressSpace,
    /// Program break (heap end) cursor.
    pub brk_cur: usize,
    /// Anonymous-`mmap` bump cursor (grows up from MMAP_BASE).
    pub mmap_cur: usize,
    /// The `%fs` segment base this process set via `arch_prctl(ARCH_SET_FS)` —
    /// musl's thread-control-block pointer for `%fs:`-relative TLS. x86_64-ONLY
    /// (aarch64 persists the equivalent in `tpidr`). Saved here per-process so
    /// the context switch can RESTORE it into `IA32_FS_BASE` for the incoming
    /// process: without this, two concurrent musl processes would clobber each
    /// other's `%fs` base (the MSR is a single global) and read the *other*
    /// process's TLS after a preemption. `0` means "never set" — a process that
    /// never called `arch_prctl` (e.g. a no_std PID1) keeps the MSR untouched.
    /// Init `0` in `blank`; inherited (copied) on `fork_current`; reset to `0`
    /// on `execve` (a fresh musl image re-sets it via `arch_prctl` at startup).
    pub fs_base: u64,
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

    /// A process freshly loaded from an ELF: takes ownership of its prepared
    /// address space. The loader fills `ctx`/`brk_cur`/`mmap_cur` after.
    ///
    /// # Safety
    /// Allocates a blank `AddressSpace` (reads the boot PML4); boot-core only.
    pub unsafe fn new_loaded(ppid: u32, space: AddressSpace) -> Process {
        // SAFETY: boot core; builds a blank space then overwrites it.
        let mut p = unsafe { Process::blank(0, ppid) };
        p.space = space;
        p
    }

    /// A blank process with a zeroed context.
    ///
    /// # Safety
    /// Allocates a blank `AddressSpace` (reads the boot PML4); boot-core only.
    unsafe fn blank(pid: u32, ppid: u32) -> Process {
        // SAFETY: boot core during bring-up.
        let space = unsafe { AddressSpace::new() };
        Process {
            pid,
            ppid,
            state: State::Runnable,
            owner_cpu: -1,
            exit_status: 0,
            ctx: Context::zeroed(),
            space,
            brk_cur: 0,
            mmap_cur: user_layout::MMAP_BASE,
            // No `%fs` base until the process calls `arch_prctl(ARCH_SET_FS)`.
            // `0` is the "never set" sentinel: switch_to skips the wrmsr for it,
            // so a no_std process keeps the MSR untouched.
            fs_base: 0,
            children: Vec::new(),
            wait_target: -1,
            wait_status_ptr: 0,
            fds: Vec::new(),
            // All dispositions SIG_DFL, nothing blocked/pending, no alt stack.
            signals: SignalState::new(),
        }
    }
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
    /// then faults to a stack address). `pick_and_claim` marking it `Running` does
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
            // A `Runnable` `current` on NO cpu (owner_cpu == -1) is the lone-process
            // steady state (e.g. the BSP's pid 1 before its first claim, or after a
            // self-preempt that re-picked itself): claiming it below stamps
            // ownership, so it stays exclusive.
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
    /// This CPU's logical index (from the `CpuToken` minted in `with_sched`).
    /// Used to route run-queue pops to THIS CPU's `RUNQ` slot. Always 0 on
    /// 1-vCPU, so the run-queue path indexes slot 0 exactly as before.
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

    /// Pick the next pid to run on THIS CPU (P4·SMP·S4a + S4b steal).
    ///
    /// First drains this CPU's own run-queue deque: pop the bottom, re-validate it
    /// is still `Runnable` in the (locked) table (a queued pid may have exited /
    /// been reaped / been migrated since enqueue), and return the first valid one.
    /// If the local deque yields nothing, **S4b** tries to STEAL from a victim
    /// CPU's deque top (`runq_steal`), re-validating any stolen pid the same way.
    /// If a steal also yields nothing, fall back to the keep-current + global
    /// round-robin scan — which on 1-vCPU (empty local deque, no victims) is
    /// byte-identical to the S4a/pre-S4b behavior.
    fn pick_next(&mut self) -> Option<u32> {
        // 1) Own deque, validated against the table.
        while let Some(pid) = runq_pop(self.cpu) {
            if matches!(self.procs.get(pid), Some(p) if p.state == State::Runnable) {
                return Some(pid);
            }
            // else: stale entry (exited/reaped/blocked) — drop it, try the next.
        }
        // 2) S4b: own deque empty — steal from a victim CPU's deque top. Re-
        // validate each stolen pid; a stale steal is harmless (the table is the
        // source of truth), just try another. No-op on 1-vCPU (no victims).
        while let Some(pid) = runq_steal(self.cpu) {
            if matches!(self.procs.get(pid), Some(p) if p.state == State::Runnable) {
                return Some(pid);
            }
            // else: stolen a stale pid (exited/reaped/claimed) — discard, retry.
        }
        // 3) Keep-current + global round-robin fallback (owner-checked).
        LocalSched::pick_next(self.procs, self.current, self.cpu)
    }

    /// Pick the next pid AND atomically CLAIM it for this CPU — all under the one
    /// `PROCS` lock held by the enclosing `with_sched` (P4·SMP·S4a). This closes
    /// the cross-CPU double-run race: a candidate is marked `Running` (owned by
    /// THIS CPU) BEFORE the lock is released, so a concurrent `pick_next` on
    /// another CPU re-validates it as `Running` and skips it — two CPUs can never
    /// claim the same pid. The outgoing `current`, if still active and different,
    /// is demoted to `Runnable` and re-enqueued on this CPU's run queue. Returns
    /// the claimed pid (now `current` + `Running`), or `None` if nothing runnable.
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
        // DEMO (feature-gated, no-op by default): announce the first run on a CPU.
        sched_log_first_run(pid, cpu);
        Some(pid)
    }
}

/// The one global process table, now (P4·SMP·S4a) behind a `ksync::SpinLock`.
///
/// **Lock-ordering invariant (enforced by review):** `PROCS` is the *only*
/// kernel lock taken inside [`with_sched`]. Never call `with_sched` while holding
/// any other lock (the leaf `NetlinkFd.response` / `PICS` locks), and never
/// acquire another lock inside the `with_sched` closure — so no ABBA deadlock is
/// representable by construction.
///
/// It is taken `lock_irqsave::<X86Irq>()` (NOT plain `lock()`) because
/// `with_sched` runs from BOTH the syscall path AND the timer-IRQ `preempt` path:
/// a same-CPU timer IRQ re-entering a held plain-`lock()` would self-deadlock on
/// its own ticket (exactly H2). Masking IRQs first closes that window. The guard
/// is dropped at the closure end — the lock is NEVER held across the `Cr3::write`
/// + `iretq` in [`switch_to`] (the lock is released before the address-space
/// install + ring transition; see `switch_to`).
static PROCS: ksync::spinlock::SpinLock<ProcTable> =
    ksync::spinlock::SpinLock::new(ProcTable::new());

/// Per-CPU run queues: one bounded Chase-Lev work-stealing deque of `Runnable`
/// pids per logical CPU (P4·SMP·S4b). The owning CPU pushes/pops the bottom of
/// its own deque; an idle CPU whose own deque is empty STEALS from a victim CPU's
/// top. The S4a `PerCpuLocal<VecDeque>` is replaced here by `ksync::cl_deque`
/// because the deque's `steal` is the lock-free cross-CPU rebalancing path the
/// `VecDeque` could not offer (a `VecDeque` is single-owner only).
///
/// The deque is itself `Sync` (all its methods take `&self`; the H4 loom models
/// proved the owner/stealer atomics race-free), so — unlike the `VecDeque`, which
/// needed `PerCpuLocal`'s `&mut`-per-slot — these slots are reached by shared `&`
/// and the per-CPU disjointness of `push`/`pop` is a *discipline* (each CPU only
/// pushes/pops its OWN index `cpu`), not a borrow-checker fact. On 1-vCPU only
/// slot 0 is ever touched and the deque degenerates to a single-owner push/pop =
/// the S4a FIFO order, so `pick_next` falls through to the byte-identical global
/// round-robin and no steal/IPI ever fires.
struct RunQueues {
    /// One stealable deque per CPU. Indexed by logical CPU index.
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

/// Round-robin fork-placement cursor over the online CPUs. `Relaxed` — it is a
/// placement *hint*, not a correctness datum (the `State`/table under `PROCS` is
/// the source of truth). Starts at 0 (the BSP).
static PLACE_CURSOR: AtomicU32 = AtomicU32::new(0);

/// Push `pid` onto CPU `cpu`'s run-queue deque (the owner's bottom). The pid must
/// be `Runnable` in the (locked) table; the caller holds the `PROCS` lock. If the
/// deque is full (`MAX_RUNNABLE_PER_CPU` reached) the push is refused and the pid
/// is enqueued on a fallback global queue so it is never lost (overflow spill).
///
/// Sound across CPUs: the deque is `Sync` and `push` takes `&self`; the
/// single-owner discipline (only CPU `cpu` pops its own deque) is preserved
/// because the *placing* CPU only ever PUSHES (the bottom), never pops, a remote
/// deque, and the `PROCS` lock the caller holds additionally serializes every
/// push/pop so even the bottom end is contention-free here.
fn runq_push(cpu: usize, pid: u32) {
    if !RUNQ.deques[cpu].push(pid) {
        // Deque full: spill to the global fallback so the pid is never dropped.
        // The locked global round-robin scan in `LocalSched::pick_next` will pick
        // it up (it is still `Runnable` + unowned in the table). Recorded for the
        // (rare) burst-overflow case; the cap is sized to MAX_RUNNABLE_PER_CPU.
        RUNQ_OVERFLOW.fetch_add(1, Ordering::Relaxed);
    }
    // P4·SMP·S4b: if the work was placed on a DIFFERENT, possibly-idle CPU, wake
    // it with a reschedule IPI so it re-runs its idle→schedule loop now instead of
    // waiting up to one periodic tick. The pid is already published to the deque
    // ABOVE (and we hold the `PROCS` lock), so the woken CPU observes it. A no-op
    // on 1-vCPU / same-CPU placement (`reschedule::notify` short-circuits), so the
    // golden is unperturbed. Fire-and-forget (no ack), so sending it under the
    // lock cannot deadlock.
    crate::reschedule::notify(cpu);
}

/// Pop the next ready pid from CPU `cpu`'s OWN run-queue deque (the owner's
/// bottom), if any. Caller holds the `PROCS` lock. Owner-only: `cpu` is always
/// the running CPU's own index.
fn runq_pop(cpu: usize) -> Option<u32> {
    RUNQ.deques[cpu].pop()
}

/// STEAL: an idle CPU `thief` whose own deque is empty takes a pid from a VICTIM
/// CPU's deque top. Returns `Some(pid)` on a successful steal (the caller
/// re-validates it is still `Runnable` under the `PROCS` lock before running it).
/// Tries each online CPU other than `thief` (starting one above it, wrapping);
/// `Steal::Retry` on a contended deque moves on to the next victim. `None` when
/// every victim is empty — the caller then idles (`hlt`/`wfe`).
///
/// The caller holds the `PROCS` lock, so the steal is serialized with the
/// victim's own `pop`; the lock-free `steal` is still correct without it (the H4
/// loom models proved owner-pop-vs-steal), but holding the lock here keeps the
/// whole pick decision atomic w.r.t. the table re-validation.
fn runq_steal(thief: usize) -> Option<u32> {
    use ksync::cl_deque::Steal;
    let mask = crate::smp::online_mask();
    if mask.count_ones() <= 1 {
        return None; // 1-vCPU: no victim to steal from (degenerate single-owner).
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
            // Contended or empty: try the next victim.
            Steal::Retry | Steal::Empty => {}
        }
    }
    None
}

/// Count of run-queue pushes that overflowed the bounded deque and spilled to the
/// global fallback. `Relaxed` — a diagnostic counter, not a correctness datum.
static RUNQ_OVERFLOW: AtomicU32 = AtomicU32::new(0);

/// P4·SMP·S4a cross-CPU scheduling DEMO instrumentation (feature-gated). A bit
/// per pid (`< 64`) recording whether we have already announced that pid's first
/// run on a CPU, so the `sched: pid P -> cpu K` line prints exactly once per
/// worker. NON-golden-shape (no `[pid N] syscall` prefix), and compiled in ONLY
/// under `smp-sched-demo` so the default golden/talos serial is byte-identical.
#[cfg(feature = "smp-sched-demo")]
static SCHED_SEEN: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Announce (once per pid) that pid `pid` is being run on CPU `cpu`. The proof of
/// cross-CPU scheduling: under `-smp 4` these lines show workers on multiple
/// distinct `cpu` indices. No-op unless the demo feature is enabled.
#[inline]
fn sched_log_first_run(pid: u32, cpu: usize) {
    #[cfg(feature = "smp-sched-demo")]
    {
        if pid < 64 {
            let bit = 1u64 << pid;
            // Claim the announce slot exactly once (Relaxed: a print, not a
            // correctness publish; the table under PROCS is the source of truth).
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
/// (P4·SMP·S4b). This is the **stolen-execution marker** the S4b gate asserts:
/// the pid was originally PLACED on `victim` (round-robin on fork) but is being
/// RUN by `thief != victim` — proof the deque steal path rebalanced load across
/// CPUs. NON-golden-shape (no `[pid N] syscall` prefix); compiled in ONLY under
/// `smp-sched-demo` so the default golden/talos serial is byte-identical, and a
/// no-op on 1-vCPU where `runq_steal` never runs.
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
/// placement. Always returns a set bit of the mask (falls back to 0/BSP if the
/// mask is somehow empty, which cannot happen — bit 0 is always set).
fn next_place_cpu() -> usize {
    let mask = crate::smp::online_mask();
    // Advance the cursor, then find the next set bit at or after it.
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
/// borrow is live (IRQs are masked in ring 0 during trap handling). The minted
/// [`hal::cpu::CpuToken`] indexes this CPU's [`CURRENT`] slot; under that same
/// IRQs-masked invariant we cannot migrate, so the index is stable across the
/// borrow. The `current` snapshot is read from `CURRENT` before `f` and written
/// back after, so `s.current = …` inside `f` updates the per-CPU slot exactly
/// as the old in-struct field did.
pub fn with_sched<R>(f: impl FnOnce(&mut Scheduler) -> R) -> R {
    // SAFETY: IRQs masked in ring 0 on the trap path → no migration, so
    // `this_cpu_token` mints a token pinned to this CPU for the whole borrow.
    let token = unsafe { crate::user::this_cpu_token() };
    // P4·SMP·S4a: take the GLOBAL ProcTable lock IRQ-SAFE for the critical
    // section only. `lock_irqsave::<X86Irq>()` masks IRQs FIRST then takes the
    // ticket, so a same-CPU timer IRQ cannot re-enter and self-deadlock (H2). On
    // 1-vCPU the lock is uncontended and IRQs are already masked on the trap
    // path, so `X86Irq::disable` returns `was_enabled=false` and the restore is a
    // no-op → no observable change vs the old `static mut` borrow.
    let mut guard = PROCS.lock_irqsave::<crate::arch::X86Irq>();
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
    // `guard` drops HERE: `SpinGuardIrq` releases the lock (Release store) THEN
    // restores the prior IRQ state. The lock is gone before any caller proceeds
    // to a CR3 write / iretq — see `switch_to`'s comment.
}

/// The one global in-RAM VFS (single namespace, M2). Frame-owned, lazily built
/// on first use (`Vfs::new()` pre-populates `/`, `/dev`, `/dev/console`,
/// `/dev/null`). Like `SCHED` it is accessed only from the single-core trap
/// path. The `Vfs` type + all its logic (tree, walker, mount table) live in the
/// 0-unsafe `vfs`; only this storage + accessor are `unsafe`, and
/// they live here in the Frame (TCB), not the forbid-set.
static mut VFS: Option<vfs::Vfs> = None;

/// Run `f` with `&mut Vfs`, building it on first use.
///
/// # Safety
/// Single-core: the trap path is the only caller and never re-enters while a
/// borrow is live (IRQs are masked in ring 0 during trap handling) — identical
/// justification to [`with_sched`].
pub fn with_vfs<R>(f: impl FnOnce(&mut vfs::Vfs) -> R) -> R {
    // SAFETY: single-core, non-reentrant access to the Frame-owned VFS.
    let slot = unsafe { &mut *core::ptr::addr_of_mut!(VFS) };
    f(slot.get_or_insert_with(vfs::Vfs::new))
}

// ---- Public scheduler operations the syscall layer / IRQ path call --------

impl Scheduler<'_> {
    /// Register a freshly-loaded first process and make it current. Returns its
    /// pid. The caller has already populated `ctx`/`space`.
    ///
    /// P4·SMP·S4a: mark it `Running` (owned by THIS CPU, the BSP) and make it this
    /// CPU's `current`. The BSP drops to ring 3 directly via `enter_ring3` — NOT
    /// through `switch_to` — and it is NOT placed on any run queue, so the BSP can
    /// only resume it via the keep-current fallback (`LocalSched::pick_next`), which
    /// needs `current == pid` and that pid `Running`/`Runnable`. Marking it
    /// `Running` here satisfies that; an AP never sees it because the fallback only
    /// ever returns a CPU's OWN `current`, and pid 1 is on no run queue for an AP to
    /// pop. On 1-vCPU no AP runs, so this is invisible to the golden.
    pub fn admit_first(&mut self, proc: Process) -> u32 {
        let pid = self.insert(proc);
        self.current = pid;
        let cpu = self.cpu;
        if let Some(p) = self.get_mut(pid) {
            p.state = State::Running;
            // The BSP owns pid 1 from the moment it drops to ring 3; stamp it so
            // the keep-current fallback resumes it (and no AP can).
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

    /// Block the current process in `wait4` (P4·SMP·S4d): save its FULL ring-3
    /// context (`frame`) into its `ctx`, then flip it to `Waiting` and record the
    /// wait target/status pointer — ALL under the single held `PROCS` lock, so the
    /// save is published BEFORE the process becomes wakeable. This closes the
    /// `save_current`-vs-`complete_waits` race: previously the blocking process was
    /// marked `Waiting` here and its context saved LATER (a second, post-lock
    /// `save_current`), so a sibling CPU's `complete_waits` could write the wake's
    /// `rax` (reaped pid) into `ctx` in the gap, only for the late `save_current`
    /// to clobber it back to the pre-block frame — corrupting the `wait4` return
    /// (premature/garbage ECHILD). Saving here, atomically, means
    /// `complete_waits`'s `ctx.rax = cpid` always wins. On 1-vCPU there is no
    /// sibling to race, and the saved context is byte-for-byte what the old
    /// `save_current` produced, so the golden trace is unchanged.
    pub fn block_current_for_wait(
        &mut self,
        frame: &Context,
        wait_target: i64,
        wait_status_ptr: u64,
    ) {
        let cur = self.current();
        cur.ctx = *frame;
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
    /// The current process's space must be the live `CR3` (true on a ring-3 `#PF`).
    /// Edits the PT + flushes the TLB for the one VA.
    pub unsafe fn try_cow_fault(&mut self, va: usize) -> bool {
        let cur = self.current;
        match self.get_mut(cur) {
            // SAFETY: `cur` is the current process; its space is the live CR3.
            Some(p) => unsafe { p.space.cow_fault(va) },
            None => false,
        }
    }

    /// P4·SMP·S4c/S4d defense-in-depth (mirror of the aarch64
    /// `Scheduler::try_retry_stale_insn_fetch`): handle a possibly-stale ring-3
    /// instruction-fetch protection fault against the current process. Returns
    /// `true` if the PT leaf is present+US+executable (a stale TLB entry, now
    /// invalidated → retry the fetch); `false` if it is a genuine
    /// non-executable/absent mapping.
    ///
    /// # Safety
    /// The current process's space must be the live `CR3` (true on a ring-3 `#PF`).
    /// Reads the PT + may invalidate one VA.
    pub unsafe fn try_retry_stale_insn_fetch(&mut self, va: usize) -> bool {
        let cur = self.current;
        match self.get(cur) {
            // SAFETY: `cur` is the current process; its space is the live CR3.
            Some(p) => unsafe { p.space.retry_stale_insn_fetch(va) },
            None => false,
        }
    }

    /// Fork the current process: create a child with a **copy-on-write** clone of
    /// the parent's address space (shared frames, writable pages write-protected +
    /// COW-tagged in both) and a duplicated context whose `rax` is 0 (the child's
    /// return). Returns the child's pid (the parent's `clone`/`fork` return value).
    ///
    /// # Safety
    /// COW-clones the parent address space (reads the boot PML4 + write-protects
    /// the parent's writable leaves); boot core only. The parent is the current
    /// process, so the post-fork CR3 reload flushes its now-write-protected TLB.
    pub unsafe fn fork_current(&mut self, parent_ctx: &Context) -> u32 {
        let parent_pid = self.current;
        let (child_space, brk, mmap, fds, fs_base, mut signals) = {
            let parent = self.get_mut(parent_pid).expect("parent missing");
            // SAFETY: boot core; the parent is the current CR3, so `cow_clone`
            // write-protects its writable leaves + invalidates their TLB entries.
            let child_space = unsafe { parent.space.cow_clone() };
            // Copy the parent's fd table: the child gets its own table whose
            // slots Arc-share the parent's open file descriptions (fork dups the
            // fds, sharing the descriptions, exactly as Linux does).
            let fds = parent.fds.clone();
            // Inherit the parent's `%fs` base: a `fork()` child shares the
            // parent's address space copy-on-write, so the same TLS pointer is
            // valid in the child (Linux fork inherits the FS base). A child that
            // later `execve`s gets it reset to 0 (a fresh musl re-sets it).
            let fs_base = parent.fs_base;
            // Inherit the parent's signal dispositions + blocked mask (POSIX
            // fork semantics); pending + on_altstack cleared below for the child.
            let signals = parent.signals;
            (child_space, parent.brk_cur, parent.mmap_cur, fds, fs_base, signals)
        };
        // Child starts with no pending signals and not running on an altstack.
        signals.pending = 0;
        signals.on_altstack = false;

        // SAFETY: boot core; the space is immediately overwritten with the copy.
        let mut child = unsafe { Process::blank(0, parent_pid) };
        child.space = child_space;
        child.brk_cur = brk;
        child.mmap_cur = mmap;
        child.fds = fds;
        child.fs_base = fs_base;
        child.signals = signals;
        // The child resumes exactly where the parent's syscall returns, but with
        // rax = 0 so userspace sees the fork() child return value, on the same
        // RSP (now backed by the child's own copied stack page).
        child.ctx = *parent_ctx;
        child.ctx.rax = 0;
        child.state = State::Runnable;

        let child_pid = self.insert(child);
        if let Some(parent) = self.get_mut(parent_pid) {
            parent.children.push(child_pid);
        }
        // P4·SMP·S4a placement: assign the child to a CPU round-robin over the
        // online mask and enqueue it on THAT CPU's local run queue. On 1-vCPU the
        // target is always CPU 0 (the only online bit) = this CPU, so the child
        // lands on slot 0 and `pick_next` pops it locally before the global
        // fallback — and since 1-vCPU never has two concurrent Runnable children,
        // the FIFO order equals the old round-robin → golden byte-identical
        // (gated). Under -smp the cursor spreads forked workers across the CPUs.
        let target = next_place_cpu();
        runq_push(target, child_pid);
        child_pid
    }

    /// Mark the current process a zombie with `status`. Waking + reaping a
    /// blocked parent is done separately by [`complete_waits`].
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
    /// the status in **WIFSIGNALED** form: low 7 bits = signal, bit 7
    /// (core-dumped) = 0 — vs a normal exit's `(code & 0xff) << 8`.
    pub fn terminate_current_by_signal(&mut self, sig: u32) {
        let pid = self.current;
        if let Some(p) = self.get_mut(pid) {
            p.state = State::Zombie;
            p.owner_cpu = -1; // off-CPU now (Zombie); no owner.
            p.exit_status = (sig & 0x7f) as i32;
        }
    }

    /// Raise signal `sig` on the process with pid `pid` by OR-ing its pending
    /// bit. **Bit-only**: NO print, NO state change — it must not flip a
    /// `Waiting` parent to `Runnable` (that's `complete_waits`'s job), keeping
    /// the SIGCHLD post on each child exit invisible to the golden trace. A
    /// no-op if `pid` is absent.
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
    /// (de)allocating) — a use-after-free that crashed the multi-child demo.
    /// Deferring the drop to the caller (post-lock, post-shootdown) mirrors the
    /// exit path. On 1-vCPU the shootdown is a no-op and the child is dropped
    /// immediately after the syscall returns, so the golden trace is unchanged.
    #[must_use = "drop the reaped process only after a cross-CPU TLB shootdown"]
    pub fn try_reap(&mut self, target: i64) -> Option<(u32, i32, Process)> {
        let parent_pid = self.current;
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
    /// into the parent's saved context (`rax = child pid`, write `*status` into
    /// the parent's address space), and mark the parent `Runnable` so it resumes
    /// from just after its `wait4` syscall with the correct return value.
    ///
    /// The reaped child [`Process`]es are **moved out** of the table and returned
    /// rather than dropped here, because dropping a child frees its page tables /
    /// frames — which must not happen while that child's address space is still
    /// the live `CR3`. The caller switches `CR3` to a surviving process first,
    /// *then* drops the returned vector.
    #[must_use = "drop the reaped processes only after switching CR3 away from them"]
    pub fn complete_waits(&mut self) -> Vec<Process> {
        let mut reaped_procs: Vec<Process> = Vec::new();
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
                if let Some(child) = self.procs.table[cpid as usize].take() {
                    reaped_procs.push(child);
                }
                let mut woke = false;
                if let Some(parent) = self.get_mut(parent_pid) {
                    parent.children.retain(|&c| c != cpid);
                    parent.state = State::Runnable;
                    parent.ctx.rax = cpid as u64; // wait4 return = child pid
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
                    // P4·SMP·S4a: a just-woken (Waiting -> Runnable) parent is put
                    // back on THIS CPU's run queue so the local pick_next (or, in
                    // S4b, a steal) re-runs it. On 1-vCPU this is CPU 0 and the
                    // parent is popped right back here by the post-wait4 schedule,
                    // matching the old global round-robin → golden byte-identical.
                    runq_push(self.cpu, parent_pid);
                }
            }
        }
        reaped_procs
    }
}

// ---- Context switching ----------------------------------------------------

/// Save the live `frame` into the current process's `ctx`.
///
/// The current process may have just exited *and been reaped* (its slot freed)
/// before we got here — in that case there is nothing to save.
pub fn save_current(frame: &Context) {
    with_sched(|s| {
        let cur_pid = s.current;
        if let Some(cur) = s.get_mut(cur_pid) {
            cur.ctx = *frame;
        }
    });
}

/// True iff ANY process that can still make progress exists (P4·SMP·S4a): a
/// `Runnable`, `Running` (on some CPU) or `Waiting` (blocked, wakeable) process.
/// A lone `Zombie` (no live parent left to reap it) is NOT "alive" — it cannot
/// run and nothing will wait on it, so the workload is done.
///
/// The machine-wide termination condition: under `-smp`, a CPU whose `schedule`
/// returns `false` (no Runnable *for it*) must NOT power the machine off if
/// another CPU is still running a process. On 1-vCPU this is `false` exactly when
/// the old `runnable_count()==0` power-off check fired (the workloads always exit
/// with an empty table), so the BSP power-off decision is unchanged.
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

/// Install the address space of `pid` — which the caller has ALREADY claimed for
/// this CPU (set `current` + `State::Running` atomically via `pick_and_claim`) —
/// by loading its `CR3` and copying its saved `ctx` into the live `frame` so the
/// entry stub's `sysretq`/`iretq` resumes it.
///
/// The SMP ownership bookkeeping (claim/demote/enqueue) is NOT done here: it must
/// happen ATOMICALLY with the pick under one `PROCS` lock (`pick_and_claim`), or
/// two CPUs could select the same pid in the gap between pick and claim. This
/// function only READS the (already-claimed) target's AS, briefly under the lock,
/// then performs the CR3 write + frame rewrite with NO lock held — the lock is
/// never held across the `Cr3::write` + `iretq`.
///
/// # Safety
/// `pid` must be this CPU's claimed `current` (`pick_and_claim` ran). `frame` is
/// the live register frame the stub restores from; this rewrites it + edits CR3.
pub unsafe fn switch_to(pid: u32, frame: &mut Context) {
    // Read the (already-claimed) target's (cr3, ctx, fs_base) under the lock; the
    // guard drops at the closure end so the CR3 write + iretq below run lock-free.
    let (cr3, ctx, fs_base) = with_sched(|s| {
        debug_assert_eq!(s.current, pid, "switch_to target must be the claimed current");
        let p = s.get(pid).expect("switch target missing");
        (p.space.cr3(), p.ctx, p.fs_base)
    });

    // SAFETY: `cr3` is a process PML4 whose kernel-half entries re-create the
    // boot identity map, so ring 0 stays addressable across the switch. Writing
    // CR3 performs the full TLB flush we need for the address-space change.
    unsafe {
        let frame_phys = PhysFrame::containing_address(PhysAddr::new(cr3));
        Cr3::write(frame_phys, Cr3Flags::empty());
    }

    // THE FIX: restore the incoming process's `%fs` base into IA32_FS_BASE so it
    // resumes with ITS OWN musl TLS — not whatever the *previous* process left in
    // the (single, global) MSR. Without this, two concurrent musl processes that
    // each `arch_prctl(ARCH_SET_FS)` a distinct base clobber each other: after a
    // preemption the incoming process reads the outgoing one's TLS -> corruption.
    // Only write when nonzero: a process that never called `arch_prctl` (fs_base
    // == 0, e.g. a no_std PID1) must leave the MSR untouched, exactly as before.
    // The swapgs/GS handling on the syscall/IRQ paths is unaffected — FS base is
    // a separate MSR and `swapgs` only swaps GS.
    //
    // FALSIFIABILITY TOGGLE: `--features fsbase-demo-nofix` compiles this restore
    // OUT, reproducing the original bug so the demo's WITHOUT-fix serial shows the
    // real TLS corruption/crash (proving the demo actually exercises the fix).
    #[cfg(not(feature = "fsbase-demo-nofix"))]
    if fs_base != 0 {
        // SAFETY: writing IA32_FS_BASE sets the user `%fs` base for the process
        // we are switching to. It only affects ring-3 `%fs:` accesses, never
        // kernel state; the value is the TLS pointer that process itself set via
        // `arch_prctl(ARCH_SET_FS)`. Same write the CPU's WRFSBASE would do.
        unsafe {
            Msr::new(IA32_FS_BASE).write(fs_base);
        }
    }
    #[cfg(feature = "fsbase-demo-nofix")]
    let _ = fs_base; // bug-reproduction build: deliberately do NOT restore.

    *frame = ctx;
}

/// Pick the next runnable process and switch the live `frame` to it. Returns
/// `true` if a switch happened (or the current keeps running), `false` if no
/// process remains runnable at all.
///
/// # Safety
/// `frame` is the live register frame; may be rewritten by `switch_to`.
pub unsafe fn schedule(frame: &mut Context) -> bool {
    // Pick AND claim the next pid ATOMICALLY under one `PROCS` lock: it is marked
    // `Running` (owned by this CPU) before the lock is released, so a concurrent
    // `schedule` on another CPU cannot also select it (the cross-CPU double-run
    // race). `switch_to` then only reads the claimed target's AS.
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
