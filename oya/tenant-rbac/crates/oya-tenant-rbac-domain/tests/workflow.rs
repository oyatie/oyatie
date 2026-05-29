#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_tenant_rbac_domain::{
    CrossServiceWorkflowInput, DeterministicGate, GateClosureAuthority,
    ObjectGraphRelationshipOwner, TenantRbacDomainError, TenantRbacService, WorkflowRoutingOwner,
    plan_cross_service_workflow,
};

#[test]
fn test_cross_service_workflow_uses_deterministic_gates() {
    let plan = plan_cross_service_workflow(valid_input()).expect("workflow gate plan");

    assert_eq!(plan.tenant_id.value.value, "ten_acme");
    assert_eq!(
        plan.workflow_ref.value.value,
        "workflow/tenant-rbac/hr-payroll-accounting"
    );
    assert_eq!(
        plan.object_graph_relationship_ref.value.value,
        "object-graph/tenant-rbac/employee-payroll-journal"
    );
    assert_eq!(plan.required_gates.value.len(), 4);
    assert!(
        plan.required_gates
            .value
            .contains(&DeterministicGate::HumanApproval)
    );
    assert!(
        plan.required_gates
            .value
            .contains(&DeterministicGate::EvidenceAttached)
    );
    assert!(
        plan.required_gates
            .value
            .contains(&DeterministicGate::RollbackPlanAttached)
    );
    assert!(
        plan.required_gates
            .value
            .contains(&DeterministicGate::LegalEntityBoundaryChecked)
    );
    assert_eq!(
        plan.gate_closure_authority.value,
        GateClosureAuthority::DeterministicGateSet
    );
    assert_eq!(
        plan.ai_suggestion_ref
            .value
            .as_ref()
            .map(|suggestion| suggestion.value.as_str()),
        Some("ai/tenant-rbac/advice/001")
    );
    assert_eq!(plan.schema_version.value, 1);
}

#[test]
fn test_direct_service_routing_is_refused() {
    let error = plan_cross_service_workflow(CrossServiceWorkflowInput {
        routing_owner: WorkflowRoutingOwner::Service(TenantRbacService::Payroll),
        ..valid_input()
    })
    .expect_err("direct service routing must be refused");

    assert_eq!(error, TenantRbacDomainError::WorkflowRoutingBypass);
}

#[test]
fn test_object_graph_relationship_owner_required() {
    let error = plan_cross_service_workflow(CrossServiceWorkflowInput {
        relationship_owner: ObjectGraphRelationshipOwner::Service(TenantRbacService::Hr),
        ..valid_input()
    })
    .expect_err("service-owned relationship graph must be refused");

    assert_eq!(error, TenantRbacDomainError::ObjectGraphBypass);
}

#[test]
fn test_ai_suggestion_cannot_close_deterministic_gate() {
    let error = plan_cross_service_workflow(CrossServiceWorkflowInput {
        gate_closure_authority: GateClosureAuthority::AiSuggestion,
        ..valid_input()
    })
    .expect_err("AI suggestion cannot close deterministic gate");

    assert_eq!(error, TenantRbacDomainError::AiCannotCloseDeterministicGate);
}

#[test]
fn test_missing_required_gate_evidence_is_refused() {
    let error = plan_cross_service_workflow(CrossServiceWorkflowInput {
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
                DeterministicGate::LegalEntityBoundaryChecked,
                "audit/tenant-rbac/workflow/entity-boundary".to_owned(),
            ),
        ],
        ..valid_input()
    })
    .expect_err("rollback evidence must be required");

    assert_eq!(
        error,
        TenantRbacDomainError::MissingDeterministicGateEvidence
    );
}

fn valid_input() -> CrossServiceWorkflowInput {
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
