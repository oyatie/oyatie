//! What a pipeline run produced, and how a run can refuse.

use std::collections::BTreeMap;

use port_engine_api::{Declaration, Digest, Receipt, RegionId, SourceModel, UnitId};
use port_engine_emit::EmitError;
use port_engine_hash::digest_str;
use port_engine_rulepack::RulepackError;
use port_engine_snapshot::AdmitError;
use port_engine_transform::TransformError;

/// Outcome of the pin→admit→plan→transform→emit→receipt pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PipelineReport {
    /// Which unit each emitted region came from, as reported by the transform rather than parsed
    /// back out of the region id.
    pub region_units: BTreeMap<RegionId, port_engine_api::UnitId>,
    /// Bound six-axis receipt.
    pub receipt: Receipt,
    /// Kernel plan step count.
    pub plan_steps: usize,
    /// Emitted region count from syn/quote path.
    pub emit_regions: usize,
    /// Emitted region tree (for verify/delta / determinism).
    pub emitted: BTreeMap<RegionId, Vec<u8>>,
    /// Content digest of [`Self::emitted`].
    pub emit_digest: Digest,
}

/// Typed refusal from the Slice 11 pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PipelineError {
    /// Snapshot admission refused.
    Admit(AdmitError),
    /// Rulepack load failed.
    Rulepack(RulepackError),
    /// Kernel plan refused.
    Plan(port_engine_api::PortError),
    /// Construction/precondition transform refused.
    Transform(TransformError),
    /// Syn/quote emit refused.
    Emit(port_engine_api::PortError),
    /// Canary single-fixture emit refused.
    Canary(EmitError),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admit(err) => write!(f, "pipeline admit: {err}"),
            Self::Rulepack(err) => write!(f, "pipeline rulepack: {err}"),
            Self::Plan(err) => write!(f, "pipeline plan: {err}"),
            Self::Transform(err) => write!(f, "pipeline transform: {err}"),
            Self::Emit(err) => write!(f, "pipeline emit: {err}"),
            Self::Canary(err) => write!(f, "pipeline canary: {err}"),
        }
    }
}

impl std::error::Error for PipelineError {}

/// Typed refusal from the Slice 7 plan smoke.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanSmokeError {
    /// Rulepack load failed.
    Rulepack(RulepackError),
    /// Kernel plan refused.
    Port(port_engine_api::PortError),
}

impl std::fmt::Display for PlanSmokeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rulepack(err) => write!(f, "plan smoke rulepack: {err}"),
            Self::Port(err) => write!(f, "plan smoke kernel: {err}"),
        }
    }
}

impl std::error::Error for PlanSmokeError {}

pub(crate) struct RulepackModel;

impl SourceModel for RulepackModel {
    fn language(&self) -> &str {
        "go"
    }

    fn snapshot_digest(&self) -> Digest {
        digest_str("slice7-rulepack-model")
    }

    fn units(&self) -> Vec<UnitId> {
        vec![
            UnitId("example.com/a".into()),
            UnitId("example.com/b".into()),
        ]
    }

    /// The Slice 7 rulepack smoke model exists to exercise plan selection against the pack's
    /// declared example units; it carries no source and therefore declares nothing. `Some(vec![])`
    /// says exactly that, and is a different answer from the `None` an unknown unit gets.
    fn declarations(&self, unit: &UnitId) -> Option<Vec<Declaration>> {
        self.units().contains(unit).then(Vec::new)
    }
}
