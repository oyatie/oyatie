//! Tenant RBAC API DTO contract layer.
//!
//! Serializable request shapes convert into Tenant RBAC domain/app inputs
//! while staying transport-neutral. This crate does not mount routes, execute
//! Workflow, persist records, emit audit-chain rows, run OpenTofu, or roll back
//! infrastructure.
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use iam_tenant_rbac_domain::{
    CloseBoundaryState, CrossServiceWorkflowInput, DeterministicGate, GateClosureAuthority,
    GroupRollupInput, IncidentFirstAction, IncidentRemediationRoute, IncidentRollbackInput,
    IncidentTrigger, Jurisdiction, LegalEntityCloseSnapshot, ObjectGraphRelationshipOwner,
    ServiceWriteInput, TenantRbacService, TenantRbacWriteKind, WorkflowRoutingOwner,
};
use iam_tenant_rbac_usecase::{OpsCommandKind, TenantRbacOpsCommandInput, TenantRbacOpsRoute};
use data_boundary_kernel::DataClass;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorBody {
    pub code: String,            // data_class: INTERNAL_ONLY
    pub message: String,         // data_class: INTERNAL_ONLY
    pub details: Option<String>, // data_class: INTERNAL_ONLY
}

impl ApiErrorEnvelope {
    pub fn validation(message: impl Into<String>, details: Option<String>) -> Self {
        Self {
            error: ApiErrorBody {
                code: "VALIDATION_ERROR".to_owned(),
                message: message.into(),
                details,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceWriteAdmissionRequest {
    pub service: TenantRbacServiceDto,      // data_class: INTERNAL_ONLY
    pub write_kind: TenantRbacWriteKindDto, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,            // data_class: INTERNAL_ONLY
    pub payload_data_class: DataClassDto,   // data_class: INTERNAL_ONLY
    pub audit_evidence_ref: String,         // data_class: INTERNAL_ONLY
    pub policy_gateway_ref: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,            // data_class: INTERNAL_ONLY
    pub sequence: u64,                      // data_class: INTERNAL_ONLY
}

impl ServiceWriteAdmissionRequest {
    pub fn into_domain(self) -> ServiceWriteInput {
        ServiceWriteInput {
            service: self.service.into(),
            write_kind: self.write_kind.into(),
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            payload_data_class: Some(self.payload_data_class.into()),
            audit_evidence_ref: self.audit_evidence_ref,
            policy_gateway_ref: self.policy_gateway_ref,
            idempotency_key: self.idempotency_key,
            sequence: self.sequence,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupCloseRollupRequest {
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub group_id: String,                 // data_class: INTERNAL_ONLY
    pub jurisdiction: JurisdictionDto,    // data_class: INTERNAL_ONLY
    pub dashboard_projection_ref: String, // data_class: INTERNAL_ONLY
    pub legal_entities: Vec<LegalEntityCloseSnapshotRequest>, // data_class: INTERNAL_ONLY + FINANCIAL
}

impl GroupCloseRollupRequest {
    pub fn into_domain(self) -> GroupRollupInput {
        GroupRollupInput {
            tenant_id: self.tenant_id,
            group_id: self.group_id,
            jurisdiction: self.jurisdiction.into(),
            dashboard_projection_ref: self.dashboard_projection_ref,
            legal_entities: self
                .legal_entities
                .into_iter()
                .map(LegalEntityCloseSnapshotRequest::into_domain)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalEntityCloseSnapshotRequest {
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                       // data_class: INTERNAL_ONLY
    pub payroll_close_state: CloseBoundaryStateDto,    // data_class: INTERNAL_ONLY
    pub accounting_close_state: CloseBoundaryStateDto, // data_class: INTERNAL_ONLY
    pub payroll_evidence_ref: String,                  // data_class: INTERNAL_ONLY
    pub accounting_evidence_ref: String,               // data_class: INTERNAL_ONLY
    pub payroll_close_version: u64,                    // data_class: INTERNAL_ONLY
    pub accounting_close_version: u64,                 // data_class: INTERNAL_ONLY
}

impl LegalEntityCloseSnapshotRequest {
    pub fn into_domain(self) -> LegalEntityCloseSnapshot {
        LegalEntityCloseSnapshot {
            tenant_id: self.tenant_id,
            legal_entity_id: self.legal_entity_id,
            payroll_close_state: self.payroll_close_state.into(),
            accounting_close_state: self.accounting_close_state.into(),
            payroll_evidence_ref: self.payroll_evidence_ref,
            accounting_evidence_ref: self.accounting_evidence_ref,
            payroll_close_version: self.payroll_close_version,
            accounting_close_version: self.accounting_close_version,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossServiceWorkflowPlanRequest {
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub workflow_ref: String,                   // data_class: INTERNAL_ONLY
    pub object_graph_relationship_ref: String,  // data_class: INTERNAL_ONLY
    pub routing_owner: WorkflowRoutingOwnerDto, // data_class: INTERNAL_ONLY
    pub relationship_owner: ObjectGraphRelationshipOwnerDto, // data_class: INTERNAL_ONLY
    pub services: Vec<TenantRbacServiceDto>,    // data_class: INTERNAL_ONLY
    pub gate_evidence_refs: Vec<DeterministicGateEvidenceRequest>, // data_class: INTERNAL_ONLY
    pub gate_closure_authority: GateClosureAuthorityDto, // data_class: INTERNAL_ONLY
    pub ai_suggestion_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                // data_class: INTERNAL_ONLY
}

impl CrossServiceWorkflowPlanRequest {
    pub fn into_domain(self) -> CrossServiceWorkflowInput {
        CrossServiceWorkflowInput {
            tenant_id: self.tenant_id,
            workflow_ref: self.workflow_ref,
            object_graph_relationship_ref: self.object_graph_relationship_ref,
            routing_owner: self.routing_owner.into(),
            relationship_owner: self.relationship_owner.into(),
            services: self
                .services
                .into_iter()
                .map(TenantRbacServiceDto::into)
                .collect(),
            gate_evidence_refs: self
                .gate_evidence_refs
                .into_iter()
                .map(DeterministicGateEvidenceRequest::into_domain)
                .collect(),
            gate_closure_authority: self.gate_closure_authority.into(),
            ai_suggestion_ref: self.ai_suggestion_ref,
            idempotency_key: self.idempotency_key,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeterministicGateEvidenceRequest {
    pub gate: DeterministicGateDto, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,       // data_class: INTERNAL_ONLY
}

impl DeterministicGateEvidenceRequest {
    pub fn into_domain(self) -> (DeterministicGate, String) {
        (self.gate.into(), self.evidence_ref)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncidentRollbackPlanRequest {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub incident_id: String,                            // data_class: INTERNAL_ONLY
    pub trigger: IncidentTriggerDto,                    // data_class: INTERNAL_ONLY
    pub first_action: IncidentFirstActionDto,           // data_class: INTERNAL_ONLY
    pub remediation_route: IncidentRemediationRouteDto, // data_class: INTERNAL_ONLY
    pub canary_evidence_ref: String,                    // data_class: INTERNAL_ONLY
    pub incident_evidence_ref: String,                  // data_class: INTERNAL_ONLY
    pub rollback_evidence_ref: String,                  // data_class: INTERNAL_ONLY
    pub convergence_ref: String,                        // data_class: INTERNAL_ONLY
    pub idempotency_key: String,                        // data_class: INTERNAL_ONLY
}

impl IncidentRollbackPlanRequest {
    pub fn into_domain(self) -> IncidentRollbackInput {
        IncidentRollbackInput {
            tenant_id: self.tenant_id,
            incident_id: self.incident_id,
            trigger: self.trigger.into(),
            first_action: self.first_action.into(),
            remediation_route: self.remediation_route.into(),
            canary_evidence_ref: self.canary_evidence_ref,
            incident_evidence_ref: self.incident_evidence_ref,
            rollback_evidence_ref: self.rollback_evidence_ref,
            convergence_ref: self.convergence_ref,
            idempotency_key: self.idempotency_key,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantRbacOpsCommandRequest {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub route: TenantRbacOpsRouteDto,    // data_class: INTERNAL_ONLY
    pub command_kind: OpsCommandKindDto, // data_class: INTERNAL_ONLY
    pub evidence_ref: String,            // data_class: INTERNAL_ONLY
    pub change_plan_ref: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,         // data_class: INTERNAL_ONLY
}

impl TenantRbacOpsCommandRequest {
    pub fn into_app(self) -> TenantRbacOpsCommandInput {
        TenantRbacOpsCommandInput {
            tenant_id: self.tenant_id,
            route: self.route.into(),
            command_kind: self.command_kind.into(),
            evidence_ref: self.evidence_ref,
            change_plan_ref: self.change_plan_ref,
            idempotency_key: self.idempotency_key,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantRbacServiceDto {
    Hr,
    Payroll,
    Accounting,
}

impl From<TenantRbacServiceDto> for TenantRbacService {
    fn from(value: TenantRbacServiceDto) -> Self {
        match value {
            TenantRbacServiceDto::Hr => Self::Hr,
            TenantRbacServiceDto::Payroll => Self::Payroll,
            TenantRbacServiceDto::Accounting => Self::Accounting,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantRbacWriteKindDto {
    HrLifecycle,
    PayrollClose,
    AccountingJournal,
}

impl From<TenantRbacWriteKindDto> for TenantRbacWriteKind {
    fn from(value: TenantRbacWriteKindDto) -> Self {
        match value {
            TenantRbacWriteKindDto::HrLifecycle => Self::HrLifecycle,
            TenantRbacWriteKindDto::PayrollClose => Self::PayrollClose,
            TenantRbacWriteKindDto::AccountingJournal => Self::AccountingJournal,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataClassDto {
    Public,
    InternalOnly,
    PiiIdentifying,
    Financial,
}

impl From<DataClassDto> for DataClass {
    fn from(value: DataClassDto) -> Self {
        match value {
            DataClassDto::Public => Self::Public,
            DataClassDto::InternalOnly => Self::InternalOnly,
            DataClassDto::PiiIdentifying => Self::PiiIdentifying,
            DataClassDto::Financial => Self::Financial,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CloseBoundaryStateDto {
    Open,
    TrialClosed,
    ProductionClosed,
    Quarantined,
}

impl From<CloseBoundaryStateDto> for CloseBoundaryState {
    fn from(value: CloseBoundaryStateDto) -> Self {
        match value {
            CloseBoundaryStateDto::Open => Self::Open,
            CloseBoundaryStateDto::TrialClosed => Self::TrialClosed,
            CloseBoundaryStateDto::ProductionClosed => Self::ProductionClosed,
            CloseBoundaryStateDto::Quarantined => Self::Quarantined,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JurisdictionDto {
    Korea,
    UnitedStates,
    EuropeanUnion,
}

impl From<JurisdictionDto> for Jurisdiction {
    fn from(value: JurisdictionDto) -> Self {
        match value {
            JurisdictionDto::Korea => Self::Korea,
            JurisdictionDto::UnitedStates => Self::UnitedStates,
            JurisdictionDto::EuropeanUnion => Self::EuropeanUnion,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowRoutingOwnerDto {
    Workflow,
    ServiceHr,
    ServicePayroll,
    ServiceAccounting,
    AiAgent,
}

impl From<WorkflowRoutingOwnerDto> for WorkflowRoutingOwner {
    fn from(value: WorkflowRoutingOwnerDto) -> Self {
        match value {
            WorkflowRoutingOwnerDto::Workflow => Self::Workflow,
            WorkflowRoutingOwnerDto::ServiceHr => Self::Service(TenantRbacService::Hr),
            WorkflowRoutingOwnerDto::ServicePayroll => Self::Service(TenantRbacService::Payroll),
            WorkflowRoutingOwnerDto::ServiceAccounting => {
                Self::Service(TenantRbacService::Accounting)
            }
            WorkflowRoutingOwnerDto::AiAgent => Self::AiAgent,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObjectGraphRelationshipOwnerDto {
    ObjectGraph,
    ServiceHr,
    ServicePayroll,
    ServiceAccounting,
    AiAgent,
}

impl From<ObjectGraphRelationshipOwnerDto> for ObjectGraphRelationshipOwner {
    fn from(value: ObjectGraphRelationshipOwnerDto) -> Self {
        match value {
            ObjectGraphRelationshipOwnerDto::ObjectGraph => Self::ObjectGraph,
            ObjectGraphRelationshipOwnerDto::ServiceHr => Self::Service(TenantRbacService::Hr),
            ObjectGraphRelationshipOwnerDto::ServicePayroll => {
                Self::Service(TenantRbacService::Payroll)
            }
            ObjectGraphRelationshipOwnerDto::ServiceAccounting => {
                Self::Service(TenantRbacService::Accounting)
            }
            ObjectGraphRelationshipOwnerDto::AiAgent => Self::AiAgent,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeterministicGateDto {
    HumanApproval,
    EvidenceAttached,
    RollbackPlanAttached,
    LegalEntityBoundaryChecked,
}

impl From<DeterministicGateDto> for DeterministicGate {
    fn from(value: DeterministicGateDto) -> Self {
        match value {
            DeterministicGateDto::HumanApproval => Self::HumanApproval,
            DeterministicGateDto::EvidenceAttached => Self::EvidenceAttached,
            DeterministicGateDto::RollbackPlanAttached => Self::RollbackPlanAttached,
            DeterministicGateDto::LegalEntityBoundaryChecked => Self::LegalEntityBoundaryChecked,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateClosureAuthorityDto {
    DeterministicGateSet,
    HumanApprover,
    AiSuggestion,
}

impl From<GateClosureAuthorityDto> for GateClosureAuthority {
    fn from(value: GateClosureAuthorityDto) -> Self {
        match value {
            GateClosureAuthorityDto::DeterministicGateSet => Self::DeterministicGateSet,
            GateClosureAuthorityDto::HumanApprover => Self::HumanApprover,
            GateClosureAuthorityDto::AiSuggestion => Self::AiSuggestion,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IncidentTriggerDto {
    CanarySloBreach,
    ProductionIncident,
    EvidenceGateFailure,
}

impl From<IncidentTriggerDto> for IncidentTrigger {
    fn from(value: IncidentTriggerDto) -> Self {
        match value {
            IncidentTriggerDto::CanarySloBreach => Self::CanarySloBreach,
            IncidentTriggerDto::ProductionIncident => Self::ProductionIncident,
            IncidentTriggerDto::EvidenceGateFailure => Self::EvidenceGateFailure,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IncidentFirstActionDto {
    Rollback,
    Quarantine,
    Remediate,
}

impl From<IncidentFirstActionDto> for IncidentFirstAction {
    fn from(value: IncidentFirstActionDto) -> Self {
        match value {
            IncidentFirstActionDto::Rollback => Self::Rollback,
            IncidentFirstActionDto::Quarantine => Self::Quarantine,
            IncidentFirstActionDto::Remediate => Self::Remediate,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IncidentRemediationRouteDto {
    OpenTofu,
    OyaOps,
    OpsConsole,
    ManualSsh,
}

impl From<IncidentRemediationRouteDto> for IncidentRemediationRoute {
    fn from(value: IncidentRemediationRouteDto) -> Self {
        match value {
            IncidentRemediationRouteDto::OpenTofu => Self::OpenTofu,
            IncidentRemediationRouteDto::OyaOps => Self::OyaOps,
            IncidentRemediationRouteDto::OpsConsole => Self::OpsConsole,
            IncidentRemediationRouteDto::ManualSsh => Self::ManualSsh,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantRbacOpsRouteDto {
    MakefileTarget,
    OyaOps,
    OpsConsole,
    ManualSsh,
}

impl From<TenantRbacOpsRouteDto> for TenantRbacOpsRoute {
    fn from(value: TenantRbacOpsRouteDto) -> Self {
        match value {
            TenantRbacOpsRouteDto::MakefileTarget => Self::MakefileTarget,
            TenantRbacOpsRouteDto::OyaOps => Self::OyaOps,
            TenantRbacOpsRouteDto::OpsConsole => Self::OpsConsole,
            TenantRbacOpsRouteDto::ManualSsh => Self::ManualSsh,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OpsCommandKindDto {
    Bootstrap,
    Install,
    Plan,
    Apply,
    Rollback,
    Day2Change,
}

impl From<OpsCommandKindDto> for OpsCommandKind {
    fn from(value: OpsCommandKindDto) -> Self {
        match value {
            OpsCommandKindDto::Bootstrap => Self::Bootstrap,
            OpsCommandKindDto::Install => Self::Install,
            OpsCommandKindDto::Plan => Self::Plan,
            OpsCommandKindDto::Apply => Self::Apply,
            OpsCommandKindDto::Rollback => Self::Rollback,
            OpsCommandKindDto::Day2Change => Self::Day2Change,
        }
    }
}
