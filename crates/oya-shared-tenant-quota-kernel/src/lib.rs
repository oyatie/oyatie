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

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

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
    Allowed {
        remaining: u64,
    },
    Denied {
        limit: u64,
        used: u64,
        retry_after_seconds: u64,
    },
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
            QuotaError::UnknownTenant(id) => {
                write!(f, "oya-shared-tenant-quota-kernel: unknown tenant {id:?}")
            }
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
    fn release(&self, tenant_id: &TenantId, axis: QuotaAxis, amount: u64)
    -> Result<(), QuotaError>;
}

// ---------------------------------------------------------------------------
// In-memory reference implementation
// ---------------------------------------------------------------------------

/// Per-axis counter: tracks the configured limit and current usage.
#[derive(Clone, Debug)]
struct AxisState {
    limit: u64,
    used: u64,
}

impl AxisState {
    fn new(limit: u64) -> Self {
        Self { limit, used: 0 }
    }

    /// Compute the decision for `amount` additional units (non-mutating).
    fn decide(&self, amount: u64) -> QuotaDecision {
        let projected = self.used.saturating_add(amount);
        if projected <= self.limit {
            QuotaDecision::Allowed {
                remaining: self.limit - projected,
            }
        } else {
            QuotaDecision::Denied {
                limit: self.limit,
                used: self.used,
                retry_after_seconds: 1,
            }
        }
    }
}

type TenantMap = HashMap<TenantId, HashMap<QuotaAxis, AxisState>>;

/// Builder for [`InMemoryTenantQuota`].
///
/// Register each (tenant, axis, limit) triple before calling [`build`](Self::build).
#[derive(Default)]
pub struct InMemoryTenantQuotaBuilder {
    map: TenantMap,
}

impl InMemoryTenantQuotaBuilder {
    /// Register `limit` units on `axis` for `tenant_id`.
    ///
    /// Calling this multiple times for the same (tenant, axis) pair overwrites
    /// the previous limit.
    pub fn register(mut self, tenant_id: TenantId, axis: QuotaAxis, limit: u64) -> Self {
        self.map
            .entry(tenant_id)
            .or_default()
            .insert(axis, AxisState::new(limit));
        self
    }

    /// Consume the builder and produce an [`InMemoryTenantQuota`].
    pub fn build(self) -> InMemoryTenantQuota {
        InMemoryTenantQuota {
            state: Mutex::new(self.map),
        }
    }
}

/// Pure in-memory, deterministic implementation of [`TenantQuotaKernel`].
///
/// All five [`QuotaAxis`] variants are supported. Construct via
/// [`InMemoryTenantQuota::builder()`].
///
/// # Thread safety
/// Interior mutability via `std::sync::Mutex`; `Send + Sync` automatically.
pub struct InMemoryTenantQuota {
    state: Mutex<TenantMap>,
}

impl InMemoryTenantQuota {
    /// Return a fresh builder.
    pub fn builder() -> InMemoryTenantQuotaBuilder {
        InMemoryTenantQuotaBuilder::default()
    }
}

impl TenantQuotaKernel for InMemoryTenantQuota {
    fn check(
        &self,
        tenant_id: &TenantId,
        axis: QuotaAxis,
        amount: u64,
    ) -> Result<QuotaDecision, QuotaError> {
        let guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let tenant = guard
            .get(tenant_id)
            .ok_or_else(|| QuotaError::UnknownTenant(tenant_id.clone()))?;
        // Axis not configured for this tenant — treat as unknown tenant context.
        let axis_state = tenant
            .get(&axis)
            .ok_or_else(|| QuotaError::UnknownTenant(tenant_id.clone()))?;
        Ok(axis_state.decide(amount))
    }

    fn consume(
        &self,
        tenant_id: &TenantId,
        axis: QuotaAxis,
        amount: u64,
    ) -> Result<QuotaDecision, QuotaError> {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let tenant = guard
            .get_mut(tenant_id)
            .ok_or_else(|| QuotaError::UnknownTenant(tenant_id.clone()))?;
        let axis_state = tenant
            .get_mut(&axis)
            .ok_or_else(|| QuotaError::UnknownTenant(tenant_id.clone()))?;
        let decision = axis_state.decide(amount);
        if let QuotaDecision::Allowed { .. } = decision {
            axis_state.used = axis_state.used.saturating_add(amount);
        }
        Ok(decision)
    }

    fn release(
        &self,
        tenant_id: &TenantId,
        axis: QuotaAxis,
        amount: u64,
    ) -> Result<(), QuotaError> {
        let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let tenant = guard
            .get_mut(tenant_id)
            .ok_or_else(|| QuotaError::UnknownTenant(tenant_id.clone()))?;
        let axis_state = tenant
            .get_mut(&axis)
            .ok_or_else(|| QuotaError::UnknownTenant(tenant_id.clone()))?;
        axis_state.used = axis_state.used.saturating_sub(amount);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Original skeleton tests (must still pass)
    // ------------------------------------------------------------------

    #[test]
    fn axis_wire_names_are_stable() {
        assert_eq!(QuotaAxis::RequestRate.wire_name(), "request_rate");
        assert_eq!(
            QuotaAxis::ConcurrentRequests.wire_name(),
            "concurrent_requests"
        );
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
        let d = QuotaDecision::Denied {
            limit: 100,
            used: 100,
            retry_after_seconds: 30,
        };
        match d {
            QuotaDecision::Denied {
                retry_after_seconds,
                ..
            } => {
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

    // ------------------------------------------------------------------
    // InMemoryTenantQuota acceptance tests
    // ------------------------------------------------------------------

    fn tid(s: &str) -> TenantId {
        TenantId(s.to_string())
    }

    fn make_quota(limit: u64) -> InMemoryTenantQuota {
        InMemoryTenantQuota::builder()
            .register(tid("t1"), QuotaAxis::RequestRate, limit)
            .build()
    }

    // 1. check under limit returns Allowed with correct remaining
    #[test]
    fn check_under_limit_returns_allowed() {
        let q = make_quota(10);
        let result = q.check(&tid("t1"), QuotaAxis::RequestRate, 3).unwrap();
        assert_eq!(result, QuotaDecision::Allowed { remaining: 7 });
    }

    // 2. check at limit (used=0, amount=limit) returns Allowed{remaining:0}
    #[test]
    fn check_at_exact_limit_returns_allowed_zero_remaining() {
        let q = make_quota(10);
        let result = q.check(&tid("t1"), QuotaAxis::RequestRate, 10).unwrap();
        assert_eq!(result, QuotaDecision::Allowed { remaining: 0 });
    }

    // 3. check over limit returns Denied
    #[test]
    fn check_over_limit_returns_denied() {
        let q = make_quota(5);
        let result = q.check(&tid("t1"), QuotaAxis::RequestRate, 6).unwrap();
        assert_eq!(
            result,
            QuotaDecision::Denied {
                limit: 5,
                used: 0,
                retry_after_seconds: 1,
            }
        );
    }

    // 4. check is non-mutating: two consecutive checks return the same result
    #[test]
    fn check_is_non_mutating() {
        let q = make_quota(10);
        let r1 = q.check(&tid("t1"), QuotaAxis::RequestRate, 3).unwrap();
        let r2 = q.check(&tid("t1"), QuotaAxis::RequestRate, 3).unwrap();
        assert_eq!(r1, r2);
    }

    // 5. consume decrements used
    #[test]
    fn consume_decrements_used() {
        let q = make_quota(10);
        let r1 = q.consume(&tid("t1"), QuotaAxis::RequestRate, 4).unwrap();
        assert_eq!(r1, QuotaDecision::Allowed { remaining: 6 });
        // second consume sees reduced remaining
        let r2 = q.check(&tid("t1"), QuotaAxis::RequestRate, 1).unwrap();
        assert_eq!(r2, QuotaDecision::Allowed { remaining: 5 });
    }

    // 6. consume returns Denied when at limit (no mutation)
    #[test]
    fn consume_denied_at_limit_does_not_mutate() {
        let q = make_quota(3);
        // fill up
        q.consume(&tid("t1"), QuotaAxis::RequestRate, 3).unwrap();
        // now at limit — consume should deny
        let denied = q.consume(&tid("t1"), QuotaAxis::RequestRate, 1).unwrap();
        assert_eq!(
            denied,
            QuotaDecision::Denied {
                limit: 3,
                used: 3,
                retry_after_seconds: 1,
            }
        );
        // check still returns Denied (used didn't change)
        let still_denied = q.check(&tid("t1"), QuotaAxis::RequestRate, 1).unwrap();
        assert_eq!(
            still_denied,
            QuotaDecision::Denied {
                limit: 3,
                used: 3,
                retry_after_seconds: 1,
            }
        );
    }

    // 7. consume on unknown tenant returns UnknownTenant
    #[test]
    fn consume_unknown_tenant_errors() {
        let q = make_quota(10);
        let err = q
            .consume(&tid("ghost"), QuotaAxis::RequestRate, 1)
            .unwrap_err();
        assert_eq!(err, QuotaError::UnknownTenant(tid("ghost")));
    }

    // 8. release decrements used (frees capacity)
    #[test]
    fn release_frees_capacity() {
        let q = make_quota(5);
        q.consume(&tid("t1"), QuotaAxis::RequestRate, 5).unwrap();
        // at limit
        assert!(matches!(
            q.check(&tid("t1"), QuotaAxis::RequestRate, 1).unwrap(),
            QuotaDecision::Denied { .. }
        ));
        // release 2
        q.release(&tid("t1"), QuotaAxis::RequestRate, 2).unwrap();
        // now should be allowed again
        assert_eq!(
            q.check(&tid("t1"), QuotaAxis::RequestRate, 2).unwrap(),
            QuotaDecision::Allowed { remaining: 0 }
        );
    }

    // 9. release clamps to 0 (no underflow / negative used)
    #[test]
    fn release_clamps_at_zero() {
        let q = make_quota(10);
        // used is 0; releasing 5 should not wrap
        q.release(&tid("t1"), QuotaAxis::RequestRate, 5).unwrap();
        let result = q.check(&tid("t1"), QuotaAxis::RequestRate, 10).unwrap();
        assert_eq!(result, QuotaDecision::Allowed { remaining: 0 });
    }

    // 10. release on unknown tenant returns UnknownTenant
    #[test]
    fn release_unknown_tenant_errors() {
        let q = make_quota(10);
        let err = q
            .release(&tid("ghost"), QuotaAxis::RequestRate, 1)
            .unwrap_err();
        assert_eq!(err, QuotaError::UnknownTenant(tid("ghost")));
    }

    // 11. full cycle: consume to limit -> denied -> release -> allowed
    #[test]
    fn full_cycle_consume_deny_release_allow() {
        let q = make_quota(2);
        q.consume(&tid("t1"), QuotaAxis::RequestRate, 2).unwrap();
        assert!(matches!(
            q.consume(&tid("t1"), QuotaAxis::RequestRate, 1).unwrap(),
            QuotaDecision::Denied { .. }
        ));
        q.release(&tid("t1"), QuotaAxis::RequestRate, 1).unwrap();
        assert_eq!(
            q.consume(&tid("t1"), QuotaAxis::RequestRate, 1).unwrap(),
            QuotaDecision::Allowed { remaining: 0 }
        );
    }

    // 12. all five axes can be configured and queried independently
    #[test]
    fn all_five_axes_independent() {
        let q = InMemoryTenantQuota::builder()
            .register(tid("t1"), QuotaAxis::RequestRate, 10)
            .register(tid("t1"), QuotaAxis::ConcurrentRequests, 20)
            .register(tid("t1"), QuotaAxis::Memory, 30)
            .register(tid("t1"), QuotaAxis::Storage, 40)
            .register(tid("t1"), QuotaAxis::Connections, 50)
            .build();

        // consume on one axis does not affect others
        q.consume(&tid("t1"), QuotaAxis::RequestRate, 10).unwrap();

        assert!(matches!(
            q.check(&tid("t1"), QuotaAxis::RequestRate, 1).unwrap(),
            QuotaDecision::Denied { .. }
        ));
        assert_eq!(
            q.check(&tid("t1"), QuotaAxis::ConcurrentRequests, 20)
                .unwrap(),
            QuotaDecision::Allowed { remaining: 0 }
        );
        assert_eq!(
            q.check(&tid("t1"), QuotaAxis::Memory, 30).unwrap(),
            QuotaDecision::Allowed { remaining: 0 }
        );
        assert_eq!(
            q.check(&tid("t1"), QuotaAxis::Storage, 40).unwrap(),
            QuotaDecision::Allowed { remaining: 0 }
        );
        assert_eq!(
            q.check(&tid("t1"), QuotaAxis::Connections, 50).unwrap(),
            QuotaDecision::Allowed { remaining: 0 }
        );
    }

    // 13. check on unknown tenant returns UnknownTenant
    #[test]
    fn check_unknown_tenant_errors() {
        let q = make_quota(10);
        let err = q
            .check(&tid("nobody"), QuotaAxis::RequestRate, 1)
            .unwrap_err();
        assert_eq!(err, QuotaError::UnknownTenant(tid("nobody")));
    }
}
