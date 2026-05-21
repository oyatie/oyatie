use oya_cloud_billing_tax_app::{
    CLOUD_BILLING_INVOICE_GENERATE_EVIDENCE_SURFACE, CloudBillingAccountSnapshotRequest,
    CloudBillingInvoiceGenerateApiResponse, CloudBillingInvoiceGenerateApiStatus,
    CloudBillingInvoiceGenerateRequest, CloudBillingInvoiceLineItemCreateRequest,
    CloudBillingMoneyRequest, CloudBillingPeriodRequest, CloudBillingTaxMeterUnitRequest,
    generate_cloud_billing_invoice_from_api,
};

fn valid_request() -> CloudBillingInvoiceGenerateRequest {
    CloudBillingInvoiceGenerateRequest {
        id: "inv_001".to_owned(),
        account: CloudBillingAccountSnapshotRequest {
            id: "ba_001".to_owned(),
            tenant_id: "ten_001".to_owned(),
            region: "region-home-1".to_owned(),
            regional_pack: "oya-pack-alpha".to_owned(),
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
        regional_pack: "oya-pack-alpha".to_owned(),
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

#[test]
fn generates_invoice_and_exposes_evidence_surface() {
    assert_eq!(
        CLOUD_BILLING_INVOICE_GENERATE_EVIDENCE_SURFACE,
        "cloud.billing.invoice.generate"
    );
    let result = generate_cloud_billing_invoice_from_api(
        "req_001".to_owned(),
        "ten_001".to_owned(),
        "idem_001".to_owned(),
        valid_request(),
    );
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
