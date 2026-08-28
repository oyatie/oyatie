mod compliance_usecase_contract {
    use data_boundary_kernel::DataClass;
    use hr_employment_domain::{
        HrAppError, HrDomainError, Jurisdiction, LaborComplianceObligationKind,
        LegalEntityWorkforceSnapshot, plan_labor_compliance_workflows,
    };

    #[test]
    fn korea_thirty_employee_snapshot_emits_two_matching_workflow_dispatches() {
        // Catches threshold dispatch loss or dispatch metadata drifting from the obligation.
        let outcome = plan_labor_compliance_workflows(LegalEntityWorkforceSnapshot {
            tenant_id: "ten_acme".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            jurisdiction: Jurisdiction::Korea,
            active_employee_count: 30,
            rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
            rulepack_effective_date: "2026-01-01".to_owned(),
            workflow_ref: "workflow/hr-compliance/kr".to_owned(),
            evidence_ref: "audit/hr/compliance/kr-threshold".to_owned(),
            evaluated_at_epoch_seconds: 1_779_519_600,
        })
        .expect("Korea workforce snapshot is accepted");

        assert_eq!(outcome.obligations.len(), 2);
        assert_eq!(outcome.workflow_dispatches.len(), 2);
        let dispatch = outcome
            .workflow_dispatches
            .iter()
            .find(|item| {
                item.obligation_kind.value == LaborComplianceObligationKind::KoreaRulesOfEmployment
            })
            .expect("rules-of-employment dispatch is present");
        assert_eq!(dispatch.topic.value, "workflow.hr.compliance.dispatch");
        assert_eq!(dispatch.tenant_id.value.value, "ten_acme");
        assert_eq!(dispatch.legal_entity_id.value.value, "le_kr_001");
        assert_eq!(
            dispatch.workflow_ref.value.value,
            "workflow/hr-compliance/kr"
        );
        assert_eq!(
            dispatch.evidence_refs.value[0].value,
            "audit/hr/compliance/kr-threshold"
        );
        assert_eq!(
            dispatch.idempotency_key.value,
            "ten_acme:le_kr_001:korea_rules_of_employment:2026-01-01"
        );
        assert_eq!(
            dispatch
                .schema_version
                .data_class
                .compatibility_data_class(),
            DataClass::Public
        );
    }

    #[test]
    fn compliance_returns_the_domain_error_for_an_invalid_snapshot() {
        // Catches invalid snapshot errors being hidden behind a use-case-specific error.
        let error = plan_labor_compliance_workflows(LegalEntityWorkforceSnapshot {
            tenant_id: "ten_acme".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            jurisdiction: Jurisdiction::Korea,
            active_employee_count: 30,
            rulepack_ref: "policy/not-a-rulepack".to_owned(),
            rulepack_effective_date: "2026-01-01".to_owned(),
            workflow_ref: "workflow/hr-compliance/kr".to_owned(),
            evidence_ref: "audit/hr/compliance/kr-threshold".to_owned(),
            evaluated_at_epoch_seconds: 1_779_519_600,
        })
        .expect_err("invalid rulepack reference is rejected");

        assert_eq!(error, HrAppError::Domain(HrDomainError::InvalidRulepackRef));
    }
}
