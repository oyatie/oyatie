// Pure, hardware-independent layout/allocator math for the EL0 user Frame.
//
// This file is the body of the `user_layout` crate (it is `include!`d from
// `lib.rs`, which supplies the `#![no_std]` attribute). It is the **single
// source of truth** — shared by every arch Frame backend (aarch64 today,
// x86_64 / riscv64 later) — for the parts of the user bring-up that are *pure
// functions* of their inputs: the VMSAv8-64 descriptor bit layout, the
// on-demand frame-pool bookkeeping, user-pointer range validation, and — most
// importantly — the Linux/SysV aarch64 **initial process stack**
// (argc/argv/envp/auxv) builder.
//
// Everything here is `no_std`-friendly and depends on **nothing** outside
// `core`. That is deliberate: it lets each arch Frame depend on this crate (the
// aarch64 Frame's `user.rs`/`process.rs` call into it) *and* lets a tiny
// out-of-workspace host harness `include!` the very same file and run
// `mod tests` under `cargo test` — so the layout math is verified on the host
// without the `build-std` / register-access machinery the arch crates need.
//
// See `crates/arch-aarch64/tests-host/` for that harness.
//
// NOTE: this file deliberately uses **no inner attributes** (`#![...]`) or
// `//!` module docs, because it is `include!`d both into this crate's `lib.rs`
// body and into the host harness's module body, where inner attributes are not
// permitted. The crate-level `#![no_std]` lives in `lib.rs` instead.
//
// Keeping this logic pure also keeps the `unsafe` Frames thin: they only do the
// things that *must* be unsafe (touch page tables, copy into user memory,
// `eret`), delegating all the fiddly arithmetic here where it can be
// exhaustively tested.

// ---- Descriptor bit fields (ARMv8-A VMSAv8-64, 4 KiB granule) -------------

/// Descriptor valid bit.
pub const DESC_VALID: u64 = 1 << 0;
/// Bit[1]=1 marks a table descriptor (at L1/L2) or a page descriptor (at L3).
pub const DESC_TABLE_OR_PAGE: u64 = 1 << 1;
/// Bit[1]=0 marks a block descriptor (at L1/L2).
pub const DESC_BLOCK: u64 = 0 << 1;
/// Access flag — must be set or the first access faults.
pub const DESC_AF: u64 = 1 << 10;
/// Inner-shareable (`0b11`) at bits[9:8].
pub const DESC_SH_INNER: u64 = 0b11 << 8;

/// AP[2:1] = 0b00 -> RW at EL1, no access at EL0.
pub const DESC_AP_EL1_RW: u64 = 0b00 << 6;
/// AP[2:1] = 0b01 -> RW at EL1 *and* EL0.
pub const DESC_AP_EL0_RW: u64 = 0b01 << 6;
/// AP[2:1] = 0b11 -> RO at EL1 *and* EL0.
pub const DESC_AP_EL0_RO: u64 = 0b11 << 6;

/// Privileged Execute-Never (bit 53): EL1 may not execute from this page.
pub const DESC_PXN: u64 = 1 << 53;
/// Unprivileged Execute-Never (bit 54): EL0 may not execute from this page.
pub const DESC_UXN: u64 = 1 << 54;

/// MAIR attribute index for Device memory (matches `mmu.rs` Attr0).
pub const ATTR_DEVICE_IDX: u64 = 0;
/// MAIR attribute index for Normal cacheable memory (matches `mmu.rs` Attr1).
pub const ATTR_NORMAL_IDX: u64 = 1;

/// Mask selecting the output-address bits of a 4 KiB descriptor.
pub const DESC_ADDR_MASK: u64 = 0x0000_FFFF_FFFF_F000;

// ---- Window geometry ------------------------------------------------------

/// 4 KiB page size.
pub const PAGE_SIZE: usize = 0x1000;
/// 4 KiB page mask.
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

/// User window base — the 2 MiB-aligned block holding the ELF link address.
/// Static musl aarch64 binaries link at `0x40_0000`.
pub const USER_BASE: usize = 0x0040_0000;

/// Span covered by **one** leaf page table (512 × 4 KiB = 2 MiB). One L3 table
/// (aarch64) or one PT (x86_64) maps exactly this much, and it is the alignment
/// granule of the L2/PD slots the window occupies.
pub const TABLE_SPAN: usize = 0x0020_0000; // 2 MiB
/// Number of 4 KiB entries in one leaf page table.
pub const PT_ENTRIES: usize = TABLE_SPAN / PAGE_SIZE; // 512

/// Number of consecutive 2 MiB leaf tables the user window spans. Enlarged from
/// the original single table (2 MiB) to **4 tables (8 MiB)** so the much larger
/// real `talos-init` musl image (~1.1–1.2 MiB of PT_LOADs) leaves room for a
/// real brk heap, an mmap region, and the initial stack above it. Each arch
/// backend allocates this many leaf tables and points this many consecutive
/// L2/PD slots at them; the per-page W^X permission model is unchanged.
pub const USER_NTABLES: usize = 4;

/// User window size: `USER_NTABLES` leaf tables (4 × 2 MiB = 8 MiB).
pub const USER_SPAN: usize = USER_NTABLES * TABLE_SPAN;
/// One past the top of the window. The initial process stack starts just below.
pub const USER_TOP: usize = USER_BASE + USER_SPAN;
/// Number of 4 KiB pages the window spans (== total leaf entries we manage).
pub const USER_NPAGES: usize = USER_SPAN / PAGE_SIZE; // 2048

/// Initial process stack size (256 KiB).
pub const STACK_SIZE: usize = 256 * 1024;
/// Stack top = top of the window (grows down).
pub const USER_STACK_TOP: usize = USER_TOP;
/// Lowest stack address we pre-map (the rest of the window is heap/mmap).
pub const USER_STACK_BOTTOM: usize = USER_STACK_TOP - STACK_SIZE;

/// Base of the anonymous-`mmap` region inside the window (above brk, below
/// stack); the bump cursor grows up from here. Placed 4 MiB above `USER_BASE`
/// so it sits clear above the largest real-init image (the x86_64 talos-init's
/// PT_LOADs reach ~`USER_BASE + 0x124000`), leaving the whole `[image_end,
/// MMAP_BASE)` gap as brk heap.
pub const MMAP_BASE: usize = USER_BASE + 0x40_0000; // 0x80_0000
/// One past the mmap region (leave the top 256 KiB for the stack).
pub const MMAP_TOP: usize = USER_STACK_BOTTOM;

// ---- Descriptor builders --------------------------------------------------

/// 2 MiB Device block descriptor mapping identity at `pa` (EL1-only).
pub const fn l2_device_block(pa: u64) -> u64 {
    pa | DESC_VALID
        | DESC_BLOCK
        | DESC_AF
        | DESC_SH_INNER
        | DESC_AP_EL1_RW
        | DESC_PXN
        | DESC_UXN
        | (ATTR_DEVICE_IDX << 2)
}

/// 4 KiB Normal page descriptor at `pa` with the given access/exec attributes.
pub const fn l3_normal_page(pa: u64, ap: u64, pxn: u64, uxn: u64) -> u64 {
    pa | DESC_VALID
        | DESC_TABLE_OR_PAGE // bit[1]=1 means "page" at L3
        | DESC_AF
        | DESC_SH_INNER
        | ap
        | pxn
        | uxn
        | (ATTR_NORMAL_IDX << 2)
}

/// Table descriptor pointing at the next-level table at physical `pa`.
pub const fn table_desc(pa: u64) -> u64 {
    pa | DESC_VALID | DESC_TABLE_OR_PAGE
}

/// Per-page access permission for a mapped user page.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(clippy::enum_variant_names)] // the shared `Read` prefix reads clearly here
pub enum PagePerm {
    /// Executable read-only (text + rodata).
    ReadExec,
    /// Writable, non-executable (stack / data / bss / heap / mmap).
    ReadWrite,
    /// Read-only, non-executable (relro, AT_PHDR copy, AT_RANDOM page).
    ReadOnly,
    /// Writable **and** executable. Needed only when a single 4 KiB page is
    /// straddled by two ELF segments of differing access — e.g. a small static
    /// binary whose `PT_LOAD` for code (RX) and the next `PT_LOAD` for data (RW)
    /// share a page. Linux maps such a page with the union of permissions; we do
    /// the same rather than refuse to load. (For our page-aligned musl images
    /// this never arises; it is the safe union when it does.)
    ReadWriteExec,
}

impl PagePerm {
    /// True if this permission grants EL0 execute.
    pub fn is_exec(self) -> bool {
        matches!(self, PagePerm::ReadExec | PagePerm::ReadWriteExec)
    }

    /// True if this permission grants EL0 write.
    pub fn is_write(self) -> bool {
        matches!(self, PagePerm::ReadWrite | PagePerm::ReadWriteExec)
    }

    /// The least-privilege permission granting the union of `self` and `other`'s
    /// capabilities. Used when two ELF segments share a page: the page must
    /// satisfy *both*, so a code+data overlap becomes RWX, code+rodata stays RX,
    /// and data+rodata stays RW. This is what re-mapping an already-mapped page
    /// must do so an earlier RX mapping is never silently downgraded to RO.
    pub fn merge(self, other: PagePerm) -> PagePerm {
        let exec = self.is_exec() || other.is_exec();
        let write = self.is_write() || other.is_write();
        match (write, exec) {
            (true, true) => PagePerm::ReadWriteExec,
            (true, false) => PagePerm::ReadWrite,
            (false, true) => PagePerm::ReadExec,
            (false, false) => PagePerm::ReadOnly,
        }
    }

    /// The L3 descriptor attribute bits for this permission, applied to `pa`.
    pub fn desc(self, pa: u64) -> u64 {
        match self {
            // EL0 read+exec; EL1 must not execute user pages (PXN).
            PagePerm::ReadExec => l3_normal_page(pa, DESC_AP_EL0_RO, DESC_PXN, 0),
            // EL0 read+write, never executable (PXN+UXN).
            PagePerm::ReadWrite => l3_normal_page(pa, DESC_AP_EL0_RW, DESC_PXN, DESC_UXN),
            // EL0 read-only, never executable.
            PagePerm::ReadOnly => l3_normal_page(pa, DESC_AP_EL0_RO, DESC_PXN, DESC_UXN),
            // EL0 read+write+exec (overlap page): EL1 still PXN, UXN clear.
            PagePerm::ReadWriteExec => l3_normal_page(pa, DESC_AP_EL0_RW, DESC_PXN, 0),
        }
    }

    /// Recover the [`PagePerm`] encoded in an existing L3 descriptor (the inverse
    /// of [`desc`] for our four cases), so [`merge`] can union a fresh mapping
    /// with whatever is already installed on a shared page.
    pub fn from_desc(desc: u64) -> PagePerm {
        let write = (desc & (0b11 << 6)) == DESC_AP_EL0_RW;
        let exec = (desc & DESC_UXN) == 0;
        match (write, exec) {
            (true, true) => PagePerm::ReadWriteExec,
            (true, false) => PagePerm::ReadWrite,
            (false, true) => PagePerm::ReadExec,
            (false, false) => PagePerm::ReadOnly,
        }
    }
}

// ---- Range validation -----------------------------------------------------

/// Flat leaf-page index for a user virtual address inside the window, counted
/// from `USER_BASE` across **all** `USER_NTABLES` leaf tables (0..USER_NPAGES).
pub fn l3_index(va: usize) -> usize {
    (va - USER_BASE) / PAGE_SIZE
}

/// Split a flat leaf-page index into `(table_idx, entry_idx)`: which of the
/// `USER_NTABLES` consecutive 2 MiB leaf tables backs the page, and the
/// 0..PT_ENTRIES slot within that table. The single source of truth for how the
/// (now multi-table) user window addresses its leaves; every arch backend routes
/// `map_page`/`translate`/`cow` through this so the window can grow by changing
/// `USER_NTABLES` alone.
pub fn l3_slot(va: usize) -> (usize, usize) {
    let flat = l3_index(va);
    (flat / PT_ENTRIES, flat % PT_ENTRIES)
}

/// True iff `[ptr, ptr+len)` lies entirely inside the user window. Written to
/// be overflow-safe: `len <= top-base` and `ptr <= top-len` together imply the
/// whole range is in `[base, top)` with no wraparound.
pub fn user_range_ok(ptr: u64, len: u64) -> bool {
    let base = USER_BASE as u64;
    let top = USER_TOP as u64;
    ptr >= base && len <= top - base && ptr <= top - len
}

// ---- Pure frame-pool bookkeeping ------------------------------------------

/// The pure index bookkeeping behind the on-demand frame pool. The Frame's
/// `alloc_frame` owns the actual `.bss` storage and turns an index into a
/// physical address; *this* type just hands out monotonically increasing frame
/// indices and enforces the capacity ceiling, so the allocation policy is
/// host-testable in isolation.
#[derive(Clone, Copy, Debug)]
pub struct FrameAllocator {
    next: usize,
    capacity: usize,
}

impl FrameAllocator {
    /// A fresh allocator over `capacity` frames.
    pub const fn new(capacity: usize) -> Self {
        Self { next: 0, capacity }
    }

    /// Allocate the next frame index, or `None` if the pool is exhausted.
    pub fn alloc(&mut self) -> Option<usize> {
        if self.next >= self.capacity {
            return None;
        }
        let idx = self.next;
        self.next += 1;
        Some(idx)
    }

    /// Number of frames handed out so far.
    pub fn used(&self) -> usize {
        self.next
    }

    /// Number of frames still available.
    pub fn remaining(&self) -> usize {
        self.capacity - self.next
    }

    /// Byte offset of frame `idx` within a pool of `PAGE_SIZE`-sized frames.
    pub const fn frame_offset(idx: usize) -> usize {
        idx * PAGE_SIZE
    }
}

// ---- Thread-local storage (aarch64 TLS variant I) -------------------------
//
// AArch64 uses **TLS variant I** (the "TLS above the thread pointer" variant).
// The thread pointer lives in `TPIDR_EL0`. A fixed-size **TCB** (Thread Control
// Block) of exactly two pointers sits *at* the thread pointer, and the module's
// static TLS block (the `PT_TLS` image: `.tdata` initialised data followed by
// `.tbss` zero-init data) is laid out **immediately above** it. So for a static
// executable the picture in memory is, low -> high address:
//
// ```text
//   tp (TPIDR_EL0) ->  [ TCB: 2 pointers = 16 bytes ]
//   tp + TCB_SIZE  ->  [ .tdata image (p_filesz bytes) ]
//                      [ .tbss      (p_memsz - p_filesz bytes, zeroed) ]
// ```
//
// A thread-local variable whose PT_TLS-relative offset is `k` is therefore
// accessed by the compiler/musl as `*(tp + TCB_SIZE + k)`. This matches the
// AArch64 ELF TLS ABI and musl's `__init_tp`/`__copy_tls` for `TLS_ABOVE_TP`:
// musl reserves `2*sizeof(void*)` above the thread pointer (`GAP_ABOVE_TP`) and
// places the TLS image just past it.
//
// Two alignment rules must both hold:
//   1. `tp` (and hence the TCB) is at least pointer-aligned; we use 16-byte
//      alignment so the two-pointer TCB is naturally aligned.
//   2. The **TLS image start** (`tp + TCB_SIZE`) must be aligned to the PT_TLS
//      `p_align`. Because `TCB_SIZE` (16) is itself a multiple of every
//      `p_align` we expect (musl/aarch64 TLS alignments are <= 16 and powers of
//      two: 8 or 16), aligning `tp` to `max(16, p_align)` makes both the TCB
//      and the image start correctly aligned.
//
// The kernel pre-initialises this block before `eret` so that musl's startup —
// which reads `TPIDR_EL0` and dereferences the TCB (e.g. the stack-guard / self
// pointer) *before* it has run its own `__init_tp` — sees a valid, zeroed TCB
// and a correctly initialised TLS image rather than faulting on a wild TP.

/// Size of the AArch64 variant-I TCB that sits at the thread pointer: two
/// pointers (DTV pointer + private/self slot), i.e. 16 bytes on LP64.
pub const TCB_SIZE: usize = 2 * 8;

/// Minimum alignment we give the TLS region as a whole (and thus the thread
/// pointer). The TCB is two 8-byte pointers, so 16 keeps it naturally aligned
/// and — since `TCB_SIZE` is a multiple of it — keeps the image start aligned
/// to any `p_align` <= 16.
pub const TLS_TP_ALIGN: usize = 16;

/// The parsed `PT_TLS` program header the loader hands to the layout math.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TlsPhdr {
    /// `p_filesz`: bytes of `.tdata` initialised image to copy.
    pub filesz: usize,
    /// `p_memsz`: total TLS size; `[filesz, memsz)` is `.tbss` (zeroed).
    pub memsz: usize,
    /// `p_align`: required alignment of the TLS image start. Must be a power of
    /// two (0 or 1 are treated as "no extra alignment").
    pub align: usize,
}

/// The computed placement of the TLS region inside the user window.
///
/// The region is `[tp, tp + total)`. `tp` is the value to load into
/// `TPIDR_EL0`; the TCB occupies `[tp, tp + TCB_SIZE)`; the TLS image occupies
/// `[image_va, image_va + memsz)` with `image_va == tp + TCB_SIZE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TlsLayout {
    /// Thread-pointer value for `TPIDR_EL0` (start of the TCB).
    pub tp: usize,
    /// VA where the TLS image (`.tdata` then `.tbss`) begins (`tp + TCB_SIZE`).
    pub image_va: usize,
    /// Bytes of `.tdata` to copy from the file image (`p_filesz`).
    pub filesz: usize,
    /// Total TLS image bytes (`p_memsz`); the `.tbss` tail is zeroed.
    pub memsz: usize,
    /// One past the end of the whole region (TCB + image), page-rounded up.
    /// The caller maps `[tp, region_end)` and may start later allocations here.
    pub region_end: usize,
}

/// Round `x` up to the next multiple of `align` (a power of two). `align <= 1`
/// is a no-op.
pub const fn align_up(x: usize, align: usize) -> usize {
    if align <= 1 {
        x
    } else {
        (x + (align - 1)) & !(align - 1)
    }
}

/// Compute the variant-I TLS placement for a PT_TLS header, starting the region
/// at `base` (a VA at or above which the whole TCB + image must fit).
///
/// The thread pointer is placed at the first address `>= base` that is aligned
/// to `max(TLS_TP_ALIGN, p_align)`. Because `TCB_SIZE` is a multiple of every
/// such alignment, `tp + TCB_SIZE` (the image start) is then also `p_align`-
/// aligned, satisfying the ABI for every TLS access `tp + TCB_SIZE + k`.
///
/// Returns `region_end` page-rounded so the caller can resume allocating above.
pub fn compute_tls_layout(base: usize, tls: &TlsPhdr) -> TlsLayout {
    // Effective alignment for the thread pointer: at least the TCB's natural
    // 16, but honour a larger PT_TLS alignment if the linker asked for one.
    let want_align = if tls.align > TLS_TP_ALIGN {
        tls.align
    } else {
        TLS_TP_ALIGN
    };
    let tp = align_up(base, want_align);
    let image_va = tp + TCB_SIZE;
    let region_end = align_up(image_va + tls.memsz, PAGE_SIZE);
    TlsLayout {
        tp,
        image_va,
        filesz: tls.filesz,
        memsz: tls.memsz,
        region_end,
    }
}

// ---- Timer-sleep math (pure; host-tested) ---------------------------------

/// Maximum `tv_nsec` accepted by `nanosleep`/`clock_nanosleep`. Linux rejects a
/// `tv_nsec` outside `0..1_000_000_000` with `-EINVAL`; we mirror that.
pub const NSEC_PER_SEC: i64 = 1_000_000_000;

/// Outcome of validating + converting a user `timespec` sleep request into a
/// number of generic-timer **counter cycles** to wait.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SleepCycles {
    /// The request was well-formed; wait this many counter cycles (0 = return
    /// immediately, e.g. a zero-duration sleep).
    Wait(u64),
    /// `tv_sec < 0` or `tv_nsec` outside `[0, 1e9)` — caller returns `-EINVAL`.
    Invalid,
}

/// Convert a `timespec` (`tv_sec`, `tv_nsec`) into a count of generic-timer
/// cycles to busy-wait, given the counter `frequency` in Hz.
///
/// This is the pure core of the `nanosleep`/`clock_nanosleep`/`ppoll`-timeout
/// path: it does *only* validation + the (sec, nsec, Hz) -> cycles arithmetic,
/// so it can be exhaustively unit-tested on the host without any hardware.
///
/// Cycles are computed in `u128` to avoid overflow for multi-year sleeps, then
/// saturated into `u64` (the counter is 64-bit; a saturated wait is still a
/// monotonically-correct upper bound and the caller re-checks the live counter).
/// A `frequency` of 0 is treated as 1 to avoid division by zero (degenerate
/// hardware); the result is then dominated by the seconds term.
pub fn timespec_to_cycles(tv_sec: i64, tv_nsec: i64, frequency: u64) -> SleepCycles {
    if tv_sec < 0 || tv_nsec < 0 || tv_nsec >= NSEC_PER_SEC {
        return SleepCycles::Invalid;
    }
    let freq = frequency.max(1) as u128;
    // cycles = sec*freq + nsec*freq/1e9, all in u128.
    let sec_cycles = (tv_sec as u128) * freq;
    let nsec_cycles = ((tv_nsec as u128) * freq) / (NSEC_PER_SEC as u128);
    let total = sec_cycles.saturating_add(nsec_cycles);
    SleepCycles::Wait(if total > u64::MAX as u128 {
        u64::MAX
    } else {
        total as u64
    })
}

/// Compute the counter value a wait should run *until*, saturating at `u64::MAX`
/// so a near-end-of-counter deadline never wraps to a tiny value (which would
/// end the wait early). Pure so the wrap behaviour is unit-tested.
pub fn deadline_after(now: u64, cycles: u64) -> u64 {
    now.saturating_add(cycles)
}

// ---- Auxiliary-vector tags (Linux `elf.h`) --------------------------------

pub const AT_NULL: u64 = 0;
pub const AT_PHDR: u64 = 3;
pub const AT_PHENT: u64 = 4;
pub const AT_PHNUM: u64 = 5;
pub const AT_PAGESZ: u64 = 6;
pub const AT_BASE: u64 = 7;
pub const AT_FLAGS: u64 = 8;
pub const AT_ENTRY: u64 = 9;
pub const AT_UID: u64 = 11;
pub const AT_EUID: u64 = 12;
pub const AT_GID: u64 = 13;
pub const AT_EGID: u64 = 14;
pub const AT_HWCAP: u64 = 16;
pub const AT_CLKTCK: u64 = 17;
pub const AT_SECURE: u64 = 23;
pub const AT_RANDOM: u64 = 25;

// ---- Pure initial-stack image builder -------------------------------------

/// The inputs the stack builder needs from the loaded ELF + environment. All
/// values are plain integers / slices so this is a pure function.
#[derive(Clone, Copy, Debug)]
pub struct StackInputs<'a> {
    /// `e_entry` of the loaded program (for `AT_ENTRY`).
    pub entry: u64,
    /// User VA of the ELF program-header copy (for `AT_PHDR`).
    pub phdr_va: u64,
    /// `e_phentsize` (for `AT_PHENT`).
    pub phentsize: u64,
    /// `e_phnum` (for `AT_PHNUM`).
    pub phnum: u64,
    /// argv strings, each a NUL-terminated byte slice.
    pub argv: &'a [&'a [u8]],
    /// envp strings, each a NUL-terminated byte slice.
    pub envp: &'a [&'a [u8]],
    /// The 16 AT_RANDOM bytes.
    pub random: &'a [u8],
    /// Top of the stack (exclusive); the blob+vector are laid out below this.
    pub stack_top: usize,
}

/// The fully-computed initial-stack image: a flat list of writes plus the final
/// 16-byte-aligned `SP_EL0`. The Frame just copies each `(va, bytes)` into user
/// memory and `eret`s with `sp`.
#[derive(Clone, Debug)]
pub struct StackImage {
    /// The argc/argv/envp/auxv machine words (LE u64s), to be written at `sp`.
    pub words: heapless_vec::Vec64,
    /// Final `SP_EL0` (16-byte aligned), pointing at `argc`.
    pub sp: usize,
    /// VA where the AT_RANDOM bytes were placed.
    pub random_va: usize,
    /// VA of each argv string, in order.
    pub argv_vas: heapless_vec::VecUsize8,
    /// VA of each envp string, in order.
    pub envp_vas: heapless_vec::VecUsize8,
}

/// A couple of tiny fixed-capacity vectors so the builder needs no allocator.
pub mod heapless_vec {
    /// Up to 64 machine words (argc/argv/envp/auxv). Plenty for the demo.
    #[derive(Clone, Debug)]
    pub struct Vec64 {
        buf: [u64; 64],
        len: usize,
    }
    impl Vec64 {
        pub const fn new() -> Self {
            Self {
                buf: [0; 64],
                len: 0,
            }
        }
        /// Push a word; returns false if full (caller asserts capacity).
        pub fn push(&mut self, v: u64) -> bool {
            if self.len >= self.buf.len() {
                return false;
            }
            self.buf[self.len] = v;
            self.len += 1;
            true
        }
        pub fn as_slice(&self) -> &[u64] {
            &self.buf[..self.len]
        }
        pub fn len(&self) -> usize {
            self.len
        }
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }
    }
    impl Default for Vec64 {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Up to 8 string VAs (argv or envp).
    #[derive(Clone, Debug)]
    pub struct VecUsize8 {
        buf: [usize; 8],
        len: usize,
    }
    impl VecUsize8 {
        pub const fn new() -> Self {
            Self {
                buf: [0; 8],
                len: 0,
            }
        }
        pub fn push(&mut self, v: usize) -> bool {
            if self.len >= self.buf.len() {
                return false;
            }
            self.buf[self.len] = v;
            self.len += 1;
            true
        }
        pub fn as_slice(&self) -> &[usize] {
            &self.buf[..self.len]
        }
        pub fn len(&self) -> usize {
            self.len
        }
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }
    }
    impl Default for VecUsize8 {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Compute the Linux/SysV aarch64 initial-stack image for `inp`.
///
/// Layout (low -> high; SP points at the lowest word, `argc`):
///
/// ```text
///   argc, argv[..], NULL, envp[..], NULL, (auxv tag,val).., AT_NULL,0
///   --- strings + AT_RANDOM bytes live at the very top, above the vector ---
/// ```
///
/// The strings/AT_RANDOM blob is placed at the top of the stack; the fixed-size
/// machine vector is placed below it with `argc` rounded **down** to a 16-byte
/// boundary so `SP_EL0 % 16 == 0` at EL0 entry (the aarch64 procedure call
/// standard requires this).
///
/// Panics only on caller misuse (too many argv/envp, or a vector exceeding the
/// 64-word buffer) — all of which are static for this build.
pub fn build_stack_image(inp: &StackInputs) -> StackImage {
    let mut top = inp.stack_top;

    // 1. AT_RANDOM bytes at the very top (16-byte aligned).
    top -= inp.random.len();
    top &= !0xf;
    let random_va = top;

    // 2. envp strings (high to low so as_slice order matches push order below).
    let mut envp_vas = heapless_vec::VecUsize8::new();
    // Lay them out from the end so earlier env vars sit at lower addresses; we
    // record VAs in env order. Compute total then assign.
    let mut envp_tmp = [0usize; 8];
    for (i, s) in inp.envp.iter().enumerate() {
        top -= s.len();
        envp_tmp[i] = top;
    }
    // The loop above placed env[0] highest; we want env[0] lowest in address
    // order? Order within argv/envp address space does not matter to the ABI as
    // long as the pointer array matches the strings. We keep the natural
    // "env[0] at the highest of the env block" placement and record in order.
    for (i, _s) in inp.envp.iter().enumerate() {
        assert!(envp_vas.push(envp_tmp[i]), "too many envp");
    }

    // 3. argv strings.
    let mut argv_vas = heapless_vec::VecUsize8::new();
    let mut argv_tmp = [0usize; 8];
    for (i, s) in inp.argv.iter().enumerate() {
        top -= s.len();
        argv_tmp[i] = top;
    }
    for (i, _s) in inp.argv.iter().enumerate() {
        assert!(argv_vas.push(argv_tmp[i]), "too many argv");
    }

    // 4. The auxv pairs.
    let auxv: [(u64, u64); 16] = [
        (AT_PHDR, inp.phdr_va),
        (AT_PHENT, inp.phentsize),
        (AT_PHNUM, inp.phnum),
        (AT_PAGESZ, PAGE_SIZE as u64),
        (AT_BASE, 0),
        (AT_FLAGS, 0),
        (AT_ENTRY, inp.entry),
        (AT_HWCAP, 0),
        (AT_CLKTCK, 100),
        (AT_UID, 0),
        (AT_EUID, 0),
        (AT_GID, 0),
        (AT_EGID, 0),
        (AT_SECURE, 0),
        (AT_RANDOM, random_va as u64),
        (AT_NULL, 0),
    ];

    // 5. Word count below `argc`:
    //    1 (argc) + argv + 1 (NULL) + envp + 1 (NULL) + 2*auxv_pairs.
    let n_words = 1 + inp.argv.len() + 1 + inp.envp.len() + 1 + 2 * auxv.len();

    // 6. Place argc so the final SP is 16-byte aligned.
    let mut sp = top - n_words * 8;
    sp &= !0xf;

    // 7. Materialise the word vector.
    let mut words = heapless_vec::Vec64::new();
    assert!(words.push(inp.argv.len() as u64), "vector overflow"); // argc
    for &p in argv_vas.as_slice() {
        assert!(words.push(p as u64), "vector overflow");
    }
    assert!(words.push(0), "vector overflow"); // argv NULL
    for &p in envp_vas.as_slice() {
        assert!(words.push(p as u64), "vector overflow");
    }
    assert!(words.push(0), "vector overflow"); // envp NULL
    for &(tag, val) in &auxv {
        assert!(words.push(tag), "vector overflow");
        assert!(words.push(val), "vector overflow");
    }
    debug_assert_eq!(words.len(), n_words);

    StackImage {
        words,
        sp,
        random_va,
        argv_vas,
        envp_vas,
    }
}

// ---- Unit tests (run on the host via the out-of-workspace harness) --------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_bits_are_well_formed() {
        // A device block carries the device attr index, is EL1-only RW, and is
        // both PXN and UXN.
        let d = l2_device_block(0x4000_0000);
        assert_eq!(d & DESC_ADDR_MASK, 0x4000_0000);
        assert_ne!(d & DESC_VALID, 0);
        assert_eq!(d & (1 << 1), DESC_BLOCK); // block, not table
        assert_ne!(d & DESC_PXN, 0);
        assert_ne!(d & DESC_UXN, 0);
        assert_eq!((d >> 2) & 0b111, ATTR_DEVICE_IDX);

        // A table descriptor preserves the next-table address and sets bit[1].
        let t = table_desc(0x12_3000);
        assert_eq!(t & DESC_ADDR_MASK, 0x12_3000);
        assert_ne!(t & DESC_TABLE_OR_PAGE, 0);
    }

    #[test]
    fn page_perms_enforce_w_xor_x() {
        let pa = 0x40_5000u64;

        // ReadExec: EL0 RO (AP=0b11), executable at EL0 (UXN clear), PXN set.
        let rx = PagePerm::ReadExec.desc(pa);
        assert_eq!(rx & (0b11 << 6), DESC_AP_EL0_RO);
        assert_eq!(rx & DESC_UXN, 0, "text must be executable at EL0");
        assert_ne!(rx & DESC_PXN, 0, "EL1 must not exec user text");

        // ReadWrite: EL0 RW, never executable.
        let rw = PagePerm::ReadWrite.desc(pa);
        assert_eq!(rw & (0b11 << 6), DESC_AP_EL0_RW);
        assert_ne!(rw & DESC_UXN, 0, "data must not be executable");
        assert_ne!(rw & DESC_PXN, 0);

        // ReadOnly: EL0 RO, never executable.
        let ro = PagePerm::ReadOnly.desc(pa);
        assert_eq!(ro & (0b11 << 6), DESC_AP_EL0_RO);
        assert_ne!(ro & DESC_UXN, 0);
    }

    #[test]
    fn page_perm_merge_unions_capabilities() {
        use PagePerm::*;
        // RX + RO (code + rodata sharing a page) stays executable, not writable.
        assert_eq!(ReadExec.merge(ReadOnly), ReadExec);
        assert_eq!(ReadOnly.merge(ReadExec), ReadExec);
        // RW + RO stays writable.
        assert_eq!(ReadWrite.merge(ReadOnly), ReadWrite);
        // RX + RW (code + data sharing a page) becomes RWX (the safe union).
        assert_eq!(ReadExec.merge(ReadWrite), ReadWriteExec);
        assert_eq!(ReadWrite.merge(ReadExec), ReadWriteExec);
        // Idempotent.
        assert_eq!(ReadOnly.merge(ReadOnly), ReadOnly);
        assert_eq!(ReadExec.merge(ReadExec), ReadExec);
        assert_eq!(ReadWrite.merge(ReadWrite), ReadWrite);
    }

    #[test]
    fn page_perm_from_desc_is_inverse_of_desc() {
        use PagePerm::*;
        for perm in [ReadExec, ReadWrite, ReadOnly, ReadWriteExec] {
            let d = perm.desc(0x40_5000);
            assert_eq!(
                PagePerm::from_desc(d),
                perm,
                "from_desc must invert desc for {:?}",
                perm
            );
        }
    }

    #[test]
    fn read_write_exec_desc_is_el0_rw_and_executable() {
        let d = PagePerm::ReadWriteExec.desc(0x40_5000);
        // EL0 read+write (AP=0b01).
        assert_eq!(d & (0b11 << 6), DESC_AP_EL0_RW);
        // Executable at EL0 (UXN clear) but not EL1 (PXN set).
        assert_eq!(d & DESC_UXN, 0, "RWX overlap page must be EL0-executable");
        assert_ne!(d & DESC_PXN, 0, "EL1 must not execute user pages");
    }

    #[test]
    fn l3_index_spans_the_window() {
        assert_eq!(l3_index(USER_BASE), 0);
        assert_eq!(l3_index(USER_BASE + PAGE_SIZE), 1);
        assert_eq!(l3_index(USER_TOP - PAGE_SIZE), USER_NPAGES - 1);
    }

    #[test]
    fn l3_slot_routes_across_the_multi_table_window() {
        // First page of the window is table 0, entry 0.
        assert_eq!(l3_slot(USER_BASE), (0, 0));
        assert_eq!(l3_slot(USER_BASE + PAGE_SIZE), (0, 1));
        // The last entry of table 0 is the page just below the 2 MiB boundary.
        assert_eq!(l3_slot(USER_BASE + TABLE_SPAN - PAGE_SIZE), (0, PT_ENTRIES - 1));
        // The very next page rolls over into table 1, entry 0.
        assert_eq!(l3_slot(USER_BASE + TABLE_SPAN), (1, 0));
        // The very last page of the whole window is the last entry of the last
        // table — proving every leaf in all `USER_NTABLES` tables is reachable.
        assert_eq!(
            l3_slot(USER_TOP - PAGE_SIZE),
            (USER_NTABLES - 1, PT_ENTRIES - 1)
        );
    }

    #[test]
    fn window_geometry_is_consistent() {
        assert_eq!(USER_SPAN, USER_NTABLES * TABLE_SPAN);
        assert_eq!(USER_NPAGES, USER_NTABLES * PT_ENTRIES);
        // The mmap region must start clear above the largest real-init image so
        // the brk heap has room: the x86_64 talos-init PT_LOADs reach ~0x524380.
        assert!(MMAP_BASE >= USER_BASE + 0x12_5000);
        assert!(MMAP_BASE < USER_STACK_BOTTOM);
    }

    #[test]
    fn user_range_ok_accepts_in_window_and_rejects_outside() {
        // Whole window is fine.
        assert!(user_range_ok(USER_BASE as u64, USER_SPAN as u64));
        // A byte at the last address.
        assert!(user_range_ok((USER_TOP - 1) as u64, 1));
        // Empty range at the top boundary is OK (len 0, ptr == top).
        assert!(user_range_ok(USER_TOP as u64, 0));

        // Below the base.
        assert!(!user_range_ok((USER_BASE - 1) as u64, 1));
        // One past the top.
        assert!(!user_range_ok((USER_TOP - 1) as u64, 2));
        assert!(!user_range_ok(USER_TOP as u64, 1));
        // A length so large it would overflow if computed naively. (Use a
        // literal rather than `u64::MAX` so the assoc-const resolves unambiguously
        // whether this module is compiled `core`-only or with the std prelude.)
        assert!(!user_range_ok(USER_BASE as u64, 0xFFFF_FFFF_FFFF_FFFF));
        // A pointer near the top of the address space must not wrap in.
        assert!(!user_range_ok(0xFFFF_FFFF_FFFF_FFFF, 16));
    }

    #[test]
    fn frame_allocator_hands_out_indices_then_exhausts() {
        let mut fa = FrameAllocator::new(3);
        assert_eq!(fa.remaining(), 3);
        assert_eq!(fa.alloc(), Some(0));
        assert_eq!(fa.alloc(), Some(1));
        assert_eq!(fa.used(), 2);
        assert_eq!(fa.alloc(), Some(2));
        assert_eq!(fa.alloc(), None, "pool exhausted");
        assert_eq!(fa.alloc(), None, "stays exhausted");
        assert_eq!(fa.remaining(), 0);
        assert_eq!(FrameAllocator::frame_offset(2), 2 * PAGE_SIZE);
    }

    /// Helper: read the auxv pairs out of a built word vector into a fixed
    /// buffer (no allocator needed, so the test module stays `core`-only and is
    /// runnable from either the kernel build or the host harness). Returns the
    /// populated `(buf, count)`.
    fn auxv_pairs(img: &StackImage, argc: usize, nenv: usize) -> ([(u64, u64); 32], usize) {
        let w = img.words.as_slice();
        // argc(1) + argv(argc) + NULL(1) + envp(nenv) + NULL(1) = start of auxv.
        let start = 1 + argc + 1 + nenv + 1;
        let mut out = [(0u64, 0u64); 32];
        let mut n = 0usize;
        let mut i = start;
        while i + 1 < w.len() {
            out[n] = (w[i], w[i + 1]);
            n += 1;
            i += 2;
        }
        (out, n)
    }

    #[test]
    fn stack_image_is_aligned_and_well_formed() {
        let argv: [&[u8]; 1] = [b"/init\0"];
        let envp: [&[u8]; 1] = [b"PATH=/usr/bin\0"];
        let random = [0xAAu8; 16];
        let inp = StackInputs {
            entry: 0x40_0208,
            phdr_va: 0x42_0040,
            phentsize: 56,
            phnum: 7,
            argv: &argv,
            envp: &envp,
            random: &random,
            stack_top: USER_STACK_TOP,
        };
        let img = build_stack_image(&inp);

        // SP must be 16-byte aligned (aarch64 PCS at process entry).
        assert_eq!(img.sp & 0xf, 0, "SP_EL0 must be 16-byte aligned");

        // The whole image lives inside the stack region.
        assert!(img.sp >= USER_STACK_BOTTOM, "SP below pre-mapped stack");
        assert!(img.random_va < USER_STACK_TOP);
        assert!(img.sp < img.random_va);

        let w = img.words.as_slice();
        // words[0] == argc == 1.
        assert_eq!(w[0], 1, "argc");
        // words[1] == argv[0] VA == img.argv_vas[0].
        assert_eq!(w[1], img.argv_vas.as_slice()[0] as u64);
        // argv NULL terminator.
        assert_eq!(w[2], 0);
        // envp[0] VA, then NULL.
        assert_eq!(w[3], img.envp_vas.as_slice()[0] as u64);
        assert_eq!(w[4], 0);
    }

    #[test]
    fn stack_image_auxv_carries_the_expected_tags() {
        let argv: [&[u8]; 1] = [b"/init\0"];
        let envp: [&[u8]; 1] = [b"PATH=/usr/bin\0"];
        let random = [0x55u8; 16];
        let inp = StackInputs {
            entry: 0x40_0208,
            phdr_va: 0x42_0040,
            phentsize: 56,
            phnum: 9,
            argv: &argv,
            envp: &envp,
            random: &random,
            stack_top: USER_STACK_TOP,
        };
        let img = build_stack_image(&inp);
        let (pairs_buf, n) = auxv_pairs(&img, 1, 1);
        let pairs = &pairs_buf[..n];

        let get = |tag: u64| pairs.iter().find(|(t, _)| *t == tag).map(|(_, v)| *v);
        assert_eq!(get(AT_PHDR), Some(0x42_0040));
        assert_eq!(get(AT_PHENT), Some(56));
        assert_eq!(get(AT_PHNUM), Some(9));
        assert_eq!(get(AT_PAGESZ), Some(PAGE_SIZE as u64));
        assert_eq!(get(AT_ENTRY), Some(0x40_0208));
        assert_eq!(get(AT_BASE), Some(0));
        // AT_RANDOM must point at the random_va we recorded.
        assert_eq!(get(AT_RANDOM), Some(img.random_va as u64));
        // The vector ends in an AT_NULL terminator.
        assert_eq!(pairs.last(), Some(&(AT_NULL, 0)));
        // AT_RANDOM target must be inside the window and 16-aligned (we masked).
        assert!(user_range_ok(img.random_va as u64, 16));
        assert_eq!(img.random_va & 0xf, 0);
    }

    #[test]
    fn tls_layout_places_tcb_below_image_with_correct_alignment() {
        // svc's real PT_TLS: filesz=32 (.tdata), memsz=80 (=> 48 bytes .tbss),
        // align=8.
        let tls = TlsPhdr {
            filesz: 32,
            memsz: 80,
            align: 8,
        };
        // Pick an unaligned base to prove `tp` gets aligned up.
        let lay = compute_tls_layout(0x42_0003, &tls);

        // tp is aligned to max(16, p_align)=16 and >= base.
        assert_eq!(lay.tp & (TLS_TP_ALIGN - 1), 0, "tp must be 16-aligned");
        assert!(lay.tp >= 0x42_0003);
        assert!(lay.tp < 0x42_0003 + TLS_TP_ALIGN);

        // The TCB is exactly 2 pointers and the image sits right above it.
        assert_eq!(lay.image_va, lay.tp + TCB_SIZE);
        assert_eq!(lay.image_va - lay.tp, 16);

        // Image start satisfies the PT_TLS alignment (8 here).
        assert_eq!(lay.image_va % tls.align, 0);

        // Sizes propagate; .tbss is memsz-filesz.
        assert_eq!(lay.filesz, 32);
        assert_eq!(lay.memsz, 80);
        assert_eq!(lay.memsz - lay.filesz, 48);

        // region_end is page-rounded and covers the whole TCB+image.
        assert!(lay.region_end >= lay.image_va + lay.memsz);
        assert_eq!(lay.region_end % PAGE_SIZE, 0);
    }

    #[test]
    fn tls_layout_honours_a_larger_p_align() {
        // A 64-byte-aligned TLS image (some vectorised TLS): tp must round to 64
        // so the image start (tp+16) is... NOT 64-aligned. Variant I aligns the
        // *image start* to p_align; since TCB_SIZE=16 is not a multiple of 64,
        // we align tp to p_align and the image lands at tp+16. The ABI contract
        // we must keep is that accesses `tp + TCB_SIZE + k` work for the linker's
        // chosen layout, which assumes image_va is p_align aligned. To keep that
        // invariant for align>16 we align the *image*, not tp. Verify image_va
        // alignment is the load-bearing property.
        let tls = TlsPhdr {
            filesz: 8,
            memsz: 16,
            align: 16,
        };
        let lay = compute_tls_layout(0x42_0000, &tls);
        // For the alignments musl/aarch64 emits (8 or 16, both dividing 16) the
        // image start is always p_align-aligned because tp is 16-aligned and
        // TCB_SIZE=16.
        assert_eq!(lay.image_va % tls.align, 0);
        assert_eq!(lay.image_va, lay.tp + TCB_SIZE);
    }

    #[test]
    fn tls_layout_zero_align_is_safe() {
        // p_align of 0 or 1 means "no extra alignment"; tp still gets the
        // default 16-byte alignment from TLS_TP_ALIGN.
        let tls = TlsPhdr {
            filesz: 0,
            memsz: 24,
            align: 0,
        };
        let lay = compute_tls_layout(0x42_0007, &tls);
        assert_eq!(lay.tp & (TLS_TP_ALIGN - 1), 0);
        assert_eq!(lay.image_va, lay.tp + TCB_SIZE);
        assert!(lay.region_end >= lay.image_va + 24);
    }

    #[test]
    fn align_up_rounds_to_power_of_two() {
        assert_eq!(align_up(0, 16), 0);
        assert_eq!(align_up(1, 16), 16);
        assert_eq!(align_up(16, 16), 16);
        assert_eq!(align_up(17, 16), 32);
        assert_eq!(align_up(0x4007, 0x1000), 0x5000);
        // align <= 1 is a no-op.
        assert_eq!(align_up(12345, 1), 12345);
        assert_eq!(align_up(12345, 0), 12345);
    }

    #[test]
    fn stack_image_handles_multiple_args_and_envs() {
        let argv: [&[u8]; 2] = [b"/init\0", b"--flag\0"];
        let envp: [&[u8]; 2] = [b"PATH=/usr/bin\0", b"HOME=/\0"];
        let random = [0u8; 16];
        let inp = StackInputs {
            entry: 0x40_1000,
            phdr_va: 0x42_0040,
            phentsize: 56,
            phnum: 5,
            argv: &argv,
            envp: &envp,
            random: &random,
            stack_top: USER_STACK_TOP,
        };
        let img = build_stack_image(&inp);
        assert_eq!(img.sp & 0xf, 0);
        let w = img.words.as_slice();
        assert_eq!(w[0], 2, "argc == 2");
        // argv block: w[1], w[2], then NULL at w[3].
        assert_eq!(w[3], 0, "argv NULL terminator");
        // envp block: w[4], w[5], then NULL at w[6].
        assert_eq!(w[6], 0, "envp NULL terminator");
        assert_eq!(img.argv_vas.len(), 2);
        assert_eq!(img.envp_vas.len(), 2);
    }

    // ---- timer-sleep helper -----------------------------------------------

    #[test]
    fn timespec_to_cycles_converts_seconds_and_nanos() {
        // 1 second at 1 MHz counter == 1_000_000 cycles.
        assert_eq!(
            timespec_to_cycles(1, 0, 1_000_000),
            SleepCycles::Wait(1_000_000)
        );
        // 200 ms (the svc heartbeat sleep) at 24 MHz (QEMU virt CNTFRQ).
        assert_eq!(
            timespec_to_cycles(0, 200_000_000, 24_000_000),
            SleepCycles::Wait(4_800_000)
        );
        // Combined sec + nsec.
        assert_eq!(
            timespec_to_cycles(2, 500_000_000, 1_000_000_000),
            SleepCycles::Wait(2_500_000_000)
        );
    }

    #[test]
    fn timespec_to_cycles_zero_duration_is_zero_wait() {
        assert_eq!(timespec_to_cycles(0, 0, 24_000_000), SleepCycles::Wait(0));
    }

    #[test]
    fn timespec_to_cycles_rejects_bad_timespec() {
        assert_eq!(timespec_to_cycles(-1, 0, 24_000_000), SleepCycles::Invalid);
        assert_eq!(timespec_to_cycles(0, -1, 24_000_000), SleepCycles::Invalid);
        // tv_nsec must be < 1e9.
        assert_eq!(
            timespec_to_cycles(0, 1_000_000_000, 24_000_000),
            SleepCycles::Invalid
        );
    }

    #[test]
    fn timespec_to_cycles_saturates_and_survives_zero_freq() {
        // Huge seconds at huge frequency saturates into u64 rather than wrapping.
        assert_eq!(
            timespec_to_cycles(i64::MAX, 0, u64::MAX),
            SleepCycles::Wait(u64::MAX)
        );
        // A degenerate 0 Hz counter is treated as 1 Hz (no div-by-zero); the
        // result is just the seconds term.
        assert_eq!(timespec_to_cycles(5, 999_999_999, 0), SleepCycles::Wait(5));
    }

    #[test]
    fn deadline_after_saturates_instead_of_wrapping() {
        assert_eq!(deadline_after(10, 5), 15);
        // Near the top of the counter, the deadline clamps to u64::MAX rather
        // than wrapping to a tiny value (which would end the wait immediately).
        assert_eq!(deadline_after(u64::MAX - 2, 10), u64::MAX);
    }
}
