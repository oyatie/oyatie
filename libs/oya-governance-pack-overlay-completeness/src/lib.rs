//! Scaffolded governance check for compliance pack overlay completeness.
//!
//! This crate will enforce ADR-0251 by requiring every microservice to carry
//! `packs/` overlays for each applicable compliance pack.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Stable identifier for the governance rule scaffolded by this crate.
pub const RULE_ID: &str = "ADR-0251";

/// Human-readable summary of the rule this crate enforces.
pub const ENFORCED_RULE: &str =
    "Every microservice has packs/ overlays for each applicable compliance pack.";

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

/// Enforces compliance pack overlay completeness for every microservice.
///
/// The real Wave-3-I implementation will resolve applicable compliance packs
/// for each service and fail when required `packs/` overlays are absent.
pub fn enforce_pack_overlay_completeness(
    repo_root: impl AsRef<Path>,
) -> anyhow::Result<GovernanceCheckOutcome> {
    let repo_root = repo_root.as_ref().to_path_buf();

    Ok(GovernanceCheckOutcome {
        rule_id: RULE_ID,
        enforced_rule: ENFORCED_RULE,
        repo_root,
        status: EnforcementStatus::Scaffolded,
    })
}
