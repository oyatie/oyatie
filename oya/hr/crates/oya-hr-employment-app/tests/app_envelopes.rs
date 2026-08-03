#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_data_boundary_kernel::DataClass;
use oya_hr_employment_app::{
    HrWorkflowExecutionAdapterError, HrWorkflowExecutionScope, OnboardEmployeeCommand,
    onboard_employee, plan_hr_workflow_execution_start, plan_labor_compliance_workflows,
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

#[test]
fn workflow_execution_adapter_maps_hr_dispatch_to_workflow_start_input() {
    let outcome =
        plan_labor_compliance_workflows(snapshot_with_count(30)).expect("workflow plan outcome");
    let dispatch = outcome
        .workflow_dispatches
        .iter()
        .find(|dispatch| {
            dispatch.obligation_kind.value == LaborComplianceObligationKind::KoreaRulesOfEmployment
        })
        .expect("rules-of-employment dispatch");

    let start_input = plan_hr_workflow_execution_start(dispatch, workflow_scope())
        .expect("HR dispatch should plan Workflow start input");

    assert_eq!(
        start_input.request_id,
        "req:hr-compliance:ten_acme:le_kr_001:korea_rules_of_employment"
    );
    assert_eq!(start_input.idempotency_key, dispatch.idempotency_key.value);
    assert_eq!(start_input.trace_ref, "trace:hr-compliance:kr-threshold");
    assert_eq!(
        format!("{:?}", start_input.domain_request.command),
        "StartRun"
    );
    assert_eq!(start_input.domain_request.expected_tenant_id, "ten_acme");
    assert_eq!(
        start_input.domain_request.expected_spec_id,
        "workflow-ref:workflow/hr-compliance/kr"
    );
    assert_eq!(
        start_input.domain_request.expected_version_sha,
        "workflow-version:hr-compliance-kr:v1"
    );
    assert_eq!(start_input.domain_request.expected_cell_id, "cell:hr:kr");
    assert_eq!(
        start_input.domain_request.policy_evidence_ref,
        "cedar://workflow/hr-compliance/start"
    );
    assert_eq!(start_input.domain_request.run.tenant_id, "ten_acme");
    assert_eq!(
        start_input.domain_request.run.run_id,
        "hr-workflow-run:ten_acme:le_kr_001:korea_rules_of_employment"
    );
    assert!(
        start_input
            .domain_request
            .run
            .evidence_refs
            .contains(&"workflow-execution-hr-intake:legal-entity:le_kr_001".to_owned())
    );
    assert!(start_input.domain_request.run.evidence_refs.contains(
        &"workflow-execution-hr-intake:evidence:audit/hr/compliance/kr-threshold".to_owned()
    ));

    let first_step = start_input
        .domain_request
        .step
        .expect("Workflow start input should include first HR step");
    assert_eq!(first_step.step_id, "hr-workflow-step:drafted");
    assert_eq!(
        first_step.idempotency_key,
        "ten_acme:le_kr_001:korea_rules_of_employment:2026-01-01:step:drafted"
    );
}

#[test]
fn workflow_execution_adapter_rejects_missing_audit_and_scope_evidence_without_echo() {
    let outcome =
        plan_labor_compliance_workflows(snapshot_with_count(30)).expect("workflow plan outcome");
    let dispatch = outcome
        .workflow_dispatches
        .iter()
        .find(|dispatch| {
            dispatch.obligation_kind.value == LaborComplianceObligationKind::KoreaRulesOfEmployment
        })
        .expect("rules-of-employment dispatch");

    let mut missing_audit = workflow_scope();
    missing_audit.audit_refs.clear();
    let error = plan_hr_workflow_execution_start(dispatch, missing_audit).unwrap_err();
    assert_eq!(error, HrWorkflowExecutionAdapterError::MissingAuditRefs);

    let mut missing_scope = workflow_scope();
    missing_scope.policy_evidence_ref.clear();
    let error = plan_hr_workflow_execution_start(dispatch, missing_scope).unwrap_err();
    assert_eq!(error, HrWorkflowExecutionAdapterError::MissingScopeEvidence);

    let mut unsafe_scope = workflow_scope();
    unsafe_scope.trace_ref = "Authorization: Bearer *** raw prompt".to_owned();
    let error = plan_hr_workflow_execution_start(dispatch, unsafe_scope).unwrap_err();
    assert_eq!(
        error,
        HrWorkflowExecutionAdapterError::WorkflowIntakeRejected
    );
    let rendered = format!("{error:?}").to_ascii_lowercase();
    assert!(!rendered.contains("authorization"));
    assert!(!rendered.contains("bearer"));
    assert!(!rendered.contains("raw prompt"));
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

fn workflow_scope() -> HrWorkflowExecutionScope {
    HrWorkflowExecutionScope {
        audit_refs: vec!["audit-chain:hr-compliance-workflow-start".to_owned()],
        cell_id: "cell:hr:kr".to_owned(),
        workflow_version_ref: "workflow-version:hr-compliance-kr:v1".to_owned(),
        policy_evidence_ref: "cedar://workflow/hr-compliance/start".to_owned(),
        spec_integrity_ref: "spec-integrity:workflow:hr-compliance-kr".to_owned(),
        replay_epoch_ref: "replay-epoch:hr-compliance:2026-01-01".to_owned(),
        scheduler_epoch_ref: "scheduler-epoch:hr-compliance:kr-threshold".to_owned(),
        trace_ref: "trace:hr-compliance:kr-threshold".to_owned(),
    }
}
