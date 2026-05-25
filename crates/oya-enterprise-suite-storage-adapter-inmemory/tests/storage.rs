#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_enterprise_suite_app::{
    EnterpriseOpsCommandInput, EnterpriseOpsRoute, OpsCommandKind,
    prepare_cross_product_workflow_envelope, prepare_enterprise_ops_envelope,
    prepare_incident_rollback_envelope,
};
use oya_enterprise_suite_domain::{
    ChildWriteInput, CloseBoundaryState, CrossProductWorkflowInput, DeterministicGate,
    EnterpriseChildProduct, GateClosureAuthority, GroupRollupInput, IncidentFirstAction,
    IncidentRemediationRoute, IncidentRollbackInput, IncidentTrigger, Jurisdiction,
    LegalEntityCloseSnapshot, ObjectGraphRelationshipOwner, SuiteWriteKind, WorkflowRoutingOwner,
    admit_child_write, plan_cross_product_workflow, plan_incident_rollback,
    roll_up_group_close_status,
};
use oya_enterprise_suite_storage_adapter_inmemory::{
    EnterpriseSuiteStorageError, EnterpriseSuiteStoragePort, EnterpriseSuiteStoredRecordKind,
    InMemoryEnterpriseSuiteStore, enterprise_suite_storage_capabilities, group_close_rollup_key,
};

#[test]
fn enterprise_suite_storage_records_metadata_without_durable_backend_claim() {
    let mut store = InMemoryEnterpriseSuiteStore::new();

    let policy = admit_child_write(policy_input()).expect("policy decision");
    let policy_record = store
        .persist_policy_decision(&policy)
        .expect("persist policy");
    assert_eq!(
        policy_record.kind,
        EnterpriseSuiteStoredRecordKind::PolicyAdmission
    );
    assert_eq!(
        policy_record.topic,
        "policy.enterprise-suite.child-write.admission"
    );
    assert_eq!(policy_record.storage_backend, "in-memory-reference");

    let rollup = roll_up_group_close_status(group_rollup_input()).expect("group rollup");
    let group_key = group_close_rollup_key(&rollup);
    let group_record = store
        .persist_group_close_rollup(&rollup)
        .expect("persist group rollup");
    assert_eq!(
        group_record.kind,
        EnterpriseSuiteStoredRecordKind::GroupCloseRollup
    );
    assert_eq!(group_record.idempotency_key, group_key);

    let workflow_plan = plan_cross_product_workflow(workflow_input()).expect("workflow plan");
    let workflow_envelope = prepare_cross_product_workflow_envelope(&workflow_plan);
    let workflow_record = store
        .persist_cross_product_workflow(&workflow_envelope)
        .expect("persist workflow");
    assert_eq!(
        workflow_record.kind,
        EnterpriseSuiteStoredRecordKind::CrossProductWorkflowPlan
    );
    assert_eq!(
        workflow_record.topic,
        "workflow.enterprise-suite.cross-product.dispatch"
    );

    let incident_plan = plan_incident_rollback(incident_input()).expect("incident plan");
    let incident_envelope = prepare_incident_rollback_envelope(&incident_plan);
    let incident_record = store
        .persist_incident_rollback(&incident_envelope)
        .expect("persist incident");
    assert_eq!(
        incident_record.kind,
        EnterpriseSuiteStoredRecordKind::IncidentRollbackPlan
    );
    assert_eq!(
        incident_record.topic,
        "incident.enterprise-suite.rollback.plan"
    );

    let ops_envelope = prepare_enterprise_ops_envelope(ops_input()).expect("ops envelope");
    let ops_record = store
        .persist_ops_command(&ops_envelope)
        .expect("persist ops");
    assert_eq!(ops_record.kind, EnterpriseSuiteStoredRecordKind::OpsCommand);
    assert_eq!(ops_record.topic, "audit.enterprise-suite.ops.command");

    assert_eq!(store.len(), 5);
    assert!(
        store
            .require_record("ten_acme:workflow:hr-payroll-accounting")
            .is_ok()
    );
    assert_eq!(store.list_records().len(), 5);

    let capabilities = enterprise_suite_storage_capabilities();
    assert_eq!(capabilities.adapter, "in-memory-reference");
    assert!(!capabilities.durable_backend_attached);
    assert!(!capabilities.postgres_rls_attached);
    assert!(!capabilities.cloud_object_store_attached);
    assert!(!capabilities.runtime_write_path_attached);
    assert!(!capabilities.workflow_execution_attached);
}

#[test]
fn enterprise_suite_storage_refuses_duplicate_idempotency_keys() {
    let mut store = InMemoryEnterpriseSuiteStore::new();
    let policy = admit_child_write(policy_input()).expect("policy decision");
    store
        .persist_policy_decision(&policy)
        .expect("first persist");

    let error = store
        .persist_policy_decision(&policy)
        .expect_err("duplicate idempotency key must be refused");
    assert_eq!(
        error,
        EnterpriseSuiteStorageError::DuplicateIdempotencyKey(
            "ten_acme:le_kr_001:payroll-close".to_owned()
        )
    );
}

#[test]
fn enterprise_suite_storage_reservation_validates_key_shape_and_allows_commit() {
    let mut store = InMemoryEnterpriseSuiteStore::new();
    assert_eq!(
        store.reserve_idempotency_key("bad key"),
        Err(EnterpriseSuiteStorageError::InvalidIdempotencyKey(
            "bad key".to_owned()
        ))
    );

    store
        .reserve_idempotency_key("ten_acme:reserved:ops")
        .expect("reserve key");
    let mut ops = ops_input();
    ops.idempotency_key = "ten_acme:reserved:ops".to_owned();
    let envelope = prepare_enterprise_ops_envelope(ops).expect("ops envelope");
    store
        .persist_ops_command(&envelope)
        .expect("reserved key can be committed once");
    assert_eq!(store.len(), 1);
}

fn policy_input() -> ChildWriteInput {
    ChildWriteInput {
        child_product: EnterpriseChildProduct::Payroll,
        write_kind: SuiteWriteKind::PayrollClose,
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payload_data_class: Some(DataClass::Financial),
        audit_evidence_ref: "audit/enterprise-suite/write/payroll-close".to_owned(),
        policy_gateway_ref: "policy/enterprise-suite/shared-gateway".to_owned(),
        idempotency_key: "ten_acme:le_kr_001:payroll-close".to_owned(),
        sequence: 1,
    }
}

fn group_rollup_input() -> GroupRollupInput {
    GroupRollupInput {
        tenant_id: "ten_acme".to_owned(),
        group_id: "grp_acme_kr".to_owned(),
        jurisdiction: Jurisdiction::Korea,
        dashboard_projection_ref: "projection/enterprise-suite/group/kr".to_owned(),
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
) -> LegalEntityCloseSnapshot {
    LegalEntityCloseSnapshot {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: legal_entity_id.to_owned(),
        payroll_close_state: CloseBoundaryState::ProductionClosed,
        accounting_close_state: CloseBoundaryState::ProductionClosed,
        payroll_evidence_ref: format!("audit/payroll/{legal_entity_id}"),
        accounting_evidence_ref: format!("audit/accounting/{legal_entity_id}"),
        payroll_close_version,
        accounting_close_version,
    }
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

fn incident_input() -> IncidentRollbackInput {
    IncidentRollbackInput {
        tenant_id: "ten_acme".to_owned(),
        incident_id: "inc_canary_slo_001".to_owned(),
        trigger: IncidentTrigger::CanarySloBreach,
        first_action: IncidentFirstAction::Rollback,
        remediation_route: IncidentRemediationRoute::OpenTofu,
        canary_evidence_ref: "audit/enterprise-suite/incidents/canary-slo".to_owned(),
        incident_evidence_ref: "audit/enterprise-suite/incidents/inc_canary_slo_001.json"
            .to_owned(),
        rollback_evidence_ref: "audit/enterprise-suite/incidents/rollback-first".to_owned(),
        convergence_ref: "opentofu/enterprise-suite/fixes/inc_canary_slo_001".to_owned(),
        idempotency_key: "ten_acme:incident:inc_canary_slo_001".to_owned(),
    }
}

fn ops_input() -> EnterpriseOpsCommandInput {
    EnterpriseOpsCommandInput {
        tenant_id: "ten_acme".to_owned(),
        route: EnterpriseOpsRoute::OyaOps,
        command_kind: OpsCommandKind::Day2Change,
        evidence_ref: "audit/enterprise-suite/ops/day2".to_owned(),
        change_plan_ref: "opentofu/enterprise-suite/day2-plan".to_owned(),
        idempotency_key: "ten_acme:day2:plan".to_owned(),
    }
}
