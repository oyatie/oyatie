//! Foundry-to-Cloud mutation control kernel.
//!
//! This crate owns the cross-axis guard named `FOUNDRY_CLOUD_MUTATION_CONTROL`:
//! agent-driven Cloud control-plane changes must prove a dry run, collect the
//! declared approval quorum, carry an exercised rollback plan, and emit
//! tamper-evident audit evidence before execution.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use audit_chain_domain::{AuditChain, AuditChainError, AuditEvent, Plane};
use intelligence_capability_domain::{AutonomyTier, Capability};
use intelligence_policy_domain::{AutonomyDecision, AutonomyVerdict};
use data_boundary_kernel::{
    Classified, DataClass, DataClassification, OperationalDataClass, PrivacyDataClass, Purpose,
};

const MUTATION_SCHEMA_VERSION: u32 = 1;
const MUTATION_ID_PREFIX: &str = "fcm_";
const APPROVAL_ID_PREFIX: &str = "fcma_";
const ROLLBACK_PLAN_ID_PREFIX: &str = "fcmr_";
const INCIDENT_REF_PREFIX: &str = "inc_";
const TENANT_ID_PREFIX: &str = "ten_";
const USER_PRINCIPAL_PREFIX: &str = "usr_";
const SERVICE_PRINCIPAL_PREFIX: &str = "svc_";
const CAPABILITY_ID_PREFIX: &str = "cap.cloud.";
const CLOUD_NAMESPACE_PREFIX: &str = "oya.cloud";
const CLOUD_SURFACE_PREFIX: &str = "cloud.";
const REQUEST_FINGERPRINT_PREFIX: &str = "sha256:";
const MAX_EMERGENCY_WINDOW_SECONDS: u64 = 4 * 60 * 60;
const DEFAULT_EXECUTION_WINDOW_SECONDS: u64 = 15 * 60;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudMutationId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudMutationApprovalId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RollbackPlanId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IncidentRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloudMutationKind {
    IamRolePublish,
    RegionRegister,
    CapacityRebalance,
    ComputeProvision,
    StorageProvision,
    NetworkProvision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DryRunOutcome {
    Pass,
    Fail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IncidentSeverity {
    Sev1,
    Sev2,
    Sev3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloudMutationState {
    Proposed,
    Approved,
    Executed,
    RolledBack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DryRunReport {
    pub request_fingerprint: String,    // data_class: INTERNAL_ONLY
    pub checked_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub outcome: DryRunOutcome,         // data_class: INTERNAL_ONLY
    pub expected_delta_summary: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalQuorum {
    pub required_approvals: u8, // data_class: INTERNAL_ONLY
    pub eligible_approvers: u8, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackPlanCreate {
    pub id: String,                   // data_class: INTERNAL_ONLY
    pub request_fingerprint: String,  // data_class: INTERNAL_ONLY
    pub steps_count: u8,              // data_class: INTERNAL_ONLY
    pub max_recovery_seconds: u32,    // data_class: INTERNAL_ONLY
    pub tested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackPlan {
    pub id: Classified<RollbackPlanId>, // data_class: INTERNAL_ONLY
    pub request_fingerprint: Classified<String>, // data_class: INTERNAL_ONLY
    pub steps_count: Classified<u8>,    // data_class: INTERNAL_ONLY
    pub max_recovery_seconds: Classified<u32>, // data_class: INTERNAL_ONLY
    pub tested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreakGlassJustificationCreate {
    pub incident_ref: String,            // data_class: INTERNAL_ONLY
    pub severity: IncidentSeverity,      // data_class: INTERNAL_ONLY
    pub reason: String,                  // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreakGlassJustification {
    pub incident_ref: Classified<IncidentRef>, // data_class: INTERNAL_ONLY
    pub severity: Classified<IncidentSeverity>, // data_class: INTERNAL_ONLY
    pub reason: Classified<String>,            // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMutationPropose {
    pub id: String,                                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                                  // data_class: INTERNAL_ONLY
    pub capability_id: String,                              // data_class: INTERNAL_ONLY
    pub kind: CloudMutationKind,                            // data_class: PUBLIC
    pub target_surface: String,                             // data_class: PUBLIC
    pub target_region: String,                              // data_class: PUBLIC
    pub dry_run: DryRunReport,                              // data_class: INTERNAL_ONLY
    pub approval_quorum: ApprovalQuorum,                    // data_class: INTERNAL_ONLY
    pub rollback_plan: RollbackPlanCreate,                  // data_class: INTERNAL_ONLY
    pub break_glass: Option<BreakGlassJustificationCreate>, // data_class: INTERNAL_ONLY
    pub requested_by_principal: String,                     // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,                    // data_class: INTERNAL_ONLY
    pub data_class: DataClass,                              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMutation {
    pub id: Classified<CloudMutationId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub capability_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub kind: Classified<CloudMutationKind>, // data_class: PUBLIC
    pub target_surface: Classified<String>, // data_class: PUBLIC
    pub target_region: Classified<String>, // data_class: PUBLIC
    pub dry_run: Classified<DryRunReport>, // data_class: INTERNAL_ONLY
    pub approval_quorum: Classified<ApprovalQuorum>, // data_class: INTERNAL_ONLY
    pub rollback_plan: Classified<RollbackPlan>, // data_class: INTERNAL_ONLY
    pub break_glass: Classified<Option<BreakGlassJustification>>, // data_class: INTERNAL_ONLY
    pub requested_by_principal: Classified<String>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub state: Classified<CloudMutationState>, // data_class: INTERNAL_ONLY
    pub proposal_audit_hash: Classified<String>, // data_class: AUDIT
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMutationApprovalCreate {
    pub id: String,                     // data_class: INTERNAL_ONLY
    pub mutation_id: String,            // data_class: INTERNAL_ONLY
    pub approver_principal: String,     // data_class: INTERNAL_ONLY
    pub rationale: String,              // data_class: INTERNAL_ONLY
    pub approved_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMutationApproval {
    pub id: Classified<CloudMutationApprovalId>, // data_class: INTERNAL_ONLY
    pub mutation_id: Classified<CloudMutationId>, // data_class: INTERNAL_ONLY
    pub approver_principal: Classified<String>,  // data_class: INTERNAL_ONLY
    pub rationale: Classified<String>,           // data_class: INTERNAL_ONLY
    pub approved_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub audit_hash: Classified<String>,          // data_class: AUDIT
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMutationExecutionReceipt {
    pub mutation_id: Classified<CloudMutationId>, // data_class: INTERNAL_ONLY
    pub execution_deadline_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub approval_count: Classified<u8>,           // data_class: INTERNAL_ONLY
    pub audit_hash: Classified<String>,           // data_class: AUDIT
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMutationRollbackCreate {
    pub mutation_id: String,               // data_class: INTERNAL_ONLY
    pub rollback_plan_id: String,          // data_class: INTERNAL_ONLY
    pub request_fingerprint: String,       // data_class: INTERNAL_ONLY
    pub rolled_back_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMutationRollbackReceipt {
    pub mutation_id: Classified<CloudMutationId>, // data_class: INTERNAL_ONLY
    pub rollback_plan_id: Classified<RollbackPlanId>, // data_class: INTERNAL_ONLY
    pub rolled_back_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub audit_hash: Classified<String>,           // data_class: AUDIT
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FoundryCloudMutationError {
    InvalidMutationId,
    InvalidApprovalId,
    InvalidRollbackPlanId,
    InvalidIncidentRef,
    InvalidTenantId,
    InvalidCapability,
    InvalidAutonomyDecision,
    InvalidSurface,
    InvalidRegion,
    InvalidPrincipal,
    InvalidFingerprint,
    InvalidDryRun,
    InvalidApprovalQuorum,
    InvalidRollbackPlan,
    InvalidBreakGlass,
    InvalidTimeOrder,
    InvalidDataClass,
    DuplicateMutation,
    DuplicateApproval,
    DuplicateApprover,
    UnknownMutation,
    NotApproved,
    NotExecuted,
    EmergencyWindowExpired,
    /// Wraps an upstream `AuditChainError` per ADR-0083 amendment 2026-05-15
    /// (`append_classifications` Tier 1 conformance — `Result<&AuditEvent,
    /// AuditChainError>`).
    AuditChainEmissionFailed(AuditChainError),
}

impl From<AuditChainError> for FoundryCloudMutationError {
    fn from(error: AuditChainError) -> Self {
        Self::AuditChainEmissionFailed(error)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FoundryCloudMutationControl {
    mutations: BTreeMap<CloudMutationId, CloudMutation>,
    approvals: BTreeMap<CloudMutationApprovalId, CloudMutationApproval>,
    approvers_by_mutation: BTreeMap<CloudMutationId, BTreeSet<String>>,
    audit_chain: AuditChain,
}

impl CloudMutationId {
    pub fn new(value: impl Into<String>) -> Result<Self, FoundryCloudMutationError> {
        prefixed_token(
            value.into(),
            MUTATION_ID_PREFIX,
            FoundryCloudMutationError::InvalidMutationId,
        )
        .map(|value| Self { value })
    }
}

impl CloudMutationApprovalId {
    pub fn new(value: impl Into<String>) -> Result<Self, FoundryCloudMutationError> {
        prefixed_token(
            value.into(),
            APPROVAL_ID_PREFIX,
            FoundryCloudMutationError::InvalidApprovalId,
        )
        .map(|value| Self { value })
    }
}

impl RollbackPlanId {
    pub fn new(value: impl Into<String>) -> Result<Self, FoundryCloudMutationError> {
        prefixed_token(
            value.into(),
            ROLLBACK_PLAN_ID_PREFIX,
            FoundryCloudMutationError::InvalidRollbackPlanId,
        )
        .map(|value| Self { value })
    }
}

impl IncidentRef {
    pub fn new(value: impl Into<String>) -> Result<Self, FoundryCloudMutationError> {
        prefixed_token(
            value.into(),
            INCIDENT_REF_PREFIX,
            FoundryCloudMutationError::InvalidIncidentRef,
        )
        .map(|value| Self { value })
    }
}

impl RollbackPlan {
    pub fn new(input: RollbackPlanCreate) -> Result<Self, FoundryCloudMutationError> {
        validate_fingerprint(&input.request_fingerprint)?;
        if input.steps_count == 0
            || input.max_recovery_seconds == 0
            || input.tested_at_epoch_seconds == 0
        {
            return Err(FoundryCloudMutationError::InvalidRollbackPlan);
        }
        Ok(Self {
            id: internal(RollbackPlanId::new(input.id)?),
            request_fingerprint: internal(input.request_fingerprint),
            steps_count: internal(input.steps_count),
            max_recovery_seconds: internal(input.max_recovery_seconds),
            tested_at_epoch_seconds: internal(input.tested_at_epoch_seconds),
        })
    }
}

impl BreakGlassJustification {
    pub fn new(input: BreakGlassJustificationCreate) -> Result<Self, FoundryCloudMutationError> {
        validate_nonempty(&input.reason, FoundryCloudMutationError::InvalidBreakGlass)?;
        validate_time(input.requested_at_epoch_seconds)?;
        if input.expires_at_epoch_seconds <= input.requested_at_epoch_seconds
            || input.expires_at_epoch_seconds - input.requested_at_epoch_seconds
                > MAX_EMERGENCY_WINDOW_SECONDS
            || !matches!(
                input.severity,
                IncidentSeverity::Sev1 | IncidentSeverity::Sev2
            )
        {
            return Err(FoundryCloudMutationError::InvalidBreakGlass);
        }
        Ok(Self {
            incident_ref: internal(IncidentRef::new(input.incident_ref)?),
            severity: internal(input.severity),
            reason: internal(input.reason),
            requested_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            expires_at_epoch_seconds: internal(input.expires_at_epoch_seconds),
        })
    }
}

impl FoundryCloudMutationControl {
    pub fn propose_mutation(
        &mut self,
        input: CloudMutationPropose,
        capability: &Capability,
        autonomy_decision: &AutonomyDecision,
    ) -> Result<CloudMutation, FoundryCloudMutationError> {
        validate_capability(input.capability_id.as_str(), capability, autonomy_decision)?;
        validate_tenant_match(&input.tenant_id, autonomy_decision)?;
        validate_principal(&input.requested_by_principal)?;
        validate_time(input.requested_at_epoch_seconds)?;
        validate_surface(&input.target_surface)?;
        validate_region(&input.target_region)?;
        validate_dry_run(&input.dry_run)?;
        validate_quorum(&input.approval_quorum, input.break_glass.is_some())?;
        let rollback_plan = RollbackPlan::new(input.rollback_plan)?;
        if rollback_plan.request_fingerprint.value != input.dry_run.request_fingerprint {
            return Err(FoundryCloudMutationError::InvalidRollbackPlan);
        }
        let break_glass = input
            .break_glass
            .map(BreakGlassJustification::new)
            .transpose()?;
        let data_class = internal_class(input.data_class)?;
        let id = CloudMutationId::new(input.id)?;
        if self.mutations.contains_key(&id) {
            return Err(FoundryCloudMutationError::DuplicateMutation);
        }
        let audit_hash = self
            .append_audit(
                &input.tenant_id,
                "foundry.cloud.mutation.propose",
                format!("PROPOSED:{}:{}", id.value, input.target_surface),
            )?
            .hash
            .clone();
        let mutation = CloudMutation {
            id: internal(id.clone()),
            tenant_id: internal(input.tenant_id),
            capability_id: internal(input.capability_id),
            kind: public(input.kind),
            target_surface: public(input.target_surface),
            target_region: public(input.target_region),
            dry_run: internal(input.dry_run),
            approval_quorum: internal(input.approval_quorum),
            rollback_plan: internal(rollback_plan),
            break_glass: internal(break_glass),
            requested_by_principal: internal(input.requested_by_principal),
            requested_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            state: internal(CloudMutationState::Proposed),
            proposal_audit_hash: audit(audit_hash),
            data_class,
            schema_version: public(MUTATION_SCHEMA_VERSION),
        };
        self.mutations.insert(id, mutation.clone());
        Ok(mutation)
    }

    pub fn record_approval(
        &mut self,
        input: CloudMutationApprovalCreate,
    ) -> Result<CloudMutationApproval, FoundryCloudMutationError> {
        validate_principal(&input.approver_principal)?;
        validate_nonempty(
            &input.rationale,
            FoundryCloudMutationError::InvalidApprovalQuorum,
        )?;
        validate_time(input.approved_at_epoch_seconds)?;
        let data_class = internal_class(input.data_class)?;
        let approval_id = CloudMutationApprovalId::new(input.id)?;
        if self.approvals.contains_key(&approval_id) {
            return Err(FoundryCloudMutationError::DuplicateApproval);
        }
        let mutation_id = CloudMutationId::new(input.mutation_id)?;
        let mutation = self
            .mutations
            .get(&mutation_id)
            .ok_or(FoundryCloudMutationError::UnknownMutation)?;
        if input.approved_at_epoch_seconds < mutation.requested_at_epoch_seconds.value {
            return Err(FoundryCloudMutationError::InvalidTimeOrder);
        }
        let approvers = self
            .approvers_by_mutation
            .entry(mutation_id.clone())
            .or_default();
        if !approvers.insert(input.approver_principal.clone()) {
            return Err(FoundryCloudMutationError::DuplicateApprover);
        }
        let audit_hash = self
            .append_audit(
                &mutation.tenant_id.value.clone(),
                "foundry.cloud.mutation.approve",
                format!(
                    "APPROVED:{}:{}",
                    mutation_id.value, input.approver_principal
                ),
            )?
            .hash
            .clone();
        let approval = CloudMutationApproval {
            id: internal(approval_id.clone()),
            mutation_id: internal(mutation_id.clone()),
            approver_principal: internal(input.approver_principal),
            rationale: internal(input.rationale),
            approved_at_epoch_seconds: internal(input.approved_at_epoch_seconds),
            audit_hash: audit(audit_hash),
            data_class,
        };
        self.approvals.insert(approval_id, approval.clone());
        self.promote_if_quorum_met(&mutation_id)?;
        Ok(approval)
    }

    pub fn authorize_execution(
        &mut self,
        mutation_id: &str,
        now_epoch_seconds: u64,
    ) -> Result<CloudMutationExecutionReceipt, FoundryCloudMutationError> {
        validate_time(now_epoch_seconds)?;
        let mutation_id = CloudMutationId::new(mutation_id.to_string())?;
        let mutation = self
            .mutations
            .get(&mutation_id)
            .ok_or(FoundryCloudMutationError::UnknownMutation)?
            .clone();
        if mutation.state.value != CloudMutationState::Approved {
            return Err(FoundryCloudMutationError::NotApproved);
        }
        if let Some(break_glass) = &mutation.break_glass.value
            && now_epoch_seconds > break_glass.expires_at_epoch_seconds.value
        {
            return Err(FoundryCloudMutationError::EmergencyWindowExpired);
        }
        let approval_count = self.approval_count(&mutation_id);
        let audit_hash = self
            .append_audit(
                &mutation.tenant_id.value,
                "foundry.cloud.mutation.execute",
                format!("EXECUTION_AUTHORIZED:{}", mutation_id.value),
            )?
            .hash
            .clone();
        let stored = self
            .mutations
            .get_mut(&mutation_id)
            .ok_or(FoundryCloudMutationError::UnknownMutation)?;
        stored.state = internal(CloudMutationState::Executed);
        Ok(CloudMutationExecutionReceipt {
            mutation_id: internal(mutation_id),
            execution_deadline_epoch_seconds: internal(
                now_epoch_seconds + DEFAULT_EXECUTION_WINDOW_SECONDS,
            ),
            approval_count: internal(approval_count),
            audit_hash: audit(audit_hash),
        })
    }

    pub fn record_rollback(
        &mut self,
        input: CloudMutationRollbackCreate,
    ) -> Result<CloudMutationRollbackReceipt, FoundryCloudMutationError> {
        validate_time(input.rolled_back_at_epoch_seconds)?;
        internal_class(input.data_class)?;
        validate_fingerprint(&input.request_fingerprint)?;
        let mutation_id = CloudMutationId::new(input.mutation_id)?;
        let rollback_plan_id = RollbackPlanId::new(input.rollback_plan_id)?;
        let mutation = self
            .mutations
            .get(&mutation_id)
            .ok_or(FoundryCloudMutationError::UnknownMutation)?
            .clone();
        if mutation.state.value != CloudMutationState::Executed {
            return Err(FoundryCloudMutationError::NotExecuted);
        }
        if mutation.rollback_plan.value.id.value != rollback_plan_id
            || mutation.rollback_plan.value.request_fingerprint.value != input.request_fingerprint
        {
            return Err(FoundryCloudMutationError::InvalidRollbackPlan);
        }
        let audit_hash = self
            .append_audit(
                &mutation.tenant_id.value,
                "foundry.cloud.mutation.rollback",
                format!("ROLLED_BACK:{}", mutation_id.value),
            )?
            .hash
            .clone();
        let stored = self
            .mutations
            .get_mut(&mutation_id)
            .ok_or(FoundryCloudMutationError::UnknownMutation)?;
        stored.state = internal(CloudMutationState::RolledBack);
        Ok(CloudMutationRollbackReceipt {
            mutation_id: internal(mutation_id),
            rollback_plan_id: internal(rollback_plan_id),
            rolled_back_at_epoch_seconds: internal(input.rolled_back_at_epoch_seconds),
            audit_hash: audit(audit_hash),
        })
    }

    pub fn mutation(&self, id: &CloudMutationId) -> Option<&CloudMutation> {
        self.mutations.get(id)
    }

    pub fn approvals(&self) -> impl Iterator<Item = &CloudMutationApproval> {
        self.approvals.values()
    }

    pub fn audit_chain(&self) -> &AuditChain {
        &self.audit_chain
    }

    fn promote_if_quorum_met(
        &mut self,
        mutation_id: &CloudMutationId,
    ) -> Result<(), FoundryCloudMutationError> {
        let approval_count = self.approval_count(mutation_id);
        let mutation = self
            .mutations
            .get_mut(mutation_id)
            .ok_or(FoundryCloudMutationError::UnknownMutation)?;
        if approval_count >= mutation.approval_quorum.value.required_approvals {
            mutation.state = internal(CloudMutationState::Approved);
        }
        Ok(())
    }

    fn approval_count(&self, mutation_id: &CloudMutationId) -> u8 {
        self.approvers_by_mutation
            .get(mutation_id)
            .map(|approvers| approvers.len().min(u8::MAX as usize) as u8)
            .unwrap_or(0)
    }

    fn append_audit(
        &mut self,
        tenant_id: &str,
        surface: &str,
        decision: String,
    ) -> Result<&AuditEvent, FoundryCloudMutationError> {
        // ADR-0083 Tier 1: `append_classifications` returns
        // `Result<&AuditEvent, AuditChainError>`; propagate via `?` and the
        // `From<AuditChainError>` impl so failure surfaces as a matchable
        // `FoundryCloudMutationError::AuditChainEmissionFailed` variant.
        Ok(self.audit_chain.append_classifications(
            tenant_id.to_string(),
            surface.to_string(),
            Plane::Audit,
            Purpose::CapabilityInvocation,
            [
                DataClassification::from(DataClass::InternalOnly),
                DataClassification::from(OperationalDataClass::Audit),
            ],
            decision,
        )?)
    }
}

fn validate_capability(
    capability_id: &str,
    capability: &Capability,
    autonomy_decision: &AutonomyDecision,
) -> Result<(), FoundryCloudMutationError> {
    if autonomy_decision.capability_id != capability_id
        || autonomy_decision.verdict != AutonomyVerdict::Allow
        || autonomy_decision.required_tier != capability.required_tier
    {
        return Err(FoundryCloudMutationError::InvalidAutonomyDecision);
    }
    if capability.id != capability_id
        || !capability.id.starts_with(CAPABILITY_ID_PREFIX)
        || !capability
            .namespace
            .value
            .starts_with(CLOUD_NAMESPACE_PREFIX)
        || !capability.evidence_topic.value.starts_with("oya.cloud.")
        || capability.required_tier > AutonomyTier::T2Advisory
    {
        return Err(FoundryCloudMutationError::InvalidCapability);
    }
    Ok(())
}

fn validate_tenant_match(
    tenant_id: &str,
    autonomy_decision: &AutonomyDecision,
) -> Result<(), FoundryCloudMutationError> {
    validate_tenant_id(tenant_id)?;
    if autonomy_decision.tenant_id != tenant_id {
        return Err(FoundryCloudMutationError::InvalidAutonomyDecision);
    }
    Ok(())
}

fn validate_tenant_id(value: &str) -> Result<(), FoundryCloudMutationError> {
    if value.starts_with(TENANT_ID_PREFIX) && value.len() > TENANT_ID_PREFIX.len() {
        Ok(())
    } else {
        Err(FoundryCloudMutationError::InvalidTenantId)
    }
}

fn validate_principal(value: &str) -> Result<(), FoundryCloudMutationError> {
    if (value.starts_with(USER_PRINCIPAL_PREFIX) || value.starts_with(SERVICE_PRINCIPAL_PREFIX))
        && value.len() > 4
    {
        Ok(())
    } else {
        Err(FoundryCloudMutationError::InvalidPrincipal)
    }
}

fn validate_surface(value: &str) -> Result<(), FoundryCloudMutationError> {
    if value.starts_with(CLOUD_SURFACE_PREFIX) && value.len() > CLOUD_SURFACE_PREFIX.len() {
        Ok(())
    } else {
        Err(FoundryCloudMutationError::InvalidSurface)
    }
}

fn validate_region(value: &str) -> Result<(), FoundryCloudMutationError> {
    if value.contains('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-' || byte.is_ascii_digit())
    {
        Ok(())
    } else {
        Err(FoundryCloudMutationError::InvalidRegion)
    }
}

fn validate_dry_run(dry_run: &DryRunReport) -> Result<(), FoundryCloudMutationError> {
    validate_fingerprint(&dry_run.request_fingerprint)?;
    validate_time(dry_run.checked_at_epoch_seconds)?;
    validate_nonempty(
        &dry_run.expected_delta_summary,
        FoundryCloudMutationError::InvalidDryRun,
    )?;
    if dry_run.outcome == DryRunOutcome::Pass {
        Ok(())
    } else {
        Err(FoundryCloudMutationError::InvalidDryRun)
    }
}

fn validate_quorum(
    quorum: &ApprovalQuorum,
    emergency: bool,
) -> Result<(), FoundryCloudMutationError> {
    if quorum.required_approvals == 0
        || quorum.eligible_approvers < quorum.required_approvals
        || (!emergency && quorum.required_approvals < 2)
    {
        return Err(FoundryCloudMutationError::InvalidApprovalQuorum);
    }
    Ok(())
}

fn validate_fingerprint(value: &str) -> Result<(), FoundryCloudMutationError> {
    if value.starts_with(REQUEST_FINGERPRINT_PREFIX)
        && value.len() == REQUEST_FINGERPRINT_PREFIX.len() + 64
        && value[REQUEST_FINGERPRINT_PREFIX.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        Ok(())
    } else {
        Err(FoundryCloudMutationError::InvalidFingerprint)
    }
}

fn validate_time(value: u64) -> Result<(), FoundryCloudMutationError> {
    if value == 0 {
        Err(FoundryCloudMutationError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_nonempty(
    value: &str,
    error: FoundryCloudMutationError,
) -> Result<(), FoundryCloudMutationError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn prefixed_token(
    value: String,
    prefix: &str,
    error: FoundryCloudMutationError,
) -> Result<String, FoundryCloudMutationError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(value)
    } else {
        Err(error)
    }
}

fn internal_class(
    data_class: DataClass,
) -> Result<Classified<PrivacyDataClass>, FoundryCloudMutationError> {
    let class = PrivacyDataClass::new(data_class)
        .map_err(|_| FoundryCloudMutationError::InvalidDataClass)?;
    if class.data_class() == DataClass::InternalOnly {
        Ok(internal(class))
    } else {
        Err(FoundryCloudMutationError::InvalidDataClass)
    }
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

fn audit<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Audit)
}

#[cfg(test)]
mod tests {
    use intelligence_policy_domain::{AutonomyCeilingInputs, evaluate_autonomy_inputs};
    use data_boundary_kernel::SubjectClass;

    use super::*;

    const FINGERPRINT: &str =
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn privacy_class(data_class: DataClass) -> PrivacyDataClass {
        PrivacyDataClass::new(data_class).expect("privacy class")
    }

    fn capability(required_tier: AutonomyTier) -> Capability {
        Capability::new_with_privacy_data_classes(
            "cap.cloud.capacity.rebalance".to_string(),
            "oya.cloud.capacity".to_string(),
            required_tier,
            vec![privacy_class(DataClass::InternalOnly)],
            "oya.cloud.capacity.rebalanced.v1".to_string(),
        )
        .expect("capability")
    }

    fn decision(required_tier: AutonomyTier, verdict_tenant: AutonomyTier) -> AutonomyDecision {
        evaluate_autonomy_inputs(AutonomyCeilingInputs::new(
            "ten_cloud".to_string(),
            "cap.cloud.capacity.rebalance".to_string(),
            verdict_tenant,
            verdict_tenant,
            required_tier,
            AutonomyTier::T4AutoExecute,
            AutonomyTier::T4AutoExecute,
            SubjectClass::Adult,
            AutonomyTier::T4AutoExecute,
        ))
    }

    fn rollback_plan() -> RollbackPlanCreate {
        RollbackPlanCreate {
            id: "fcmr_capacity_rebalance_001".to_string(),
            request_fingerprint: FINGERPRINT.to_string(),
            steps_count: 3,
            max_recovery_seconds: 900,
            tested_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn dry_run() -> DryRunReport {
        DryRunReport {
            request_fingerprint: FINGERPRINT.to_string(),
            checked_at_epoch_seconds: 1_700_000_020,
            outcome: DryRunOutcome::Pass,
            expected_delta_summary: "move 5% capacity from cell-a to cell-b".to_string(),
        }
    }

    fn proposal(id: &str, quorum: ApprovalQuorum) -> CloudMutationPropose {
        CloudMutationPropose {
            id: id.to_string(),
            tenant_id: "ten_cloud".to_string(),
            capability_id: "cap.cloud.capacity.rebalance".to_string(),
            kind: CloudMutationKind::CapacityRebalance,
            target_surface: "cloud.capacity.rebalance".to_string(),
            target_region: "region-alpha1".to_string(),
            dry_run: dry_run(),
            approval_quorum: quorum,
            rollback_plan: rollback_plan(),
            break_glass: None,
            requested_by_principal: "svc_foundry_agent".to_string(),
            requested_at_epoch_seconds: 1_700_000_030,
            data_class: DataClass::InternalOnly,
        }
    }

    fn approval(id: &str, approver: &str) -> CloudMutationApprovalCreate {
        CloudMutationApprovalCreate {
            id: id.to_string(),
            mutation_id: "fcm_capacity_rebalance_001".to_string(),
            approver_principal: approver.to_string(),
            rationale: "dry run and rollback reviewed".to_string(),
            approved_at_epoch_seconds: 1_700_000_040,
            data_class: DataClass::InternalOnly,
        }
    }

    #[test]
    fn proposes_cloud_mutation_only_after_dry_run_and_allowed_t2_capability() {
        let mut control = FoundryCloudMutationControl::default();
        let cap = capability(AutonomyTier::T2Advisory);
        let mutation = control
            .propose_mutation(
                proposal(
                    "fcm_capacity_rebalance_001",
                    ApprovalQuorum {
                        required_approvals: 2,
                        eligible_approvers: 3,
                    },
                ),
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T2Advisory),
            )
            .expect("proposal");

        assert_eq!(mutation.state.value, CloudMutationState::Proposed);
        assert_eq!(mutation.target_surface.value, "cloud.capacity.rebalance");
        assert_eq!(control.audit_chain().events().len(), 1);
        assert!(control.audit_chain().verify());
    }

    #[test]
    fn rejects_failed_dry_run_auto_execute_capability_and_denied_autonomy() {
        let mut control = FoundryCloudMutationControl::default();
        let cap = capability(AutonomyTier::T4AutoExecute);
        let auto_error = control
            .propose_mutation(
                proposal(
                    "fcm_auto_execute",
                    ApprovalQuorum {
                        required_approvals: 2,
                        eligible_approvers: 3,
                    },
                ),
                &cap,
                &decision(AutonomyTier::T4AutoExecute, AutonomyTier::T4AutoExecute),
            )
            .expect_err("cloud mutations cannot be T4 auto-execute");
        assert_eq!(auto_error, FoundryCloudMutationError::InvalidCapability);

        let cap = capability(AutonomyTier::T2Advisory);
        let denied_error = control
            .propose_mutation(
                proposal(
                    "fcm_denied_autonomy",
                    ApprovalQuorum {
                        required_approvals: 2,
                        eligible_approvers: 3,
                    },
                ),
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T1ViewOnly),
            )
            .expect_err("denied autonomy decision blocks proposal");
        assert_eq!(
            denied_error,
            FoundryCloudMutationError::InvalidAutonomyDecision
        );

        let mut failed = proposal(
            "fcm_failed_dry_run",
            ApprovalQuorum {
                required_approvals: 2,
                eligible_approvers: 3,
            },
        );
        failed.dry_run.outcome = DryRunOutcome::Fail;
        let dry_run_error = control
            .propose_mutation(
                failed,
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T2Advisory),
            )
            .expect_err("failed dry run blocks proposal");
        assert_eq!(dry_run_error, FoundryCloudMutationError::InvalidDryRun);
    }

    #[test]
    fn enforces_m_of_n_distinct_approvals_before_execution() {
        let mut control = FoundryCloudMutationControl::default();
        let cap = capability(AutonomyTier::T2Advisory);
        control
            .propose_mutation(
                proposal(
                    "fcm_capacity_rebalance_001",
                    ApprovalQuorum {
                        required_approvals: 2,
                        eligible_approvers: 3,
                    },
                ),
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T2Advisory),
            )
            .expect("proposal");

        control
            .record_approval(approval("fcma_001", "usr_alice"))
            .expect("approval");
        let not_approved = control
            .authorize_execution("fcm_capacity_rebalance_001", 1_700_000_050)
            .expect_err("one approval is insufficient");
        assert_eq!(not_approved, FoundryCloudMutationError::NotApproved);

        let duplicate = control
            .record_approval(approval("fcma_002", "usr_alice"))
            .expect_err("same approver cannot count twice");
        assert_eq!(duplicate, FoundryCloudMutationError::DuplicateApprover);

        control
            .record_approval(approval("fcma_003", "usr_bob"))
            .expect("approval");
        let receipt = control
            .authorize_execution("fcm_capacity_rebalance_001", 1_700_000_060)
            .expect("execution authorization");
        assert_eq!(receipt.approval_count.value, 2);
        assert_eq!(control.audit_chain().events().len(), 4);
    }

    #[test]
    fn break_glass_is_timeboxed_sev1_or_sev2_and_still_requires_approval() {
        let mut control = FoundryCloudMutationControl::default();
        let cap = capability(AutonomyTier::T2Advisory);
        let mut emergency = proposal(
            "fcm_capacity_rebalance_001",
            ApprovalQuorum {
                required_approvals: 1,
                eligible_approvers: 2,
            },
        );
        emergency.break_glass = Some(BreakGlassJustificationCreate {
            incident_ref: "inc_sev1_capacity_hotspot".to_string(),
            severity: IncidentSeverity::Sev1,
            reason: "tenant-critical cell is exhausting headroom".to_string(),
            requested_at_epoch_seconds: 1_700_000_031,
            expires_at_epoch_seconds: 1_700_003_000,
        });
        control
            .propose_mutation(
                emergency,
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T2Advisory),
            )
            .expect("emergency proposal");
        control
            .record_approval(approval("fcma_001", "usr_incident_commander"))
            .expect("approval");
        let receipt = control
            .authorize_execution("fcm_capacity_rebalance_001", 1_700_000_070)
            .expect("emergency execution authorization");
        assert_eq!(receipt.approval_count.value, 1);

        let mut invalid = proposal(
            "fcm_invalid_break_glass",
            ApprovalQuorum {
                required_approvals: 1,
                eligible_approvers: 2,
            },
        );
        invalid.break_glass = Some(BreakGlassJustificationCreate {
            incident_ref: "inc_sev3_capacity".to_string(),
            severity: IncidentSeverity::Sev3,
            reason: "routine tuning".to_string(),
            requested_at_epoch_seconds: 1_700_000_031,
            expires_at_epoch_seconds: 1_700_100_000,
        });
        let error = control
            .propose_mutation(
                invalid,
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T2Advisory),
            )
            .expect_err("non-critical or long emergency windows are rejected");
        assert_eq!(error, FoundryCloudMutationError::InvalidBreakGlass);
    }

    #[test]
    fn rollback_requires_executed_mutation_and_matching_plan_fingerprint() {
        let mut control = FoundryCloudMutationControl::default();
        let cap = capability(AutonomyTier::T2Advisory);
        control
            .propose_mutation(
                proposal(
                    "fcm_capacity_rebalance_001",
                    ApprovalQuorum {
                        required_approvals: 2,
                        eligible_approvers: 3,
                    },
                ),
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T2Advisory),
            )
            .expect("proposal");
        let premature = control
            .record_rollback(CloudMutationRollbackCreate {
                mutation_id: "fcm_capacity_rebalance_001".to_string(),
                rollback_plan_id: "fcmr_capacity_rebalance_001".to_string(),
                request_fingerprint: FINGERPRINT.to_string(),
                rolled_back_at_epoch_seconds: 1_700_000_050,
                data_class: DataClass::InternalOnly,
            })
            .expect_err("rollback waits for execution");
        assert_eq!(premature, FoundryCloudMutationError::NotExecuted);

        control
            .record_approval(approval("fcma_001", "usr_alice"))
            .expect("approval");
        control
            .record_approval(approval("fcma_002", "usr_bob"))
            .expect("approval");
        control
            .authorize_execution("fcm_capacity_rebalance_001", 1_700_000_060)
            .expect("execute");
        let receipt = control
            .record_rollback(CloudMutationRollbackCreate {
                mutation_id: "fcm_capacity_rebalance_001".to_string(),
                rollback_plan_id: "fcmr_capacity_rebalance_001".to_string(),
                request_fingerprint: FINGERPRINT.to_string(),
                rolled_back_at_epoch_seconds: 1_700_000_080,
                data_class: DataClass::InternalOnly,
            })
            .expect("rollback");
        assert!(receipt.audit_hash.value.starts_with("sha256:"));
        let mutation_id = CloudMutationId::new("fcm_capacity_rebalance_001").expect("id");
        assert_eq!(
            control
                .mutation(&mutation_id)
                .expect("mutation")
                .state
                .value,
            CloudMutationState::RolledBack
        );
    }

    #[test]
    fn rejects_duplicate_mutation_bad_quorum_and_forged_public_data_class() {
        let mut control = FoundryCloudMutationControl::default();
        let cap = capability(AutonomyTier::T2Advisory);
        let bad_quorum = control
            .propose_mutation(
                proposal(
                    "fcm_bad_quorum",
                    ApprovalQuorum {
                        required_approvals: 1,
                        eligible_approvers: 3,
                    },
                ),
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T2Advisory),
            )
            .expect_err("ordinary cloud mutation requires M-of-N approvals");
        assert_eq!(bad_quorum, FoundryCloudMutationError::InvalidApprovalQuorum);

        let duplicate_proposal = proposal(
            "fcm_capacity_rebalance_001",
            ApprovalQuorum {
                required_approvals: 2,
                eligible_approvers: 3,
            },
        );
        control
            .propose_mutation(
                duplicate_proposal.clone(),
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T2Advisory),
            )
            .expect("proposal");
        let duplicate = control
            .propose_mutation(
                duplicate_proposal,
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T2Advisory),
            )
            .expect_err("duplicate mutation id rejected");
        assert_eq!(duplicate, FoundryCloudMutationError::DuplicateMutation);

        let mut forged_public = proposal(
            "fcm_public_class",
            ApprovalQuorum {
                required_approvals: 2,
                eligible_approvers: 3,
            },
        );
        forged_public.data_class = DataClass::Public;
        let data_class_error = control
            .propose_mutation(
                forged_public,
                &cap,
                &decision(AutonomyTier::T2Advisory, AutonomyTier::T2Advisory),
            )
            .expect_err("mutation ledger is internal only");
        assert_eq!(
            data_class_error,
            FoundryCloudMutationError::InvalidDataClass
        );
    }
}
