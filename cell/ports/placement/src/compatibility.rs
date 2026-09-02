use crate::{
    BoxCellFuture, CellAdmissionEpoch, CellCapacityRevision, CellDeploymentUnitId, CellId,
    CellLifecycleRevision, CellProofEnvelopeV1, CellProofVerifier, CellReadinessRingV1,
    CellSpecRevision, Digest32, ImmutableEvidenceRefV1, PlacementReadAuthorityV1, ProducerId,
    ProofConstructionError, ProofVerificationError,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProtocolVersionRangeV1 {
    pub minimum: u32,
    pub maximum: u32,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OwnerReleaseId(String);

impl OwnerReleaseId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(_value: impl Into<String>) -> Result<Self, ProofConstructionError> {
        Err(ProofConstructionError::NotImplemented)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeploymentOrderV1 {
    ExpandReadersBeforeWriters,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DurableStateRollbackV1 {
    RollbackBeforeWriterCutover,
    ForwardRecoveryAfterWriterCutover,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ControlPlaneOwnerV1 {
    Cell,
    Tenancy,
    GatewayProjection,
    CapabilityParticipant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerReleaseCompatibilityPayloadV1 {
    pub schema_version: u32,
    pub owner: ControlPlaneOwnerV1,
    pub owner_id: String,
    pub release_id: OwnerReleaseId,
    pub predecessor_release_id: OwnerReleaseId,
    pub api_read_versions: ProtocolVersionRangeV1,
    pub api_write_versions: ProtocolVersionRangeV1,
    pub proof_read_versions: ProtocolVersionRangeV1,
    pub proof_write_versions: ProtocolVersionRangeV1,
    pub durable_state_read_versions: ProtocolVersionRangeV1,
    pub durable_state_write_version: u32,
    pub minimum_forward_recovery_version: u32,
    pub deployment_order: DeploymentOrderV1,
    pub rollback: DurableStateRollbackV1,
    pub compatibility_matrix_digest: Digest32,
    pub record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedOwnerReleaseCompatibilityV1 {
    pub payload: OwnerReleaseCompatibilityPayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedOwnerReleaseCompatibility(SignedOwnerReleaseCompatibilityV1);

impl VerifiedOwnerReleaseCompatibility {
    #[must_use]
    pub fn signed(&self) -> &SignedOwnerReleaseCompatibilityV1 {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseCompatibilitySetV1 {
    pub immutable_set: ImmutableEvidenceRefV1,
    pub ordered_release_root_digest: Digest32,
    pub release_count: u64,
    pub set_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseCompatibilityMemberV1 {
    pub ordinal: u64,
    pub release: SignedOwnerReleaseCompatibilityV1,
    pub inclusion_path: Vec<Digest32>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedReleaseCompatibilityMember(ReleaseCompatibilityMemberV1);

impl VerifiedReleaseCompatibilityMember {
    #[must_use]
    pub fn member(&self) -> &ReleaseCompatibilityMemberV1 {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PromotionPrerequisiteStatusV1 {
    NotSatisfied,
    Satisfied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalRolloutCompatibilityEvidenceV1 {
    pub source_record: ImmutableEvidenceRefV1,
    pub contracts_signed: PromotionPrerequisiteStatusV1,
    pub controls_validated: PromotionPrerequisiteStatusV1,
    pub compliance_evidence_accepted: PromotionPrerequisiteStatusV1,
    pub capacity_reserved: PromotionPrerequisiteStatusV1,
    pub compatibility_record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellRevisionIdentityV1 {
    pub cell_id: CellId,
    pub cell_identity_digest: Digest32,
    pub deployment_unit_id: CellDeploymentUnitId,
    pub spec_revision: CellSpecRevision,
    pub spec_digest: Digest32,
    pub resource_digest: Digest32,
    pub lifecycle_revision: CellLifecycleRevision,
    pub admission_epoch: CellAdmissionEpoch,
    pub capacity_revision: CellCapacityRevision,
    pub capacity_record_digest: Digest32,
    pub topology_inventory_digest: Digest32,
    pub resilience_objective_digest: Digest32,
    pub capability_inventory_digest: Digest32,
    pub revision_identity_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TwoCellIsolationEvidenceV1 {
    pub subject: CellRevisionIdentityV1,
    pub counterpart: CellRevisionIdentityV1,
    pub evidence: ImmutableEvidenceRefV1,
    pub observation_window_start_unix_seconds: u64,
    pub observation_window_end_unix_seconds: u64,
    pub record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionMetricPopulationV1 {
    pub population_name: String,
    pub denominator: u64,
    pub population_root_digest: Digest32,
    pub population_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellPromotionMetricsV1 {
    pub slo_snapshot: ImmutableEvidenceRefV1,
    pub two_cell_isolation_evidence: TwoCellIsolationEvidenceV1,
    pub failure_injection_evidence: ImmutableEvidenceRefV1,
    pub version_skew_evidence: ImmutableEvidenceRefV1,
    pub progressive_delivery_evidence: ImmutableEvidenceRefV1,
    pub rollback_rehearsal_evidence: ImmutableEvidenceRefV1,
    pub local_control_plane_resilience_evidence: ImmutableEvidenceRefV1,
    pub backup_restore_evidence: ImmutableEvidenceRefV1,
    pub noisy_tenant_backpressure_evidence: ImmutableEvidenceRefV1,
    pub bad_deployment_containment_evidence: ImmutableEvidenceRefV1,
    pub observation_window_start_unix_seconds: u64,
    pub observation_window_end_unix_seconds: u64,
    pub tenant_population: PromotionMetricPopulationV1,
    pub reservation_population: PromotionMetricPopulationV1,
    pub movement_population: PromotionMetricPopulationV1,
    pub economics_evidence: ImmutableEvidenceRefV1,
    pub slo_threshold_policy: ImmutableEvidenceRefV1,
    pub calculation_policy_version: String,
    pub calculation_policy_digest: Digest32,
    pub regional_rollout_compatibility: RegionalRolloutCompatibilityEvidenceV1,
    pub unit_cost_microunits: u64,
    pub currency: crate::CurrencyCode,
    pub stranded_headroom_basis_points: u32,
    pub recovery_reserve_headroom_basis_points: u32,
    pub reservation_leakage_count: u64,
    pub oldest_reconciliation_age_seconds: u64,
    pub maximum_projection_age_seconds: u64,
    pub cross_cell_traffic_bytes: u64,
    pub movement_churn_basis_points: u32,
    pub bytes_moved: u64,
    pub correlated_risk_overlap_millionths: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellPromotionEvidencePayloadV1 {
    pub schema_version: u32,
    pub cell: CellRevisionIdentityV1,
    pub target_readiness: CellReadinessRingV1,
    pub current_release_set: ReleaseCompatibilitySetV1,
    pub target_release_set: ReleaseCompatibilitySetV1,
    pub metrics: CellPromotionMetricsV1,
    pub required_release_owner_root_digest: Digest32,
    pub required_release_owner_count: u64,
    pub current_release_coverage_proof_digest: Digest32,
    pub target_release_coverage_proof_digest: Digest32,
    pub policy_generation: u64,
    pub observed_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    pub record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedCellPromotionEvidenceV1 {
    pub payload: CellPromotionEvidencePayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedCellPromotionEvidence(SignedCellPromotionEvidenceV1);

impl VerifiedCellPromotionEvidence {
    #[must_use]
    pub fn signed(&self) -> &SignedCellPromotionEvidenceV1 {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellPromotionEvidenceExpectationV1 {
    pub cell: CellRevisionIdentityV1,
    pub target_readiness: CellReadinessRingV1,
    pub required_release_owner_root_digest: Digest32,
    pub required_release_owner_count: u64,
    pub expected_policy_generation: u64,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

pub fn verify_owner_release_compatibility(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedOwnerReleaseCompatibilityV1,
    _expected_predecessor: &OwnerReleaseId,
    _expected_producer: &ProducerId,
    _expected_audience: &ProducerId,
    _now_unix_seconds: u64,
) -> Result<VerifiedOwnerReleaseCompatibility, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub fn verify_cell_promotion_evidence(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedCellPromotionEvidenceV1,
    _expectation: &CellPromotionEvidenceExpectationV1,
) -> Result<VerifiedCellPromotionEvidence, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub fn verify_release_compatibility_member(
    _set: &ReleaseCompatibilitySetV1,
    _member: ReleaseCompatibilityMemberV1,
    _maximum_inclusion_path_depth: u32,
) -> Result<VerifiedReleaseCompatibilityMember, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub trait ReleaseCompatibilityReader: Send + Sync {
    fn read_member<'a>(
        &'a self,
        authority: &'a PlacementReadAuthorityV1,
        set: &'a ReleaseCompatibilitySetV1,
        ordinal: u64,
    ) -> BoxCellFuture<'a, Result<ReleaseCompatibilityMemberV1, crate::PlacementContractError>>;
}
