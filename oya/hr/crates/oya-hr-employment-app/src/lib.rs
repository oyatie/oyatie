//! HR employment application layer.
//!
//! This crate orchestrates the pure HR employment domain into metadata-only
//! audit, Workflow dispatch, and Workflow start-run planning envelopes for later
//! cloud/runtime adapters. It does not persist data, execute Workflow runtime,
//! emit audit-chain records, or perform network I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use oya_hr_employment_domain::{
    AuditEvidenceRef, Employee, EmployeeCreate, EmployeeId, EmployeeLifecycleEvent, HrDomainError,
    HrLifecycleKind, Jurisdiction, LaborComplianceObligation, LaborComplianceObligationKind,
    LaborComplianceWorkflowStep, LeaveDecision, LeavePayrollImpactInput, LeavePayrollImpactPlan,
    LeaveRequestId, LeaveRoutingMode, LegalEntityId, LegalEntityWorkforceSnapshot,
    PayrollImpactKind, PolicyRef, RulepackEffectiveDate, RulepackRef, SensitiveHrDataKind,
    SensitiveHrReadDecision, SensitiveHrReadInput, SensitiveReadDecisionStatus,
    SensitiveReadLegalBasis, SensitiveReadPurpose, TenantId, WorkflowRef,
    evaluate_labor_compliance, evaluate_sensitive_hr_read, plan_leave_payroll_impact,
};
use oya_workflow_engine_execution_engine_usecase::{
    ExecutionEngineUsecaseInput, HrLaborComplianceWorkflowIntake, HrWorkflowIntakeError,
    plan_hr_labor_compliance_workflow_start,
};

const HR_LIFECYCLE_TOPIC: &str = "audit.hr.employment.lifecycle";
const HR_COMPLIANCE_WORKFLOW_TOPIC: &str = "workflow.hr.compliance.dispatch";
const HR_LEAVE_PAYROLL_IMPACT_TOPIC: &str = "integration.hr.payroll.leave-impact";
const HR_SENSITIVE_READ_TOPIC: &str = "audit.hr.sensitive-read.policy";
const TENANT_RBAC_HR_SENSITIVE_READ_SCOPE_EVIDENCE_PREFIX: &str =
    "audit/tenant-rbac/hr-sensitive-read/";
const HR_SENSITIVE_READ_AUDIT_EVENT_CLASS: &str = "HrSensitiveReadPolicyEvaluated";
pub const HR_SENSITIVE_READ_AUDIT_EMISSION_CONTRACT_REF: &str =
    "audit-event-class/HrSensitiveReadPolicyEvaluated";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrAuditEnvelope {
    pub topic: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub aggregate_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub payload_kind: Classified<String>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub payload_data_class: Classified<DataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrWorkflowDispatchEnvelope {
    pub topic: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub workflow_ref: Classified<WorkflowRef>, // data_class: INTERNAL_ONLY
    pub obligation_kind: Classified<LaborComplianceObligationKind>, // data_class: INTERNAL_ONLY
    pub jurisdiction: Classified<Jurisdiction>, // data_class: INTERNAL_ONLY
    pub required_steps: Classified<Vec<LaborComplianceWorkflowStep>>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Classified<Vec<AuditEvidenceRef>>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrLeavePayrollImpactEnvelope {
    pub topic: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub leave_request_id: Classified<LeaveRequestId>, // data_class: INTERNAL_ONLY
    pub approver_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub decision: Classified<LeaveDecision>, // data_class: INTERNAL_ONLY
    pub routing_mode: Classified<LeaveRoutingMode>, // data_class: INTERNAL_ONLY
    pub workflow_ref: Classified<WorkflowRef>, // data_class: INTERNAL_ONLY
    pub payroll_period: Classified<String>, // data_class: FINANCIAL
    pub payroll_impact_kind: Classified<PayrollImpactKind>, // data_class: FINANCIAL
    pub decision_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub routing_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub payroll_impact_evidence_ref: Classified<AuditEvidenceRef>, // data_class: FINANCIAL
    pub rulepack_ref: Classified<RulepackRef>, // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub decided_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub payload_data_class: Classified<DataClass>, // data_class: INTERNAL_ONLY
    pub payroll_calculation_attached: Classified<bool>, // data_class: PUBLIC
    pub payroll_network_call: Classified<bool>, // data_class: PUBLIC
    pub workflow_execution: Classified<bool>, // data_class: PUBLIC
    pub storage_attached: Classified<bool>, // data_class: PUBLIC
    pub runtime_audit_emission: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrSensitiveReadEnvelope {
    pub topic: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub actor_employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub subject_employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub data_kind: Classified<SensitiveHrDataKind>, // data_class: SENSITIVE_PIPA_ART23
    pub purpose: Classified<SensitiveReadPurpose>, // data_class: INTERNAL_ONLY
    pub legal_basis: Classified<SensitiveReadLegalBasis>, // data_class: INTERNAL_ONLY
    pub policy_ref: Classified<PolicyRef>, // data_class: INTERNAL_ONLY
    pub basis_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub consent_evidence_ref: Classified<Option<AuditEvidenceRef>>, // data_class: INTERNAL_ONLY
    pub request_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub read_log_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub decision_status: Classified<SensitiveReadDecisionStatus>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub payload_data_class: Classified<DataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SensitiveHrRuntimeReadBoundaryInput {
    pub policy_input: SensitiveHrReadInput, // data_class: SENSITIVE_PIPA_ART23
    pub tenant_rbac_scope_evidence_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub audit_emission_contract_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrSensitiveReadRuntimeBoundaryEnvelope {
    pub topic: Classified<String>,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub actor_employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub subject_employee_id: Classified<EmployeeId>, // data_class: INTERNAL_ONLY
    pub data_kind: Classified<SensitiveHrDataKind>, // data_class: SENSITIVE_PIPA_ART23
    pub purpose: Classified<SensitiveReadPurpose>, // data_class: INTERNAL_ONLY
    pub legal_basis: Classified<SensitiveReadLegalBasis>, // data_class: INTERNAL_ONLY
    pub policy_ref: Classified<PolicyRef>, // data_class: INTERNAL_ONLY
    pub tenant_rbac_scope_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub audit_emission_contract_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub audit_event_class: Classified<String>, // data_class: INTERNAL_ONLY
    pub decision_status: Classified<SensitiveReadDecisionStatus>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub payload_data_class: Classified<DataClass>, // data_class: INTERNAL_ONLY
    pub sensitive_data_fetch: Classified<bool>, // data_class: PUBLIC
    pub raw_sensitive_data_echo: Classified<bool>, // data_class: PUBLIC
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnboardEmployeeCommand {
    pub employee: EmployeeCreate,        // data_class: PII_IDENTIFYING
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub lifecycle_kind: HrLifecycleKind, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnboardEmployeeOutcome {
    pub employee: Employee,                      // data_class: PII_IDENTIFYING
    pub lifecycle_event: EmployeeLifecycleEvent, // data_class: INTERNAL_ONLY
    pub audit_envelope: HrAuditEnvelope,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaborWorkflowPlanOutcome {
    pub obligations: Vec<LaborComplianceObligation>, // data_class: INTERNAL_ONLY
    pub workflow_dispatches: Vec<HrWorkflowDispatchEnvelope>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeavePayrollImpactOutcome {
    pub plan: LeavePayrollImpactPlan, // data_class: FINANCIAL
    pub payroll_impact_envelope: HrLeavePayrollImpactEnvelope, // data_class: FINANCIAL
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SensitiveHrReadOutcome {
    pub decision: SensitiveHrReadDecision, // data_class: SENSITIVE_PIPA_ART23
    pub audit_envelope: HrSensitiveReadEnvelope, // data_class: SENSITIVE_PIPA_ART23
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SensitiveHrRuntimeReadBoundaryOutcome {
    pub decision: SensitiveHrReadDecision, // data_class: SENSITIVE_PIPA_ART23
    pub audit_envelope: HrSensitiveReadRuntimeBoundaryEnvelope, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HrWorkflowExecutionScope {
    pub audit_refs: Vec<String>,      // data_class: INTERNAL_ONLY
    pub cell_id: String,              // data_class: INTERNAL_ONLY
    pub workflow_version_ref: String, // data_class: INTERNAL_ONLY
    pub policy_evidence_ref: String,  // data_class: INTERNAL_ONLY
    pub spec_integrity_ref: String,   // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,     // data_class: INTERNAL_ONLY
    pub scheduler_epoch_ref: String,  // data_class: INTERNAL_ONLY
    pub trace_ref: String,            // data_class: INTERNAL_ONLY
}

impl HrWorkflowExecutionScope {
    fn has_required_scope_evidence(&self) -> bool {
        !blank(&self.cell_id)
            && !blank(&self.workflow_version_ref)
            && !blank(&self.policy_evidence_ref)
            && !blank(&self.spec_integrity_ref)
            && !blank(&self.replay_epoch_ref)
            && !blank(&self.scheduler_epoch_ref)
            && !blank(&self.trace_ref)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HrWorkflowExecutionAdapterError {
    MissingAuditRefs,
    MissingScopeEvidence,
    WorkflowIntakeRejected,
}

impl HrWorkflowExecutionAdapterError {
    fn from_workflow_intake_error(error: HrWorkflowIntakeError) -> Self {
        match error {
            HrWorkflowIntakeError::MissingAuditRefs => Self::MissingAuditRefs,
            HrWorkflowIntakeError::MissingEvidenceRefs
            | HrWorkflowIntakeError::MissingRequiredSteps
            | HrWorkflowIntakeError::UnsafeMetadata => Self::WorkflowIntakeRejected,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HrAppError {
    Domain(HrDomainError),
    MissingTenantRbacScopeEvidence,
    InvalidTenantRbacScopeEvidence,
    MissingSensitiveReadAuditContract,
    InvalidSensitiveReadAuditContract,
}

impl From<HrDomainError> for HrAppError {
    fn from(error: HrDomainError) -> Self {
        Self::Domain(error)
    }
}

pub fn onboard_employee(
    command: OnboardEmployeeCommand,
) -> Result<OnboardEmployeeOutcome, HrAppError> {
    let employee = Employee::new(command.employee)?;
    let lifecycle_event = employee.lifecycle_event(&command.event_id, command.lifecycle_kind)?;
    let audit_envelope = lifecycle_audit_envelope(&lifecycle_event);
    Ok(OnboardEmployeeOutcome {
        employee,
        lifecycle_event,
        audit_envelope,
    })
}

pub fn plan_labor_compliance_workflows(
    snapshot: LegalEntityWorkforceSnapshot,
) -> Result<LaborWorkflowPlanOutcome, HrAppError> {
    let obligations = evaluate_labor_compliance(snapshot)?;
    let workflow_dispatches = obligations.iter().map(workflow_dispatch).collect();
    Ok(LaborWorkflowPlanOutcome {
        obligations,
        workflow_dispatches,
    })
}

pub fn plan_hr_workflow_execution_start(
    dispatch: &HrWorkflowDispatchEnvelope,
    scope: HrWorkflowExecutionScope,
) -> Result<ExecutionEngineUsecaseInput, HrWorkflowExecutionAdapterError> {
    if scope.audit_refs.is_empty() {
        return Err(HrWorkflowExecutionAdapterError::MissingAuditRefs);
    }
    if !scope.has_required_scope_evidence() {
        return Err(HrWorkflowExecutionAdapterError::MissingScopeEvidence);
    }

    plan_hr_labor_compliance_workflow_start(HrLaborComplianceWorkflowIntake {
        tenant_id: dispatch.tenant_id.value.value.clone(),
        legal_entity_id: dispatch.legal_entity_id.value.value.clone(),
        workflow_ref: dispatch.workflow_ref.value.value.clone(),
        obligation_kind: obligation_kind_key(dispatch.obligation_kind.value).to_owned(),
        required_steps: dispatch
            .required_steps
            .value
            .iter()
            .map(|step| workflow_step_key(*step).to_owned())
            .collect(),
        evidence_refs: dispatch
            .evidence_refs
            .value
            .iter()
            .map(|evidence_ref| evidence_ref.value.clone())
            .collect(),
        idempotency_key: dispatch.idempotency_key.value.clone(),
        audit_refs: scope.audit_refs,
        cell_id: scope.cell_id,
        workflow_version_ref: scope.workflow_version_ref,
        policy_evidence_ref: scope.policy_evidence_ref,
        spec_integrity_ref: scope.spec_integrity_ref,
        replay_epoch_ref: scope.replay_epoch_ref,
        scheduler_epoch_ref: scope.scheduler_epoch_ref,
        trace_ref: scope.trace_ref,
    })
    .map_err(HrWorkflowExecutionAdapterError::from_workflow_intake_error)
}

pub fn plan_leave_payroll_impact_envelope(
    input: LeavePayrollImpactInput,
) -> Result<LeavePayrollImpactOutcome, HrAppError> {
    let plan = plan_leave_payroll_impact(input)?;
    let payroll_impact_envelope = leave_payroll_impact_envelope(&plan);
    Ok(LeavePayrollImpactOutcome {
        plan,
        payroll_impact_envelope,
    })
}

pub fn prepare_sensitive_hr_read_envelope(
    input: SensitiveHrReadInput,
) -> Result<SensitiveHrReadOutcome, HrAppError> {
    let decision = evaluate_sensitive_hr_read(input)?;
    let audit_envelope = sensitive_read_envelope(&decision);
    Ok(SensitiveHrReadOutcome {
        decision,
        audit_envelope,
    })
}

pub fn authorize_sensitive_hr_runtime_read_boundary(
    input: SensitiveHrRuntimeReadBoundaryInput,
) -> Result<SensitiveHrRuntimeReadBoundaryOutcome, HrAppError> {
    let tenant_rbac_scope_evidence_ref =
        require_tenant_rbac_scope_evidence(input.tenant_rbac_scope_evidence_ref)?;
    let audit_emission_contract_ref =
        require_sensitive_read_audit_contract(input.audit_emission_contract_ref)?;
    let decision = evaluate_sensitive_hr_read(input.policy_input)?;
    let audit_envelope = sensitive_read_runtime_boundary_envelope(
        &decision,
        tenant_rbac_scope_evidence_ref,
        audit_emission_contract_ref,
    );
    Ok(SensitiveHrRuntimeReadBoundaryOutcome {
        decision,
        audit_envelope,
    })
}

fn lifecycle_audit_envelope(event: &EmployeeLifecycleEvent) -> HrAuditEnvelope {
    HrAuditEnvelope {
        topic: internal(HR_LIFECYCLE_TOPIC.to_owned()),
        tenant_id: internal(event.tenant_id.value.clone()),
        legal_entity_id: internal(event.legal_entity_id.value.clone()),
        aggregate_ref: internal(format!("hr/employee/{}", event.employee_id.value.value)),
        evidence_ref: internal(event.audit_evidence_ref.value.clone()),
        payload_kind: internal(format!("{:?}", event.lifecycle_kind.value)),
        idempotency_key: internal(event.idempotency_key.value.clone()),
        payload_data_class: internal(DataClass::PiiIdentifying),
        schema_version: public(1),
    }
}

fn workflow_dispatch(obligation: &LaborComplianceObligation) -> HrWorkflowDispatchEnvelope {
    HrWorkflowDispatchEnvelope {
        topic: internal(HR_COMPLIANCE_WORKFLOW_TOPIC.to_owned()),
        tenant_id: internal(obligation.tenant_id.value.clone()),
        legal_entity_id: internal(obligation.legal_entity_id.value.clone()),
        workflow_ref: internal(obligation.workflow_ref.value.clone()),
        obligation_kind: internal(obligation.kind.value),
        jurisdiction: internal(obligation.jurisdiction.value),
        required_steps: internal(obligation.workflow_steps.value.clone()),
        evidence_refs: internal(obligation.evidence_paths.value.clone()),
        idempotency_key: internal(obligation.idempotency_key.value.clone()),
        schema_version: public(1),
    }
}

fn leave_payroll_impact_envelope(plan: &LeavePayrollImpactPlan) -> HrLeavePayrollImpactEnvelope {
    HrLeavePayrollImpactEnvelope {
        topic: internal(HR_LEAVE_PAYROLL_IMPACT_TOPIC.to_owned()),
        tenant_id: internal(plan.tenant_id.value.clone()),
        legal_entity_id: internal(plan.legal_entity_id.value.clone()),
        employee_id: internal(plan.employee_id.value.clone()),
        leave_request_id: internal(plan.leave_request_id.value.clone()),
        approver_id: internal(plan.approver_id.value.clone()),
        decision: internal(plan.decision.value),
        routing_mode: internal(plan.routing_mode.value),
        workflow_ref: internal(plan.workflow_ref.value.clone()),
        payroll_period: Classified::new(plan.payroll_period.value.clone(), DataClass::Financial),
        payroll_impact_kind: Classified::new(plan.payroll_impact_kind.value, DataClass::Financial),
        decision_evidence_ref: internal(plan.decision_evidence_ref.value.clone()),
        routing_evidence_ref: internal(plan.routing_evidence_ref.value.clone()),
        payroll_impact_evidence_ref: Classified::new(
            plan.payroll_impact_evidence_ref.value.clone(),
            DataClass::Financial,
        ),
        rulepack_ref: internal(plan.rulepack_ref.value.clone()),
        rulepack_effective_date: internal(plan.rulepack_effective_date.value.clone()),
        decided_at_epoch_seconds: internal(plan.decided_at_epoch_seconds.value),
        idempotency_key: internal(plan.idempotency_key.value.clone()),
        payload_data_class: internal(DataClass::Financial),
        payroll_calculation_attached: public(false),
        payroll_network_call: public(false),
        workflow_execution: public(false),
        storage_attached: public(false),
        runtime_audit_emission: public(false),
        schema_version: public(1),
    }
}

fn sensitive_read_envelope(decision: &SensitiveHrReadDecision) -> HrSensitiveReadEnvelope {
    HrSensitiveReadEnvelope {
        topic: internal(HR_SENSITIVE_READ_TOPIC.to_owned()),
        tenant_id: internal(decision.tenant_id.value.clone()),
        legal_entity_id: internal(decision.legal_entity_id.value.clone()),
        actor_employee_id: internal(decision.actor_employee_id.value.clone()),
        subject_employee_id: internal(decision.subject_employee_id.value.clone()),
        data_kind: Classified::new(decision.data_kind.value, DataClass::SensitivePipaArticle23),
        purpose: internal(decision.purpose.value),
        legal_basis: internal(decision.legal_basis.value),
        policy_ref: internal(decision.policy_ref.value.clone()),
        basis_evidence_ref: internal(decision.basis_evidence_ref.value.clone()),
        consent_evidence_ref: internal(decision.consent_evidence_ref.value.clone()),
        request_evidence_ref: internal(decision.request_evidence_ref.value.clone()),
        read_log_evidence_ref: internal(decision.read_log_evidence_ref.value.clone()),
        decision_status: internal(decision.decision_status.value),
        idempotency_key: internal(decision.idempotency_key.value.clone()),
        payload_data_class: internal(sensitive_read_payload_data_class(decision.data_kind.value)),
        schema_version: public(1),
    }
}

fn sensitive_read_runtime_boundary_envelope(
    decision: &SensitiveHrReadDecision,
    tenant_rbac_scope_evidence_ref: AuditEvidenceRef,
    audit_emission_contract_ref: String,
) -> HrSensitiveReadRuntimeBoundaryEnvelope {
    HrSensitiveReadRuntimeBoundaryEnvelope {
        topic: internal(HR_SENSITIVE_READ_TOPIC.to_owned()),
        tenant_id: internal(decision.tenant_id.value.clone()),
        legal_entity_id: internal(decision.legal_entity_id.value.clone()),
        actor_employee_id: internal(decision.actor_employee_id.value.clone()),
        subject_employee_id: internal(decision.subject_employee_id.value.clone()),
        data_kind: Classified::new(decision.data_kind.value, DataClass::SensitivePipaArticle23),
        purpose: internal(decision.purpose.value),
        legal_basis: internal(decision.legal_basis.value),
        policy_ref: internal(decision.policy_ref.value.clone()),
        tenant_rbac_scope_evidence_ref: internal(tenant_rbac_scope_evidence_ref),
        audit_emission_contract_ref: internal(audit_emission_contract_ref),
        audit_event_class: internal(HR_SENSITIVE_READ_AUDIT_EVENT_CLASS.to_owned()),
        decision_status: internal(decision.decision_status.value),
        idempotency_key: internal(decision.idempotency_key.value.clone()),
        payload_data_class: internal(sensitive_read_payload_data_class(decision.data_kind.value)),
        sensitive_data_fetch: public(false),
        raw_sensitive_data_echo: public(false),
        schema_version: public(1),
    }
}

fn require_tenant_rbac_scope_evidence(
    evidence_ref: Option<String>,
) -> Result<AuditEvidenceRef, HrAppError> {
    let evidence_ref = evidence_ref.ok_or(HrAppError::MissingTenantRbacScopeEvidence)?;
    let Some(scope_suffix) =
        evidence_ref.strip_prefix(TENANT_RBAC_HR_SENSITIVE_READ_SCOPE_EVIDENCE_PREFIX)
    else {
        return Err(HrAppError::InvalidTenantRbacScopeEvidence);
    };
    if scope_suffix.is_empty() || !safe_audit_evidence_ref_shape(&evidence_ref) {
        return Err(HrAppError::InvalidTenantRbacScopeEvidence);
    }
    Ok(AuditEvidenceRef {
        value: evidence_ref,
    })
}

fn safe_audit_evidence_ref_shape(value: &str) -> bool {
    if value.trim() != value
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
        || value.contains('\\')
        || value
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '.'))
    {
        return false;
    }
    let lowered = value.to_ascii_lowercase();
    !(lowered.contains("token")
        || lowered.contains("secret")
        || lowered.contains("bearer")
        || lowered.contains("password"))
}

fn require_sensitive_read_audit_contract(
    contract_ref: Option<String>,
) -> Result<String, HrAppError> {
    let contract_ref = contract_ref.ok_or(HrAppError::MissingSensitiveReadAuditContract)?;
    if contract_ref != HR_SENSITIVE_READ_AUDIT_EMISSION_CONTRACT_REF {
        return Err(HrAppError::InvalidSensitiveReadAuditContract);
    }
    Ok(contract_ref)
}

fn sensitive_read_payload_data_class(data_kind: SensitiveHrDataKind) -> DataClass {
    match data_kind {
        SensitiveHrDataKind::Medical | SensitiveHrDataKind::DisabilityAccommodation => {
            DataClass::Phi
        }
        SensitiveHrDataKind::Compensation => DataClass::Financial,
        SensitiveHrDataKind::GovernmentIdentifier => DataClass::PiiIdentifying,
        SensitiveHrDataKind::Disciplinary => DataClass::SensitivePipaArticle23,
    }
}

fn obligation_kind_key(kind: LaborComplianceObligationKind) -> &'static str {
    match kind {
        LaborComplianceObligationKind::KoreaRulesOfEmployment => "korea_rules_of_employment",
        LaborComplianceObligationKind::KoreaLaborManagementCouncil => {
            "korea_labor_management_council"
        }
    }
}

fn workflow_step_key(step: LaborComplianceWorkflowStep) -> &'static str {
    match step {
        LaborComplianceWorkflowStep::Drafted => "drafted",
        LaborComplianceWorkflowStep::EmployeeReviewSent => "employee-review-sent",
        LaborComplianceWorkflowStep::MajorityConsentObtained => "majority-consent-obtained",
        LaborComplianceWorkflowStep::MoelFiled => "moel-filed",
        LaborComplianceWorkflowStep::CouncilRosterRequired => "council-roster-required",
        LaborComplianceWorkflowStep::MeetingCadenceRequired => "meeting-cadence-required",
        LaborComplianceWorkflowStep::MinutesEvidenceRequired => "minutes-evidence-required",
        LaborComplianceWorkflowStep::Active => "active",
    }
}

fn blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, PrivacyDataClass::internal_only())
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}
