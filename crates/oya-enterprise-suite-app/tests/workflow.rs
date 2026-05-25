#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_enterprise_suite_app::prepare_cross_product_workflow_envelope;
use oya_enterprise_suite_domain::{
    CrossProductWorkflowInput, DeterministicGate, EnterpriseChildProduct, GateClosureAuthority,
    ObjectGraphRelationshipOwner, WorkflowRoutingOwner, plan_cross_product_workflow,
};

#[test]
fn workflow_envelope_routes_to_workflow_without_executing_it() {
    let plan = plan_cross_product_workflow(workflow_input()).expect("workflow plan");
    let envelope = prepare_cross_product_workflow_envelope(&plan);

    assert_eq!(
        envelope.topic.value,
        "workflow.enterprise-suite.cross-product.dispatch"
    );
    assert_eq!(envelope.tenant_id.value.value, "ten_acme");
    assert_eq!(
        envelope.workflow_ref.value.value,
        "workflow/enterprise-suite/hr-payroll-accounting"
    );
    assert_eq!(envelope.required_gates.value.len(), 4);
    assert_eq!(envelope.gate_evidence_refs.value.len(), 4);
    assert_eq!(
        envelope
            .ai_suggestion_ref
            .value
            .as_ref()
            .map(|item| item.value.as_str()),
        Some("ai/enterprise-suite/advice/001")
    );
    assert_eq!(envelope.payload_data_class.value, DataClass::InternalOnly);
    assert_eq!(envelope.schema_version.value, 1);
}

fn workflow_input() -> CrossProductWorkflowInput {
    CrossProductWorkflowInput {
        tenant_id: "ten_acme".to_owned(),
        workflow_ref: "workflow/enterprise-suite/hr-payroll-accounting".to_owned(),
        object_graph_relationship_ref: "object-graph/enterprise-suite/employee-payroll-journal"
            .to_owned(),
        routing_owner: WorkflowRoutingOwner::Workflow,
        relationship_owner: ObjectGraphRelationshipOwner::ObjectGraph,
        child_products: vec![
            EnterpriseChildProduct::Hr,
            EnterpriseChildProduct::Payroll,
            EnterpriseChildProduct::Accounting,
        ],
        gate_evidence_refs: vec![
            (
                DeterministicGate::HumanApproval,
                "audit/enterprise-suite/workflow/approval".to_owned(),
            ),
            (
                DeterministicGate::EvidenceAttached,
                "audit/enterprise-suite/workflow/evidence".to_owned(),
            ),
            (
                DeterministicGate::RollbackPlanAttached,
                "audit/enterprise-suite/workflow/rollback".to_owned(),
            ),
            (
                DeterministicGate::LegalEntityBoundaryChecked,
                "audit/enterprise-suite/workflow/entity-boundary".to_owned(),
            ),
        ],
        gate_closure_authority: GateClosureAuthority::DeterministicGateSet,
        ai_suggestion_ref: Some("ai/enterprise-suite/advice/001".to_owned()),
        idempotency_key: "ten_acme:workflow:hr-payroll-accounting".to_owned(),
    }
}
