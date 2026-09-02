use crate::{CurrencyCode, Digest32, MovementBudgetV1, PlacementPolicyGeneration};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferBudgetRequirementV1 {
    pub transfer_effect_manifest_digest: Digest32,
    pub maximum_total_bytes: u64,
    pub maximum_total_effects: u64,
    pub maximum_total_cost_microunits: u64,
    pub currency: CurrencyCode,
    pub requirement_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementBudgetRequestV1 {
    pub placement_policy_generation: PlacementPolicyGeneration,
    pub transfer: TransferBudgetRequirementV1,
    pub ordinary_budget: MovementBudgetV1,
    pub required_forward_completion_reserve: MovementBudgetV1,
    pub requested_window_start_unix_seconds: u64,
    pub requested_window_end_unix_seconds: u64,
    pub request_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForwardCompletionCoverageV1 {
    pub transfer_requirement_digest: Digest32,
    pub transfer_effect_manifest_digest: Digest32,
    pub required_bytes: u64,
    pub required_effects: u64,
    pub required_cost_microunits: u64,
    pub reserved_bytes: u64,
    pub reserved_effects: u64,
    pub reserved_cost_microunits: u64,
    pub disposition: ForwardCompletionCoverageDispositionV1,
    pub coverage_proof_digest: Digest32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ForwardCompletionCoverageDispositionV1 {
    CompleteForSealedManifestAndNonRevocableAfterFence,
}
