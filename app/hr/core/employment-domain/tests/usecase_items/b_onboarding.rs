mod onboarding_usecase_contract {
    use data_boundary_kernel::DataClass;
    use hr_employment_domain::{
        EmployeeCreate, EmploymentStatus, HrAppError, HrDomainError, HrLifecycleKind,
        OnboardEmployeeCommand, TenantTierSnapshot, onboard_employee,
    };

    #[test]
    fn onboarding_preserves_employee_lifecycle_and_metadata_only_audit_values() {
        // Catches dropped lifecycle metadata or a PII audit payload classification change.
        let outcome = onboard_employee(OnboardEmployeeCommand {
            employee: EmployeeCreate {
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
            },
            event_id: "hrev_employee_created_001".to_owned(),
            lifecycle_kind: HrLifecycleKind::Created,
        })
        .expect("literal onboarding command is accepted");

        assert_eq!(outcome.employee.employee_id.value.value, "emp_001");
        assert_eq!(outcome.employee.tenant_id.value.value, "ten_acme");
        assert_eq!(outcome.employee.legal_entity_id.value.value, "le_kr_001");
        assert_eq!(outcome.employee.person_ref.value.value, "person/acme/001");
        assert_eq!(
            outcome
                .employee
                .manager_id
                .value
                .expect("manager is retained")
                .value,
            "emp_mgr_001"
        );
        assert_eq!(
            outcome.employee.employment_status.value,
            EmploymentStatus::Active
        );
        assert_eq!(
            outcome.employee.tenant_tier_snapshot.value,
            TenantTierSnapshot::EnterpriseGroup
        );
        assert_eq!(
            outcome.employee.audit_evidence_ref.value.value,
            "audit/hr/employee/001"
        );
        assert_eq!(outcome.employee.version.value, 1);
        assert_eq!(outcome.employee.schema_version.value, 1);
        assert_eq!(
            outcome
                .employee
                .schema_version
                .data_class
                .compatibility_data_class(),
            DataClass::Public
        );
        assert_eq!(
            outcome.lifecycle_event.event_id.value.value,
            "hrev_employee_created_001"
        );
        assert_eq!(outcome.lifecycle_event.tenant_id.value.value, "ten_acme");
        assert_eq!(
            outcome.lifecycle_event.legal_entity_id.value.value,
            "le_kr_001"
        );
        assert_eq!(outcome.lifecycle_event.employee_id.value.value, "emp_001");
        assert_eq!(
            outcome.lifecycle_event.lifecycle_kind.value,
            HrLifecycleKind::Created
        );
        assert_eq!(
            outcome.lifecycle_event.audit_evidence_ref.value.value,
            "audit/hr/employee/001"
        );
        assert_eq!(outcome.lifecycle_event.idempotency_key.value, "emp_001:1");
        assert_eq!(outcome.lifecycle_event.schema_version.value, 1);
        assert_eq!(
            outcome
                .lifecycle_event
                .schema_version
                .data_class
                .compatibility_data_class(),
            DataClass::Public
        );
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
        assert_eq!(outcome.audit_envelope.payload_kind.value, "Created");
        assert_eq!(outcome.audit_envelope.idempotency_key.value, "emp_001:1");
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
        assert_eq!(outcome.audit_envelope.schema_version.value, 1);
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
    fn onboarding_returns_the_domain_error_for_a_serialized_boundary_employee_id() {
        // Catches a boundary-validation error being erased or translated incorrectly.
        let error = onboard_employee(OnboardEmployeeCommand {
            employee: EmployeeCreate {
                employee_id: "emp_../001".to_owned(),
                tenant_id: "ten_acme".to_owned(),
                legal_entity_id: "le_kr_001".to_owned(),
                person_ref: "person/acme/001".to_owned(),
                manager_id: None,
                employment_status: EmploymentStatus::Draft,
                tenant_tier_snapshot: TenantTierSnapshot::SmbSelfServe,
                audit_evidence_ref: "audit/hr/employee/001".to_owned(),
                data_class: None,
                version: 1,
            },
            event_id: "hrev_employee_created_001".to_owned(),
            lifecycle_kind: HrLifecycleKind::Created,
        })
        .expect_err("serialized-boundary employee id is rejected");

        assert_eq!(error, HrAppError::Domain(HrDomainError::InvalidEmployeeId));
    }
}
