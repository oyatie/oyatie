//! Tenant lifecycle kernel — ports + entity types + value objects + error types.
//!
//! Wave 15-IMPL-truth-up scaffold; full implementation lands in IP-002 execution.
//! Per ADR-0105 the kernel layer is pure types: zero I/O, zero business logic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

/// Opaque tenant identifier. IP-002 will replace this with a ULID-backed newtype.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TenantId(pub String);

/// Lifecycle states per IP-003 FSM (Created → Activated → Suspended/Resumed → DeletionRequested → Deleted).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TenantStatus {
    Created,
    Activated,
    Suspended,
    Resumed,
    DeletionRequested,
    Deleted,
}

/// Plan tier classification per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanTier {
    DemoTrial,
    Paid,
}

/// Jurisdiction code (ISO 3166-1 alpha-2 plus optional sub-jurisdiction).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct JurisdictionCode(pub String);

/// Aggregate tenant root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tenant {
    pub id: TenantId,
    pub status: TenantStatus,
    pub plan_tier: PlanTier,
    pub jurisdiction: JurisdictionCode,
}

/// Cross-cutting context every adapter checkout receives.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantContext {
    pub tenant_id: TenantId,
    pub jurisdiction: JurisdictionCode,
}

/// Sealed port for persistence access.
pub trait TenantRepository {
    fn find(&self, id: &TenantId) -> Result<Option<Tenant>, TenantKernelError>;
    fn save(&self, tenant: &Tenant) -> Result<(), TenantKernelError>;
}

/// Sealed port for resolving request → tenant context.
pub trait TenantContextResolver {
    fn resolve(&self, principal: &str) -> Result<TenantContext, TenantKernelError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantKernelError {
    NotFound,
    InvalidTransition { from: TenantStatus, to: TenantStatus },
    PersistenceUnavailable,
}
