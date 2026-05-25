#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_accounting_journal_app::record_payroll_posting;
use oya_accounting_journal_domain::{JournalLineInput, PayrollPostingInput};
use oya_enterprise_suite_app::prepare_cross_product_workflow_envelope;
use oya_enterprise_suite_domain::{
    CrossProductWorkflowInput, DeterministicGate, EnterpriseChildProduct, GateClosureAuthority,
    ObjectGraphRelationshipOwner, WorkflowRoutingOwner, plan_cross_product_workflow,
};
use oya_enterprise_suite_local_inmemory_harness::{
    EnterpriseLocalInMemoryHarness, EnterpriseLocalInMemoryHarnessError,
};
use oya_hr_employment_app::plan_leave_payroll_impact_envelope;
use oya_hr_employment_domain::{
    LeaveDecision, LeavePayrollImpactInput, LeaveRoutingMode, PayrollImpactKind,
};
use oya_hr_employment_storage_adapter_inmemory::HrStorageError;
use oya_payroll_run_app::{prepare_accounting_dispatch, prepare_hr_leave_impact_intake};
use oya_payroll_run_domain::{
    HrLeaveImpactIntakeInput, HrLeaveImpactKind, PayrollJournalInput, PayrollJournalLineInput,
};

#[test]
fn enterprise_local_inmemory_harness_records_cross_service_metadata_without_cloud_claim() {
    let mut harness = EnterpriseLocalInMemoryHarness::new();

    let hr_leave = plan_leave_payroll_impact_envelope(hr_leave_input()).expect("HR leave impact");
    let hr_record = harness
        .record_hr_leave_payroll_impact(&hr_leave.payroll_impact_envelope)
        .expect("persist HR leave impact");
    assert_eq!(hr_record.topic, "integration.hr.payroll.leave-impact");
    assert_eq!(hr_record.primary_ref, "leave_001");

    let payroll_intake =
        prepare_hr_leave_impact_intake(payroll_hr_leave_input()).expect("payroll leave intake");
    let payroll_intake_record = harness
        .record_payroll_hr_leave_impact_intake(&payroll_intake.intake_envelope)
        .expect("persist payroll HR leave intake");
    assert_eq!(
        payroll_intake_record.topic,
        "integration.payroll.hr.leave-impact-intake"
    );
    assert_eq!(payroll_intake_record.primary_ref, "leave_001");

    let payroll_dispatch =
        prepare_accounting_dispatch(payroll_journal_input()).expect("payroll accounting dispatch");
    let payroll_dispatch_record = harness
        .record_payroll_accounting_dispatch(&payroll_dispatch.dispatch_envelope)
        .expect("persist payroll accounting dispatch");
    assert_eq!(
        payroll_dispatch_record.topic,
        "enterprise.payroll.accounting.journal_draft"
    );
    assert_eq!(payroll_dispatch_record.primary_ref, "jrn_payroll_2026_06");

    let accounting_posting =
        record_payroll_posting(accounting_payroll_input()).expect("accounting payroll posting");
    let accounting_record = harness
        .record_accounting_payroll_posting(&accounting_posting.audit_envelope)
        .expect("persist accounting payroll posting");
    assert_eq!(accounting_record.topic, "audit.accounting.payroll.posted");
    assert_eq!(accounting_record.primary_ref, "jrn_payroll_2026_06");

    let workflow_plan =
        plan_cross_product_workflow(suite_workflow_input()).expect("suite workflow plan");
    let workflow_envelope = prepare_cross_product_workflow_envelope(&workflow_plan);
    let workflow_record = harness
        .record_enterprise_suite_workflow_dispatch(&workflow_envelope)
        .expect("persist and queue suite workflow");
    assert_eq!(
        workflow_record.storage_record.topic,
        "workflow.enterprise-suite.cross-product.dispatch"
    );
    assert_eq!(
        workflow_record.dispatch_record.queue_backend,
        "in-memory-workflow-reference"
    );
    assert_eq!(workflow_record.dispatch_record.required_gate_count, 4);

    let snapshot = harness.snapshot();
    assert_eq!(snapshot.hr_records, 1);
    assert_eq!(snapshot.payroll_records, 2);
    assert_eq!(snapshot.accounting_records, 1);
    assert_eq!(snapshot.enterprise_suite_records, 1);
    assert_eq!(snapshot.enterprise_workflow_dispatches, 1);

    let capabilities = harness.capabilities();
    assert_eq!(capabilities.harness, "enterprise-local-inmemory-harness");
    assert!(capabilities.in_memory_storage_integration_attached);
    assert!(!capabilities.durable_storage_attached);
    assert!(!capabilities.postgres_rls_attached);
    assert!(!capabilities.deployed_listener_attached);
    assert!(!capabilities.child_network_calls_attached);
    assert!(!capabilities.workflow_engine_attached);
    assert!(!capabilities.broker_publish_attached);
    assert!(!capabilities.statutory_filing_rails_attached);
    assert!(!capabilities.disbursement_rails_attached);
    assert!(!capabilities.cloud_deployment_attached);
    assert!(!capabilities.runtime_audit_chain_emission_attached);
}

#[test]
fn enterprise_local_inmemory_harness_surfaces_duplicate_store_errors() {
    let mut harness = EnterpriseLocalInMemoryHarness::new();
    let hr_leave = plan_leave_payroll_impact_envelope(hr_leave_input()).expect("HR leave impact");
    harness
        .record_hr_leave_payroll_impact(&hr_leave.payroll_impact_envelope)
        .expect("first persist");

    let error = harness
        .record_hr_leave_payroll_impact(&hr_leave.payroll_impact_envelope)
        .expect_err("duplicate HR storage key must surface");
    assert!(matches!(
        error,
        EnterpriseLocalInMemoryHarnessError::HrStorage(HrStorageError::DuplicateIdempotencyKey(_))
    ));
}

fn digest() -> String {
    format!("sha256:{}", "c".repeat(64))
}

fn hr_leave_input() -> LeavePayrollImpactInput {
    LeavePayrollImpactInput {
        leave_request_id: "leave_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        employee_id: "emp_001".to_owned(),
        approver_id: "emp_hr_001".to_owned(),
        decision: LeaveDecision::Approved,
        routing_mode: LeaveRoutingMode::EscalatedHr,
        start_date: "2026-06-01".to_owned(),
        end_date: "2026-06-03".to_owned(),
        payroll_period: "2026-06".to_owned(),
        payroll_impact_kind: PayrollImpactKind::UnpaidLeaveDeduction,
        workflow_ref: "workflow/hr-leave/kr".to_owned(),
        rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        decision_evidence_ref: "audit/hr/leave/leave_001/decision".to_owned(),
        routing_evidence_ref: "audit/hr/leave/leave_001/escalation".to_owned(),
        payroll_impact_evidence_ref: "audit/hr/leave/leave_001/payroll-impact".to_owned(),
        decided_at_epoch_seconds: 1_779_532_800,
    }
}

fn payroll_hr_leave_input() -> HrLeaveImpactIntakeInput {
    HrLeaveImpactIntakeInput {
        run_id: "prun_kr_2026_06".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        payroll_period: "2026-06".to_owned(),
        payee_id: "payee_001".to_owned(),
        employee_id: "emp_001".to_owned(),
        leave_request_id: "leave_001".to_owned(),
        impact_kind: HrLeaveImpactKind::UnpaidLeaveDeduction,
        source_topic: "integration.hr.payroll.leave-impact".to_owned(),
        source_hr_idempotency_key: "ten_acme:leave_001:Approved:2026-06".to_owned(),
        decision_evidence_ref: "audit/hr/leave/leave_001/decision".to_owned(),
        routing_evidence_ref: "audit/hr/leave/leave_001/escalation".to_owned(),
        payroll_impact_evidence_ref: "audit/hr/leave/leave_001/payroll-impact".to_owned(),
        payroll_intake_evidence_ref: "audit/payroll/hr-leave/leave_001/intake".to_owned(),
        rulepack_ref: "rulepack/kr-payroll-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        received_at_epoch_seconds: 1_779_535_200,
    }
}

fn payroll_journal_input() -> PayrollJournalInput {
    PayrollJournalInput {
        journal_id: "jrn_payroll_2026_06".to_owned(),
        run_id: "prun_kr_2026_06".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-06".to_owned(),
        source_payroll_digest: digest(),
        approval_evidence_ref: "audit/payroll/approval/cfo".to_owned(),
        lines: vec![
            PayrollJournalLineInput {
                account_code: "EXP-WAGES".to_owned(),
                debit_minor: 1_000_000,
                credit_minor: 0,
            },
            PayrollJournalLineInput {
                account_code: "LIAB-NETPAY".to_owned(),
                debit_minor: 0,
                credit_minor: 1_000_000,
            },
        ],
    }
}

fn accounting_payroll_input() -> PayrollPostingInput {
    PayrollPostingInput {
        journal_id: "jrn_payroll_2026_06".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-06".to_owned(),
        source_payroll_digest: digest(),
        wage_ledger_refs: vec!["audit/payroll/wage-ledger/001".to_owned()],
        approval_evidence_ref: "audit/accounting/payroll/approval".to_owned(),
        reversal_path_ref: "audit/accounting/payroll/reversal".to_owned(),
        lines: vec![
            JournalLineInput {
                account_code: "EXP-WAGES".to_owned(),
                debit_minor: 1_000_000,
                credit_minor: 0,
            },
            JournalLineInput {
                account_code: "LIAB-NETPAY".to_owned(),
                debit_minor: 0,
                credit_minor: 1_000_000,
            },
        ],
    }
}

fn suite_workflow_input() -> CrossProductWorkflowInput {
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
        ai_suggestion_ref: None,
        idempotency_key: "ten_acme:workflow:hr-payroll-accounting:harness".to_owned(),
    }
}
