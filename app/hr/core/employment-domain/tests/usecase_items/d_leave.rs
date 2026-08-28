mod leave_usecase_contract {
    use data_boundary_kernel::DataClass;
    use hr_employment_domain::{
        HrAppError, HrDomainError, LeaveDecision, LeavePayrollImpactInput, LeaveRoutingMode,
        PayrollImpactKind, plan_leave_payroll_impact_envelope,
    };

    #[test]
    fn accepted_leave_emits_the_financial_metadata_only_payroll_envelope() {
        // Catches payroll routing, evidence, or financial classification being omitted.
        let outcome = plan_leave_payroll_impact_envelope(LeavePayrollImpactInput {
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
        })
        .expect("literal accepted leave input is valid");

        let envelope = outcome.payroll_impact_envelope;
        assert_eq!(envelope.topic.value, "integration.hr.payroll.leave-impact");
        assert_eq!(envelope.tenant_id.value.value, "ten_acme");
        assert_eq!(envelope.legal_entity_id.value.value, "le_kr_001");
        assert_eq!(envelope.employee_id.value.value, "emp_001");
        assert_eq!(envelope.leave_request_id.value.value, "leave_001");
        assert_eq!(envelope.decision.value, LeaveDecision::Approved);
        assert_eq!(envelope.routing_mode.value, LeaveRoutingMode::EscalatedHr);
        assert_eq!(envelope.payroll_period.value, "2026-06");
        assert_eq!(
            envelope.payroll_impact_kind.value,
            PayrollImpactKind::UnpaidLeaveDeduction
        );
        assert_eq!(
            envelope.payroll_impact_evidence_ref.value.value,
            "audit/hr/leave/leave_001/payroll-impact"
        );
        assert_eq!(envelope.payload_data_class.value, DataClass::Financial);
        assert_eq!(envelope.schema_version.value, 1);
    }

    #[test]
    fn leave_returns_the_domain_error_for_a_malformed_payroll_period() {
        // Catches malformed leave input being accepted or remapped by the use case.
        let error = plan_leave_payroll_impact_envelope(LeavePayrollImpactInput {
            leave_request_id: "leave_001".to_owned(),
            tenant_id: "ten_acme".to_owned(),
            legal_entity_id: "le_kr_001".to_owned(),
            employee_id: "emp_001".to_owned(),
            approver_id: "emp_hr_001".to_owned(),
            decision: LeaveDecision::Approved,
            routing_mode: LeaveRoutingMode::EscalatedHr,
            start_date: "2026-06-01".to_owned(),
            end_date: "2026-06-03".to_owned(),
            payroll_period: "2026-99".to_owned(),
            payroll_impact_kind: PayrollImpactKind::UnpaidLeaveDeduction,
            workflow_ref: "workflow/hr-leave/kr".to_owned(),
            rulepack_ref: "rulepack/kr-labor-2026".to_owned(),
            rulepack_effective_date: "2026-01-01".to_owned(),
            decision_evidence_ref: "audit/hr/leave/leave_001/decision".to_owned(),
            routing_evidence_ref: "audit/hr/leave/leave_001/escalation".to_owned(),
            payroll_impact_evidence_ref: "audit/hr/leave/leave_001/payroll-impact".to_owned(),
            decided_at_epoch_seconds: 1_779_532_800,
        })
        .expect_err("malformed payroll period is rejected");

        assert_eq!(
            error,
            HrAppError::Domain(HrDomainError::InvalidPayrollPeriod)
        );
    }
}
