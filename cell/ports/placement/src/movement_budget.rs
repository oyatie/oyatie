use crate::{
    CellId, CurrencyCode, Digest32, ImmutableEvidenceRefV1, PlacementPartitionV1,
    PlacementPolicyGeneration, ProofConstructionError, TenantId,
};

macro_rules! opaque_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn parse(_value: impl Into<String>) -> Result<Self, ProofConstructionError> {
                Err(ProofConstructionError::NotImplemented)
            }
        }
    };
}

opaque_id!(MovementBudgetAuthorityPartition);
opaque_id!(MovementBudgetGrantId);
opaque_id!(MovementBudgetDelegationId);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementBudgetV1 {
    pub maximum_bytes: u64,
    pub maximum_effects: u64,
    pub maximum_cost_microunits: u64,
    pub currency: CurrencyCode,
    pub budget_relation_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForwardCompletionReserveV1 {
    pub reserved_bytes: u64,
    pub reserved_effects: u64,
    pub reserved_cost_microunits: u64,
    pub disposition: ForwardCompletionReserveDispositionV1,
    pub reserve_digest: Digest32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ForwardCompletionReserveDispositionV1 {
    NonRevocableAfterFence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MovementBudgetScopeV1 {
    FleetPartition(PlacementPartitionV1),
    Cell(CellId),
    Tenant(TenantId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementBudgetScopeAllocationV1 {
    pub scope: MovementBudgetScopeV1,
    pub window_start_unix_seconds: u64,
    pub window_end_unix_seconds: u64,
    pub ordinary_budget: MovementBudgetV1,
    pub forward_completion_reserve: ForwardCompletionReserveV1,
    pub quota_allocation_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementBudgetDelegationV1 {
    pub authority_partition: MovementBudgetAuthorityPartition,
    pub delegation_id: MovementBudgetDelegationId,
    pub parent_scope: MovementBudgetScopeV1,
    pub child_allocation: MovementBudgetScopeAllocationV1,
    pub parent_delegation_digest: Option<Digest32>,
    pub parent_state_revision_at_commit: MovementBudgetAuthorityRevision,
    pub parent_state_record_digest_at_commit: Digest32,
    pub child_state_revision_at_commit: MovementBudgetAuthorityRevision,
    pub child_state_record_digest_at_commit: Digest32,
    pub record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementBudgetLineageV1 {
    pub fleet_to_cell: MovementBudgetDelegationV1,
    pub cell_to_tenant: MovementBudgetDelegationV1,
    pub ordered_delegation_root_digest: Digest32,
    pub delegation_count: u64,
    pub lineage_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementBudgetLineageExpectationV1 {
    pub authority_partition: MovementBudgetAuthorityPartition,
    pub fleet_partition: PlacementPartitionV1,
    pub cell_id: CellId,
    pub tenant_id: TenantId,
    pub policy_generation: PlacementPolicyGeneration,
    pub window_start_unix_seconds: u64,
    pub window_end_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedMovementBudgetLineage(MovementBudgetLineageV1);

impl VerifiedMovementBudgetLineage {
    #[must_use]
    pub fn lineage(&self) -> &MovementBudgetLineageV1 {
        &self.0
    }
}

pub fn verify_movement_budget_lineage(
    _lineage: MovementBudgetLineageV1,
    _fleet_state: &MovementBudgetAuthorityStateV1,
    _cell_state: &MovementBudgetAuthorityStateV1,
    _tenant_state: &MovementBudgetAuthorityStateV1,
    _expectation: &MovementBudgetLineageExpectationV1,
) -> Result<VerifiedMovementBudgetLineage, crate::PlacementContractError> {
    Err(crate::PlacementContractError::NotImplemented)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementBudgetGrantV1 {
    pub authority_partition: MovementBudgetAuthorityPartition,
    pub grant_id: MovementBudgetGrantId,
    pub lineage: MovementBudgetLineageV1,
    pub leaf_scope_allocation: MovementBudgetScopeAllocationV1,
    pub leaf_state_revision_at_commit: MovementBudgetAuthorityRevision,
    pub leaf_state_record_digest_at_commit: Digest32,
    pub grant_digest: Digest32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MovementBudgetAuthorityRevision(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementBudgetAuthorityStateV1 {
    pub authority_partition: MovementBudgetAuthorityPartition,
    pub scope: MovementBudgetScopeV1,
    pub parent_delegation_digest: Option<Digest32>,
    pub policy_generation: PlacementPolicyGeneration,
    pub window_start_unix_seconds: u64,
    pub window_end_unix_seconds: u64,
    pub ordinary_ceiling: MovementBudgetV1,
    pub ordinary_delegated_bytes: u64,
    pub ordinary_delegated_effects: u64,
    pub ordinary_delegated_cost_microunits: u64,
    pub ordinary_granted_bytes: u64,
    pub ordinary_granted_effects: u64,
    pub ordinary_granted_cost_microunits: u64,
    pub forward_completion_ceiling: ForwardCompletionReserveV1,
    pub forward_completion_delegated_bytes: u64,
    pub forward_completion_delegated_effects: u64,
    pub forward_completion_delegated_cost_microunits: u64,
    pub forward_completion_outstanding_bytes: u64,
    pub forward_completion_outstanding_effects: u64,
    pub forward_completion_outstanding_cost_microunits: u64,
    pub outstanding_movement_count: u64,
    pub ordered_delegation_root_digest: Digest32,
    pub delegation_count: u64,
    pub ordered_grant_root_digest: Digest32,
    pub grant_count: u64,
    pub policy_evidence: ImmutableEvidenceRefV1,
    pub revision: MovementBudgetAuthorityRevision,
    pub record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovementBudgetAuthorityPreconditionV1 {
    pub authority_partition: MovementBudgetAuthorityPartition,
    pub scope: MovementBudgetScopeV1,
    pub expected_revision: MovementBudgetAuthorityRevision,
    pub expected_record_digest: Digest32,
}
