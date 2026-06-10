//! Minimal identity-mapping MMU bring-up for the QEMU `virt` machine.
//!
//! Frame code: programs `MAIR_EL1`, `TCR_EL1`, `TTBR0_EL1`, and `SCTLR_EL1`.
//! We build the simplest table that works: a single level-1 translation table
//! using 1 GiB block descriptors over a 4 GiB (32-bit) address space.
//!
//! QEMU `virt` memory map (the parts we care about):
//!   * `0x0000_0000 .. 0x4000_0000` — devices (GICv2 @ 0x0800_0000,
//!     PL011 UART @ 0x0900_0000). Mapped as **Device-nGnRE**.
//!   * `0x4000_0000 .. 0x8000_0000` — RAM. Mapped as **Normal** cacheable.
//!
//! This identity map keeps every physical address equal to its virtual address,
//! so no pointers in the already-running kernel need fixing up when the MMU
//! turns on.

use aarch64_cpu::asm::barrier;
use aarch64_cpu::registers::{MAIR_EL1, SCTLR_EL1, TCR_EL1, TTBR0_EL1};
use tock_registers::interfaces::{ReadWriteable, Writeable};

/// MAIR attribute index for Device-nGnRE memory.
const ATTR_DEVICE_IDX: u64 = 0;
/// MAIR attribute index for Normal Write-Back cacheable memory.
const ATTR_NORMAL_IDX: u64 = 1;

// ---- Level-1 block descriptor bit fields (ARMv8-A VMSAv8-64) -------------

/// Descriptor is valid.
const DESC_VALID: u64 = 1 << 0;
/// Block descriptor (bit[1]=0) vs table (bit[1]=1). For a block we clear bit 1.
const DESC_BLOCK: u64 = 0 << 1;
/// Access flag — must be set or the first access faults.
const DESC_AF: u64 = 1 << 10;
/// Shareability: inner-shareable (`0b11`) at bits[9:8].
const DESC_SH_INNER: u64 = 0b11 << 8;
/// AP[2:1] = 0b00 -> read/write at EL1.
const DESC_AP_RW: u64 = 0b00 << 6;

/// Build the lower attributes for a block pointing at MAIR index `attr_idx`.
const fn block_attrs(attr_idx: u64) -> u64 {
    DESC_VALID | DESC_BLOCK | DESC_AF | DESC_SH_INNER | DESC_AP_RW | (attr_idx << 2)
}

/// The level-1 translation table: 4 entries, each mapping a 1 GiB block.
///
/// 16 KiB-aligned is more than the required 4 KiB alignment for a table whose
/// 4 used entries fit in the first 32 bytes; we over-align to a page so the
/// physical base is a clean page address for `TTBR0_EL1`.
#[repr(C, align(4096))]
struct PageTable {
    entries: [u64; 512],
}

static mut L1_TABLE: PageTable = PageTable { entries: [0; 512] };

/// Build the identity page table (BSP-only) and enable the MMU on this CPU.
///
/// This is the boot-core path: it populates the SHARED [`L1_TABLE`] once, then
/// turns translation on. It is the composition of [`build_table`] (idempotent,
/// BSP-only) and [`enable_translation`] (per-CPU). An AP (P4·SMP·S3) calls ONLY
/// [`enable_translation`] — the table is already built, and rebuilding it would
/// race the BSP.
///
/// # Safety
/// Must be called once, early, on the boot core with the MMU off. After it
/// returns, translation is active with the identity map described above.
pub unsafe fn enable() {
    // SAFETY: single-core bring-up; we are the only writer of `L1_TABLE` here.
    unsafe {
        build_table();
        enable_translation();
    }
}

/// Populate the SHARED level-1 identity table. **BSP-only**, idempotent.
///
/// # Safety
/// Call once on the boot core before any AP starts. APs must NOT call this —
/// they consume the already-built table via [`enable_translation`].
unsafe fn build_table() {
    // SAFETY: boot core, before any AP runs; we are the only writer of the
    // shared `L1_TABLE`.
    let table = unsafe { &mut *core::ptr::addr_of_mut!(L1_TABLE) };

    // 0 GiB: devices (GIC, UART, ...) — Device memory.
    table.entries[0] = 0x0000_0000 | block_attrs(ATTR_DEVICE_IDX);
    // 1 GiB: RAM at 0x4000_0000 — Normal cacheable memory.
    table.entries[1] = 0x4000_0000 | block_attrs(ATTR_NORMAL_IDX);
    // 2 GiB / 3 GiB: more device/MMIO space (PCIe ECAM etc.) — Device memory.
    table.entries[2] = 0x8000_0000 | block_attrs(ATTR_DEVICE_IDX);
    table.entries[3] = 0xC000_0000 | block_attrs(ATTR_DEVICE_IDX);
}

/// Enable translation on THIS CPU against the already-built SHARED [`L1_TABLE`]:
/// program `MAIR_EL1`/`TCR_EL1`/`TTBR0_EL1`, barrier, then set `SCTLR_EL1.M/C/I`.
///
/// Per-CPU and re-entrant-by-design: the BSP runs it once (via [`enable`]); each
/// AP (P4·SMP·S3) runs it from `_ap_start` to bring up its own MMU on the shared
/// table. It loads **TTBR0 only** — the kernel uses a single 32-bit-VA TTBR0 map
/// and sets `EPD1::DisableTTBR1Walks` below, so there is no TTBR1 to load. It
/// touches no `static mut` (only this CPU's banked translation registers).
///
/// # Safety
/// Call on a CPU with the MMU off, after [`build_table`] has run on the BSP. The
/// caller must be running at an identity-mapped PC so translation can turn on
/// without invalidating the current fetch (true: the whole image is identity
/// mapped by the table above).
pub unsafe fn enable_translation() {
    let ttbr0 = core::ptr::addr_of!(L1_TABLE) as u64;

    // --- Memory attributes (MAIR_EL1). ---
    // Attr0 = Device-nGnRE (0x00); Attr1 = Normal WB/WA inner+outer (0xFF).
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck
            + MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc,
    );

    // --- Translation control (TCR_EL1). ---
    // 32-bit VA (T0SZ=32) so the 4-entry L1 covers the whole space; 4 KiB
    // granule; inner-shareable; WB cacheable table walks; IPS from the CPU.
    // EPD1 disables TTBR1 walks — there is NO high-half map, so APs load TTBR0
    // only (the S3 grounded correction).
    TCR_EL1.write(
        TCR_EL1::T0SZ.val(32)
            + TCR_EL1::TG0::KiB_4
            + TCR_EL1::SH0::Inner
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD1::DisableTTBR1Walks
            + TCR_EL1::IPS::Bits_40,
    );

    TTBR0_EL1.set_baddr(ttbr0);

    // Ensure all the above reach memory/registers before enabling translation.
    barrier::isb(barrier::SY);
    barrier::dsb(barrier::SY);

    // --- Turn the MMU on (and the caches). ---
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);

    barrier::isb(barrier::SY);
}
