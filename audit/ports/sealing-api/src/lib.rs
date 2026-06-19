//! Audit-chain sealing API: command/result DTOs for the worker cycle.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full schema in IP-010.
#![allow(dead_code)]

use audit_sealing_kernel::{SealRecord, SealStatus};

/// Manual replay or periodic worker-cycle request.
#[derive(Clone, Debug)]
pub struct SealCycleCommand {
    pub pack: String,
    pub tenant_partition: String,
    pub period_id: String,
}

/// Worker-cycle result. Full enum in IP-010.
#[derive(Clone, Debug)]
pub enum SealCycleResult {
    Minted(SealRecord),
    AlreadyMinted(SealRecord),
    Degraded { reason: String, status: SealStatus },
}
