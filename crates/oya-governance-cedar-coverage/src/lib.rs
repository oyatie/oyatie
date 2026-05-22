//! Scaffolded governance check for public API Cedar policy coverage.
//!
//! This crate will enforce ADR-0243 by requiring every public API endpoint to
//! have a corresponding Cedar policy under `policies/*.cedar`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Stable identifier for the governance rule scaffolded by this crate.
pub const RULE_ID: &str = "ADR-0243";

/// Human-readable summary of the rule this crate enforces.
pub const ENFORCED_RULE: &str =
    "Every public API endpoint has a corresponding Cedar policy in policies/*.cedar.";

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

/// Enforces Cedar policy coverage for public API endpoints.
///
/// The real Wave-3-I implementation will derive the public endpoint inventory,
/// parse `policies/*.cedar`, and fail endpoints that lack a matching policy.
pub fn enforce_cedar_coverage(
    repo_root: impl AsRef<Path>,
) -> anyhow::Result<GovernanceCheckOutcome> {
    let repo_root = repo_root.as_ref().to_path_buf();
    let _serialized_status = serde_yaml::to_string(&EnforcementStatus::Scaffolded)?;
    let _root_probe = walkdir::WalkDir::new(&repo_root).max_depth(0);

    Ok(GovernanceCheckOutcome {
        rule_id: RULE_ID,
        enforced_rule: ENFORCED_RULE,
        repo_root,
        status: EnforcementStatus::Scaffolded,
    })
}
