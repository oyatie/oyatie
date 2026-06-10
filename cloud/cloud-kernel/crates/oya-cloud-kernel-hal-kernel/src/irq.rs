//! Interrupt-controller shape: the modern [`IrqChip`] (x2APIC / GICv3 / AIA).
//!
//! Pure trait definitions. The legacy 8259/PIT and GICv2 paths stay behind the
//! existing [`crate::InterruptApi`] as the bring-up fallback (NO-LEGACY §7: the
//! modern chip is the floor *shape*, its body lands in P4). Sealed so only the
//! arch backends implement it.

use crate::mm::DmaAddr;
use crate::sealed::Sealed;
use crate::ArchError;

/// An abstract interrupt identifier (GIC INTID / APIC vector / AIA source).
/// Opaque to the safe kernel; the arch backend maps it onto its real vector
/// space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct IrqVector(pub u32);

/// Logical target CPU for IPIs and MSI steering. The arch backend resolves it
/// to an APIC ID / GIC affinity / hart ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CpuId(pub u32);

/// A programmed MSI(-X) message: the address/data pair the device writes to
/// raise its interrupt. Produced by [`IrqChip::map_msi`] for a target CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MsiMessage {
    /// The DMA/MMIO address the device writes to signal the interrupt.
    pub address: DmaAddr,
    /// The data payload written to `address`.
    pub data: u32,
}

/// The modern per-CPU interrupt controller (x86 x2APIC / aarch64 GICv3+ITS /
/// riscv64 AIA-IMSIC). Sealed; implemented by the arch backends only.
///
/// Replaces the narrow [`crate::InterruptApi`] enable/disable pair with the
/// full floor shape: per-line enable/disable, end-of-interrupt, inter-processor
/// interrupts, and MSI mapping for virtio multi-queue steering (lessons A16/A20
/// and roadmap P4/P5). Method bodies belong to the arch backend.
pub trait IrqChip: Sealed {
    /// Enable delivery of `vector` to this CPU's controller.
    fn enable(&mut self, vector: IrqVector) -> Result<(), ArchError>;

    /// Disable delivery of `vector`.
    fn disable(&mut self, vector: IrqVector) -> Result<(), ArchError>;

    /// Signal end-of-interrupt for the vector currently in service.
    fn eoi(&mut self, vector: IrqVector);

    /// Send an inter-processor interrupt carrying `vector` to `target`.
    fn send_ipi(&mut self, target: CpuId, vector: IrqVector) -> Result<(), ArchError>;

    /// Allocate and program an MSI(-X) message that, when written by a device,
    /// raises `vector` on `target` (the per-CPU queue-affinity primitive for
    /// virtio multi-queue, lesson A21 / roadmap P5).
    fn map_msi(&mut self, target: CpuId, vector: IrqVector) -> Result<MsiMessage, ArchError>;
}
