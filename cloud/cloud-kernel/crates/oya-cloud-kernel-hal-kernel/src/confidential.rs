//! The confidential-compute platform seam — **stubbed shape only** (P6).
//!
//! Open question Q3 reserves *only* the type-level seam now: the
//! [`crate::mm::PhysFrame`] (private) vs [`crate::mm::SharedPhysFrame`] (shared)
//! distinction and the [`UnacceptedFrame`]/`AcceptedFrame` acceptance type-state
//! below. There is **zero** confidential-compute logic — no `#VC`/`#VE`/GHCB/
//! TDCALL/PSC, no method bodies. SEV-SNP, then TDX, then ARM CCA Realm impls
//! land in P6. This module exists so the DMA/frame paths already carry the
//! shared/private bit and never need reworking when the bodies arrive.

use crate::mm::{PhysAddr, SharedPhysFrame};
use crate::sealed::Sealed;
use crate::ArchError;

/// A guest-physical frame that has **not** yet been accepted into the
/// confidential VM's private memory (TDX lazy acceptance / SNP validation).
///
/// The acceptance type-state ([`UnacceptedFrame`] → `AcceptedFrame`) is the
/// shape reserved in P0; the accept operation itself is a P6 body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnacceptedFrame {
    addr: PhysAddr,
}

/// A guest-physical frame that has been accepted/validated into private memory
/// and is safe for the guest to touch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptedFrame {
    addr: PhysAddr,
}

impl UnacceptedFrame {
    /// Wrap an as-yet-unaccepted guest-physical address.
    pub const fn new(addr: PhysAddr) -> Self {
        Self { addr }
    }

    /// The guest-physical base address.
    pub const fn address(self) -> PhysAddr {
        self.addr
    }
}

impl AcceptedFrame {
    /// The guest-physical base address.
    pub const fn address(self) -> PhysAddr {
        self.addr
    }
}

/// The confidential-platform contract (SEV-SNP / TDX / ARM CCA Realm).
///
/// **Stub.** Every method is a `;`-terminated signature with no implementor and
/// no body anywhere — P6 work (open question Q3). Defined now only so the seam's
/// shape (page acceptance + shared/private conversion + attestation) is fixed.
/// Sealed: only an arch/platform backend may implement it, later.
pub trait ConfidentialPlatform: Sealed {
    /// Accept (validate) a guest-physical frame into private memory, yielding a
    /// usable [`AcceptedFrame`]. P6 body: SNP `PVALIDATE` / TDX `TDG.MEM.PAGE.ACCEPT`.
    fn accept_frame(&self, frame: UnacceptedFrame) -> Result<AcceptedFrame, ArchError>;

    /// Convert a private frame into a host-**shared** window (for DMA bounce
    /// buffers). P6 body: SNP page-state change / TDX `TDG.MEM.PAGE.ATTR.WR`.
    fn share_frame(&self, frame: AcceptedFrame) -> Result<SharedPhysFrame, ArchError>;

    /// Reclaim a previously-shared frame back into private memory.
    fn unshare_frame(&self, frame: SharedPhysFrame) -> Result<AcceptedFrame, ArchError>;

    /// Produce a signed attestation report over the supplied report data.
    /// P6 body: SNP `SNP_REPORT` / TDX `TDG.MR.REPORT`. Returns the number of
    /// report bytes written into `report`.
    fn attestation_report(
        &self,
        report_data: &[u8; 64],
        report: &mut [u8],
    ) -> Result<usize, ArchError>;
}
