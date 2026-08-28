#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LeaveDecision {
    Approved,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LeaveRoutingMode {
    DirectManager,
    DelegatedApprover,
    EscalatedHr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PayrollImpactKind {
    PaidLeave,
    UnpaidLeaveDeduction,
    AttendanceCorrection,
    NoPayrollImpact,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeavePayrollImpactInput {
    pub leave_request_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                      // data_class: INTERNAL_ONLY
    pub legal_entity_id: String,                // data_class: INTERNAL_ONLY
    pub employee_id: String,                    // data_class: INTERNAL_ONLY
    pub approver_id: String,                    // data_class: INTERNAL_ONLY
    pub decision: LeaveDecision,                // data_class: INTERNAL_ONLY
    pub routing_mode: LeaveRoutingMode,         // data_class: INTERNAL_ONLY
    pub start_date: String,                     // data_class: INTERNAL_ONLY
    pub end_date: String,                       // data_class: INTERNAL_ONLY
    pub payroll_period: String,                 // data_class: FINANCIAL
    pub payroll_impact_kind: PayrollImpactKind, // data_class: FINANCIAL
    pub workflow_ref: String,                   // data_class: INTERNAL_ONLY
    pub rulepack_ref: String,                   // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: String,        // data_class: INTERNAL_ONLY
    pub decision_evidence_ref: String,          // data_class: INTERNAL_ONLY
    pub routing_evidence_ref: String,           // data_class: INTERNAL_ONLY
    pub payroll_impact_evidence_ref: String,    // data_class: FINANCIAL
    pub decided_at_epoch_seconds: u64,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeavePayrollImpactPlan {
    pub leave_request_id: Classified<LeaveRequestId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<TenantId>,              // data_class: INTERNAL_ONLY
    pub legal_entity_id: Classified<LegalEntityId>,   // data_class: INTERNAL_ONLY
    pub employee_id: Classified<EmployeeId>,          // data_class: INTERNAL_ONLY
    pub approver_id: Classified<EmployeeId>,          // data_class: INTERNAL_ONLY
    pub decision: Classified<LeaveDecision>,          // data_class: INTERNAL_ONLY
    pub routing_mode: Classified<LeaveRoutingMode>,   // data_class: INTERNAL_ONLY
    pub start_date: Classified<String>,               // data_class: INTERNAL_ONLY
    pub end_date: Classified<String>,                 // data_class: INTERNAL_ONLY
    pub payroll_period: Classified<String>,           // data_class: FINANCIAL
    pub payroll_impact_kind: Classified<PayrollImpactKind>, // data_class: FINANCIAL
    pub workflow_ref: Classified<WorkflowRef>,        // data_class: INTERNAL_ONLY
    pub rulepack_ref: Classified<RulepackRef>,        // data_class: INTERNAL_ONLY
    pub rulepack_effective_date: Classified<RulepackEffectiveDate>, // data_class: INTERNAL_ONLY
    pub decision_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub routing_evidence_ref: Classified<AuditEvidenceRef>, // data_class: INTERNAL_ONLY
    pub payroll_impact_evidence_ref: Classified<AuditEvidenceRef>, // data_class: FINANCIAL
    pub idempotency_key: Classified<String>,          // data_class: INTERNAL_ONLY
    pub decided_at_epoch_seconds: Classified<u64>,    // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,              // data_class: PUBLIC
}

pub fn plan_leave_payroll_impact(
    input: LeavePayrollImpactInput,
) -> Result<LeavePayrollImpactPlan, HrDomainError> {
    validate_identifier(
        &input.leave_request_id,
        LEAVE_REQUEST_ID_PREFIX,
        HrDomainError::InvalidLeaveRequestId,
    )?;
    validate_identifier(
        &input.tenant_id,
        TENANT_ID_PREFIX,
        HrDomainError::InvalidTenantId,
    )?;
    validate_identifier(
        &input.legal_entity_id,
        LEGAL_ENTITY_ID_PREFIX,
        HrDomainError::InvalidLegalEntityId,
    )?;
    validate_identifier(
        &input.employee_id,
        EMPLOYEE_ID_PREFIX,
        HrDomainError::InvalidEmployeeId,
    )?;
    validate_identifier(
        &input.approver_id,
        EMPLOYEE_ID_PREFIX,
        HrDomainError::InvalidApproverId,
    )?;
    validate_leave_dates(&input.start_date, &input.end_date)?;
    validate_payroll_period(&input.payroll_period)?;
    validate_ref(
        &input.workflow_ref,
        WORKFLOW_REF_PREFIX,
        HrDomainError::InvalidWorkflowRef,
    )?;
    validate_ref(
        &input.rulepack_ref,
        RULEPACK_REF_PREFIX,
        HrDomainError::InvalidRulepackRef,
    )?;
    validate_iso_date(&input.rulepack_effective_date)?;
    validate_evidence_ref(&input.decision_evidence_ref)?;
    validate_evidence_ref(&input.routing_evidence_ref)?;
    validate_evidence_ref(&input.payroll_impact_evidence_ref)?;
    if input.decided_at_epoch_seconds == 0 {
        return Err(HrDomainError::InvalidDecisionTimestamp);
    }

    let idempotency_key = format!(
        "{}:{}:{:?}:{}",
        input.tenant_id, input.leave_request_id, input.decision, input.payroll_period
    );

    Ok(LeavePayrollImpactPlan {
        leave_request_id: internal(LeaveRequestId {
            value: input.leave_request_id,
        }),
        tenant_id: internal(TenantId {
            value: input.tenant_id,
        }),
        legal_entity_id: internal(LegalEntityId {
            value: input.legal_entity_id,
        }),
        employee_id: internal(EmployeeId {
            value: input.employee_id,
        }),
        approver_id: internal(EmployeeId {
            value: input.approver_id,
        }),
        decision: internal(input.decision),
        routing_mode: internal(input.routing_mode),
        start_date: internal(input.start_date),
        end_date: internal(input.end_date),
        payroll_period: Classified::new(input.payroll_period, DataClass::Financial),
        payroll_impact_kind: Classified::new(input.payroll_impact_kind, DataClass::Financial),
        workflow_ref: internal(WorkflowRef {
            value: input.workflow_ref,
        }),
        rulepack_ref: internal(RulepackRef {
            value: input.rulepack_ref,
        }),
        rulepack_effective_date: internal(RulepackEffectiveDate {
            value: input.rulepack_effective_date,
        }),
        decision_evidence_ref: internal(AuditEvidenceRef {
            value: input.decision_evidence_ref,
        }),
        routing_evidence_ref: internal(AuditEvidenceRef {
            value: input.routing_evidence_ref,
        }),
        payroll_impact_evidence_ref: Classified::new(
            AuditEvidenceRef {
                value: input.payroll_impact_evidence_ref,
            },
            DataClass::Financial,
        ),
        idempotency_key: internal(idempotency_key),
        decided_at_epoch_seconds: internal(input.decided_at_epoch_seconds),
        schema_version: public(LEAVE_PAYROLL_IMPACT_SCHEMA_VERSION),
    })
}
