//! The virtio transport seam.
//!
//! kernel is a virtio-only device stack (NO-LEGACY §7). The two
//! transports — virtio-pci+MSI-X on x86, virtio-mmio+GIC-SPI on aarch64 — differ
//! only in how the driver reaches the device's config space, queues, and
//! notification doorbell. [`VirtioTransport`] is that arch-neutral surface; the
//! generic `VirtQueue<SIZE>` driver layer (roadmap P5) sits above it. Sealed;
//! the arch backend (or a thin transport shim) implements it. Shape only — no
//! device logic here.

use crate::mm::DmaAddr;
use crate::sealed::Sealed;
use crate::ArchError;

/// The negotiated feature bitmap (virtio 1.2 split into 32-bit windows).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VirtioFeatures {
    /// Feature bits 0..32.
    pub low: u32,
    /// Feature bits 32..64.
    pub high: u32,
}

/// Arch-neutral access to a virtio device's transport (config space, queue
/// programming, and the notify doorbell).
///
/// Sealed; bodies belong to the transport shim/arch backend in P5.
pub trait VirtioTransport: Sealed {
    /// The virtio device type ID (1 = net, 2 = blk, …) read from the device.
    fn device_id(&self) -> u32;

    /// Read the device-offered feature bits.
    fn device_features(&self) -> VirtioFeatures;

    /// Write back the subset of features the driver accepts.
    fn set_driver_features(&mut self, features: VirtioFeatures) -> Result<(), ArchError>;

    /// Read the current device status byte.
    fn status(&self) -> u8;

    /// Write the device status byte (drives the
    /// `Uninit→FeatureNegotiated→Ready` lifecycle).
    fn set_status(&mut self, status: u8);

    /// Program virtqueue `queue` with the device-visible base address of its
    /// descriptor area (the DMA ring lives in a [`crate::mm::DmaRegion`]).
    fn set_queue(&mut self, queue: u16, descriptor_addr: DmaAddr) -> Result<(), ArchError>;

    /// Ring the notification doorbell for `queue` (tells the device new buffers
    /// are available).
    fn notify(&mut self, queue: u16);
}
