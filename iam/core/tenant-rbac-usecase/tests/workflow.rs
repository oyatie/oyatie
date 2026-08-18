#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iam_tenant_rbac_domain::{
    CrossServiceWorkflowInput, DeterministicGate, GateClosureAuthority,
    ObjectGraphRelationshipOwner, TenantRbacService, WorkflowRoutingOwner,
    plan_cross_service_workflow,
};
use iam_tenant_rbac_usecase::prepare_cross_service_workflow_envelope;
use oya_data_boundary_kernel::DataClass;

#[test]
fn workflow_envelope_routes_to_workflow_without_executing_it() {
    let plan = plan_cross_service_workflow(workflow_input()).expect("workflow plan");
    let envelope = prepare_cross_service_workflow_envelope(&plan);

    assert_eq!(
        envelope.topic.value,
        "workflow.tenant-rbac.cross-service.dispatch"
    );
    assert_eq!(envelope.tenant_id.value.value, "ten_acme");
    assert_eq!(
        envelope.workflow_ref.value.value,
        "workflow/tenant-rbac/hr-payroll-accounting"
    );
    assert_eq!(envelope.required_gates.value.len(), 4);
    assert_eq!(envelope.gate_evidence_refs.value.len(), 4);
    assert_eq!(
        envelope
            .ai_suggestion_ref
            .value
            .as_ref()
            .map(|item| item.value.as_str()),
        Some("ai/tenant-rbac/advice/001")
    );
    assert_eq!(envelope.payload_data_class.value, DataClass::InternalOnly);
    assert_eq!(envelope.schema_version.value, 1);
}

fn workflow_input() -> CrossServiceWorkflowInput {
    CrossServiceWorkflowInput {
        tenant_id: "ten_acme".to_owned(),
        workflow_ref: "workflow/tenant-rbac/hr-payroll-accounting".to_owned(),
        object_graph_relationship_ref: "object-graph/tenant-rbac/employee-payroll-journal"
            .to_owned(),
        routing_owner: WorkflowRoutingOwner::Workflow,
        relationship_owner: ObjectGraphRelationshipOwner::ObjectGraph,
        services: vec![
            TenantRbacService::Hr,
            TenantRbacService::Payroll,
            TenantRbacService::Accounting,
        ],
        gate_evidence_refs: vec![
            (
                DeterministicGate::HumanApproval,
                "audit/tenant-rbac/workflow/approval".to_owned(),
            ),
            (
                DeterministicGate::EvidenceAttached,
                "audit/tenant-rbac/workflow/evidence".to_owned(),
            ),
            (
                DeterministicGate::RollbackPlanAttached,
                "audit/tenant-rbac/workflow/rollback".to_owned(),
            ),
            (
                DeterministicGate::LegalEntityBoundaryChecked,
                "audit/tenant-rbac/workflow/entity-boundary".to_owned(),
            ),
        ],
        gate_closure_authority: GateClosureAuthority::DeterministicGateSet,
        ai_suggestion_ref: Some("ai/tenant-rbac/advice/001".to_owned()),
        idempotency_key: "ten_acme:workflow:hr-payroll-accounting".to_owned(),
    }
}
