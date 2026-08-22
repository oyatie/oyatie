use billing_tax_api::{
    CLOUD_BILLING_INVOICE_GENERATE_EVIDENCE_SURFACE, CloudBillingAccountSnapshotRequest,
    CloudBillingInvoiceGenerateApiResponse, CloudBillingInvoiceGenerateApiResult,
    CloudBillingInvoiceGenerateApiStatus, CloudBillingInvoiceGenerateRequest,
    CloudBillingInvoiceLineItemCreateRequest, CloudBillingMoneyRequest, CloudBillingPeriodRequest,
    CloudBillingTaxInvoiceFormatPolicy, CloudBillingTaxMeterUnitRequest,
    generate_cloud_billing_invoice_from_api,
};

fn valid_request() -> CloudBillingInvoiceGenerateRequest {
    CloudBillingInvoiceGenerateRequest {
        id: "inv_001".to_owned(),
        account: CloudBillingAccountSnapshotRequest {
            id: "ba_001".to_owned(),
            tenant_id: "ten_001".to_owned(),
            region: "region-home-1".to_owned(),
            regional_pack: "pack-kr".to_owned(),
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
        regional_pack: "pack-kr".to_owned(),
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
        ("pack-kr", "kr_electronic_tax_invoice"),
        ("pack-jp", "jp_qualified_invoice"),
        ("pack-eu", "eu_country_e_invoice"),
        ("pack-in", "in_gst"),
        ("pack-br", "br_nfe"),
        ("pack-ksa", "ksa_fatoora"),
        ("pack-uae", "uae_vat"),
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
        regional_pack: "pack-jp".to_owned(),
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
    let result = generate(request_for_pack("pack-jp", "kr_electronic_tax_invoice"));

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
        "pack-mars",
        "kr_electronic_tax_invoice",
    ));

    assert_eq!(
        result.status,
        CloudBillingInvoiceGenerateApiStatus::BadRequest
    );
    assert_eq!(error_code(result.response), "unsupported_regional_pack");
}
