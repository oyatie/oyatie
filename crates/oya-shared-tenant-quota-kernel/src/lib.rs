//! Per-tenant resource-quota kernel — ADR-0155.
//!
//! # ADR-0155 (Tier-A hyperscaler pattern)
//!
//! AWS Well-Architected SaaS Lens prescribes per-tenant isolation
//! across five axes: request rate, concurrent in-flight requests,
//! memory, storage, and connection count. The tenancy µservice owns
//! the canonical quota definitions; runtime µservices query via this
//! trait.
//!
//! # Naming justification
//!
//! `oya-shared-tenant-quota-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:tenant-quota>-<layer:kernel>`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

/// ULID-shaped tenant id.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TenantId(pub String);

/// Canonical quota axes (five).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum QuotaAxis {
    RequestRate,
    ConcurrentRequests,
    Memory,
    Storage,
    Connections,
}

impl QuotaAxis {
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            QuotaAxis::RequestRate => "request_rate",
            QuotaAxis::ConcurrentRequests => "concurrent_requests",
            QuotaAxis::Memory => "memory",
            QuotaAxis::Storage => "storage",
            QuotaAxis::Connections => "connections",
        }
    }
}

/// Decision returned by the quota check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuotaDecision {
    Allowed { remaining: u64 },
    Denied { limit: u64, used: u64, retry_after_seconds: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuotaError {
    UnknownTenant(TenantId),
    NegativeAmount,
    SkeletonNotYetImplemented(&'static str),
}

impl fmt::Display for QuotaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuotaError::UnknownTenant(id) => write!(
                f,
                "oya-shared-tenant-quota-kernel: unknown tenant {id:?}"
            ),
            QuotaError::NegativeAmount => write!(
                f,
                "oya-shared-tenant-quota-kernel: amount must be non-negative"
            ),
            QuotaError::SkeletonNotYetImplemented(method) => write!(
                f,
                "oya-shared-tenant-quota-kernel: {method} is skeleton-only \
                 (tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0155-quota-impl)"
            ),
        }
    }
}

impl std::error::Error for QuotaError {}

/// The trait every µservice integrates to gate per-tenant resource use.
pub trait TenantQuotaKernel: Send + Sync {
    /// Non-mutating preview: would `amount` units of `axis` be allowed?
    ///
    /// # Errors
    /// - `UnknownTenant` when the tenant has no quota record.
    /// - `SkeletonNotYetImplemented` for the skeleton impl.
    fn check(
        &self,
        tenant_id: &TenantId,
        axis: QuotaAxis,
        amount: u64,
    ) -> Result<QuotaDecision, QuotaError>;

    /// Atomic decrement: consume `amount` if allowed.
    ///
    /// # Errors
    /// - `UnknownTenant` when the tenant has no quota record.
    /// - `SkeletonNotYetImplemented` for the skeleton impl.
    fn consume(
        &self,
        tenant_id: &TenantId,
        axis: QuotaAxis,
        amount: u64,
    ) -> Result<QuotaDecision, QuotaError>;

    /// Atomic increment: release `amount` (e.g. concurrent-requests
    /// counter when a request completes).
    ///
    /// # Errors
    /// - `UnknownTenant` when the tenant has no quota record.
    /// - `SkeletonNotYetImplemented` for the skeleton impl.
    fn release(
        &self,
        tenant_id: &TenantId,
        axis: QuotaAxis,
        amount: u64,
    ) -> Result<(), QuotaError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_wire_names_are_stable() {
        assert_eq!(QuotaAxis::RequestRate.wire_name(), "request_rate");
        assert_eq!(QuotaAxis::ConcurrentRequests.wire_name(), "concurrent_requests");
        assert_eq!(QuotaAxis::Memory.wire_name(), "memory");
        assert_eq!(QuotaAxis::Storage.wire_name(), "storage");
        assert_eq!(QuotaAxis::Connections.wire_name(), "connections");
    }

    #[test]
    fn decision_allowed_carries_remaining() {
        let d = QuotaDecision::Allowed { remaining: 5 };
        match d {
            QuotaDecision::Allowed { remaining } => assert_eq!(remaining, 5),
            QuotaDecision::Denied { .. } => panic!("wrong variant"),
        }
    }

    #[test]
    fn decision_denied_carries_retry_after() {
        let d = QuotaDecision::Denied { limit: 100, used: 100, retry_after_seconds: 30 };
        match d {
            QuotaDecision::Denied { retry_after_seconds, .. } => {
                assert_eq!(retry_after_seconds, 30);
            }
            QuotaDecision::Allowed { .. } => panic!("wrong variant"),
        }
    }

    #[test]
    fn error_display_carries_follow_up_pointer() {
        let err = QuotaError::SkeletonNotYetImplemented("check");
        let msg = format!("{err}");
        assert!(msg.contains("adr-0155-quota-impl"));
    }

    #[test]
    fn five_quota_axes_exist() {
        let axes = [
            QuotaAxis::RequestRate,
            QuotaAxis::ConcurrentRequests,
            QuotaAxis::Memory,
            QuotaAxis::Storage,
            QuotaAxis::Connections,
        ];
        assert_eq!(axes.len(), 5);
    }
}
