//! Scaffolded governance check for ADR-0316 capability-tier coverage.
//!
//! This crate will enforce that `registry/capability-tiers/microservice-tier-mapping.yaml`
//! declares one capability-tier mapping entry for every microservice.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Stable identifier for the governance rule scaffolded by this crate.
pub const RULE_ID: &str = "ADR-0316";

/// Human-readable summary of the rule this crate enforces.
pub const ENFORCED_RULE: &str = "Every microservice has a capability-tier entry in registry/capability-tiers/microservice-tier-mapping.yaml.";

/// Status returned by the Wave-3-I scaffold before full enforcement lands.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EnforcementStatus {
    Scaffolded,
}

/// Machine-readable outcome from the scaffolded enforcement entrypoint.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GovernanceCheckOutcome {
    pub rule_id: &'static str,
    pub enforced_rule: &'static str,
    pub repo_root: PathBuf,
    pub status: EnforcementStatus,
}

impl GovernanceCheckOutcome {
    pub fn is_scaffolded(&self) -> bool {
        self.status == EnforcementStatus::Scaffolded
    }
}

/// Enforces ADR-0316 capability-tier coverage for every microservice.
///
/// The real Wave-3-I implementation will compare the canonical microservice
/// inventory with `registry/capability-tiers/microservice-tier-mapping.yaml`
/// and fail when any service is missing a tier row.
pub fn enforce_capability_tier_coverage(
    repo_root: impl AsRef<Path>,
) -> anyhow::Result<GovernanceCheckOutcome> {
    let repo_root = repo_root.as_ref().to_path_buf();
    let _root_probe = walkdir::WalkDir::new(&repo_root).max_depth(0);

    Ok(GovernanceCheckOutcome {
        rule_id: RULE_ID,
        enforced_rule: ENFORCED_RULE,
        repo_root,
        status: EnforcementStatus::Scaffolded,
    })
}
