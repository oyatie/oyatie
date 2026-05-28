use oya_tenant_rbac_disbursement_evidence::{
    DisbursementJurisdiction, DisbursementRailKind, TenantRbacDisbursementEvidenceError,
    disbursement_network_or_authority_urls, tenant_rbac_disbursement_evidence_plan,
    validate_tenant_rbac_disbursement_evidence_plan,
};

#[test]
fn disbursement_evidence_plan_validates_controls_and_nonclaims() {
    let plan = tenant_rbac_disbursement_evidence_plan();
    validate_tenant_rbac_disbursement_evidence_plan(&plan)
        .expect("disbursement evidence plan validates");

    assert_eq!(plan.plan_name, "tenant-rbac-disbursement-evidence-plan");
    assert_eq!(plan.service_name, "tenant-rbac");
    assert_eq!(plan.requirements.len(), 4);
    assert!(plan.source_rulepack_or_invoice_evidence_required);
    assert!(plan.bank_network_registry_required);
    assert!(plan.payment_file_digest_required);
    assert!(plan.beneficiary_tokenization_required);
    assert!(plan.approval_workflow_required);
    assert!(plan.segregation_of_duties_required);
    assert!(plan.dual_approval_required);
    assert!(plan.reconciliation_receipt_required);
    assert!(plan.rollback_or_reversal_runbook_required);
    assert!(!plan.manual_bank_portal_workaround_allowed);
    assert!(!plan.runtime_payment_execution_attached);
    assert!(!plan.bank_credential_attached);
    assert!(!plan.bank_connection_attached);
    assert!(!plan.disbursement_rail_runtime_attached);
    assert!(!plan.tax_payment_execution_attached);
    assert!(!plan.durable_payment_archive_attached);
    assert!(!plan.cloud_deployment_attached);
    assert!(!plan.production_disbursement_evidence_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
}

#[test]
fn disbursement_evidence_plan_covers_payroll_tax_social_insurance_and_vendor_rails() {
    let plan = tenant_rbac_disbursement_evidence_plan();
    let urls = disbursement_network_or_authority_urls(&plan);

    assert!(
        urls.iter()
            .any(|url| url.starts_with("https://achdevguide.nacha.org/"))
    );
    assert!(
        urls.iter()
            .any(|url| url.starts_with("https://www.irs.gov/"))
    );
    assert!(
        urls.iter()
            .any(|url| url.starts_with("https://eng.kftc.or.kr/"))
    );
    assert!(
        urls.iter()
            .any(|url| { url.starts_with("https://www.europeanpaymentscouncil.eu/") })
    );
    for jurisdiction in [
        DisbursementJurisdiction::Korea,
        DisbursementJurisdiction::UnitedStates,
        DisbursementJurisdiction::Europe,
    ] {
        assert!(
            plan.requirements
                .iter()
                .any(|requirement| requirement.jurisdiction == jurisdiction),
            "missing {jurisdiction:?}"
        );
    }
    for rail_kind in [
        DisbursementRailKind::PayrollAchCredit,
        DisbursementRailKind::TaxPaymentEftps,
        DisbursementRailKind::KoreanSocialInsuranceBankTransfer,
        DisbursementRailKind::SepaVendorCreditTransfer,
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
fn disbursement_evidence_plan_preserves_source_payment_and_reconciliation_boundaries() {
    let plan = tenant_rbac_disbursement_evidence_plan();

    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .source_evidence_ref
            .starts_with("evidence/multispectrum/cs-ent-")
    }));
    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .payment_file_schema_ref
            .starts_with("schemas/disbursement/")
    }));
    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .payment_digest_ref
            .starts_with("evidence/disbursement/")
    }));
    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .beneficiary_account_tokenization_ref
            .starts_with("privacy-boundary/disbursement/")
    }));
    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .approval_workflow_ref
            .starts_with("workflow/disbursement/")
    }));
    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .reconciliation_receipt_schema_ref
            .starts_with("schemas/disbursement-reconciliation/")
    }));
    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .rollback_or_reversal_runbook_ref
            .starts_with("rollback/disbursement/")
    }));
    assert!(
        plan.requirements
            .iter()
            .all(|requirement| requirement.legal_entity_scope_required)
    );
    assert!(
        plan.requirements
            .iter()
            .all(|requirement| requirement.segregation_of_duties_required)
    );
    assert!(
        plan.requirements
            .iter()
            .all(|requirement| requirement.dual_approval_required)
    );
    assert!(
        plan.requirements
            .iter()
            .all(|requirement| requirement.reconciliation_required)
    );
}

#[test]
fn disbursement_evidence_plan_rejects_missing_requirements_duplicate_and_network_drift() {
    let mut plan = tenant_rbac_disbursement_evidence_plan();
    plan.requirements.truncate(1);
    assert_eq!(
        validate_tenant_rbac_disbursement_evidence_plan(&plan),
        Err(TenantRbacDisbursementEvidenceError::MissingRequirements)
    );

    let mut plan = tenant_rbac_disbursement_evidence_plan();
    plan.requirements[1].requirement_id = plan.requirements[0].requirement_id;
    assert_eq!(
        validate_tenant_rbac_disbursement_evidence_plan(&plan),
        Err(TenantRbacDisbursementEvidenceError::DuplicateRequirement(
            "us-payroll-ach-direct-deposit".to_owned()
        ))
    );

    let mut plan = tenant_rbac_disbursement_evidence_plan();
    plan.requirements[0].network_or_authority_url = "https://example.com/payment";
    assert_eq!(
        validate_tenant_rbac_disbursement_evidence_plan(&plan),
        Err(TenantRbacDisbursementEvidenceError::InvalidOfficialNetworkOrAuthorityUrl)
    );
}

#[test]
fn disbursement_evidence_plan_rejects_unsafe_refs_missing_controls_and_runtime_overclaims() {
    let mut plan = tenant_rbac_disbursement_evidence_plan();
    plan.requirements[0].payment_digest_ref = "evidence/disbursement/us/payroll/secret-api-key";
    assert_eq!(
        validate_tenant_rbac_disbursement_evidence_plan(&plan),
        Err(TenantRbacDisbursementEvidenceError::InvalidPaymentDigestRef)
    );

    let mut plan = tenant_rbac_disbursement_evidence_plan();
    plan.dual_approval_required = false;
    assert_eq!(
        validate_tenant_rbac_disbursement_evidence_plan(&plan),
        Err(TenantRbacDisbursementEvidenceError::MissingRequiredControl(
            "dual_approval_required"
        ))
    );

    let mut plan = tenant_rbac_disbursement_evidence_plan();
    plan.runtime_payment_execution_attached = true;
    assert_eq!(
        validate_tenant_rbac_disbursement_evidence_plan(&plan),
        Err(TenantRbacDisbursementEvidenceError::RuntimeAttachmentOverclaim)
    );
}
