//! M02-P05-IP-002 — Autonomy ceiling kernel.
//!
//! Pure tier-comparison kernel. T4Actuate is the highest tier and
//! is *disabled by default* — only a tenant ceiling that explicitly
//! permits T4Actuate will allow a T4 cap through.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod tenant_ceiling;
pub use tenant_ceiling::{
    BatchCeilingVerdict, TenantCeiling, TenantCeilingVerdict, resolve, resolve_batch,
};

use std::fmt;

/// Re-export-style local tier (kernel-internal copy to keep this crate
/// kernel-pure with respect to its own surface). For consumers wanting a
/// single source of truth, see `intelligence_capability_registry_kernel::AutonomyTier`;
/// both enums are wire-compatible by name.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum AutonomyTier {
    T1Read = 1,
    T2Suggest = 2,
    T3PropAct = 3,
    T4Actuate = 4,
}

impl AutonomyTier {
    pub fn label(&self) -> &'static str {
        match self {
            Self::T1Read => "T1Read",
            Self::T2Suggest => "T2Suggest",
            Self::T3PropAct => "T3PropAct",
            Self::T4Actuate => "T4Actuate",
        }
    }
}

impl fmt::Display for AutonomyTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CeilingVerdict {
    Allow,
    /// Block: capability requested a tier strictly greater than the ceiling.
    Block {
        capability_tier: AutonomyTier,
        ceiling: AutonomyTier,
    },
}

/// Compare capability tier against a tenant ceiling.
///
/// Rule: capability is allowed iff `capability_tier <= ceiling`.
/// The seed default ceiling for new tenants is `T3PropAct`, which
/// blocks T4Actuate (mirrors Cedar policy `Action::"actuate-t4"`).
pub fn check_tier(capability_tier: AutonomyTier, ceiling: AutonomyTier) -> CeilingVerdict {
    if capability_tier <= ceiling {
        CeilingVerdict::Allow
    } else {
        CeilingVerdict::Block {
            capability_tier,
            ceiling,
        }
    }
}

/// Default tenant ceiling: T3PropAct. T4 is opt-in only.
pub fn default_ceiling() -> AutonomyTier {
    AutonomyTier::T3PropAct
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t1_under_t3_ceiling_allowed() {
        assert_eq!(
            check_tier(AutonomyTier::T1Read, AutonomyTier::T3PropAct),
            CeilingVerdict::Allow
        );
    }

    #[test]
    fn t3_under_t3_ceiling_allowed() {
        assert_eq!(
            check_tier(AutonomyTier::T3PropAct, AutonomyTier::T3PropAct),
            CeilingVerdict::Allow
        );
    }

    #[test]
    fn t4_under_t3_ceiling_blocked() {
        let verdict = check_tier(AutonomyTier::T4Actuate, AutonomyTier::T3PropAct);
        assert!(matches!(verdict, CeilingVerdict::Block { .. }));
    }

    #[test]
    fn t4_under_t4_ceiling_allowed_when_explicitly_opted_in() {
        assert_eq!(
            check_tier(AutonomyTier::T4Actuate, AutonomyTier::T4Actuate),
            CeilingVerdict::Allow
        );
    }

    #[test]
    fn default_ceiling_blocks_t4() {
        let v = check_tier(AutonomyTier::T4Actuate, default_ceiling());
        assert!(matches!(v, CeilingVerdict::Block { .. }));
    }

    #[test]
    fn tier_ordering_preserved() {
        assert!(AutonomyTier::T1Read < AutonomyTier::T4Actuate);
    }

    #[test]
    fn block_carries_diagnostic_tiers() {
        let v = check_tier(AutonomyTier::T4Actuate, AutonomyTier::T2Suggest);
        match v {
            CeilingVerdict::Block {
                capability_tier,
                ceiling,
            } => {
                assert_eq!(capability_tier, AutonomyTier::T4Actuate);
                assert_eq!(ceiling, AutonomyTier::T2Suggest);
            }
            _ => panic!("expected Block verdict"),
        }
    }
}
