//! Cell assignment kernel — CellId, ShardKey, CellHealth, RebalanceTask + ports.
//!
//! Wave 15-IMPL-truth-up scaffold; full implementation lands in IP-008 execution.
//! Per ADR-0248 + oyatie-shuffle-sharding crate, cellular architecture is the
//! pattern; this crate owns the assignment-decision concern.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CellId(pub String); // data_class: INTERNAL_ONLY

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ShardKey(pub u64); // data_class: INTERNAL_ONLY

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellHealth {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebalanceTask {
    pub tenant: String,    // data_class: INTERNAL_ONLY
    pub from_cell: CellId, // data_class: INTERNAL_ONLY
    pub to_cell: CellId,   // data_class: INTERNAL_ONLY
    pub reason: String,    // data_class: INTERNAL_ONLY
}

pub trait CellAssignmentRepository {
    fn assigned_cell(&self, tenant: &str) -> Result<Option<CellId>, CellKernelError>;
    fn record_assignment(&self, tenant: &str, cell: &CellId) -> Result<(), CellKernelError>;
}

pub trait CellHealthProbe {
    fn probe(&self, cell: &CellId) -> Result<CellHealth, CellKernelError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellKernelError {
    NoHealthyCell,
    ProbeFailed,
    PersistenceUnavailable,
    RebalanceConflict,
}
