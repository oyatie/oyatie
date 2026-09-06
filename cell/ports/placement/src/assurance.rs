use crate::{
    BoxCellFuture, CapacityVectorV1, CellProofEnvelopeV1, CellProofVerifier, Digest32,
    PlacementLocationV1, PlacementReadAuthorityV1, ProducerId, ProofConstructionError,
    ProofVerificationError, ResilienceObjectiveV1, TenantId,
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

opaque_id!(CertificationId);
opaque_id!(KeyCustodyAuthorityId);
opaque_id!(AssuranceCompilerVersion);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssuranceGeneration(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocationConstraintV1 {
    PlatformPolicyOnly,
    Only(Vec<PlacementLocationV1>),
    DenyAll,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IsolationClassV1 {
    SharedCertified,
    DedicatedLogical,
    DedicatedPhysical,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HardwareClassV1 {
    GeneralPurpose,
    ConfidentialCompute,
    Accelerator,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EncryptionRequirementV1 {
    ProviderManaged,
    CustomerManaged,
    ExternalKeyManager,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExclusiveTenantKeyRequirementV1 {
    NotRequired,
    Required,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ImmutableAuditStorageRequirementV1 {
    NotRequired,
    Required,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyCustodyRequirementV1 {
    pub permitted_authorities: Vec<KeyCustodyAuthorityId>,
    pub exclusive_tenant_key: ExclusiveTenantKeyRequirementV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditRequirementV1 {
    pub minimum_retention_seconds: u64,
    pub immutable_storage: ImmutableAuditStorageRequirementV1,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectiveDurationSecondsV1(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DedicatedWarmRecoveryReserveV1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WarmRecoveryRequirementV1 {
    pub allowed_recovery_locations: LocationConstraintV1,
    pub recovery_capacity: CapacityVectorV1,
    pub recovery_resilience: ResilienceObjectiveV1,
    pub reserve_model: DedicatedWarmRecoveryReserveV1,
    pub maximum_rpo: ObjectiveDurationSecondsV1,
    pub maximum_rto: ObjectiveDurationSecondsV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardRecoveryRequirementV1 {
    pub backup_storage_locations: LocationConstraintV1,
    pub restore_staging_locations: LocationConstraintV1,
    pub allowed_restore_locations: LocationConstraintV1,
    pub maximum_rpo: ObjectiveDurationSecondsV1,
    pub maximum_rto: ObjectiveDurationSecondsV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryRequirementV1 {
    Standard(StandardRecoveryRequirementV1),
    Warm(WarmRecoveryRequirementV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveAssuranceRequirementsV1 {
    pub generation: AssuranceGeneration,
    pub compiler_version: AssuranceCompilerVersion,
    pub primary_locations: LocationConstraintV1,
    pub required_certifications: Vec<CertificationId>,
    pub minimum_isolation: IsolationClassV1,
    pub required_hardware: HardwareClassV1,
    pub encryption: EncryptionRequirementV1,
    pub key_custody: KeyCustodyRequirementV1,
    pub audit: AuditRequirementV1,
    pub recovery: RecoveryRequirementV1,
    pub capability_requirements_digest: Digest32,
    pub compiled_requirements_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceRequirementSetV1 {
    pub source_authority_id: String,
    pub source_repository_id: String,
    pub source_object_id: String,
    pub source_object_version: u64,
    pub ordered_requirement_root_digest: Digest32,
    pub requirement_count: u64,
    pub source_content_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceRequirementV1 {
    pub primary_locations: LocationConstraintV1,
    pub required_certifications: Vec<CertificationId>,
    pub minimum_isolation: IsolationClassV1,
    pub required_hardware: HardwareClassV1,
    pub encryption: EncryptionRequirementV1,
    pub key_custody: KeyCustodyRequirementV1,
    pub audit: AuditRequirementV1,
    pub recovery: RecoveryRequirementV1,
    pub requirement_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceRequirementMemberV1 {
    pub ordinal: u64,
    pub source_id: String,
    pub requirement: AssuranceRequirementV1,
    pub member_digest: Digest32,
    pub inclusion_path: Vec<Digest32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceRequirementPageTokenV1(Vec<u8>);

impl AssuranceRequirementPageTokenV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn parse(_value: Vec<u8>) -> Result<Self, crate::PlacementContractError> {
        Err(crate::PlacementContractError::NotImplemented)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceRequirementPageRequestV1 {
    pub set: AssuranceRequirementSetV1,
    pub page_size: u32,
    pub page_token: Option<AssuranceRequirementPageTokenV1>,
    pub maximum_inclusion_path_depth: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceRequirementPageV1 {
    pub members: Vec<AssuranceRequirementMemberV1>,
    pub next_page_token: Option<AssuranceRequirementPageTokenV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceCompilationPayloadV1 {
    pub schema_version: u32,
    pub tenant_id: TenantId,
    pub generation: AssuranceGeneration,
    pub compiler_version: AssuranceCompilerVersion,
    pub tenant_floor: AssuranceRequirementSetV1,
    pub capability_requirements: AssuranceRequirementSetV1,
    pub platform_policy_generation: u64,
    pub platform_policy_digest: Digest32,
    pub effective: EffectiveAssuranceRequirementsV1,
    pub compiled_at_unix_seconds: u64,
    pub record_digest: Digest32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedAssuranceCompilationV1 {
    pub payload: AssuranceCompilationPayloadV1,
    pub envelope: CellProofEnvelopeV1,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssuranceCompilationExpectationV1 {
    pub tenant_id: TenantId,
    pub generation: AssuranceGeneration,
    pub compiler_version: AssuranceCompilerVersion,
    pub tenant_floor_digest: Digest32,
    pub capability_requirements_digest: Digest32,
    pub platform_policy_generation: u64,
    pub platform_policy_digest: Digest32,
    pub expected_producer: ProducerId,
    pub expected_audience: ProducerId,
    pub now_unix_seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct VerifiedAssuranceCompilation(SignedAssuranceCompilationV1);

impl VerifiedAssuranceCompilation {
    #[must_use]
    pub fn signed(&self) -> &SignedAssuranceCompilationV1 {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AssuranceCompilationErrorV1 {
    MissingTenantFloor,
    MissingCapabilityRequirement,
    DuplicateRequirement,
    ContradictoryPrimaryLocation,
    ContradictoryRecoveryLocation,
    ContradictoryRealmOrSovereignty,
    ContradictoryKeyCustody,
    UnsupportedIsolation,
    UnsupportedHardware,
    UnprovableCertification,
    UnprovableRecoveryObjective,
    ArithmeticOverflow,
    VerificationFailed,
    NotImplemented,
}

pub fn verify_assurance_compilation(
    _verifier: &dyn CellProofVerifier,
    _signed: SignedAssuranceCompilationV1,
    _expectation: &AssuranceCompilationExpectationV1,
) -> Result<VerifiedAssuranceCompilation, ProofVerificationError> {
    Err(ProofVerificationError::NotImplemented)
}

pub trait AssuranceCompiler: Send + Sync {
    fn compile<'a>(
        &'a self,
        authority: &'a PlacementReadAuthorityV1,
        tenant_id: &'a TenantId,
        tenant_floor: &'a AssuranceRequirementSetV1,
        capability_requirements: &'a AssuranceRequirementSetV1,
        generation: AssuranceGeneration,
    ) -> BoxCellFuture<'a, Result<SignedAssuranceCompilationV1, AssuranceCompilationErrorV1>>;
}

pub trait AssuranceRequirementReader: Send + Sync {
    fn read_page<'a>(
        &'a self,
        authority: &'a PlacementReadAuthorityV1,
        request: &'a AssuranceRequirementPageRequestV1,
    ) -> BoxCellFuture<'a, Result<AssuranceRequirementPageV1, AssuranceCompilationErrorV1>>;
}
