use oya_enterprise_suite_statutory_filing_evidence::{
    EnterpriseStatutoryFilingEvidenceError, StatutoryFilingJurisdiction, StatutoryFilingRailKind,
    enterprise_suite_statutory_filing_evidence_plan, statutory_authority_urls,
    validate_enterprise_suite_statutory_filing_evidence_plan,
};

#[test]
fn statutory_filing_evidence_plan_validates_controls_and_nonclaims() {
    let plan = enterprise_suite_statutory_filing_evidence_plan();
    validate_enterprise_suite_statutory_filing_evidence_plan(&plan)
        .expect("statutory filing evidence plan validates");

    assert_eq!(
        plan.plan_name,
        "enterprise-suite-statutory-filing-evidence-plan"
    );
    assert_eq!(plan.service_name, "enterprise-suite");
    assert_eq!(plan.requirements.len(), 4);
    assert!(plan.source_rulepack_manifests_required);
    assert!(plan.authority_endpoint_registry_required);
    assert!(plan.payload_digest_required);
    assert!(plan.agency_receipt_required);
    assert!(plan.legal_entity_isolation_required);
    assert!(plan.credential_attestation_required);
    assert!(plan.human_approval_required);
    assert!(!plan.manual_submission_workaround_allowed);
    assert!(!plan.runtime_submission_attached);
    assert!(!plan.agency_credential_attached);
    assert!(!plan.agency_connection_attached);
    assert!(!plan.filing_rail_runtime_attached);
    assert!(!plan.disbursement_rail_attached);
    assert!(!plan.tax_payment_execution_attached);
    assert!(!plan.durable_statutory_archive_attached);
    assert!(!plan.cloud_deployment_attached);
    assert!(!plan.production_filing_evidence_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
}

#[test]
fn statutory_filing_evidence_plan_covers_kr_us_payroll_accounting_authorities() {
    let plan = enterprise_suite_statutory_filing_evidence_plan();
    let urls = statutory_authority_urls(&plan);

    assert!(
        urls.iter()
            .any(|url| url.starts_with("https://s.nts.go.kr/"))
    );
    assert!(
        urls.iter()
            .any(|url| url.starts_with("https://www.hometax.go.kr/"))
    );
    assert!(
        urls.iter()
            .any(|url| url.starts_with("https://edi.nps.or.kr/"))
    );
    assert!(
        urls.iter()
            .any(|url| url.starts_with("https://www.irs.gov/"))
    );
    assert!(
        plan.requirements
            .iter()
            .any(|requirement| requirement.jurisdiction == StatutoryFilingJurisdiction::Korea)
    );
    assert!(
        plan.requirements.iter().any(
            |requirement| requirement.jurisdiction == StatutoryFilingJurisdiction::UnitedStates
        )
    );
    for rail_kind in [
        StatutoryFilingRailKind::PayrollWithholding,
        StatutoryFilingRailKind::SocialInsurance,
        StatutoryFilingRailKind::ValueAddedTax,
        StatutoryFilingRailKind::CorporateIncomeTax,
    ] {
        assert!(
            plan.requirements
                .iter()
                .any(|requirement| requirement.rail_kind == rail_kind),
            "missing {rail_kind:?}"
        );
    }
}

#[test]
fn statutory_filing_evidence_plan_preserves_source_manifest_and_receipt_boundaries() {
    let plan = enterprise_suite_statutory_filing_evidence_plan();

    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .source_rulepack_evidence_ref
            .starts_with("evidence/multispectrum/cs-ent-")
    }));
    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .payload_schema_ref
            .starts_with("schemas/statutory-filing/")
    }));
    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .required_receipt_schema_ref
            .starts_with("schemas/statutory-filing/")
    }));
    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .payload_digest_ref
            .starts_with("evidence/statutory/")
    }));
    assert!(
        plan.requirements
            .iter()
            .all(|requirement| requirement.legal_entity_scope_required)
    );
    assert!(
        plan.requirements
            .iter()
            .all(|requirement| requirement.human_approval_required)
    );
    assert!(
        plan.requirements
            .iter()
            .all(|requirement| requirement.agency_acceptance_receipt_required)
    );
}

#[test]
fn statutory_filing_evidence_plan_rejects_missing_requirements_duplicate_and_authority_drift() {
    let mut plan = enterprise_suite_statutory_filing_evidence_plan();
    plan.requirements.truncate(1);
    assert_eq!(
        validate_enterprise_suite_statutory_filing_evidence_plan(&plan),
        Err(EnterpriseStatutoryFilingEvidenceError::MissingRequirements)
    );

    let mut plan = enterprise_suite_statutory_filing_evidence_plan();
    plan.requirements[1].requirement_id = plan.requirements[0].requirement_id;
    assert_eq!(
        validate_enterprise_suite_statutory_filing_evidence_plan(&plan),
        Err(
            EnterpriseStatutoryFilingEvidenceError::DuplicateRequirement(
                "kr-payroll-withholding-hometax".to_owned()
            )
        )
    );

    let mut plan = enterprise_suite_statutory_filing_evidence_plan();
    plan.requirements[0].authority_url = "https://example.com/filing";
    assert_eq!(
        validate_enterprise_suite_statutory_filing_evidence_plan(&plan),
        Err(EnterpriseStatutoryFilingEvidenceError::InvalidOfficialAuthorityUrl)
    );
}

#[test]
fn statutory_filing_evidence_plan_rejects_unsafe_refs_and_runtime_overclaims() {
    let mut plan = enterprise_suite_statutory_filing_evidence_plan();
    plan.requirements[0].payload_digest_ref = "evidence/statutory/kr/secret-token";
    assert_eq!(
        validate_enterprise_suite_statutory_filing_evidence_plan(&plan),
        Err(EnterpriseStatutoryFilingEvidenceError::InvalidPayloadDigestRef)
    );

    let mut plan = enterprise_suite_statutory_filing_evidence_plan();
    plan.credential_attestation_required = false;
    assert_eq!(
        validate_enterprise_suite_statutory_filing_evidence_plan(&plan),
        Err(
            EnterpriseStatutoryFilingEvidenceError::MissingRequiredControl(
                "credential_attestation_required"
            )
        )
    );

    let mut plan = enterprise_suite_statutory_filing_evidence_plan();
    plan.filing_rail_runtime_attached = true;
    assert_eq!(
        validate_enterprise_suite_statutory_filing_evidence_plan(&plan),
        Err(EnterpriseStatutoryFilingEvidenceError::RuntimeAttachmentOverclaim)
    );
}
