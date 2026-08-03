use oya_cloud_billing_tax_app::{
    CLOUD_BILLING_INVOICE_GENERATE_EVIDENCE_SURFACE, CloudBillingAccountSnapshotRequest,
    CloudBillingInvoiceGenerateApiResponse, CloudBillingInvoiceGenerateApiResult,
    CloudBillingInvoiceGenerateApiStatus, CloudBillingInvoiceGenerateRequest,
    CloudBillingInvoiceLineItemCreateRequest, CloudBillingMoneyRequest, CloudBillingPeriodRequest,
    CloudBillingTaxEmissionPlanRequest, CloudBillingTaxEmissionPlanValidationError,
    CloudBillingTaxInvoiceFormatPolicy, CloudBillingTaxMeterUnitRequest,
    generate_cloud_billing_invoice_from_api, validate_cloud_billing_tax_emission_plan,
};

fn valid_request() -> CloudBillingInvoiceGenerateRequest {
    CloudBillingInvoiceGenerateRequest {
        id: "inv_001".to_owned(),
        account: CloudBillingAccountSnapshotRequest {
            id: "ba_001".to_owned(),
            tenant_id: "ten_001".to_owned(),
            region: "region-home-1".to_owned(),
            regional_pack: "oya-pack-kr".to_owned(),
            payment_method: "pm_001".to_owned(),
            credit_balance: CloudBillingMoneyRequest {
                currency: "KRW".to_owned(),
                minor_units: 0,
            },
            state: "active".to_owned(),
            data_class: "INTERNAL_ONLY".to_owned(),
            created_at_epoch_seconds: 1,
        },
        tenant_id: "ten_001".to_owned(),
        regional_pack: "oya-pack-kr".to_owned(),
        period: CloudBillingPeriodRequest {
            start_epoch_seconds: 10,
            end_epoch_seconds: 20,
        },
        line_items: vec![CloudBillingInvoiceLineItemCreateRequest {
            id: "ili_001".to_owned(),
            resource_id: "res_001".to_owned(),
            description: "compute".to_owned(),
            units: vec![CloudBillingTaxMeterUnitRequest {
                kind: "resource_second".to_owned(),
                quantity_microunits: 1,
            }],
            subtotal: CloudBillingMoneyRequest {
                currency: "KRW".to_owned(),
                minor_units: 100,
            },
            data_class: "INTERNAL_ONLY".to_owned(),
        }],
        subtotal: CloudBillingMoneyRequest {
            currency: "KRW".to_owned(),
            minor_units: 100,
        },
        tax: CloudBillingMoneyRequest {
            currency: "KRW".to_owned(),
            minor_units: 10,
        },
        total: CloudBillingMoneyRequest {
            currency: "KRW".to_owned(),
            minor_units: 110,
        },
        tax_invoice_format: "kr_electronic_tax_invoice".to_owned(),
        tax_registration_id: "tax_001".to_owned(),
        issued_at_epoch_seconds: 30,
        due_at_epoch_seconds: 40,
        data_class: "INTERNAL_ONLY".to_owned(),
    }
}

fn error_code(response: CloudBillingInvoiceGenerateApiResponse) -> String {
    match response {
        CloudBillingInvoiceGenerateApiResponse::Error(error) => error.error.code,
        CloudBillingInvoiceGenerateApiResponse::Created(_) => {
            panic!("expected error response")
        }
    }
}

fn generate(request: CloudBillingInvoiceGenerateRequest) -> CloudBillingInvoiceGenerateApiResult {
    generate_cloud_billing_invoice_from_api(
        "req_001".to_owned(),
        "ten_001".to_owned(),
        "idem_001".to_owned(),
        request,
    )
}

fn request_for_pack(
    regional_pack: &str,
    tax_invoice_format: &str,
) -> CloudBillingInvoiceGenerateRequest {
    let mut request = valid_request();
    request.account.regional_pack = regional_pack.to_owned();
    request.regional_pack = regional_pack.to_owned();
    request.tax_invoice_format = tax_invoice_format.to_owned();
    request
}

fn valid_tax_emission_plan(env_tier: &str) -> CloudBillingTaxEmissionPlanRequest {
    CloudBillingTaxEmissionPlanRequest {
        tenant_id: "ten_001".to_owned(),
        recipient_tenant_id: "ten_001".to_owned(),
        env_tier: Some(env_tier.to_owned()),
        regional_pack: "oya-pack-kr".to_owned(),
        compliance_pack_label: "kr-tax-compliance-pack".to_owned(),
        tax_invoice_format_evidence_ref: "evidence://cloud-billing-tax/kr-format-policy".to_owned(),
        destination_binding_ref: Some("tax-intercept-log:ten_001:inv_001".to_owned()),
        policy_evidence_ref: Some("policy://tenancy/env-tier/ten_001/kr-tax".to_owned()),
        requested_runtime_delivery: false,
    }
}

fn tax_emission_error(
    request: CloudBillingTaxEmissionPlanRequest,
) -> CloudBillingTaxEmissionPlanValidationError {
    validate_cloud_billing_tax_emission_plan(request).expect_err("expected fail-closed guardrail")
}

#[test]
fn generates_invoice_and_exposes_evidence_surface() {
    assert_eq!(
        CLOUD_BILLING_INVOICE_GENERATE_EVIDENCE_SURFACE,
        "cloud.billing.invoice.generate"
    );
    let result = generate(valid_request());
    assert_eq!(result.status, CloudBillingInvoiceGenerateApiStatus::Created);
    assert!(matches!(
        result.response,
        CloudBillingInvoiceGenerateApiResponse::Created(_)
    ));
}

#[test]
fn covers_documented_status_codes() {
    assert_eq!(CloudBillingInvoiceGenerateApiStatus::Created.code(), 201);
    assert_eq!(CloudBillingInvoiceGenerateApiStatus::BadRequest.code(), 400);
    assert_eq!(
        CloudBillingInvoiceGenerateApiStatus::Unauthorized.code(),
        401
    );
    assert_eq!(CloudBillingInvoiceGenerateApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudBillingInvoiceGenerateApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudBillingInvoiceGenerateApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn maps_supported_regional_packs_to_documented_tax_formats() {
    for (regional_pack, tax_invoice_format) in [
        ("oya-pack-kr", "kr_electronic_tax_invoice"),
        ("oya-pack-jp", "jp_qualified_invoice"),
        ("oya-pack-eu", "eu_country_e_invoice"),
        ("oya-pack-in", "in_gst"),
        ("oya-pack-br", "br_nfe"),
        ("oya-pack-ksa", "ksa_fatoora"),
        ("oya-pack-uae", "uae_vat"),
    ] {
        assert_eq!(
            CloudBillingTaxInvoiceFormatPolicy::expected_for_regional_pack(regional_pack),
            Some(tax_invoice_format)
        );
    }
}

#[test]
fn rejects_account_and_invoice_regional_pack_mismatch() {
    let result = generate(CloudBillingInvoiceGenerateRequest {
        regional_pack: "oya-pack-jp".to_owned(),
        tax_invoice_format: "jp_qualified_invoice".to_owned(),
        ..valid_request()
    });

    assert_eq!(
        result.status,
        CloudBillingInvoiceGenerateApiStatus::BadRequest
    );
    assert_eq!(error_code(result.response), "regional_pack_mismatch");
}

#[test]
fn rejects_tax_invoice_format_that_does_not_match_regional_pack() {
    let result = generate(request_for_pack("oya-pack-jp", "kr_electronic_tax_invoice"));

    assert_eq!(
        result.status,
        CloudBillingInvoiceGenerateApiStatus::BadRequest
    );
    assert_eq!(
        error_code(result.response),
        "invalid_tax_invoice_format".to_owned()
    );
}

#[test]
fn rejects_unsupported_regional_pack_tax_format_pairs() {
    let result = generate(request_for_pack(
        "oya-pack-mars",
        "kr_electronic_tax_invoice",
    ));

    assert_eq!(
        result.status,
        CloudBillingInvoiceGenerateApiStatus::BadRequest
    );
    assert_eq!(error_code(result.response), "unsupported_regional_pack");
}

#[test]
fn tax_emission_test_tier_derives_intercept_log_only_mode() {
    let plan = validate_cloud_billing_tax_emission_plan(valid_tax_emission_plan("test"))
        .expect("test tier metadata plan should validate");

    assert_eq!(plan.env_tier, "test");
    assert_eq!(plan.outbound_mode, "intercept");
    assert_eq!(
        plan.destination_binding_ref,
        "tax-intercept-log:ten_001:inv_001"
    );
    assert!(!plan.runtime_delivery_authorized);
}

#[test]
fn tax_emission_staging_requires_tenant_qa_tax_endpoint() {
    let mut request = valid_tax_emission_plan("staging");
    request.destination_binding_ref = None;

    assert_eq!(
        tax_emission_error(request),
        CloudBillingTaxEmissionPlanValidationError::MissingStagingQaTaxEndpoint
    );
}

#[test]
fn tax_emission_staging_derives_test_recipients_for_qa_tax_endpoint() {
    let mut request = valid_tax_emission_plan("staging");
    request.destination_binding_ref =
        Some("qa-tax-endpoint:ten_001:kr-einvoice-sandbox".to_owned());

    let plan = validate_cloud_billing_tax_emission_plan(request)
        .expect("staging QA endpoint metadata plan should validate");

    assert_eq!(plan.env_tier, "staging");
    assert_eq!(plan.outbound_mode, "test_recipients");
    assert_eq!(
        plan.destination_binding_ref,
        "qa-tax-endpoint:ten_001:kr-einvoice-sandbox"
    );
    assert!(!plan.runtime_delivery_authorized);
}

#[test]
fn tax_emission_prod_requires_policy_evidence_before_live_mode() {
    let mut request = valid_tax_emission_plan("prod");
    request.destination_binding_ref = Some("prod-tax-authority-binding:ten_001:kr-nts".to_owned());
    request.policy_evidence_ref = None;

    assert_eq!(
        tax_emission_error(request),
        CloudBillingTaxEmissionPlanValidationError::MissingProdPolicyEvidence
    );
}

#[test]
fn tax_emission_prod_derives_live_only_with_policy_evidence() {
    let mut request = valid_tax_emission_plan("prod");
    request.destination_binding_ref = Some("prod-tax-authority-binding:ten_001:kr-nts".to_owned());

    let plan = validate_cloud_billing_tax_emission_plan(request)
        .expect("prod metadata plan should validate with policy evidence");

    assert_eq!(plan.env_tier, "prod");
    assert_eq!(plan.outbound_mode, "live");
    assert_eq!(
        plan.destination_binding_ref,
        "prod-tax-authority-binding:ten_001:kr-nts"
    );
    assert!(!plan.runtime_delivery_authorized);
}

#[test]
fn tax_emission_rejects_missing_env_tier() {
    let mut request = valid_tax_emission_plan("test");
    request.env_tier = None;

    assert_eq!(
        tax_emission_error(request),
        CloudBillingTaxEmissionPlanValidationError::MissingEnvTier
    );
}

#[test]
fn tax_emission_rejects_test_mode_tax_authority_delivery_attempt() {
    let mut request = valid_tax_emission_plan("test");
    request.requested_runtime_delivery = true;

    assert_eq!(
        tax_emission_error(request),
        CloudBillingTaxEmissionPlanValidationError::TestTierRuntimeDeliveryAttempt
    );
}

#[test]
fn tax_emission_rejects_cross_tenant_tax_recipient_leakage() {
    let mut request = valid_tax_emission_plan("prod");
    request.recipient_tenant_id = "ten_other".to_owned();
    request.destination_binding_ref =
        Some("prod-tax-authority-binding:ten_other:kr-nts".to_owned());

    assert_eq!(
        tax_emission_error(request),
        CloudBillingTaxEmissionPlanValidationError::CrossTenantTaxRecipient
    );
}

#[test]
fn tax_emission_rejects_cross_tenant_destination_binding_leakage() {
    for (env_tier, destination_binding_ref) in [
        ("test", "tax-intercept-log:ten_other:inv_001"),
        ("staging", "qa-tax-endpoint:ten_other:kr-einvoice-sandbox"),
        ("prod", "prod-tax-authority-binding:ten_other:kr-nts"),
    ] {
        let mut request = valid_tax_emission_plan(env_tier);
        request.destination_binding_ref = Some(destination_binding_ref.to_owned());

        assert_eq!(
            tax_emission_error(request),
            CloudBillingTaxEmissionPlanValidationError::CrossTenantTaxRecipient
        );
    }
}

#[test]
fn tax_emission_rejects_raw_tax_authority_credential_marker() {
    let mut request = valid_tax_emission_plan("staging");
    request.destination_binding_ref =
        Some("qa-tax-endpoint:ten_001:raw-tax-authority-credential".to_owned());

    assert_eq!(
        tax_emission_error(request),
        CloudBillingTaxEmissionPlanValidationError::SecretLikeFixtureContent
    );
}
