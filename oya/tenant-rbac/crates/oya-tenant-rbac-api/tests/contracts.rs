#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_tenant_rbac_api::{
    ApiErrorEnvelope, CloseBoundaryStateDto, CrossServiceWorkflowPlanRequest, DataClassDto,
    DeterministicGateDto, DeterministicGateEvidenceRequest, GateClosureAuthorityDto,
    GroupCloseRollupRequest, IncidentFirstActionDto, IncidentRemediationRouteDto,
    IncidentRollbackPlanRequest, IncidentTriggerDto, JurisdictionDto,
    LegalEntityCloseSnapshotRequest, ObjectGraphRelationshipOwnerDto, OpsCommandKindDto,
    ServiceWriteAdmissionRequest, TenantRbacOpsCommandRequest, TenantRbacOpsRouteDto,
    TenantRbacServiceDto, TenantRbacWriteKindDto, WorkflowRoutingOwnerDto,
};
use oya_tenant_rbac_domain::{
    admit_service_write, plan_cross_service_workflow, plan_incident_rollback,
    roll_up_group_close_status,
};
use oya_tenant_rbac_usecase::{
    prepare_cross_service_workflow_envelope, prepare_incident_rollback_envelope,
    prepare_tenant_rbac_ops_envelope,
};
use serde_json::json;

#[test]
fn child_write_admission_request_uses_camel_case_and_stable_enums() {
    let request = service_write_request();
    let body = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(body["service"], "PAYROLL");
    assert_eq!(body["writeKind"], "PAYROLL_CLOSE");
    assert_eq!(body["payloadDataClass"], "FINANCIAL");

    let decision = admit_service_write(request.into_domain()).expect("policy decision");
    assert_eq!(
        decision.service.value,
        oya_tenant_rbac_domain::TenantRbacService::Payroll
    );
    assert_eq!(decision.payload_data_class.value, DataClass::Financial);
}

#[test]
fn group_rollup_request_converts_to_domain_input() {
    let rollup = roll_up_group_close_status(group_rollup_request().into_domain()).expect("rollup");

    assert_eq!(rollup.group_id.value.value, "grp_acme_kr");
    assert_eq!(rollup.legal_entity_count.value, 2);
    assert!(rollup.all_entities_closed.value);
}

#[test]
fn workflow_request_converts_to_app_envelope() {
    let plan = plan_cross_service_workflow(workflow_request().into_domain()).expect("plan");
    let envelope = prepare_cross_service_workflow_envelope(&plan);

    assert_eq!(
        envelope.topic.value,
        "workflow.tenant-rbac.cross-service.dispatch"
    );
    assert_eq!(envelope.required_gates.value.len(), 4);
    assert_eq!(envelope.gate_evidence_refs.value.len(), 4);
}

#[test]
fn incident_request_converts_to_incident_envelope() {
    let plan = plan_incident_rollback(incident_request().into_domain()).expect("incident plan");
    let envelope = prepare_incident_rollback_envelope(&plan);

    assert_eq!(envelope.topic.value, "incident.tenant-rbac.rollback.plan");
    assert_eq!(envelope.incident_id.value.value, "inc_canary_slo_001");
    assert_eq!(
        envelope.first_action.value,
        oya_tenant_rbac_domain::IncidentFirstAction::Rollback
    );
}

#[test]
fn ops_request_converts_to_no_manual_route() {
    let envelope =
        prepare_tenant_rbac_ops_envelope(ops_request().into_app()).expect("ops envelope");

    assert_eq!(envelope.topic.value, "audit.tenant-rbac.ops.command");
    assert_eq!(envelope.tenant_id.value.value, "ten_acme");
}

#[test]
fn error_envelope_has_consistent_shape() {
    let envelope =
        ApiErrorEnvelope::validation("Invalid tenant-rbac request", Some("tenantId".to_owned()));

    assert_eq!(
        serde_json::to_value(envelope).expect("serialize error"),
        json!({
            "error": {
                "code": "VALIDATION_ERROR",
                "message": "Invalid tenant-rbac request",
                "details": "tenantId"
            }
        })
    );
}

fn service_write_request() -> ServiceWriteAdmissionRequest {
    ServiceWriteAdmissionRequest {
        service: TenantRbacServiceDto::Payroll,
        write_kind: TenantRbacWriteKindDto::PayrollClose,
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payload_data_class: DataClassDto::Financial,
        audit_evidence_ref: "audit/tenant-rbac/write/payroll-close".to_owned(),
        policy_gateway_ref: "policy/tenant-rbac/shared-gateway".to_owned(),
        idempotency_key: "ten_acme:le_kr_001:payroll-close".to_owned(),
        sequence: 1,
    }
}

fn group_rollup_request() -> GroupCloseRollupRequest {
    GroupCloseRollupRequest {
        tenant_id: "ten_acme".to_owned(),
        group_id: "grp_acme_kr".to_owned(),
        jurisdiction: JurisdictionDto::Korea,
        dashboard_projection_ref: "projection/tenant-rbac/group/kr".to_owned(),
        legal_entities: vec![
            closed_snapshot("le_kr_001", 7, 11),
            closed_snapshot("le_kr_002", 3, 5),
        ],
    }
}

fn closed_snapshot(
    legal_entity_id: &str,
    payroll_close_version: u64,
    accounting_close_version: u64,
) -> LegalEntityCloseSnapshotRequest {
    LegalEntityCloseSnapshotRequest {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: legal_entity_id.to_owned(),
        payroll_close_state: CloseBoundaryStateDto::ProductionClosed,
        accounting_close_state: CloseBoundaryStateDto::ProductionClosed,
        payroll_evidence_ref: format!("audit/payroll/{legal_entity_id}"),
        accounting_evidence_ref: format!("audit/accounting/{legal_entity_id}"),
        payroll_close_version,
        accounting_close_version,
    }
}

fn workflow_request() -> CrossServiceWorkflowPlanRequest {
    CrossServiceWorkflowPlanRequest {
        tenant_id: "ten_acme".to_owned(),
        workflow_ref: "workflow/tenant-rbac/hr-payroll-accounting".to_owned(),
        object_graph_relationship_ref: "object-graph/tenant-rbac/employee-payroll-journal"
            .to_owned(),
        routing_owner: WorkflowRoutingOwnerDto::Workflow,
        relationship_owner: ObjectGraphRelationshipOwnerDto::ObjectGraph,
        services: vec![
            TenantRbacServiceDto::Hr,
            TenantRbacServiceDto::Payroll,
            TenantRbacServiceDto::Accounting,
        ],
        gate_evidence_refs: vec![
            gate(DeterministicGateDto::HumanApproval, "approval"),
            gate(DeterministicGateDto::EvidenceAttached, "evidence"),
            gate(DeterministicGateDto::RollbackPlanAttached, "rollback"),
            gate(
                DeterministicGateDto::LegalEntityBoundaryChecked,
                "entity-boundary",
            ),
        ],
        gate_closure_authority: GateClosureAuthorityDto::DeterministicGateSet,
        ai_suggestion_ref: Some("ai/tenant-rbac/advice/001".to_owned()),
        idempotency_key: "ten_acme:workflow:hr-payroll-accounting".to_owned(),
    }
}

fn gate(gate: DeterministicGateDto, suffix: &str) -> DeterministicGateEvidenceRequest {
    DeterministicGateEvidenceRequest {
        gate,
        evidence_ref: format!("audit/tenant-rbac/workflow/{suffix}"),
    }
}

fn incident_request() -> IncidentRollbackPlanRequest {
    IncidentRollbackPlanRequest {
        tenant_id: "ten_acme".to_owned(),
        incident_id: "inc_canary_slo_001".to_owned(),
        trigger: IncidentTriggerDto::CanarySloBreach,
        first_action: IncidentFirstActionDto::Rollback,
        remediation_route: IncidentRemediationRouteDto::OpenTofu,
        canary_evidence_ref: "audit/tenant-rbac/incidents/canary-slo".to_owned(),
        incident_evidence_ref: "audit/tenant-rbac/incidents/inc_canary_slo_001.json".to_owned(),
        rollback_evidence_ref: "audit/tenant-rbac/incidents/rollback-first".to_owned(),
        convergence_ref: "opentofu/tenant-rbac/fixes/inc_canary_slo_001".to_owned(),
        idempotency_key: "ten_acme:incident:inc_canary_slo_001".to_owned(),
    }
}

fn ops_request() -> TenantRbacOpsCommandRequest {
    TenantRbacOpsCommandRequest {
        tenant_id: "ten_acme".to_owned(),
        route: TenantRbacOpsRouteDto::OyaOps,
        command_kind: OpsCommandKindDto::Day2Change,
        evidence_ref: "audit/tenant-rbac/ops/day2".to_owned(),
        change_plan_ref: "opentofu/tenant-rbac/day2-plan".to_owned(),
        idempotency_key: "ten_acme:day2:plan".to_owned(),
    }
}
