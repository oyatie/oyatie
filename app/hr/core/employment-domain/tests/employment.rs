#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use hr_employment_domain::{
    Employee, EmployeeCreate, EmploymentStatus, HrDomainError, HrLifecycleKind, TenantTierSnapshot,
};
use oya_data_boundary_kernel::{DataClass, PrivacyDataClass};

#[test]
fn test_employment_requires_entity_and_audit() {
    let employee = Employee::new(employee_input()).expect("valid employee");

    assert_eq!(employee.tenant_id.value.value, "ten_acme");
    assert_eq!(employee.legal_entity_id.value.value, "le_kr_001");
    assert_eq!(employee.person_ref.value.value, "person/acme/001");

    let mut missing_legal_entity = employee_input();
    missing_legal_entity.legal_entity_id.clear();
    assert_eq!(
        Employee::new(missing_legal_entity),
        Err(HrDomainError::InvalidLegalEntityId)
    );

    let mut prefix_only_tenant = employee_input();
    prefix_only_tenant.tenant_id = "ten_".to_owned();
    assert_eq!(
        Employee::new(prefix_only_tenant),
        Err(HrDomainError::InvalidTenantId)
    );

    let mut traversal_entity = employee_input();
    traversal_entity.legal_entity_id = "le_../tenant".to_owned();
    assert_eq!(
        Employee::new(traversal_entity),
        Err(HrDomainError::InvalidLegalEntityId)
    );

    let mut token_shaped_evidence = employee_input();
    token_shaped_evidence.audit_evidence_ref = "audit/bearer-token".to_owned();
    assert_eq!(
        Employee::new(token_shaped_evidence),
        Err(HrDomainError::InvalidAuditEvidenceRef)
    );

    let mut unsafe_evidence_path = employee_input();
    unsafe_evidence_path.audit_evidence_ref = "audit/hr/../employee".to_owned();
    assert_eq!(
        Employee::new(unsafe_evidence_path),
        Err(HrDomainError::InvalidAuditEvidenceRef)
    );

    let mut public_record_class = employee_input();
    public_record_class.data_class =
        Some(PrivacyDataClass::new(DataClass::Public).expect("public is a privacy-program class"));
    assert_eq!(
        Employee::new(public_record_class),
        Err(HrDomainError::InvalidDataClass)
    );
}

#[test]
fn test_employee_lifecycle_event_is_tenant_scoped_and_idempotent() {
    let employee = Employee::new(employee_input()).expect("valid employee");
    let event = employee
        .lifecycle_event("hrev_employee_created_001", HrLifecycleKind::Created)
        .expect("event");

    assert_eq!(event.event_id.value.value, "hrev_employee_created_001");
    assert_eq!(event.tenant_id.value.value, "ten_acme");
    assert_eq!(event.legal_entity_id.value.value, "le_kr_001");
    assert_eq!(event.idempotency_key.value, "emp_001:1");

    assert_eq!(
        employee.lifecycle_event("audit/not-an-event", HrLifecycleKind::Updated),
        Err(HrDomainError::InvalidHrEventId)
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
