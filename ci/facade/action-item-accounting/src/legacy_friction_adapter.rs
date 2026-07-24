//! Transitional predecessor-ledger adapter.
//!
//! The durable `fixuptask_v2` gate has no dependency on this module. This
//! adapter retains predecessor census and mapping checks until a separately
//! authorized qualified-human migration completes.

use std::collections::BTreeSet;
use std::path::Path;

use crate::{CollectError, Finding, evaluate_legacy_friction_materialized_gate};

pub const GATE_ID: &str = "cloud-ci-legacy-friction-adapter";

pub fn evaluate_materialized_gate(root: &Path) -> Result<BTreeSet<Finding>, CollectError> {
    evaluate_legacy_friction_materialized_gate(root)
}
