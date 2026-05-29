#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_hr_employment_app::{
    OnboardEmployeeCommand, onboard_employee, plan_labor_compliance_workflows,
    plan_leave_payroll_impact_envelope, prepare_sensitive_hr_read_envelope,
};
use oya_hr_employment_domain::{
    EmployeeCreate, EmploymentStatus, HrLifecycleKind, Jurisdiction, LeaveDecision,
    LeavePayrollImpactInput, LeaveRoutingMode, LegalEntityWorkforceSnapshot, PayrollImpactKind,
    SensitiveHrDataKind, SensitiveHrReadInput, SensitiveReadLegalBasis, SensitiveReadPurpose,
    TenantTierSnapshot,
};
use oya_hr_employment_storage_adapter_inmemory::{
    HrEmploymentStoragePort, HrStorageError, HrStoredRecordKind, InMemoryHrEmploymentStore,
    hr_storage_capabilities,
};

#[test]
fn hr_storage_records_metadata_without_durable_backend_claim() {
    let mut store = InMemoryHrEmploymentStore::new();

    let onboard = onboard_employee(OnboardEmployeeCommand {
        employee: employee_input(),
        event_id: "hrev_employee_created_001".to_owned(),
        lifecycle_kind: HrLifecycleKind::Created,
    })
    .expect("onboarding outcome");
    let lifecycle = store
        .persist_lifecycle_audit(&onboard.audit_envelope)
        .expect("persist lifecycle audit");
    assert_eq!(lifecycle.kind, HrStoredRecordKind::LifecycleAudit);
    assert_eq!(lifecycle.topic, "audit.hr.employment.lifecycle");
    assert_eq!(lifecycle.primary_ref, "hr/employee/emp_001");
    assert_eq!(lifecycle.storage_backend, "in-memory-hr-reference");

    let labor =
        plan_labor_compliance_workflows(snapshot_with_count(30)).expect("labor compliance outcome");
    let workflow = store
        .persist_labor_workflow_dispatch(&labor.workflow_dispatches[0])
        .expect("persist labor workflow dispatch");
    assert_eq!(workflow.kind, HrStoredRecordKind::LaborWorkflowDispatch);
    assert_eq!(workflow.topic, "workflow.hr.compliance.dispatch");
    assert_eq!(workflow.evidence_ref_count, 2);

    let leave = plan_leave_payroll_impact_envelope(leave_input()).expect("leave outcome");
    let leave_record = store
        .persist_leave_payroll_impact(&leave.payroll_impact_envelope)
        .expect("persist leave payroll impact");
    assert_eq!(leave_record.kind, HrStoredRecordKind::LeavePayrollImpact);
    assert_eq!(leave_record.topic, "integration.hr.payroll.leave-impact");
    assert_eq!(leave_record.primary_ref, "leave_001");

    let sensitive =
        prepare_sensitive_hr_read_envelope(sensitive_input()).expect("sensitive read outcome");
    let sensitive_record = store
        .persist_sensitive_read_policy(&sensitive.audit_envelope)
        .expect("persist sensitive read policy");
    assert_eq!(
        sensitive_record.kind,
        HrStoredRecordKind::SensitiveReadPolicy
    );
    assert_eq!(sensitive_record.topic, "audit.hr.sensitive-read.policy");
    assert_eq!(sensitive_record.evidence_ref_count, 4);

    assert_eq!(store.len(), 4);
    assert!(store.require_record(&lifecycle.idempotency_key).is_ok());

    let capabilities = hr_storage_capabilities();
    assert_eq!(capabilities.adapter, "in-memory-hr-reference");
    assert!(!capabilities.durable_backend_attached);
    assert!(!capabilities.postgres_rls_attached);
    assert!(!capabilities.sensitive_data_retrieval_attached);
    assert!(!capabilities.workflow_execution_attached);
    assert!(!capabilities.payroll_network_call_attached);
    assert!(!capabilities.audit_chain_emission_attached);
}

#[test]
fn hr_storage_refuses_duplicate_idempotency_keys() {
    let mut store = InMemoryHrEmploymentStore::new();
    let outcome = onboard_employee(OnboardEmployeeCommand {
        employee: employee_input(),
        event_id: "hrev_employee_created_001".to_owned(),
        lifecycle_kind: HrLifecycleKind::Created,
    })
    .expect("onboarding outcome");
    store
        .persist_lifecycle_audit(&outcome.audit_envelope)
        .expect("first persist");

    let error = store
        .persist_lifecycle_audit(&outcome.audit_envelope)
        .expect_err("duplicate idempotency key must be refused");
    assert_eq!(
        error,
        HrStorageError::DuplicateIdempotencyKey("emp_001:1".to_owned())
    );
}

#[test]
fn hr_storage_reservation_validates_key_shape_and_allows_commit() {
    let mut store = InMemoryHrEmploymentStore::new();
    assert_eq!(
        store.reserve_idempotency_key("bad key"),
        Err(HrStorageError::InvalidIdempotencyKey("bad key".to_owned()))
    );

    let leave = plan_leave_payroll_impact_envelope(leave_input()).expect("leave outcome");
    let key = leave.payroll_impact_envelope.idempotency_key.value.clone();
    store.reserve_idempotency_key(&key).expect("reserve key");
    store
        .persist_leave_payroll_impact(&leave.payroll_impact_envelope)
        .expect("reserved key can be committed once");
    assert_eq!(store.len(), 1);
}

fn employee_input() -> EmployeeCreate {
    EmployeeCreate {
        employee_id: "emp_001".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        person_ref: "person/acme/001".to_owned(),
        manager_id: Some("emp_mgr_001".to_owned()),
        employment_status: EmploymentStatus::Active,
        tenant_tier_snapshot: TenantTierSnapshot::EnterpriseGroup,
        audit_evidence_ref: "audit/hr/employee/001".to_owned(),
        data_class: None,
        version: 1,
    }
}

fn snapshot_with_count(active_employee_count: u32) -> LegalEntityWorkforceSnapshot {
    LegalEntityWorkforceSnapshot {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        jurisdiction: Jurisdiction::Korea,
        active_employee_count,
        rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        workflow_ref: "workflow/hr-compliance/kr".to_owned(),
        evidence_ref: "audit/hr/compliance/kr-threshold".to_owned(),
        evaluated_at_epoch_seconds: 1_779_519_600,
    }
}

fn leave_input() -> LeavePayrollImpactInput {
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

fn sensitive_input() -> SensitiveHrReadInput {
    SensitiveHrReadInput {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        actor_employee_id: "emp_admin_001".to_owned(),
        subject_employee_id: "emp_001".to_owned(),
        data_kind: SensitiveHrDataKind::Medical,
        purpose: SensitiveReadPurpose::BenefitsAdministration,
        legal_basis: SensitiveReadLegalBasis::Consent,
        policy_ref: "policy/hr/sensitive-read/benefits-2026".to_owned(),
        basis_evidence_ref: "audit/hr/privacy/emp_001/basis".to_owned(),
        consent_evidence_ref: Some("audit/hr/privacy/emp_001/consent".to_owned()),
        request_evidence_ref: "audit/hr/privacy/emp_001/request".to_owned(),
        read_log_evidence_ref: "audit/hr/privacy/emp_001/read-log".to_owned(),
        evaluated_at_epoch_seconds: 1_779_533_400,
    }
}
