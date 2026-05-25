#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_enterprise_suite_app::prepare_cross_product_workflow_envelope;
use oya_enterprise_suite_domain::{
    CrossProductWorkflowInput, DeterministicGate, EnterpriseChildProduct, GateClosureAuthority,
    ObjectGraphRelationshipOwner, WorkflowRoutingOwner, plan_cross_product_workflow,
};
use oya_enterprise_suite_workflow_adapter_inmemory::{
    EnterpriseSuiteWorkflowDispatchPort, EnterpriseSuiteWorkflowDispatchStatus,
    EnterpriseSuiteWorkflowQueueError, InMemoryEnterpriseSuiteWorkflowQueue,
    enterprise_suite_workflow_queue_capabilities,
};

#[test]
fn enterprise_suite_workflow_queue_records_dispatch_without_execution_claim() {
    let plan = plan_cross_product_workflow(workflow_input()).expect("workflow plan");
    let envelope = prepare_cross_product_workflow_envelope(&plan);
    let mut queue = InMemoryEnterpriseSuiteWorkflowQueue::new();

    let record = queue
        .enqueue_dispatch(&envelope)
        .expect("enqueue workflow dispatch intent");

    assert_eq!(
        record.topic,
        "workflow.enterprise-suite.cross-product.dispatch"
    );
    assert_eq!(record.tenant_id, "ten_acme");
    assert_eq!(
        record.workflow_ref,
        "workflow/enterprise-suite/hr-payroll-accounting"
    );
    assert_eq!(record.required_gate_count, 4);
    assert_eq!(record.gate_evidence_count, 4);
    assert_eq!(
        record.ai_suggestion_ref.as_deref(),
        Some("ai/enterprise-suite/advice/001")
    );
    assert_eq!(
        record.dispatch_status,
        EnterpriseSuiteWorkflowDispatchStatus::QueuedMetadataOnly
    );
    assert_eq!(record.queue_backend, "in-memory-workflow-reference");
    assert_eq!(queue.len(), 1);
    assert!(
        queue
            .require_dispatch("ten_acme:workflow:hr-payroll-accounting")
            .is_ok()
    );

    let capabilities = enterprise_suite_workflow_queue_capabilities();
    assert_eq!(capabilities.adapter, "in-memory-workflow-reference");
    assert!(!capabilities.durable_queue_attached);
    assert!(!capabilities.workflow_engine_attached);
    assert!(!capabilities.broker_publish_attached);
    assert!(!capabilities.runtime_execution_attached);
    assert!(capabilities.in_memory_execution_reference_attached);
    assert!(!capabilities.child_service_calls_attached);
    assert!(!capabilities.audit_chain_emission_attached);
}

#[test]
fn enterprise_suite_workflow_queue_refuses_duplicate_dispatches() {
    let plan = plan_cross_product_workflow(workflow_input()).expect("workflow plan");
    let envelope = prepare_cross_product_workflow_envelope(&plan);
    let mut queue = InMemoryEnterpriseSuiteWorkflowQueue::new();

    queue
        .enqueue_dispatch(&envelope)
        .expect("first dispatch should queue");
    let error = queue
        .enqueue_dispatch(&envelope)
        .expect_err("duplicate workflow dispatch must be refused");

    assert_eq!(
        error,
        EnterpriseSuiteWorkflowQueueError::DuplicateDispatch(
            "ten_acme:workflow:hr-payroll-accounting".to_owned()
        )
    );
}

#[test]
fn enterprise_suite_workflow_queue_reservation_validates_key_shape_and_allows_commit() {
    let mut queue = InMemoryEnterpriseSuiteWorkflowQueue::new();
    assert_eq!(
        queue.reserve_dispatch_key("bad key"),
        Err(EnterpriseSuiteWorkflowQueueError::InvalidIdempotencyKey(
            "bad key".to_owned()
        ))
    );

    queue
        .reserve_dispatch_key("ten_acme:workflow:hr-payroll-accounting")
        .expect("reserve valid key");
    let plan = plan_cross_product_workflow(workflow_input()).expect("workflow plan");
    let envelope = prepare_cross_product_workflow_envelope(&plan);
    queue
        .enqueue_dispatch(&envelope)
        .expect("reserved key can be committed once");
    assert_eq!(queue.len(), 1);
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

#[test]
fn enterprise_suite_workflow_queue_executes_required_gates_in_memory_without_broker_claim() {
    let plan = plan_cross_product_workflow(workflow_input()).expect("workflow plan");
    let envelope = prepare_cross_product_workflow_envelope(&plan);
    let mut queue = InMemoryEnterpriseSuiteWorkflowQueue::new();

    queue
        .enqueue_dispatch(&envelope)
        .expect("dispatch intent queued before execution");
    let execution = queue
        .execute_dispatch(&envelope)
        .expect("in-memory workflow execution closes deterministic gates");

    assert_eq!(execution.tenant_id, "ten_acme");
    assert_eq!(
        execution.workflow_ref,
        "workflow/enterprise-suite/hr-payroll-accounting"
    );
    assert_eq!(
        execution.dispatch_idempotency_key,
        "ten_acme:workflow:hr-payroll-accounting"
    );
    assert_eq!(
        execution.idempotency_key,
        "ten_acme:workflow:hr-payroll-accounting:execution"
    );
    assert_eq!(execution.executed_gate_count, 4);
    assert_eq!(execution.gate_records.len(), 4);
    assert!(
        execution
            .gate_records
            .iter()
            .all(|gate| gate.gate_satisfied)
    );
    assert!(!execution.child_service_calls_attached);
    assert!(!execution.broker_publish_attached);
    assert!(!execution.durable_queue_attached);
    assert!(!execution.audit_chain_emission_attached);
    assert_eq!(queue.list_executions().len(), 1);
    assert_eq!(
        queue
            .get_execution("ten_acme:workflow:hr-payroll-accounting")
            .expect("execution is keyed by dispatch idempotency")
            .idempotency_key,
        "ten_acme:workflow:hr-payroll-accounting:execution"
    );
    assert_eq!(
        queue
            .require_dispatch("ten_acme:workflow:hr-payroll-accounting")
            .expect("dispatch should remain discoverable")
            .dispatch_status,
        EnterpriseSuiteWorkflowDispatchStatus::ExecutedInMemoryReference
    );

    let capabilities = enterprise_suite_workflow_queue_capabilities();
    assert!(capabilities.in_memory_execution_reference_attached);
    assert!(!capabilities.workflow_engine_attached);
    assert!(!capabilities.broker_publish_attached);
    assert!(!capabilities.runtime_execution_attached);
}

#[test]
fn enterprise_suite_workflow_queue_requires_dispatch_before_execution() {
    let plan = plan_cross_product_workflow(workflow_input()).expect("workflow plan");
    let envelope = prepare_cross_product_workflow_envelope(&plan);
    let mut queue = InMemoryEnterpriseSuiteWorkflowQueue::new();

    let error = queue
        .execute_dispatch(&envelope)
        .expect_err("execution without queued dispatch is refused");

    assert_eq!(
        error,
        EnterpriseSuiteWorkflowQueueError::MissingDispatch(
            "ten_acme:workflow:hr-payroll-accounting".to_owned()
        )
    );
}

#[test]
fn enterprise_suite_workflow_queue_rejects_execution_gate_evidence_drift() {
    let plan = plan_cross_product_workflow(workflow_input()).expect("workflow plan");
    let envelope = prepare_cross_product_workflow_envelope(&plan);
    let mut mutated_envelope = envelope.clone();
    mutated_envelope
        .gate_evidence_refs
        .value
        .pop()
        .expect("fixture has evidence refs");
    let mut queue = InMemoryEnterpriseSuiteWorkflowQueue::new();

    queue.enqueue_dispatch(&envelope).expect("dispatch queued");
    let error = queue
        .execute_dispatch(&mutated_envelope)
        .expect_err("execution must reject gate/evidence drift after dispatch");

    assert_eq!(
        error,
        EnterpriseSuiteWorkflowQueueError::GateEvidenceMismatch
    );
}

#[test]
fn enterprise_suite_workflow_queue_refuses_duplicate_execution() {
    let plan = plan_cross_product_workflow(workflow_input()).expect("workflow plan");
    let envelope = prepare_cross_product_workflow_envelope(&plan);
    let mut queue = InMemoryEnterpriseSuiteWorkflowQueue::new();

    queue.enqueue_dispatch(&envelope).expect("dispatch queued");
    queue.execute_dispatch(&envelope).expect("first execution");
    let error = queue
        .execute_dispatch(&envelope)
        .expect_err("duplicate execution must be refused");

    assert_eq!(
        error,
        EnterpriseSuiteWorkflowQueueError::DuplicateExecution(
            "ten_acme:workflow:hr-payroll-accounting".to_owned()
        )
    );
}
