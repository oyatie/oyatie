use oya_hr_employment_domain::{
    HrDomainError, HrStatutoryFilingAuditEventClass, HrStatutoryFilingEvidenceReceiptStatus,
    HrStatutoryFilingKind, HrStatutoryFilingManifestInput, HrStatutoryFilingRollbackAction,
    Jurisdiction, prepare_hr_statutory_filing_manifest,
};

fn digest(ch: char) -> String {
    format!("sha256:{}", ch.to_string().repeat(64))
}

fn valid_filing_manifest_input() -> HrStatutoryFilingManifestInput {
    HrStatutoryFilingManifestInput {
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        jurisdiction: Jurisdiction::Korea,
        filing_kind: HrStatutoryFilingKind::KoreaRulesOfEmploymentReport,
        workflow_ref: "workflow/hr-compliance/kr".to_owned(),
        workflow_run_ref: "workflow-run/hr-compliance/ten_acme/le_kr_001/rules-of-employment"
            .to_owned(),
        rulepack_ref: "rulepack/hr/korea/2026".to_owned(),
        rulepack_effective_date: "2026-01-01".to_owned(),
        source_manifest_digest: digest('f'),
        source_evidence_refs: vec![
            "audit/hr-rulepack/korea/labor-standards-act".to_owned(),
            "audit/hr-rulepack/korea/rules-of-employment".to_owned(),
            "audit/hr-rulepack/korea/labor-management-council".to_owned(),
        ],
        workflow_evidence_refs: vec![
            "audit/hr/compliance/kr-threshold".to_owned(),
            "audit/le_kr_001/moel/rules-of-employment/report".to_owned(),
        ],
        filing_authority_ref: "filing-authority/kr/moel/rules-of-employment".to_owned(),
        audit_event_class: HrStatutoryFilingAuditEventClass::ManifestPrepared,
        rollback_evidence_ref: "audit/hr-filing/kr/rules-of-employment/rollback-plan".to_owned(),
        filing_window_start_date: "2026-01-01".to_owned(),
        filing_window_end_date: "2026-03-31".to_owned(),
        prepared_at_epoch_seconds: 1_779_544_204,
        production_filing_transport_attached: false,
        government_submission_attached: false,
        legal_certification_claimed: false,
        payroll_calculation_attached: false,
        runtime_audit_emission_attached: false,
        cloud_deployment_attached: false,
    }
}

#[test]
fn hr_statutory_filing_manifest_prepares_review_packet_without_transport_or_certification() {
    let manifest = prepare_hr_statutory_filing_manifest(valid_filing_manifest_input())
        .expect("valid statutory filing manifest should prepare review metadata");

    assert_eq!(manifest.tenant_id.value.value, "ten_acme");
    assert_eq!(manifest.legal_entity_id.value.value, "le_kr_001");
    assert_eq!(manifest.jurisdiction.value, Jurisdiction::Korea);
    assert_eq!(
        manifest.filing_kind.value,
        HrStatutoryFilingKind::KoreaRulesOfEmploymentReport
    );
    assert_eq!(
        manifest.workflow_ref.value.value,
        "workflow/hr-compliance/kr"
    );
    assert_eq!(
        manifest.workflow_run_ref.value.value,
        "workflow-run/hr-compliance/ten_acme/le_kr_001/rules-of-employment"
    );
    assert_eq!(manifest.rulepack_ref.value.value, "rulepack/hr/korea/2026");
    assert_eq!(manifest.source_manifest_digest.value.value, digest('f'));
    assert_eq!(manifest.source_evidence_count.value, 3);
    assert_eq!(manifest.workflow_evidence_count.value, 2);
    assert_eq!(
        manifest.evidence_receipt_status.value,
        HrStatutoryFilingEvidenceReceiptStatus::AcceptedForReview
    );
    assert_eq!(
        manifest.audit_event_class.value,
        HrStatutoryFilingAuditEventClass::ManifestPrepared
    );
    assert_eq!(
        manifest.audit_event_class_name.value,
        "HrStatutoryFilingManifestPrepared"
    );
    assert_eq!(
        manifest.rollback_action.value,
        HrStatutoryFilingRollbackAction::QuarantinePreparedManifest
    );
    assert_eq!(
        manifest.rollback_evidence_ref.value.value,
        "audit/hr-filing/kr/rules-of-employment/rollback-plan"
    );
    assert_eq!(
        manifest.idempotency_key.value,
        "ten_acme:le_kr_001:KoreaRulesOfEmploymentReport:rulepack/hr/korea/2026:2026-01-01"
    );
    assert!(!manifest.production_filing_transport_attached.value);
    assert!(!manifest.government_submission_attached.value);
    assert!(!manifest.legal_certification_claimed.value);
    assert!(!manifest.payroll_calculation_attached.value);
    assert!(!manifest.runtime_audit_emission_attached.value);
    assert!(!manifest.cloud_deployment_attached.value);
}

#[test]
fn hr_statutory_filing_manifest_rejects_non_rulepack_source_evidence() {
    let mut manifest_input = valid_filing_manifest_input();
    manifest_input.source_evidence_refs = vec!["audit/hr/compliance/kr-threshold".to_owned()];

    assert_eq!(
        prepare_hr_statutory_filing_manifest(manifest_input),
        Err(HrDomainError::FilingEvidenceRequired),
        "filing manifests must be backed by source-versioned HR rulepack evidence, not workflow-only evidence"
    );
}

#[test]
fn hr_statutory_filing_manifest_rejects_authority_kind_mismatch() {
    let mut manifest_input = valid_filing_manifest_input();
    manifest_input.filing_kind = HrStatutoryFilingKind::KoreaLaborManagementCouncilMinutes;

    assert_eq!(
        prepare_hr_statutory_filing_manifest(manifest_input),
        Err(HrDomainError::InvalidFilingAuthorityRef),
        "rules-of-employment authority refs must not authorize a labor-management council filing manifest"
    );
}

#[test]
fn hr_statutory_filing_manifest_rejects_non_korea_jurisdiction_for_korea_filing_kind() {
    let mut manifest_input = valid_filing_manifest_input();
    manifest_input.jurisdiction = Jurisdiction::UnitedStates;

    assert_eq!(
        prepare_hr_statutory_filing_manifest(manifest_input),
        Err(HrDomainError::InvalidFilingAuthorityRef),
        "Korea statutory filing kinds must not be prepared under a non-Korea jurisdiction"
    );
}
