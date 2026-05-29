#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_hr_employment_app::{
    OnboardEmployeeCommand, onboard_employee, plan_labor_compliance_workflows,
};
use oya_hr_employment_domain::{
    EmployeeCreate, EmploymentStatus, HrLifecycleKind, Jurisdiction, LaborComplianceObligationKind,
    LegalEntityWorkforceSnapshot, TenantTierSnapshot,
};

#[test]
fn onboarding_emits_metadata_only_audit_event() {
    let outcome = onboard_employee(OnboardEmployeeCommand {
        employee: employee_input(),
        event_id: "hrev_employee_created_001".to_owned(),
        lifecycle_kind: HrLifecycleKind::Created,
    })
    .expect("onboarding outcome");

    assert_eq!(outcome.employee.employee_id.value.value, "emp_001");
    assert_eq!(outcome.lifecycle_event.tenant_id.value.value, "ten_acme");
    assert_eq!(
        outcome.audit_envelope.topic.value,
        "audit.hr.employment.lifecycle"
    );
    assert_eq!(
        outcome.audit_envelope.aggregate_ref.value,
        "hr/employee/emp_001"
    );
    assert_eq!(
        outcome.audit_envelope.evidence_ref.value.value,
        "audit/hr/employee/001"
    );
    assert_eq!(
        outcome.audit_envelope.payload_data_class.value,
        DataClass::PiiIdentifying
    );
    assert_eq!(
        outcome
            .audit_envelope
            .payload_data_class
            .data_class
            .compatibility_data_class(),
        DataClass::InternalOnly
    );
    assert_eq!(
        outcome
            .audit_envelope
            .schema_version
            .data_class
            .compatibility_data_class(),
        DataClass::Public
    );
}

#[test]
fn kr_obligations_create_workflow_dispatches() {
    let outcome =
        plan_labor_compliance_workflows(snapshot_with_count(30)).expect("workflow plan outcome");

    assert_eq!(outcome.obligations.len(), 2);
    assert_eq!(outcome.workflow_dispatches.len(), 2);

    let rules = outcome
        .workflow_dispatches
        .iter()
        .find(|dispatch| {
            dispatch.obligation_kind.value == LaborComplianceObligationKind::KoreaRulesOfEmployment
        })
        .expect("rules-of-employment dispatch");
    assert_eq!(rules.topic.value, "workflow.hr.compliance.dispatch");
    assert_eq!(rules.tenant_id.value.value, "ten_acme");
    assert_eq!(rules.legal_entity_id.value.value, "le_kr_001");
    assert_eq!(rules.workflow_ref.value.value, "workflow/hr-compliance/kr");
    assert_eq!(
        rules.evidence_refs.value[0].value,
        "audit/hr/compliance/kr-threshold"
    );
    assert!(
        rules
            .idempotency_key
            .value
            .contains("korea_rules_of_employment")
    );
    assert_eq!(
        rules.schema_version.data_class.compatibility_data_class(),
        DataClass::Public
    );
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
