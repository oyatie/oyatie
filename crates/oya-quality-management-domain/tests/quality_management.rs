use oya_quality_management_domain::{
    CertificateState, InspectionLotResultInput, InspectionPlanInput, InspectionPlanState,
    InspectionPlanType, QualityCertificateInput, QualityManagementError, QualityNotificationInput,
    QualityNotificationState, QualitySeverity, UsageDecisionState, approve_inspection_plan,
    open_quality_notification, prepare_quality_certificate, record_inspection_lot_result,
};

fn inspection_plan_input() -> InspectionPlanInput {
    InspectionPlanInput {
        inspection_plan_id: "qplan_laptop_final_acceptance".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        plant_id: "plant_us001".to_owned(),
        item_id: "item_laptop_finished".to_owned(),
        plan_type: InspectionPlanType::WorkInProcess,
        characteristic_count: 8,
        sample_size: 100,
        acceptable_quality_limit_basis_points: 200,
        effective_from_yyyymmdd: 20260523,
        specification_source_ref: "src/quality-management/spec/laptop-final-acceptance".to_owned(),
        approval_evidence_ref: "audit/quality-management/qplan_laptop_final_acceptance/approval"
            .to_owned(),
    }
}

fn accepted_lot_input(inspection_plan_approved: bool) -> InspectionLotResultInput {
    InspectionLotResultInput {
        inspection_lot_id: "qlot_laptop_batch_01".to_owned(),
        inspection_plan_id: "qplan_laptop_final_acceptance".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        plant_id: "plant_us001".to_owned(),
        item_id: "item_laptop_finished".to_owned(),
        batch_id: "batch_laptop_2026_06_01".to_owned(),
        source_document_ref: "src/production-planning/prod_laptop_june".to_owned(),
        inspection_evidence_ref: "audit/quality-management/qlot_laptop_batch_01/results".to_owned(),
        inspection_plan_approved,
        inspected_quantity: 100,
        accepted_quantity: 99,
        rejected_quantity: 1,
        defect_count: 1,
        acceptable_quality_limit_basis_points: 200,
    }
}

fn rejected_lot_input() -> InspectionLotResultInput {
    InspectionLotResultInput {
        rejected_quantity: 8,
        accepted_quantity: 92,
        defect_count: 8,
        ..accepted_lot_input(true)
    }
}

fn certificate_input(
    usage_decision_made: bool,
    inspection_passed: bool,
) -> QualityCertificateInput {
    QualityCertificateInput {
        quality_certificate_id: "qcert_laptop_batch_01".to_owned(),
        inspection_lot_id: "qlot_laptop_batch_01".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        plant_id: "plant_us001".to_owned(),
        item_id: "item_laptop_finished".to_owned(),
        batch_id: "batch_laptop_2026_06_01".to_owned(),
        customer_id: "cust_contoso".to_owned(),
        usage_decision_made,
        inspection_passed,
        characteristic_result_count: 8,
        certificate_profile_ref: "src/quality-management/certificate-profile/standard-customer"
            .to_owned(),
        certificate_evidence_ref: "audit/quality-management/qcert_laptop_batch_01/prepared"
            .to_owned(),
    }
}

fn notification_input(rejected_inspection: bool) -> QualityNotificationInput {
    QualityNotificationInput {
        quality_notification_id: "qnot_laptop_batch_01".to_owned(),
        inspection_lot_id: "qlot_laptop_batch_01".to_owned(),
        tenant_id: "ten_enterprise".to_owned(),
        legal_entity_id: "le_us001".to_owned(),
        plant_id: "plant_us001".to_owned(),
        item_id: "item_laptop_finished".to_owned(),
        rejected_inspection,
        severity: QualitySeverity::Major,
        defect_count: 8,
        corrective_action_due_days: 14,
        defect_source_ref: "src/quality-management/defect/laptop-final-acceptance".to_owned(),
        notification_evidence_ref: "audit/quality-management/qnot_laptop_batch_01/opened"
            .to_owned(),
    }
}

#[test]
fn inspection_plan_drives_quality_lot_certificate_and_notification() {
    let plan = approve_inspection_plan(inspection_plan_input()).unwrap();
    assert_eq!(plan.state.value, InspectionPlanState::Approved);
    assert_eq!(plan.characteristic_count.value, 8);
    assert!(!plan.inspection_runtime_attached.value);
    assert!(!plan.inventory_blocking_mutation_attached.value);
    assert!(!plan.workflow_execution_attached.value);

    let accepted_lot = record_inspection_lot_result(accepted_lot_input(true)).unwrap();
    assert_eq!(accepted_lot.state.value, UsageDecisionState::Accepted);
    assert_eq!(accepted_lot.max_rejected_quantity.value, 2);
    assert!(accepted_lot.inspection_passed.value);
    assert!(accepted_lot.certificate_preparation_allowed.value);
    assert!(!accepted_lot.quality_notification_required.value);
    assert!(!accepted_lot.inventory_mutation_attached.value);

    let certificate = prepare_quality_certificate(certificate_input(true, true)).unwrap();
    assert_eq!(certificate.state.value, CertificateState::Prepared);
    assert!(certificate.certificate_ready_for_output.value);
    assert!(!certificate.pdf_rendering_attached.value);
    assert!(!certificate.email_delivery_attached.value);
    assert!(!certificate.cloud_deployment_attached.value);

    let rejected_lot = record_inspection_lot_result(rejected_lot_input()).unwrap();
    assert_eq!(rejected_lot.state.value, UsageDecisionState::Rejected);
    assert!(!rejected_lot.inspection_passed.value);
    assert!(rejected_lot.quality_notification_required.value);

    let notification = open_quality_notification(notification_input(true)).unwrap();
    assert_eq!(notification.state.value, QualityNotificationState::Opened);
    assert!(notification.corrective_action_required.value);
    assert!(!notification.capa_workflow_execution_attached.value);
    assert!(!notification.supplier_collaboration_network_attached.value);
    assert!(!notification.cloud_deployment_attached.value);
}

#[test]
fn quality_management_refuses_unapproved_plan_and_invalid_results() {
    assert_eq!(
        record_inspection_lot_result(accepted_lot_input(false)),
        Err(QualityManagementError::InspectionPlanApprovalRequired)
    );

    let mut invalid_totals = accepted_lot_input(true);
    invalid_totals.accepted_quantity = 98;
    assert_eq!(
        record_inspection_lot_result(invalid_totals),
        Err(QualityManagementError::InvalidInspectionResult)
    );

    let mut invalid_defects = accepted_lot_input(true);
    invalid_defects.defect_count = 0;
    assert_eq!(
        record_inspection_lot_result(invalid_defects),
        Err(QualityManagementError::InvalidInspectionResult)
    );
}

#[test]
fn quality_management_refuses_certificate_and_notification_overclaims() {
    assert_eq!(
        prepare_quality_certificate(certificate_input(false, true)),
        Err(QualityManagementError::UsageDecisionRequired)
    );
    assert_eq!(
        prepare_quality_certificate(certificate_input(true, false)),
        Err(QualityManagementError::CertificateRequiresAcceptedInspection)
    );
    assert_eq!(
        open_quality_notification(notification_input(false)),
        Err(QualityManagementError::RejectedInspectionRequired)
    );
}

#[test]
fn quality_management_validates_refs_dates_and_limits() {
    let mut unsafe_plan = inspection_plan_input();
    unsafe_plan.approval_evidence_ref = "audit/quality-management/secret-token".to_owned();
    assert_eq!(
        approve_inspection_plan(unsafe_plan),
        Err(QualityManagementError::InvalidEvidenceRef)
    );

    let mut bad_date = inspection_plan_input();
    bad_date.effective_from_yyyymmdd = 20260230;
    assert_eq!(
        approve_inspection_plan(bad_date),
        Err(QualityManagementError::InvalidEffectiveDate)
    );

    let mut bad_aql = inspection_plan_input();
    bad_aql.acceptable_quality_limit_basis_points = 0;
    assert_eq!(
        approve_inspection_plan(bad_aql),
        Err(QualityManagementError::InvalidAcceptableQualityLimit)
    );

    let mut bad_ref = accepted_lot_input(true);
    bad_ref.source_document_ref = "src/../production".to_owned();
    assert_eq!(
        record_inspection_lot_result(bad_ref),
        Err(QualityManagementError::InvalidSourceDocumentRef)
    );

    let mut bad_due = notification_input(true);
    bad_due.corrective_action_due_days = 0;
    assert_eq!(
        open_quality_notification(bad_due),
        Err(QualityManagementError::InvalidCorrectiveActionDueDays)
    );
}
