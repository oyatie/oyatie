//! Scaffolded governance check for audit event emission.
//!
//! This crate will enforce ADR-0263 by requiring every state-changing endpoint
//! to emit a registered audit event class.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Stable identifier for the governance rule scaffolded by this crate.
pub const RULE_ID: &str = "ADR-0263";

/// Human-readable summary of the rule this crate enforces.
pub const ENFORCED_RULE: &str =
    "Every state-changing endpoint emits an ADR-0263 registered audit event class.";

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

/// Enforces audit event emission for state-changing endpoints.
///
/// The real Wave-3-I implementation will identify mutating routes, compare them
/// to ADR-0263 audit event class registrations, and fail any route without an
/// emitted registered class.
pub fn enforce_audit_event_emission(
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
