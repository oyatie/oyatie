use crate::{
    BoxCellFuture, CapacityVectorV1, CellCapacityLedgerV1, CellControlAuditRecordV1,
    CellControlIdempotencyRecordV1, CellControlOperationPreconditionV1, CellControlOperationV1,
    CellControlPersistenceAuthorityV1, CellControlReadAuthorityV1, CellId, CellLifecycleRecordV1,
    CellTopologyInventoryV1, Digest32, HardwareClassV1, ImmutableEvidenceRefV1, IsolationClassV1,
    PlacementContractError, PlacementPartitionV1, ProofConstructionError, ResilienceObjectiveV1,
    VerifiedCellCapacityLedgerV1, VerifiedCellLifecycleTransitionV1,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CellSpecRevision(pub u64);

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CellDeploymentUnitId(String);

impl CellDeploymentUnitId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, ProofConstructionError> {
        Err(ProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellCapabilityInventoryV1 {
    pub source: ImmutableEvidenceRefV1,
    pub required_capability_root_digest: Digest32,
    pub required_capability_count: u64,
    pub available_capability_root_digest: Digest32,
    pub available_capability_count: u64,
    pub coverage_proof_digest: Digest32,
    pub inventory_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellIdentityV1 {
    pub cell_id: CellId,
    pub partition: PlacementPartitionV1,
    pub deployment_unit_id: CellDeploymentUnitId,
    pub identity_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellSpecV1 {
    pub topology: CellTopologyInventoryV1,
    pub resilience_objective: ResilienceObjectiveV1,
    pub declared_capacity: CapacityVectorV1,
    pub isolation_classes: Vec<IsolationClassV1>,
    pub hardware_classes: Vec<HardwareClassV1>,
    pub certification_root_digest: Digest32,
    pub certification_count: u64,
    pub capabilities: CellCapabilityInventoryV1,
    pub spec_digest: Digest32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CellConditionKindV1 {
    Health,
    TopologyEvidenceCurrent,
    CapabilityCoverage,
    CapacityPressure,
    DeploymentHealth,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CellConditionStatusV1 {
    Unknown,
    False,
    True,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellConditionV1 {
    pub kind: CellConditionKindV1,
    pub status: CellConditionStatusV1,
    pub reason_code: String,
    pub evidence: ImmutableEvidenceRefV1,
    pub observed_at_unix_seconds: u64,
    pub condition_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellResourceV1 {
    pub identity: CellIdentityV1,
    pub spec: CellSpecV1,
    pub spec_revision: CellSpecRevision,
    pub lifecycle: CellLifecycleRecordV1,
    pub conditions: Vec<CellConditionV1>,
    pub ordered_condition_root_digest: Digest32,
    pub condition_count: u64,
    pub resource_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellViewV1 {
    pub resource: CellResourceV1,
    pub capacity: CellCapacityLedgerV1,
    pub view_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellPageTokenV1(Vec<u8>);

impl CellPageTokenV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse(_value: Vec<u8>) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellPageRequestV1 {
    pub partition: PlacementPartitionV1,
    pub page_size: u32,
    pub page_token: Option<CellPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellPageV1 {
    pub cells: Vec<CellViewV1>,
    pub next_page_token: Option<CellPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellResourcePreconditionV1 {
    Absent {
        identity: CellIdentityV1,
    },
    Matches {
        identity_digest: Digest32,
        spec_revision: CellSpecRevision,
        resource_digest: Digest32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellResourceCapacityMutationV1 {
    Initialize(VerifiedCellCapacityLedgerV1),
    Update {
        precondition: crate::CellCapacityPreconditionV1,
        next_capacity: VerifiedCellCapacityLedgerV1,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CellResourceWriteSetV1 {
    authority: CellControlPersistenceAuthorityV1,
    precondition: CellResourcePreconditionV1,
    resource: CellResourceV1,
    lifecycle_transition: VerifiedCellLifecycleTransitionV1,
    capacity_mutation: CellResourceCapacityMutationV1,
    operation_precondition: CellControlOperationPreconditionV1,
    operation: CellControlOperationV1,
    drain_mutations: crate::DrainContributorMutationSetV1,
    idempotency: CellControlIdempotencyRecordV1,
    proof_consumptions: Vec<crate::CellProofConsumptionV1>,
    audit_outbox: CellControlAuditRecordV1,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CellResourceWriteSetPartsV1 {
    pub authority: CellControlPersistenceAuthorityV1,
    pub precondition: CellResourcePreconditionV1,
    pub resource: CellResourceV1,
    pub lifecycle_transition: VerifiedCellLifecycleTransitionV1,
    pub capacity_mutation: CellResourceCapacityMutationV1,
    pub operation_precondition: CellControlOperationPreconditionV1,
    pub operation: CellControlOperationV1,
    pub drain_mutations: crate::DrainContributorMutationSetV1,
    pub idempotency: CellControlIdempotencyRecordV1,
    pub proof_consumptions: Vec<crate::CellProofConsumptionV1>,
    pub audit_outbox: CellControlAuditRecordV1,
}

impl CellResourceWriteSetV1 {
    pub fn assemble(_parts: CellResourceWriteSetPartsV1) -> Result<Self, PlacementContractError> {
        Err(PlacementContractError::NotImplemented)
    }

    #[must_use]
    pub fn authority(&self) -> &CellControlPersistenceAuthorityV1 {
        &self.authority
    }

    #[must_use]
    pub fn precondition(&self) -> &CellResourcePreconditionV1 {
        &self.precondition
    }

    #[must_use]
    pub fn resource(&self) -> &CellResourceV1 {
        &self.resource
    }

    #[must_use]
    pub fn lifecycle_transition(&self) -> &VerifiedCellLifecycleTransitionV1 {
        &self.lifecycle_transition
    }

    #[must_use]
    pub fn capacity_mutation(&self) -> &CellResourceCapacityMutationV1 {
        &self.capacity_mutation
    }

    #[must_use]
    pub fn operation_precondition(&self) -> &CellControlOperationPreconditionV1 {
        &self.operation_precondition
    }

    #[must_use]
    pub fn operation(&self) -> &CellControlOperationV1 {
        &self.operation
    }

    #[must_use]
    pub fn drain_mutations(&self) -> &crate::DrainContributorMutationSetV1 {
        &self.drain_mutations
    }

    #[must_use]
    pub fn idempotency(&self) -> &CellControlIdempotencyRecordV1 {
        &self.idempotency
    }

    #[must_use]
    pub fn proof_consumptions(&self) -> &[crate::CellProofConsumptionV1] {
        &self.proof_consumptions
    }

    #[must_use]
    pub fn audit_outbox(&self) -> &CellControlAuditRecordV1 {
        &self.audit_outbox
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CellResourceMutationResultV1 {
    pub view: CellViewV1,
    pub operation: CellControlOperationV1,
}

pub trait CellResourceStore: Send + Sync {
    fn apply<'a>(
        &'a self,
        write_set: &'a CellResourceWriteSetV1,
    ) -> BoxCellFuture<'a, Result<CellResourceMutationResultV1, PlacementContractError>>;

    fn get<'a>(
        &'a self,
        authority: &'a CellControlReadAuthorityV1,
        cell_id: &'a CellId,
    ) -> BoxCellFuture<'a, Result<Option<CellViewV1>, PlacementContractError>>;

    fn list<'a>(
        &'a self,
        authority: &'a CellControlReadAuthorityV1,
        request: &'a CellPageRequestV1,
    ) -> BoxCellFuture<'a, Result<CellPageV1, PlacementContractError>>;
}
