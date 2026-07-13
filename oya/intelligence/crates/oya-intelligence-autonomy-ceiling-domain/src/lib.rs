//! M02-P05-IP-002 — Per-tenant ceiling policy.
//!
//! Holds the per-tenant `AutonomyTier` ceiling. New tenants get
//! `default_ceiling()` (T3PropAct) — T4 must be enabled explicitly.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::HashMap;
use std::fmt;

use intelligence_autonomy_ceiling_kernel::{AutonomyTier, default_ceiling};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TenantId(pub String);

impl TenantId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

#[derive(Default)]
pub struct CeilingPolicy {
    ceilings: HashMap<TenantId, AutonomyTier>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CeilingPolicyError {
    EmptyTenantId,
}

impl fmt::Display for CeilingPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTenantId => f.write_str("tenant id must not be empty"),
        }
    }
}

impl std::error::Error for CeilingPolicyError {}

impl CeilingPolicy {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the ceiling for a tenant. Per ADR-0003: any change is auditable.
    pub fn set(&mut self, tenant: TenantId, tier: AutonomyTier) -> Result<(), CeilingPolicyError> {
        if tenant.0.is_empty() {
            return Err(CeilingPolicyError::EmptyTenantId);
        }
        self.ceilings.insert(tenant, tier);
        Ok(())
    }

    /// Read the effective ceiling for a tenant. Falls back to `default_ceiling()`
    /// (T3PropAct) when the tenant has no explicit policy row.
    pub fn ceiling_for(&self, tenant: &TenantId) -> AutonomyTier {
        self.ceilings
            .get(tenant)
            .copied()
            .unwrap_or_else(default_ceiling)
    }

    pub fn len(&self) -> usize {
        self.ceilings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ceilings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ceiling_for_unknown_tenant() {
        let p = CeilingPolicy::new();
        assert_eq!(
            p.ceiling_for(&TenantId::new("acme")),
            AutonomyTier::T3PropAct
        );
    }

    #[test]
    fn explicit_tenant_ceiling_returned() {
        let mut p = CeilingPolicy::new();
        p.set(TenantId::new("acme"), AutonomyTier::T2Suggest)
            .unwrap();
        assert_eq!(
            p.ceiling_for(&TenantId::new("acme")),
            AutonomyTier::T2Suggest
        );
    }

    #[test]
    fn tenants_isolated_from_each_other() {
        let mut p = CeilingPolicy::new();
        p.set(TenantId::new("a"), AutonomyTier::T1Read).unwrap();
        p.set(TenantId::new("b"), AutonomyTier::T4Actuate).unwrap();
        assert_eq!(p.ceiling_for(&TenantId::new("a")), AutonomyTier::T1Read);
        assert_eq!(p.ceiling_for(&TenantId::new("b")), AutonomyTier::T4Actuate);
    }

    #[test]
    fn rejects_empty_tenant_id() {
        let mut p = CeilingPolicy::new();
        assert_eq!(
            p.set(TenantId::new(""), AutonomyTier::T1Read),
            Err(CeilingPolicyError::EmptyTenantId)
        );
    }

    #[test]
    fn set_overwrites_existing() {
        let mut p = CeilingPolicy::new();
        p.set(TenantId::new("acme"), AutonomyTier::T1Read).unwrap();
        p.set(TenantId::new("acme"), AutonomyTier::T3PropAct)
            .unwrap();
        assert_eq!(
            p.ceiling_for(&TenantId::new("acme")),
            AutonomyTier::T3PropAct
        );
    }
}
