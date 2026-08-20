//! M02-P05-IP-002 — `EnforceCeiling` use-case.
//!
//! Bridges the capability registry's tier enum into the ceiling kernel's
//! tier enum (both wire-compatible by name) and dispatches `check_tier`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use intelligence_autonomy_ceiling_domain::{CeilingPolicy, TenantId};
use intelligence_autonomy_ceiling_kernel::{
    AutonomyTier as CeilingTier, CeilingVerdict, check_tier,
};
use intelligence_capability_registry_kernel::{AutonomyTier as CapTier, Capability};

fn bridge(t: CapTier) -> CeilingTier {
    match t {
        CapTier::T1Read => CeilingTier::T1Read,
        CapTier::T2Suggest => CeilingTier::T2Suggest,
        CapTier::T3PropAct => CeilingTier::T3PropAct,
        CapTier::T4Actuate => CeilingTier::T4Actuate,
    }
}

/// Use-case: enforce the tenant ceiling against a capability.
pub fn enforce(cap: &Capability, tenant_ceiling: CeilingTier) -> CeilingVerdict {
    check_tier(bridge(cap.autonomy_tier), tenant_ceiling)
}

/// Use-case: enforce the per-tenant policy from `CeilingPolicy`.
pub fn enforce_for_tenant(
    cap: &Capability,
    tenant: &TenantId,
    policy: &CeilingPolicy,
) -> CeilingVerdict {
    enforce(cap, policy.ceiling_for(tenant))
}

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_capability_registry_kernel::CapabilityId;

    fn cap(tier: CapTier) -> Capability {
        Capability::new(CapabilityId::new("foundry.x.y"), "n", tier, true)
    }

    #[test]
    fn t1_cap_allowed_under_t3_ceiling() {
        let v = enforce(&cap(CapTier::T1Read), CeilingTier::T3PropAct);
        assert_eq!(v, CeilingVerdict::Allow);
    }

    #[test]
    fn t4_cap_blocked_under_t3_ceiling() {
        let v = enforce(&cap(CapTier::T4Actuate), CeilingTier::T3PropAct);
        assert!(matches!(v, CeilingVerdict::Block { .. }));
    }

    #[test]
    fn t4_cap_allowed_when_ceiling_t4() {
        let v = enforce(&cap(CapTier::T4Actuate), CeilingTier::T4Actuate);
        assert_eq!(v, CeilingVerdict::Allow);
    }

    #[test]
    fn enforce_for_tenant_default_blocks_t4() {
        let policy = CeilingPolicy::new();
        let v = enforce_for_tenant(&cap(CapTier::T4Actuate), &TenantId::new("acme"), &policy);
        assert!(matches!(v, CeilingVerdict::Block { .. }));
    }

    #[test]
    fn enforce_for_tenant_explicit_ceiling_allows_match() {
        let mut policy = CeilingPolicy::new();
        policy
            .set(TenantId::new("acme"), CeilingTier::T2Suggest)
            .unwrap();
        let v = enforce_for_tenant(&cap(CapTier::T2Suggest), &TenantId::new("acme"), &policy);
        assert_eq!(v, CeilingVerdict::Allow);
    }

    #[test]
    fn enforce_for_tenant_explicit_ceiling_blocks_over() {
        let mut policy = CeilingPolicy::new();
        policy
            .set(TenantId::new("acme"), CeilingTier::T2Suggest)
            .unwrap();
        let v = enforce_for_tenant(&cap(CapTier::T3PropAct), &TenantId::new("acme"), &policy);
        assert!(matches!(v, CeilingVerdict::Block { .. }));
    }
}
