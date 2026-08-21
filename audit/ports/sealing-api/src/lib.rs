//! Audit-chain sealing API: command/result DTOs for the worker cycle.
//!
//! Command/result DTOs for the sealing worker cycle. The seal-record
//! construction, status-transition and epoch-coverage rules live in
//! `audit/core/sealing-domain`.
#![allow(dead_code)]

use audit_sealing_kernel::{SealRecord, SealStatus};

/// Manual replay or periodic worker-cycle request.
#[derive(Clone, Debug)]
pub struct SealCycleCommand {
    pub pack: String,
    pub tenant_partition: String,
    pub period_id: String,
}

/// Worker-cycle result.
#[derive(Clone, Debug)]
pub enum SealCycleResult {
    Minted(SealRecord),
    AlreadyMinted(SealRecord),
    Degraded { reason: String, status: SealStatus },
}
