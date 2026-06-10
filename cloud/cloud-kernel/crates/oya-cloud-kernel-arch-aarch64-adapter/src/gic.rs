//! GICv2 interrupt-controller driver for the QEMU `virt` machine.
//!
//! Frame code. The `virt` board places the GICv2 distributor at
//! `0x0800_0000` and the CPU interface at `0x0801_0000`. We bring up just
//! enough to deliver the EL1 physical timer interrupt (PPI 30, INTID 30) to the
//! boot CPU.
//!
//! The IRQ dispatch path: a CPU IRQ exception lands in the vector table, which
//! calls [`handle_irq`]; that reads the GIC's IAR to get the pending INTID,
//! dispatches (timer -> [`crate::timer`]), then writes EOIR to retire it.

use core::sync::atomic::{AtomicU32, Ordering};

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::ReadWrite;

/// Distributor MMIO base on QEMU `virt`.
const GICD_BASE: usize = 0x0800_0000;
/// CPU-interface MMIO base on QEMU `virt`.
const GICC_BASE: usize = 0x0801_0000;

/// INTID of the EL1 physical (non-secure) generic timer (a PPI).
pub const TIMER_INTID: u32 = 30;

/// Spurious-interrupt INTIDs returned by the GIC when nothing is pending.
const SPURIOUS_MIN: u32 = 1020;

register_structs! {
    /// GICv2 distributor registers (subset we use).
    GicdRegs {
        (0x000 => ctlr: ReadWrite<u32>),
        (0x004 => _reserved0),
        (0x100 => isenabler: [ReadWrite<u32>; 32]),
        (0x180 => _reserved1),
        (0x400 => ipriorityr: [ReadWrite<u32>; 256]),
        (0x800 => _reserved2),
        (0xC00 => _reserved3),
        (0x1000 => @END),
    }
}

register_structs! {
    /// GICv2 CPU-interface registers (subset we use).
    GiccRegs {
        (0x000 => ctlr: ReadWrite<u32>),
        (0x004 => pmr: ReadWrite<u32>),
        (0x008 => _reserved0),
        (0x00C => iar: ReadWrite<u32>),
        (0x010 => eoir: ReadWrite<u32>),
        (0x014 => _reserved1),
        (0x1000 => @END),
    }
}

fn gicd() -> &'static GicdRegs {
    // SAFETY: GICD_BASE is the QEMU `virt` distributor base, identity-mapped as
    // Device memory; single-core bring-up means no aliasing writers.
    unsafe { &*(GICD_BASE as *const GicdRegs) }
}

fn gicc() -> &'static GiccRegs {
    // SAFETY: as above for the CPU interface.
    unsafe { &*(GICC_BASE as *const GiccRegs) }
}

/// Initialize the GICv2 distributor + CPU interface for the boot core.
///
/// # Safety
/// Call once on the boot core during bring-up.
pub unsafe fn init() {
    let d = gicd();
    let c = gicc();

    // Enable group 0 forwarding in the distributor.
    d.ctlr.set(1);

    // CPU interface: lowest priority mask (accept all), enable.
    c.pmr.set(0xFF);
    c.ctlr.set(1);
}

/// Enable a single interrupt ID and give it a usable priority.
///
/// # Safety
/// Call after [`init`]. `intid` must be a valid SPI/PPI for this board.
pub unsafe fn enable_interrupt(intid: u32) {
    let d = gicd();
    let reg = (intid / 32) as usize;
    let bit = intid % 32;
    // Mid-range priority (lower value = higher priority); byte per INTID.
    let pri_reg = (intid / 4) as usize;
    let pri_shift = (intid % 4) * 8;
    let old = d.ipriorityr[pri_reg].get();
    let cleared = old & !(0xFFu32 << pri_shift);
    d.ipriorityr[pri_reg].set(cleared | (0xA0u32 << pri_shift));
    d.isenabler[reg].set(1 << bit);
}

/// P4·SMP·S3: per-AP GICv2 CPU-interface bring-up + PPI enable.
///
/// On GICv2 the CPU interface (`GICC_*`) is a single MMIO block, but its key
/// registers (`PMR`, `CTLR`) are **banked per-CPU** — each core sees its own
/// copy at the same address — so every AP must enable its own CPU interface
/// itself. The distributor's per-PPI enable register `ISENABLER0` (INTIDs 0..31)
/// is likewise banked per-CPU, so the AP enables its timer PPI here too. The
/// distributor's *global* enable (`GICD_CTLR`) is shared and already set by the
/// BSP's [`init`], so the AP does not touch it.
///
/// For S3 the AP's generic timer stays masked (no scheduling); this only makes
/// the AP *able* to receive the PPI in S4.
///
/// # Safety
/// Call once from the AP's own bring-up path, after its MMU is enabled and with
/// IRQs masked. `intid` must be a valid PPI (`< 32`).
pub unsafe fn init_ap(intid: u32) {
    let c = gicc();
    // This AP's banked CPU interface: accept all priorities, enable group 0.
    c.pmr.set(0xFF);
    c.ctlr.set(1);

    // Enable + prioritize the PPI in the per-CPU banked distributor registers.
    let d = gicd();
    let reg = (intid / 32) as usize;
    let bit = intid % 32;
    let pri_reg = (intid / 4) as usize;
    let pri_shift = (intid % 4) * 8;
    let old = d.ipriorityr[pri_reg].get();
    let cleared = old & !(0xFFu32 << pri_shift);
    d.ipriorityr[pri_reg].set(cleared | (0xA0u32 << pri_shift));
    d.isenabler[reg].set(1 << bit);
}

/// Count of timer ticks dispatched (visible to the safe kernel via the HAL).
pub static TICKS: AtomicU32 = AtomicU32::new(0);

/// Handle one IRQ exception: ack, dispatch, EOI. Returns `true` iff the serviced
/// interrupt was a timer tick (a preemption source); `false` for the shootdown
/// SGI or a spurious INTID (which must NOT trigger a reschedule).
pub fn handle_irq() -> bool {
    let c = gicc();
    let iar = c.iar.get();
    let intid = iar & 0x3FF;

    if intid >= SPURIOUS_MIN {
        return false; // Spurious; nothing to retire, not a preemption tick.
    }

    let mut was_tick = false;
    if intid == TIMER_INTID {
        crate::timer::on_tick();
        TICKS.fetch_add(1, Ordering::Relaxed);
        was_tick = true;
    } else if intid == crate::shootdown::SHOOTDOWN_SGI {
        // P4·SMP·S4c: a cross-CPU TLB shootdown request. Invalidate this CPU's
        // stale translations + ack the sender (the H1 receiver step). NOT a
        // preemption tick — the receiver resumes the exact interrupted context.
        crate::shootdown::service_on_sgi();
    } else if intid == crate::reschedule::RESCHED_SGI {
        // P4·SMP·S4b: a reschedule wake — taking the SGI already woke an idle AP
        // from `wfe`; the handler does nothing. NOT a preemption tick (a busy CPU
        // resumes its current context; the idle CPU re-runs schedule in its loop).
    }

    // End-of-interrupt: write the original IAR value back.
    c.eoir.set(iar);
    was_tick
}

/// Signal end-of-interrupt for `intid` by writing `GICC_EOIR`.
///
/// Additive helper used by the reshaped-HAL floor backing
/// (`crate::hal_caps::Gicv2IrqChip::eoi`); the existing boot IRQ path retires
/// interrupts inline in [`handle_irq`] and is unchanged. Writing EOIR with the
/// INTID is the same retire mechanism that path uses.
#[allow(dead_code)]
pub fn eoi(intid: u32) {
    gicc().eoir.set(intid);
}
