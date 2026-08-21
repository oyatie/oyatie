//! Tenant RBAC domain foundation.
//!
//! This crate owns pure tenant/RBAC invariants that span HR, Payroll, and
//! Accounting: shared policy-gateway admission and legal-entity group close
//! projections. It does not perform storage, network I/O, Workflow dispatch,
//! audit-chain emission, deployment, or service writes.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const TENANT_RBAC_DECISION_SCHEMA_VERSION: u32 = 1;
const GROUP_ROLLUP_SCHEMA_VERSION: u32 = 1;
const TENANT_ID_PREFIX: &str = "ten_";
const LEGAL_ENTITY_ID_PREFIX: &str = "le_";
const GROUP_ID_PREFIX: &str = "grp_";
const AUDIT_EVIDENCE_PREFIX: &str = "audit/";
const POLICY_GATEWAY_PREFIX: &str = "policy/tenant-rbac/";
const PROJECTION_REF_PREFIX: &str = "projection/";
const WORKFLOW_REF_PREFIX: &str = "workflow/";
const OBJECT_GRAPH_REF_PREFIX: &str = "object-graph/";
const AI_SUGGESTION_REF_PREFIX: &str = "ai/";
const CROSS_SERVICE_WORKFLOW_SCHEMA_VERSION: u32 = 1;
const INCIDENT_ID_PREFIX: &str = "inc_";
const OPENTOFU_CONVERGENCE_PREFIX: &str = "opentofu/";
const OPS_CONVERGENCE_PREFIX: &str = "ops/";
const INCIDENT_ROLLBACK_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TenantId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LegalEntityId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GroupId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AuditEvidenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PolicyGatewayRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DashboardProjectionRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorkflowRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObjectGraphRelationshipRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AiSuggestionRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IncidentId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConvergenceRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TenantRbacService {
    Hr,
    Payroll,
    Accounting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TenantRbacWriteKind {
    HrLifecycle,
    PayrollClose,
    AccountingJournal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TenantRbacPolicyDecisionStatus {
    Accepted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloseBoundaryState {
    Open,
    TrialClosed,
    ProductionClosed,
    Quarantined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Jurisdiction {
    Korea,
    UnitedStates,
    EuropeanUnion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowRoutingOwner {
    Workflow,
    Service(TenantRbacService),
    AiAgent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ObjectGraphRelationshipOwner {
    ObjectGraph,
    Service(TenantRbacService),
    AiAgent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DeterministicGate {
    HumanApproval,
    EvidenceAttached,
    RollbackPlanAttached,
    LegalEntityBoundaryChecked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GateClosureAuthority {
    DeterministicGateSet,
    HumanApprover,
    AiSuggestion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IncidentTrigger {
    CanarySloBreach,
    ProductionIncident,
    EvidenceGateFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IncidentFirstAction {
    Rollback,
    Quarantine,
    Remediate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IncidentRemediationRoute {
    OpenTofu,
    OyaOps,
    OpsConsole,
    ManualSsh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IncidentPlanStatus {
    RollbackFirstAccepted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceWriteInput {
    pub service: TenantRbacService,            // data_class: INTERNAL_ONLY
    pub write_kind: TenantRbacWriteKind,       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,               // data_class: INTERNAL_ONLY
    pub payload_data_class: Option<DataClass>, // data_class: INTERNAL_ONLY
    pub audit_evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub policy_gateway_ref: String,            // data_class: INTERNAL_ONLY
    pub idempotency_key: String,               // data_class: INTERNAL_ONLY
    pub sequence: u64,                         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacPolicyDecision {
    pub service: Classified<TenantRbacService>, // data_class: INTERNAL_ONLY
    pub write_kind: Classified<TenantRbacWriteKind>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,        // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub payload_data_class: Classified<DataClass>, // data_class: INTERNAL_ONLY
    pub audit_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub policy_gateway_ref: Classified<PolicyGatewayRef>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,    // data_class: INTERNAL_ONLY
    pub sequence: Classified<u64>,              // data_class: INTERNAL_ONLY
    pub decision_status: Classified<TenantRbacPolicyDecisionStatus>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegalEntityCloseSnapshot {
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                    // data_class: INTERNAL_ONLY
    pub payroll_close_state: CloseBoundaryState,    // data_class: INTERNAL_ONLY
    pub accounting_close_state: CloseBoundaryState, // data_class: INTERNAL_ONLY
    pub payroll_evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub accounting_evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub payroll_close_version: u64,                 // data_class: INTERNAL_ONLY
    pub accounting_close_version: u64,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegalEntityCloseProjection {
    pub legal_entity_id: Classified<LegalEntityId>, // data_class: INTERNAL_ONLY
    pub payroll_close_state: Classified<CloseBoundaryState>, // data_class: INTERNAL_ONLY
    pub accounting_close_state: Classified<CloseBoundaryState>, // data_class: INTERNAL_ONLY
    pub payroll_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub accounting_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub payroll_close_version: Classified<u64>,     // data_class: INTERNAL_ONLY
    pub accounting_close_version: Classified<u64>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupRollupInput {
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub group_id: String,                              // data_class: INTERNAL_ONLY
    pub jurisdiction: Jurisdiction,                    // data_class: INTERNAL_ONLY
    pub dashboard_projection_ref: String,              // data_class: INTERNAL_ONLY
    pub legal_entities: Vec<LegalEntityCloseSnapshot>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupCloseRollup {
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub group_id: Classified<GroupId>,   // data_class: INTERNAL_ONLY
    pub jurisdiction: Classified<Jurisdiction>, // data_class: INTERNAL_ONLY
    pub dashboard_projection_ref: Classified<DashboardProjectionRef>, // data_class: INTERNAL_ONLY
    pub legal_entity_count: Classified<u32>, // data_class: INTERNAL_ONLY
    pub all_entities_closed: Classified<bool>, // data_class: INTERNAL_ONLY
    pub legal_entity_projections: Classified<Vec<LegalEntityCloseProjection>>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,                                       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossServiceWorkflowInput {
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub workflow_ref: String,                  // data_class: INTERNAL_ONLY
    pub object_graph_relationship_ref: String, // data_class: INTERNAL_ONLY
    pub routing_owner: WorkflowRoutingOwner,   // data_class: INTERNAL_ONLY
    pub relationship_owner: ObjectGraphRelationshipOwner, // data_class: INTERNAL_ONLY
    pub services: Vec<TenantRbacService>,      // data_class: INTERNAL_ONLY
    pub gate_evidence_refs: Vec<(DeterministicGate, String)>, // data_class: INTERNAL_ONLY
    pub gate_closure_authority: GateClosureAuthority, // data_class: INTERNAL_ONLY
    pub ai_suggestion_ref: Option<String>,     // data_class: INTERNAL_ONLY
    pub idempotency_key: String,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicGateEvidence {
    pub gate: Classified<DeterministicGate>, // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossServiceWorkflowPlan {
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub workflow_ref: Classified<WorkflowRef>, // data_class: INTERNAL_ONLY
    pub object_graph_relationship_ref: Classified<ObjectGraphRelationshipRef>, // data_class: INTERNAL_ONLY
    pub routing_owner: Classified<WorkflowRoutingOwner>, // data_class: INTERNAL_ONLY
    pub relationship_owner: Classified<ObjectGraphRelationshipOwner>, // data_class: INTERNAL_ONLY
    pub services: Classified<Vec<TenantRbacService>>,    // data_class: INTERNAL_ONLY
    pub required_gates: Classified<Vec<DeterministicGate>>, // data_class: INTERNAL_ONLY
    pub gate_evidence: Classified<Vec<DeterministicGateEvidence>>, // data_class: INTERNAL_ONLY
    pub gate_closure_authority: Classified<GateClosureAuthority>, // data_class: INTERNAL_ONLY
    pub ai_suggestion_ref: Classified<Option<AiSuggestionRef>>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,             // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,                 // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncidentRollbackInput {
    pub tenant_id: String,                           // data_class: INTERNAL_ONLY
    pub incident_id: String,                         // data_class: INTERNAL_ONLY
    pub trigger: IncidentTrigger,                    // data_class: INTERNAL_ONLY
    pub first_action: IncidentFirstAction,           // data_class: INTERNAL_ONLY
    pub remediation_route: IncidentRemediationRoute, // data_class: INTERNAL_ONLY
    pub canary_evidence_ref: String,                 // data_class: INTERNAL_ONLY
    pub incident_evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub rollback_evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub convergence_ref: String,                     // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncidentRollbackPlan {
    pub tenant_id: Classified<TenantId>, // data_class: INTERNAL_ONLY
    pub incident_id: Classified<IncidentId>, // data_class: INTERNAL_ONLY
    pub trigger: Classified<IncidentTrigger>, // data_class: INTERNAL_ONLY
    pub first_action: Classified<IncidentFirstAction>, // data_class: INTERNAL_ONLY
    pub remediation_route: Classified<IncidentRemediationRoute>, // data_class: INTERNAL_ONLY
    pub canary_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub incident_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub rollback_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub convergence_ref: Classified<ConvergenceRef>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub plan_status: Classified<IncidentPlanStatus>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacDomainError {
    InvalidTenantId,
    InvalidLegalEntityId,
    InvalidGroupId,
    InvalidAuditEvidenceRef,
    BypassedPlatformPolicyGateway,
    InvalidDashboardProjectionRef,
    InvalidIdempotencyKey,
    InvalidSequence,
    MissingPayloadDataClass,
    EmptyLegalEntitySet,
    CrossTenantLegalEntity,
    MissingCloseEvidence,
    InvalidWorkflowRef,
    InvalidObjectGraphRelationshipRef,
    InvalidAiSuggestionRef,
    WorkflowRoutingBypass,
    ObjectGraphBypass,
    AiCannotCloseDeterministicGate,
    MissingChildProduct,
    MissingDeterministicGateEvidence,
    InvalidIncidentId,
    InvalidConvergenceRef,
    RollbackOrQuarantineMustBeFirst,
    ManualSshRefused,
}

pub fn admit_service_write(
    input: ServiceWriteInput,
) -> Result<TenantRbacPolicyDecision, TenantRbacDomainError> {
    require_prefixed(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        TenantRbacDomainError::InvalidTenantId,
    )?;
    require_prefixed(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        TenantRbacDomainError::InvalidLegalEntityId,
    )?;
    let payload_data_class = input
        .payload_data_class
        .ok_or(TenantRbacDomainError::MissingPayloadDataClass)?;
    require_prefixed(
        &input.audit_evidence_ref,
        AUDIT_EVIDENCE_PREFIX,
        TenantRbacDomainError::InvalidAuditEvidenceRef,
    )?;
    require_prefixed(
        &input.policy_gateway_ref,
        POLICY_GATEWAY_PREFIX,
        TenantRbacDomainError::BypassedPlatformPolicyGateway,
    )?;
    require_non_empty_key(&input.idempotency_key)?;
    if input.sequence == 0 {
        return Err(TenantRbacDomainError::InvalidSequence);
    }

    Ok(TenantRbacPolicyDecision {
        service: internal(input.service),
        write_kind: internal(input.write_kind),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        payload_data_class: internal(payload_data_class),
        audit_evidence_ref: internal(AuditEvidenceRef {
            value: input.audit_evidence_ref,
        }),
        policy_gateway_ref: internal(PolicyGatewayRef {
            value: input.policy_gateway_ref,
        }),
        idempotency_key: internal(input.idempotency_key),
        sequence: internal(input.sequence),
        decision_status: internal(TenantRbacPolicyDecisionStatus::Accepted),
        schema_version: public(TENANT_RBAC_DECISION_SCHEMA_VERSION),
    })
}

pub fn roll_up_group_close_status(
    input: GroupRollupInput,
) -> Result<GroupCloseRollup, TenantRbacDomainError> {
    require_prefixed(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        TenantRbacDomainError::InvalidTenantId,
    )?;
    require_prefixed(
        &input.group_id,
        GROUP_ID_PREFIX,
        TenantRbacDomainError::InvalidGroupId,
    )?;
    require_prefixed(
        &input.dashboard_projection_ref,
        PROJECTION_REF_PREFIX,
        TenantRbacDomainError::InvalidDashboardProjectionRef,
    )?;
    if input.legal_entities.is_empty() {
        return Err(TenantRbacDomainError::EmptyLegalEntitySet);
    }

    let mut projections = Vec::with_capacity(input.legal_entities.len());
    let mut all_entities_closed = true;
    for entity in input.legal_entities {
        if entity.tenant_id != input.tenant_id {
            return Err(TenantRbacDomainError::CrossTenantLegalEntity);
        }
        require_prefixed(
            &entity.legal_entity_id,
            LEGAL_ENTITY_ID_PREFIX,
            TenantRbacDomainError::InvalidLegalEntityId,
        )?;
        require_prefixed(
            &entity.payroll_evidence_ref,
            AUDIT_EVIDENCE_PREFIX,
            TenantRbacDomainError::InvalidAuditEvidenceRef,
        )?;
        require_prefixed(
            &entity.accounting_evidence_ref,
            AUDIT_EVIDENCE_PREFIX,
            TenantRbacDomainError::InvalidAuditEvidenceRef,
        )?;
        let entity_closed = entity.payroll_close_state == CloseBoundaryState::ProductionClosed
            && entity.accounting_close_state == CloseBoundaryState::ProductionClosed;
        all_entities_closed &= entity_closed;
        if entity_closed
            && (entity.payroll_close_version == 0 || entity.accounting_close_version == 0)
        {
            return Err(TenantRbacDomainError::MissingCloseEvidence);
        }
        projections.push(LegalEntityCloseProjection {
            legal_entity_id: internal(LegalEntityId {
                value: entity.legal_entity_id,
            }),
            payroll_close_state: internal(entity.payroll_close_state),
            accounting_close_state: internal(entity.accounting_close_state),
            payroll_evidence_ref: internal(AuditEvidenceRef {
                value: entity.payroll_evidence_ref,
            }),
            accounting_evidence_ref: internal(AuditEvidenceRef {
                value: entity.accounting_evidence_ref,
            }),
            payroll_close_version: internal(entity.payroll_close_version),
            accounting_close_version: internal(entity.accounting_close_version),
        });
    }

    Ok(GroupCloseRollup {
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        group_id: internal(GroupId {
            value: input.group_id,
        }),
        jurisdiction: internal(input.jurisdiction),
        dashboard_projection_ref: internal(DashboardProjectionRef {
            value: input.dashboard_projection_ref,
        }),
        legal_entity_count: internal(projections.len() as u32),
        all_entities_closed: internal(all_entities_closed),
        legal_entity_projections: internal(projections),
        schema_version: public(GROUP_ROLLUP_SCHEMA_VERSION),
    })
}

pub fn plan_cross_service_workflow(
    input: CrossServiceWorkflowInput,
) -> Result<CrossServiceWorkflowPlan, TenantRbacDomainError> {
    require_prefixed(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        TenantRbacDomainError::InvalidTenantId,
    )?;
    require_prefixed(
        &input.workflow_ref,
        WORKFLOW_REF_PREFIX,
        TenantRbacDomainError::InvalidWorkflowRef,
    )?;
    require_prefixed(
        &input.object_graph_relationship_ref,
        OBJECT_GRAPH_REF_PREFIX,
        TenantRbacDomainError::InvalidObjectGraphRelationshipRef,
    )?;
    if input.routing_owner != WorkflowRoutingOwner::Workflow {
        return Err(TenantRbacDomainError::WorkflowRoutingBypass);
    }
    if input.relationship_owner != ObjectGraphRelationshipOwner::ObjectGraph {
        return Err(TenantRbacDomainError::ObjectGraphBypass);
    }
    if input.gate_closure_authority == GateClosureAuthority::AiSuggestion {
        return Err(TenantRbacDomainError::AiCannotCloseDeterministicGate);
    }
    require_cross_service_set(&input.services)?;
    let gate_evidence = deterministic_gate_evidence(input.gate_evidence_refs)?;
    let ai_suggestion_ref = match input.ai_suggestion_ref {
        Some(value) => {
            require_prefixed(
                &value,
                AI_SUGGESTION_REF_PREFIX,
                TenantRbacDomainError::InvalidAiSuggestionRef,
            )?;
            Some(AiSuggestionRef { value })
        }
        None => None,
    };
    require_non_empty_key(&input.idempotency_key)?;

    Ok(CrossServiceWorkflowPlan {
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        workflow_ref: internal(WorkflowRef {
            value: input.workflow_ref,
        }),
        object_graph_relationship_ref: internal(ObjectGraphRelationshipRef {
            value: input.object_graph_relationship_ref,
        }),
        routing_owner: internal(input.routing_owner),
        relationship_owner: internal(input.relationship_owner),
        services: internal(input.services),
        required_gates: internal(required_workflow_gates()),
        gate_evidence: internal(gate_evidence),
        gate_closure_authority: internal(input.gate_closure_authority),
        ai_suggestion_ref: internal(ai_suggestion_ref),
        idempotency_key: internal(input.idempotency_key),
        schema_version: public(CROSS_SERVICE_WORKFLOW_SCHEMA_VERSION),
    })
}

pub fn plan_incident_rollback(
    input: IncidentRollbackInput,
) -> Result<IncidentRollbackPlan, TenantRbacDomainError> {
    require_prefixed(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        TenantRbacDomainError::InvalidTenantId,
    )?;
    require_prefixed(
        &input.incident_id,
        INCIDENT_ID_PREFIX,
        TenantRbacDomainError::InvalidIncidentId,
    )?;
    if !matches!(
        input.first_action,
        IncidentFirstAction::Rollback | IncidentFirstAction::Quarantine
    ) {
        return Err(TenantRbacDomainError::RollbackOrQuarantineMustBeFirst);
    }
    if input.remediation_route == IncidentRemediationRoute::ManualSsh {
        return Err(TenantRbacDomainError::ManualSshRefused);
    }
    require_prefixed(
        &input.canary_evidence_ref,
        AUDIT_EVIDENCE_PREFIX,
        TenantRbacDomainError::InvalidAuditEvidenceRef,
    )?;
    require_prefixed(
        &input.incident_evidence_ref,
        AUDIT_EVIDENCE_PREFIX,
        TenantRbacDomainError::InvalidAuditEvidenceRef,
    )?;
    require_prefixed(
        &input.rollback_evidence_ref,
        AUDIT_EVIDENCE_PREFIX,
        TenantRbacDomainError::InvalidAuditEvidenceRef,
    )?;
    require_convergence_ref(&input.convergence_ref)?;
    require_non_empty_key(&input.idempotency_key)?;

    Ok(IncidentRollbackPlan {
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        incident_id: internal(IncidentId {
            value: input.incident_id,
        }),
        trigger: internal(input.trigger),
        first_action: internal(input.first_action),
        remediation_route: internal(input.remediation_route),
        canary_evidence_ref: internal(AuditEvidenceRef {
            value: input.canary_evidence_ref,
        }),
        incident_evidence_ref: internal(AuditEvidenceRef {
            value: input.incident_evidence_ref,
        }),
        rollback_evidence_ref: internal(AuditEvidenceRef {
            value: input.rollback_evidence_ref,
        }),
        convergence_ref: internal(ConvergenceRef {
            value: input.convergence_ref,
        }),
        idempotency_key: internal(input.idempotency_key),
        plan_status: internal(IncidentPlanStatus::RollbackFirstAccepted),
        schema_version: public(INCIDENT_ROLLBACK_SCHEMA_VERSION),
    })
}

fn require_cross_service_set(services: &[TenantRbacService]) -> Result<(), TenantRbacDomainError> {
    let required = [
        TenantRbacService::Hr,
        TenantRbacService::Payroll,
        TenantRbacService::Accounting,
    ];
    if required
        .iter()
        .any(|required_service| !services.contains(required_service))
    {
        return Err(TenantRbacDomainError::MissingChildProduct);
    }
    Ok(())
}

fn deterministic_gate_evidence(
    evidence_refs: Vec<(DeterministicGate, String)>,
) -> Result<Vec<DeterministicGateEvidence>, TenantRbacDomainError> {
    let mut evidence = Vec::with_capacity(evidence_refs.len());
    for (gate, evidence_ref) in evidence_refs {
        require_prefixed(
            &evidence_ref,
            AUDIT_EVIDENCE_PREFIX,
            TenantRbacDomainError::InvalidAuditEvidenceRef,
        )?;
        evidence.push(DeterministicGateEvidence {
            gate: internal(gate),
            evidence_ref: internal(AuditEvidenceRef {
                value: evidence_ref,
            }),
        });
    }
    for required_gate in required_workflow_gates() {
        if !evidence
            .iter()
            .any(|entry| entry.gate.value == required_gate)
        {
            return Err(TenantRbacDomainError::MissingDeterministicGateEvidence);
        }
    }
    Ok(evidence)
}

fn required_workflow_gates() -> Vec<DeterministicGate> {
    vec![
        DeterministicGate::HumanApproval,
        DeterministicGate::EvidenceAttached,
        DeterministicGate::RollbackPlanAttached,
        DeterministicGate::LegalEntityBoundaryChecked,
    ]
}

fn require_convergence_ref(value: &str) -> Result<(), TenantRbacDomainError> {
    if value.starts_with(OPENTOFU_CONVERGENCE_PREFIX) {
        require_prefixed(
            value,
            OPENTOFU_CONVERGENCE_PREFIX,
            TenantRbacDomainError::InvalidConvergenceRef,
        )
    } else if value.starts_with(OPS_CONVERGENCE_PREFIX) {
        require_prefixed(
            value,
            OPS_CONVERGENCE_PREFIX,
            TenantRbacDomainError::InvalidConvergenceRef,
        )
    } else {
        Err(TenantRbacDomainError::InvalidConvergenceRef)
    }
}

fn require_prefixed(
    value: &str,
    prefix: &str,
    error: TenantRbacDomainError,
) -> Result<(), TenantRbacDomainError> {
    if value.len() <= prefix.len()
        || !value.starts_with(prefix)
        || value.contains("..")
        || value.chars().any(char::is_whitespace)
    {
        return Err(error);
    }
    Ok(())
}

fn require_non_empty_key(value: &str) -> Result<(), TenantRbacDomainError> {
    if value.trim().is_empty() || value.contains("..") || value.chars().any(char::is_whitespace) {
        return Err(TenantRbacDomainError::InvalidIdempotencyKey);
    }
    Ok(())
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, PrivacyDataClass::internal_only())
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}
