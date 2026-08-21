//! Shared fixture for the rulepack tests.

use std::collections::BTreeMap;

use port_engine_api::{Declaration, Digest, PackSemantics, RuleId, RulePack, SourceModel, UnitId};
use port_engine_hash::digest_bytes;
use port_engine_rulepack::{LoadedRule, LoadedRulePack, RulepackError, w0_ready};

pub struct TinyModel {
    pub units: Vec<UnitId>,
}

impl SourceModel for TinyModel {
    fn language(&self) -> &str {
        "go"
    }
    fn snapshot_digest(&self) -> Digest {
        Digest("snap".into())
    }
    fn units(&self) -> Vec<UnitId> {
        self.units.clone()
    }
    fn declarations(&self, unit: &UnitId) -> Option<Vec<Declaration>> {
        self.units.contains(unit).then(Vec::new)
    }
}
