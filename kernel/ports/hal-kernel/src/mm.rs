//! Memory-management shapes: typed addresses, the page-table config trait, the
//! typed/untyped frame split, the W^X map-flag type-state, and the DMA seam.
//!
//! These are pure definitions. The single generic page-table / cursor engine
//! that consumes [`PageTableConfig`] lives above this crate (in `frame`); the
//! PTE encoding, paging constants, and the unsafe address-space switch live in
//! each arch backend. Re-expressed clean-room from OSTD lessons A1/A4/A21 — the
//! *shapes*, in our own words and design.

use core::marker::PhantomData;

use crate::sealed::Sealed;

// ---------------------------------------------------------------------------
// Typed physical / DMA addresses (lesson A21: distinct newtypes so a host VA, a
// physical frame address, and a device-visible bus address can never be mixed).
// ---------------------------------------------------------------------------

/// A physical address (host-side, MMU input). Distinct from a [`DmaAddr`] so
/// the two cannot be confused at a call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(pub usize);

impl PhysAddr {
    /// The raw physical address value.
    pub const fn as_usize(self) -> usize {
        self.0
    }
}

/// A device-visible bus address (what a DMA-capable peripheral sees through the
/// IOMMU/SMMU). On an identity-mapped platform it may equal the [`PhysAddr`],
/// but the types stay distinct so a missing IOMMU translation is a type error,
/// not silent corruption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DmaAddr(pub usize);

impl DmaAddr {
    /// The raw bus address value.
    pub const fn as_usize(self) -> usize {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Typed page order (lesson: typed PageOrder makes a 4 KiB-frame-as-1 GiB-entry a
// compile error — three-pillars §5 zero-cost abstractions).
// ---------------------------------------------------------------------------

/// The supported mapping granularities, as a closed typed set.
///
/// Encoding a frame's order in the type/enum (rather than a raw shift) means the
/// page-table engine can refuse to install, say, a `Base4K` frame at a level
/// that demands a `Huge1G` block. `Cont64K` is the aarch64 contiguous-hint
/// group; the others are the standard huge-page steps shared across arches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PageOrder {
    /// 4 KiB base page.
    Base4K,
    /// 64 KiB contiguous-hint group (aarch64 `Contiguous` bit).
    Cont64K,
    /// 2 MiB huge page / block descriptor.
    Huge2M,
    /// 1 GiB huge page / block descriptor.
    Huge1G,
}

impl PageOrder {
    /// Size of one mapping at this order, in bytes.
    pub const fn size(self) -> usize {
        match self {
            PageOrder::Base4K => 4 * 1024,
            PageOrder::Cont64K => 64 * 1024,
            PageOrder::Huge2M => 2 * 1024 * 1024,
            PageOrder::Huge1G => 1024 * 1024 * 1024,
        }
    }
}

// ---------------------------------------------------------------------------
// W^X as a MapFlags type-state (consensus C6 / three-pillars §5).
//   - set_write() -> WritableFlags
//   - set_exec()  -> ExecFlags
//   - there is NO combiner: a Write+Exec value is a type that cannot be built.
// ---------------------------------------------------------------------------

/// The base, permission-less mapping request: present + readable.
///
/// Branches into exactly one of [`WritableFlags`] or [`ExecFlags`]; because
/// neither offers a method back to the other, a single mapping can be writable
/// **or** executable, never both. W^X is therefore a property the compiler
/// checks, with no runtime flag to forget (it replaces a runtime `PagePerm`
/// merge that could otherwise produce `ReadWriteExec`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapFlags {
    global: bool,
    user: bool,
}

/// A mapping that is writable (and therefore can never become executable).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WritableFlags {
    base: MapFlags,
}

/// A mapping that is executable (and therefore can never become writable).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecFlags {
    base: MapFlags,
}

impl MapFlags {
    /// A fresh read-only kernel mapping request.
    pub const fn new() -> Self {
        Self {
            global: false,
            user: false,
        }
    }

    /// Mark the mapping as user-accessible (EL0 / ring-3).
    pub const fn user(mut self) -> Self {
        self.user = true;
        self
    }

    /// Mark the mapping global (survives an address-space switch; x86 PGE /
    /// aarch64 nG-clear). Typically the kernel linear map.
    pub const fn global(mut self) -> Self {
        self.global = true;
        self
    }

    /// Is this mapping user-accessible?
    pub const fn is_user(self) -> bool {
        self.user
    }

    /// Is this mapping global?
    pub const fn is_global(self) -> bool {
        self.global
    }

    /// Consume this read-only request and make it **writable**. The result is a
    /// [`WritableFlags`] with no path to executability — that is the W half of
    /// W^X, enforced by the absence of a combiner.
    pub const fn set_write(self) -> WritableFlags {
        WritableFlags { base: self }
    }

    /// Consume this read-only request and make it **executable**. The result is
    /// an [`ExecFlags`] with no path to writability — the X half of W^X.
    pub const fn set_exec(self) -> ExecFlags {
        ExecFlags { base: self }
    }
}

impl Default for MapFlags {
    fn default() -> Self {
        Self::new()
    }
}

impl WritableFlags {
    /// The read-only base flags (global/user bits) this was derived from.
    pub const fn base(self) -> MapFlags {
        self.base
    }
}

impl ExecFlags {
    /// The read-only base flags (global/user bits) this was derived from.
    pub const fn base(self) -> MapFlags {
        self.base
    }
}

// ---------------------------------------------------------------------------
// PTE typed flag set (lesson A1: GenericPteFlags-style typed flags) and PTE.
// ---------------------------------------------------------------------------

/// A typed, arch-neutral page-table-entry flag set.
///
/// The arch backend maps these abstract bits onto its real PTE layout (x86
/// `PRESENT/RW/US/NX/G`, aarch64 `AF/AP/UXN/PXN/nG`, riscv64 `V/R/W/X/U/G`).
/// Keeping the safe-side flag vocabulary abstract (the OSTD `GenericPteFlags`
/// idea) means the page-table engine never hard-codes one arch's bit numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenericPteFlags {
    /// The entry maps a present, valid translation.
    pub present: bool,
    /// Writable by the owning privilege level.
    pub writable: bool,
    /// Executable (instruction fetch permitted).
    pub executable: bool,
    /// Accessible from user mode (EL0 / ring-3 / U-bit).
    pub user: bool,
    /// Global — not flushed on an address-space switch.
    pub global: bool,
}

impl GenericPteFlags {
    /// An empty (not-present) flag set.
    pub const fn empty() -> Self {
        Self {
            present: false,
            writable: false,
            executable: false,
            user: false,
            global: false,
        }
    }
}

/// A page-table entry: the arch backend's concrete `repr(transparent)` word,
/// presented as an arch-neutral shape. The engine in `frame` reads/writes
/// entries only through this trait, so one cursor serves every paging format.
pub trait Pte: Sealed + Copy {
    /// Decode this entry's typed flags.
    fn flags(&self) -> GenericPteFlags;

    /// The output physical address this entry points at (next table or frame).
    fn address(&self) -> PhysAddr;

    /// Does this entry terminate the walk (a leaf mapping) versus point at a
    /// next-level table?
    fn is_leaf(&self) -> bool;
}

/// Paging constants describing one arch's translation regime (lesson A1's
/// `PagingConstsTrait`). Associated consts so the generic engine specializes
/// per arch with zero runtime cost.
pub trait PagingConsts {
    /// Number of translation levels (x86 4/5, aarch64 3/4, riscv64 Sv39=3).
    const NR_LEVELS: u8;
    /// Base page size in bytes (4096 today).
    const BASE_PAGE_SIZE: usize;
    /// Virtual-address width in bits.
    const ADDRESS_WIDTH: u8;
    /// Whether the top VA bit sign-extends (canonical-address regime).
    const VA_SIGN_EXT: bool;
    /// The highest level at which a leaf (huge) mapping is permitted.
    const HIGHEST_TRANSLATION_LEVEL: u8;
}

/// An opaque per-address-space TLB tag (x86 PCID / aarch64 ASID).
///
/// The value is meaningful only to the arch backend; the safe kernel passes it
/// around without interpreting it. Making it opaque now (consensus Q1
/// non-negotiable) means the future tagged-TLB body of `switch_to` is a
/// signature-compatible swap-in for today's full-flush fallback — no hot-path
/// re-cut. A `None`-equivalent ([`AsidTag::UNTAGGED`]) requests a full flush.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AsidTag(u16);

impl AsidTag {
    /// The "no tag" sentinel: the backend must full-flush rather than tag.
    pub const UNTAGGED: Self = Self(0);

    /// Wrap a backend-chosen tag value. Construction is open (the kernel/Frame
    /// allocates tags) but the inner value stays opaque to safe code.
    pub const fn from_raw(tag: u16) -> Self {
        Self(tag)
    }

    /// The raw tag, for the arch backend that knows how to program it.
    pub const fn as_raw(self) -> u16 {
        self.0
    }
}

/// The per-arch page-table configuration: PTE format, paging constants, and the
/// address-space switch contract (lesson A1). The single generic page-table
/// engine in `frame` is parameterized over this trait, so x86 4/5-level,
/// aarch64 TTBR0/TTBR1, and riscv64 Sv39/Sv48 all reuse one audited cursor.
///
/// Sealed: only this workspace's arch backends may implement it.
pub trait PageTableConfig: Sealed {
    /// The arch PTE encoding.
    type Pte: Pte;
    /// The arch paging constants.
    type Consts: PagingConsts;

    /// Switch the active translation root to `root`, tagging the new context
    /// with `asid_tag`. With [`AsidTag::UNTAGGED`] (or on hardware lacking
    /// tagged TLBs) the backend full-flushes; otherwise it programs the tag and
    /// suppresses the flush. Deferred to the arch backend — declared here so the
    /// signature is fixed (opaque tag) before any body exists (consensus Q1).
    fn switch_to(&self, root: PhysAddr, asid_tag: AsidTag);
}

// ---------------------------------------------------------------------------
// Typed/untyped physical-frame split (lesson A4) + confidential SharedPhysFrame
// seam (open question Q3: reserve the type distinction only, no impl).
// ---------------------------------------------------------------------------

/// Sealed marker for the *kind* of a physical frame. Closed set, so the engine
/// can reason exhaustively about which frames may enter a user address space.
pub trait FrameKind: Sealed {}

/// Marker for a **typed** frame — kernel-internal pages (page-table nodes,
/// kernel stacks, metadata). These must never be mapped into user space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Typed;

/// Marker for an **untyped** frame — ordinary page-cache / anonymous user
/// memory. Only these (the OSTD `UFrame`) may be handed to a user mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Untyped;

impl Sealed for Typed {}
impl Sealed for Untyped {}
impl FrameKind for Typed {}
impl FrameKind for Untyped {}

/// A physical frame handle parameterized by its [`FrameKind`].
///
/// The kind lives in the type, so an API that accepts only `PhysFrame<Untyped>`
/// (the `UFrame` alias) structurally rejects a page-table node — eliminating a
/// privilege-escalation class with zero runtime cost (lesson A4). This is the
/// shape; refcount/metadata machinery lands in `frame`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysFrame<K: FrameKind> {
    addr: PhysAddr,
    _kind: PhantomData<K>,
}

/// The untyped frame: the only kind that may cross into a user address space.
pub type UFrame = PhysFrame<Untyped>;

impl<K: FrameKind> PhysFrame<K> {
    /// The physical base address of this frame.
    pub const fn address(self) -> PhysAddr {
        self.addr
    }
}

impl PhysFrame<Typed> {
    /// Construct a typed (kernel-internal) frame handle from its base address.
    pub const fn new_typed(addr: PhysAddr) -> Self {
        Self {
            addr,
            _kind: PhantomData,
        }
    }
}

impl PhysFrame<Untyped> {
    /// Construct an untyped (user-mappable) frame handle from its base address.
    pub const fn new_untyped(addr: PhysAddr) -> Self {
        Self {
            addr,
            _kind: PhantomData,
        }
    }
}

/// A physical frame whose contents are **shared with the host/hypervisor** —
/// the confidential-VM (SEV-SNP / TDX / CCA) shared-memory seam.
///
/// Open question Q3 reserves *only* this type-level distinction now: a normal
/// [`PhysFrame`] is private (C-bit set / TDX-private / Realm-protected); a
/// `SharedPhysFrame` is the explicitly-shared window DMA bounce buffers live in.
/// **No** confidential-compute logic exists yet (no PSC/GHCB/TDCALL) — that is
/// P6. Carrying the distinction now prevents a later DMA-path rework.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedPhysFrame {
    addr: PhysAddr,
}

impl SharedPhysFrame {
    /// Wrap a physical address as an explicitly host-shared frame.
    pub const fn new(addr: PhysAddr) -> Self {
        Self { addr }
    }

    /// The physical base address of the shared window.
    pub const fn address(self) -> PhysAddr {
        self.addr
    }
}

// ---------------------------------------------------------------------------
// DMA region RAII shape (lesson A21): possession is authorization, Drop tears
// down the IOMMU mapping. Body lands in `frame`/arch; this is the seam.
// ---------------------------------------------------------------------------

/// The DMA-transfer direction, as a sealed type-state (lesson A21). Sealing the
/// set lets the engine const-assert which of `reader()`/`writer()` a given
/// region exposes.
pub trait DmaDirection: Sealed {}

/// Host → device (the device reads; we only write).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToDevice;
/// Device → host (the device writes; we only read).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FromDevice;
/// Bidirectional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bidirectional;

impl Sealed for ToDevice {}
impl Sealed for FromDevice {}
impl Sealed for Bidirectional {}
impl DmaDirection for ToDevice {}
impl DmaDirection for FromDevice {}
impl DmaDirection for Bidirectional {}

/// The RAII shape for a DMA-mappable region (lesson A21).
///
/// A live `DmaRegion<D>` is a capability: holding it means the underlying frame
/// is IOMMU-mapped for direction `D` and reachable at [`DmaRegion::device_addr`]
/// by the peripheral. The mapping is installed on construction (by `frame`/arch,
/// later) and **torn down on `Drop`** — deny-by-default isolation that a missing
/// `unmap` cannot defeat. The shared/private bit travels with the frame, so a
/// confidential bounce buffer carries a [`SharedPhysFrame`].
pub struct DmaRegion<D: DmaDirection> {
    /// Host physical base of the mapped region.
    phys: PhysAddr,
    /// Device-visible base after IOMMU translation.
    device: DmaAddr,
    /// Length of the mapped window in bytes.
    len: usize,
    _dir: PhantomData<D>,
}

impl<D: DmaDirection> DmaRegion<D> {
    /// The device-visible (bus) base address the peripheral programs into its
    /// descriptor ring.
    pub const fn device_addr(&self) -> DmaAddr {
        self.device
    }

    /// The host physical base of the region.
    pub const fn phys_addr(&self) -> PhysAddr {
        self.phys
    }

    /// The mapped length in bytes.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the region is zero-length.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<D: DmaDirection> Drop for DmaRegion<D> {
    fn drop(&mut self) {
        // SHAPE ONLY: the real IOMMU/SMMU teardown lives in the arch backend
        // and is wired in a later increment. Declaring `Drop` now fixes the
        // RAII contract (lesson A21: "implement Drop from day one, not a TODO
        // no-op that leaks VA space"). No `unsafe`, no hardware touch here.
    }
}
