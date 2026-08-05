use oya_workflow_engine_execution_engine_usecase::{
    ExecutionDomainCommandKind, ExecutionDomainOrigin, HrLaborComplianceWorkflowIntake,
    HrWorkflowIntakeError, WorkflowExecutionStatus, plan_hr_labor_compliance_workflow_start,
};

#[test]
fn hr_labor_compliance_intake_maps_metadata_envelope_to_start_run_input() {
    let planned = plan_hr_labor_compliance_workflow_start(valid_intake())
        .expect("HR labor-compliance intake should plan a Workflow start input");

    assert_eq!(
        planned.request_id,
        "req:hr-compliance:ten_acme:le_kr_001:korea_rules_of_employment"
    );
    assert_eq!(
        planned.idempotency_key,
        "ten_acme:le_kr_001:korea_rules_of_employment:2026-01-01"
    );
    assert_eq!(planned.trace_ref, "trace:hr-compliance:kr-threshold");
    assert_eq!(planned.expected_run_version, 1);

    let request = planned.domain_request;
    assert_eq!(request.command, ExecutionDomainCommandKind::StartRun);
    assert_eq!(request.origin, ExecutionDomainOrigin::ApiCommand);
    assert_eq!(request.expected_tenant_id, "ten_acme");
    assert_eq!(
        request.expected_spec_id,
        "workflow-ref:workflow/hr-compliance/kr"
    );
    assert_eq!(
        request.expected_version_sha,
        "workflow-version:hr-compliance-kr:v1"
    );
    assert_eq!(request.expected_cell_id, "cell:hr:kr");
    assert_eq!(
        request.policy_evidence_ref,
        "cedar://workflow/hr-compliance/start"
    );
    assert_eq!(
        request.spec_integrity_ref,
        "spec-integrity:workflow:hr-compliance-kr"
    );
    assert_eq!(
        request.replay_epoch_ref,
        "replay-epoch:hr-compliance:2026-01-01"
    );
    assert_eq!(
        request.scheduler_epoch_ref,
        "scheduler-epoch:hr-compliance:kr-threshold"
    );

    assert_eq!(request.run.tenant_id, "ten_acme");
    assert_eq!(
        request.run.run_id,
        "hr-workflow-run:ten_acme:le_kr_001:korea_rules_of_employment"
    );
    assert_eq!(
        request.run.spec_id,
        "workflow-ref:workflow/hr-compliance/kr"
    );
    assert_eq!(
        request.run.version_sha,
        "workflow-version:hr-compliance-kr:v1"
    );
    assert_eq!(request.run.status, WorkflowExecutionStatus::Pending);
    assert!(
        request
            .run
            .evidence_refs
            .contains(&"audit-chain:hr-compliance-workflow-start".to_owned())
    );
    assert!(
        request
            .run
            .evidence_refs
            .contains(&"workflow-execution-hr-intake:legal-entity:le_kr_001".to_owned())
    );
    assert!(request.run.evidence_refs.contains(
        &"workflow-execution-hr-intake:workflow-ref:workflow/hr-compliance/kr".to_owned()
    ));
    assert!(request.run.evidence_refs.contains(
        &"workflow-execution-hr-intake:evidence:audit/hr/compliance/kr-threshold".to_owned()
    ));
    assert!(request.run.evidence_refs.contains(&"workflow-execution-hr-intake:evidence:audit/le_kr_001/moel/rules-of-employment/report".to_owned()));

    let step = request
        .step
        .expect("start-run request should include first step");
    assert_eq!(step.tenant_id, "ten_acme");
    assert_eq!(
        step.run_id,
        "hr-workflow-run:ten_acme:le_kr_001:korea_rules_of_employment"
    );
    assert_eq!(step.step_id, "hr-workflow-step:drafted");
    assert_eq!(step.step_index, 0);
    assert_eq!(
        step.idempotency_key,
        "ten_acme:le_kr_001:korea_rules_of_employment:2026-01-01:step:drafted"
    );
    assert!(
        step.evidence_refs
            .contains(&"workflow-execution-hr-intake:required-step:drafted".to_owned())
    );
}

#[test]
fn hr_labor_compliance_intake_rejects_missing_required_steps() {
    let mut intake = valid_intake();
    intake.required_steps.clear();

    let error = plan_hr_labor_compliance_workflow_start(intake).unwrap_err();

    assert_eq!(error, HrWorkflowIntakeError::MissingRequiredSteps);
}

#[test]
fn hr_labor_compliance_intake_rejects_missing_evidence_refs() {
    let mut intake = valid_intake();
    intake.evidence_refs.clear();

    let error = plan_hr_labor_compliance_workflow_start(intake).unwrap_err();

    assert_eq!(error, HrWorkflowIntakeError::MissingEvidenceRefs);
}

#[test]
fn hr_labor_compliance_intake_rejects_missing_audit_refs() {
    let mut intake = valid_intake();
    intake.audit_refs.clear();

    let error = plan_hr_labor_compliance_workflow_start(intake).unwrap_err();

    assert_eq!(error, HrWorkflowIntakeError::MissingAuditRefs);
}

#[test]
fn hr_labor_compliance_intake_rejects_unsafe_metadata_without_echo() {
    let mut intake = valid_intake();
    intake.idempotency_key = "Authorization: Bearer *** raw prompt".to_owned();

    let error = plan_hr_labor_compliance_workflow_start(intake).unwrap_err();

    assert_eq!(error, HrWorkflowIntakeError::UnsafeMetadata);
    let rendered = format!("{error:?}").to_ascii_lowercase();
    assert!(!rendered.contains("authorization"));
    assert!(!rendered.contains("bearer"));
    assert!(!rendered.contains("raw prompt"));
}

#[test]
fn hr_labor_compliance_intake_rejects_unsafe_source_refs_without_echo() {
    let mut traversal = valid_intake();
    traversal.workflow_ref = "workflow/hr/../compliance".to_owned();

    let traversal_error = plan_hr_labor_compliance_workflow_start(traversal).unwrap_err();

    assert_eq!(traversal_error, HrWorkflowIntakeError::UnsafeMetadata);
    let rendered = format!("{traversal_error:?}").to_ascii_lowercase();
    assert!(!rendered.contains("workflow/hr"));

    let mut credential_ref = valid_intake();
    credential_ref.evidence_refs = vec!["audit/hr/compliance/secret-token".to_owned()];

    let credential_error = plan_hr_labor_compliance_workflow_start(credential_ref).unwrap_err();

    assert_eq!(credential_error, HrWorkflowIntakeError::UnsafeMetadata);
    let rendered = format!("{credential_error:?}").to_ascii_lowercase();
    assert!(!rendered.contains("secret-token"));
}

fn valid_intake() -> HrLaborComplianceWorkflowIntake {
    HrLaborComplianceWorkflowIntake {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        workflow_ref: "workflow/hr-compliance/kr".to_owned(),
        obligation_kind: "korea_rules_of_employment".to_owned(),
        required_steps: vec![
            "drafted".to_owned(),
            "employee-review-sent".to_owned(),
            "majority-consent-obtained".to_owned(),
            "moel-filed".to_owned(),
            "active".to_owned(),
        ],
        evidence_refs: vec![
            "audit/hr/compliance/kr-threshold".to_owned(),
            "audit/le_kr_001/moel/rules-of-employment/report".to_owned(),
        ],
        idempotency_key: "ten_acme:le_kr_001:korea_rules_of_employment:2026-01-01".to_owned(),
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
