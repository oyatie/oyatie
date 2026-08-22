use billing_domain::{
    BillingComponent, BillingPeriod, CloudBillingError, CloudBillingTenantGuardrail,
    CloudBillingTenantGuardrailCreate, TaxInvoiceFormat, TenantClass,
};

fn paid_guardrail_create() -> CloudBillingTenantGuardrailCreate {
    CloudBillingTenantGuardrailCreate {
        tenant_id: "ten_alpha".to_string(),
        region: "region-alpha".to_string(),
        billing_account_id: "ba_ten_alpha".to_string(),
        tenant_class: TenantClass::Paid,
        billing_components: vec![BillingComponent::PerSeat, BillingComponent::PerUsage],
        regional_pack: "pack-electronic-tax".to_string(),
        tax_invoice_format: TaxInvoiceFormat::ElectronicTaxInvoice,
        rate_card_ref: "rate/region-alpha/cloud/billing/v1".to_string(),
        invoice_id: "inv_ten_alpha_202605".to_string(),
        billing_period: BillingPeriod::new(1_700_000_000, 1_700_086_400)
            .expect("billing period fixture is valid"),
        metering_evidence_refs: vec![
            "evidence/billing/metering/ten_alpha/region-alpha/usage-202605.json".to_string(),
            "evidence/billing/metering/ten_alpha/region-alpha/idempotency-ledger.json".to_string(),
        ],
        invoice_evidence_ref:
            "evidence/billing/invoice/ten_alpha/region-alpha/inv_ten_alpha_202605.json".to_string(),
        tax_evidence_ref:
            "evidence/billing/tax/ten_alpha/region-alpha/tax-format-electronic-202605.json"
                .to_string(),
        audit_chain_ref:
            "audit-chain/billing/ten_alpha/region-alpha/2026-05-23/cs-cloud-billing-tax-001"
                .to_string(),
        demo_trial_cap_evidence_ref: None,
    }
}

#[test]
fn paid_guardrail_records_tenant_class_components_and_evidence_refs() {
    let guardrail = CloudBillingTenantGuardrail::new(paid_guardrail_create())
        .expect("paid tenant guardrail should admit complete billing evidence");

    assert_eq!(guardrail.tenant_class.value, TenantClass::Paid);
    assert_eq!(
        guardrail.billing_components.value,
        vec![BillingComponent::PerSeat, BillingComponent::PerUsage]
    );
    assert_eq!(
        guardrail.tax_invoice_format.value,
        TaxInvoiceFormat::ElectronicTaxInvoice
    );
    assert_eq!(guardrail.metering_evidence_refs.value.len(), 2);
    assert_eq!(
        guardrail.audit_chain_ref.value,
        "audit-chain/billing/ten_alpha/region-alpha/2026-05-23/cs-cloud-billing-tax-001"
    );
}

#[test]
fn demo_trial_guardrail_rejects_paid_components_and_requires_cap_evidence() {
    let mut with_paid_components = paid_guardrail_create();
    with_paid_components.tenant_class = TenantClass::DemoTrial;
    with_paid_components.demo_trial_cap_evidence_ref =
        Some("evidence/billing/demo-trial-cap/ten_alpha/region-alpha/cap-202605.json".to_string());

    let component_error = CloudBillingTenantGuardrail::new(with_paid_components)
        .expect_err("demo trial tenants cannot enable paid billing components");
    assert_eq!(
        component_error,
        CloudBillingError::InvalidBillingComponentPolicy
    );

    let mut without_cap_evidence = paid_guardrail_create();
    without_cap_evidence.tenant_class = TenantClass::DemoTrial;
    without_cap_evidence.billing_components = Vec::new();
    without_cap_evidence.demo_trial_cap_evidence_ref = None;

    let cap_error = CloudBillingTenantGuardrail::new(without_cap_evidence)
        .expect_err("demo trial tenants require cap evidence");
    assert_eq!(cap_error, CloudBillingError::InvalidTenantClassPolicy);

    let mut demo_trial = paid_guardrail_create();
    demo_trial.tenant_class = TenantClass::DemoTrial;
    demo_trial.billing_components = Vec::new();
    demo_trial.demo_trial_cap_evidence_ref =
        Some("evidence/billing/demo-trial-cap/ten_alpha/region-alpha/cap-202605.json".to_string());

    let guardrail = CloudBillingTenantGuardrail::new(demo_trial)
        .expect("demo trial with cap evidence and no paid components is valid");
    assert_eq!(guardrail.tenant_class.value, TenantClass::DemoTrial);
    assert!(guardrail.billing_components.value.is_empty());
    assert!(guardrail.demo_trial_cap_evidence_ref.value.is_some());
}

#[test]
fn paid_guardrail_requires_at_least_one_unique_paid_component() {
    let mut no_components = paid_guardrail_create();
    no_components.billing_components = Vec::new();
    let empty_error = CloudBillingTenantGuardrail::new(no_components)
        .expect_err("paid tenants need at least one billable component");
    assert_eq!(
        empty_error,
        CloudBillingError::InvalidBillingComponentPolicy
    );

    let mut duplicate_components = paid_guardrail_create();
    duplicate_components.billing_components =
        vec![BillingComponent::PerUsage, BillingComponent::PerUsage];
    let duplicate_error = CloudBillingTenantGuardrail::new(duplicate_components)
        .expect_err("paid tenant billing components are a set");
    assert_eq!(
        duplicate_error,
        CloudBillingError::InvalidBillingComponentPolicy
    );
}

#[test]
fn guardrail_rejects_secret_like_or_wrong_scope_evidence_refs() {
    let mut wrong_metering_prefix = paid_guardrail_create();
    wrong_metering_prefix.metering_evidence_refs =
        vec!["evidence/compute/metering/ten_alpha/region-alpha/usage-202605.json".to_string()];
    let metering_error = CloudBillingTenantGuardrail::new(wrong_metering_prefix)
        .expect_err("metering evidence must stay in billing metering scope");
    assert_eq!(metering_error, CloudBillingError::InvalidBillingEvidenceRef);

    let mut secret_tax_ref = paid_guardrail_create();
    secret_tax_ref.tax_evidence_ref =
        "evidence/billing/tax/ten_alpha/region-alpha/openbao-secret-token.json".to_string();
    let secret_error = CloudBillingTenantGuardrail::new(secret_tax_ref)
        .expect_err("tax evidence refs must not carry secret-like material");
    assert_eq!(secret_error, CloudBillingError::InvalidBillingEvidenceRef);

    let mut wrong_audit_ref = paid_guardrail_create();
    wrong_audit_ref.audit_chain_ref =
        "evidence/billing/audit/ten_alpha/region-alpha/not-a-chain.json".to_string();
    let audit_error = CloudBillingTenantGuardrail::new(wrong_audit_ref)
        .expect_err("audit-chain ref must point at the billing audit-chain lane");
    assert_eq!(audit_error, CloudBillingError::InvalidAuditChainRef);
}

#[test]
fn guardrail_rejects_regional_pack_tax_format_drift_and_invalid_ids() {
    let mut format_drift = paid_guardrail_create();
    format_drift.regional_pack = "pack-qualified-tax".to_string();
    let format_error = CloudBillingTenantGuardrail::new(format_drift)
        .expect_err("regional tax pack determines the required invoice format");
    assert_eq!(format_error, CloudBillingError::InvalidTaxInvoiceFormat);

    let mut invalid_tenant = paid_guardrail_create();
    invalid_tenant.tenant_id = "ten_Alpha".to_string();
    let tenant_error = CloudBillingTenantGuardrail::new(invalid_tenant)
        .expect_err("tenant ids use strict lower-case canonical segments");
    assert_eq!(tenant_error, CloudBillingError::InvalidTenantId);

    let mut invalid_account = paid_guardrail_create();
    invalid_account.billing_account_id = "inv_ten_alpha".to_string();
    let account_error = CloudBillingTenantGuardrail::new(invalid_account)
        .expect_err("billing account id must use billing account id prefix");
    assert_eq!(account_error, CloudBillingError::InvalidBillingAccountId);
}
