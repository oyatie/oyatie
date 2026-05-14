use oya_cloud_billing_domain::{CloudBillingError, CloudBillingLedger};
use oya_cloud_billing_tax_application::{
    CLOUD_BILLING_INVOICE_GENERATE_SURFACE, CloudBillingAccountSnapshotRequest,
    CloudBillingInvoiceGenerateApiRequest, CloudBillingInvoiceGenerateApiStatus,
    CloudBillingInvoiceGenerateIdempotencyLedger, CloudBillingInvoiceGenerateRequest,
    CloudBillingInvoiceLineItemCreateRequest, CloudBillingMoneyRequest, CloudBillingPeriodRequest,
    CloudBillingTaxApiAuthorization, CloudBillingTaxApiBoundaryContext, CloudBillingTaxApiError,
    CloudBillingTaxApiPrincipal, CloudBillingTaxMeterUnitRequest,
    generate_cloud_billing_invoice_from_api,
};

const INVOICE_ID: &str = "inv_kr_202605_001";
const BILLING_ACCOUNT_ID: &str = "ba_ten_kr";
const RESOURCE_ID: &str = "oya:cloud:kr-seoul:ten_kr:instance:api-001";

fn boundary_for(request_id: &str, idempotency_key: &str) -> CloudBillingTaxApiBoundaryContext {
    CloudBillingTaxApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_kr".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn principal_for(principal_id: &str) -> CloudBillingTaxApiPrincipal {
    CloudBillingTaxApiPrincipal {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
    }
}

fn authorization_for(principal_id: &str, surfaces: &[&str]) -> CloudBillingTaxApiAuthorization {
    CloudBillingTaxApiAuthorization {
        tenant_id: "ten_kr".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: format!("authz_decision_{principal_id}"),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
    }
}

fn money(minor_units: u64) -> CloudBillingMoneyRequest {
    CloudBillingMoneyRequest {
        currency: "KRW".to_string(),
        minor_units,
    }
}

fn account_snapshot() -> CloudBillingAccountSnapshotRequest {
    CloudBillingAccountSnapshotRequest {
        id: BILLING_ACCOUNT_ID.to_string(),
        tenant_id: "ten_kr".to_string(),
        region: "kr-seoul".to_string(),
        regional_pack: "oya-pack-kr".to_string(),
        payment_method: "pm_card_001".to_string(),
        credit_balance: money(10_000),
        state: "active".to_string(),
        data_class: "FINANCIAL_KR_신용정보".to_string(),
        created_at_epoch_seconds: 1_700_000_000,
    }
}

fn line_item() -> CloudBillingInvoiceLineItemCreateRequest {
    CloudBillingInvoiceLineItemCreateRequest {
        id: "ili_compute_001".to_string(),
        resource_id: RESOURCE_ID.to_string(),
        description: "instance api-001 resource seconds".to_string(),
        units: vec![CloudBillingTaxMeterUnitRequest {
            kind: "resource_second".to_string(),
            quantity_microunits: 3_600_000_000,
        }],
        subtotal: money(100_000),
        data_class: "FINANCIAL_KR_신용정보".to_string(),
    }
}

fn invoice_body(id: &str) -> CloudBillingInvoiceGenerateRequest {
    CloudBillingInvoiceGenerateRequest {
        id: id.to_string(),
        account: account_snapshot(),
        tenant_id: "ten_kr".to_string(),
        regional_pack: "oya-pack-kr".to_string(),
        period: CloudBillingPeriodRequest {
            start_epoch_seconds: 1_700_000_000,
            end_epoch_seconds: 1_700_086_400,
        },
        line_items: vec![line_item()],
        subtotal: money(100_000),
        tax: money(10_000),
        total: money(110_000),
        tax_invoice_format: "kr_electronic_tax_invoice".to_string(),
        tax_registration_id: "kr-bizreg/1234567890".to_string(),
        issued_at_epoch_seconds: 1_700_086_500,
        due_at_epoch_seconds: 1_700_604_900,
        data_class: "FINANCIAL_KR_신용정보".to_string(),
    }
}

fn create_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudBillingInvoiceGenerateApiRequest {
    CloudBillingInvoiceGenerateApiRequest {
        path_invoice_id: INVOICE_ID.to_string(),
        boundary: boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_billing_tax_admin"),
        authorization: authorization_for(
            "sp_billing_tax_admin",
            &[CLOUD_BILLING_INVOICE_GENERATE_SURFACE],
        ),
        body: invoice_body(INVOICE_ID),
    }
}

#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(
        CLOUD_BILLING_INVOICE_GENERATE_SURFACE,
        "cloud.billing.invoice.generate"
    );
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
fn invoice_generate_api_creates_invoice_once_and_replays_same_idempotent_result() {
    let mut ledger = CloudBillingLedger::default();
    let mut idempotency = CloudBillingInvoiceGenerateIdempotencyLedger::default();
    let request = create_request("req-billing-invoice-create", "idem_billing_invoice_create");

    let first =
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, request.clone())
            .expect("authorized invoice generation succeeds");
    let second = generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(idempotency.len(), 1);
    assert_eq!(ledger.invoices().count(), 1);
    assert_eq!(first.metadata.request_id, "req-billing-invoice-create");
    assert_eq!(first.data.id, INVOICE_ID);
    assert_eq!(first.data.billing_account_id, BILLING_ACCOUNT_ID);
    assert_eq!(first.data.tenant_id, "ten_kr");
    assert_eq!(first.data.regional_pack, "oya-pack-kr");
    assert_eq!(first.data.line_item_count, 1);
    assert_eq!(first.data.currency, "KRW");
    assert_eq!(first.data.subtotal_minor_units, 100_000);
    assert_eq!(first.data.tax_minor_units, 10_000);
    assert_eq!(first.data.total_minor_units, 110_000);
    assert_eq!(first.data.tax_invoice_format, "kr_electronic_tax_invoice");
    assert_eq!(first.data.tax_registration_id, "kr-bizreg/1234567890");
    assert_eq!(first.data.state, "issued");
    assert_eq!(first.data.data_class, "FINANCIAL_KR");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn invoice_generate_api_rejects_path_body_invoice_drift_before_ledger() {
    let mut ledger = CloudBillingLedger::default();
    let mut idempotency = CloudBillingInvoiceGenerateIdempotencyLedger::default();
    let mut request = create_request("req-billing-invoice-drift", "idem_billing_invoice_drift");
    request.body.id = "inv_kr_202605_002".to_string();

    let error = generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, request)
        .expect_err("path/body invoice drift is rejected");

    assert_eq!(
        error,
        CloudBillingTaxApiError::InvoiceIdMismatch {
            path_invoice_id: INVOICE_ID.to_string(),
            body_invoice_id: "inv_kr_202605_002".to_string(),
        }
    );
    assert_eq!(error.invoice_generate_status_code(), 400);
    assert!(idempotency.is_empty());
    assert_eq!(ledger.invoices().count(), 0);
}

#[test]
fn invoice_generate_api_separates_missing_authentication_from_denied_authorization() {
    let mut ledger = CloudBillingLedger::default();
    let mut idempotency = CloudBillingInvoiceGenerateIdempotencyLedger::default();
    let mut unauthenticated =
        create_request("req-billing-invoice-authn", "idem_billing_invoice_authn");
    unauthenticated.principal.principal_id = " ".to_string();

    let authn_error =
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, unauthenticated)
            .expect_err("missing principal is authentication failure");
    assert_eq!(authn_error, CloudBillingTaxApiError::EmptyPrincipalId);
    assert_eq!(authn_error.invoice_generate_status_code(), 401);

    let mut denied = create_request("req-billing-invoice-authz", "idem_billing_invoice_authz");
    denied.authorization.allowed_surfaces = vec!["cloud.billing.event.ingest".to_string()];
    let authz_error =
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, denied)
            .expect_err("authorization decision excludes invoice generate");
    assert_eq!(
        authz_error,
        CloudBillingTaxApiError::AuthorizationDenied {
            surface: CLOUD_BILLING_INVOICE_GENERATE_SURFACE.to_string(),
        }
    );
    assert_eq!(authz_error.invoice_generate_status_code(), 403);
    assert!(idempotency.is_empty());
    assert_eq!(ledger.invoices().count(), 0);
}

#[test]
fn invoice_generate_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut ledger = CloudBillingLedger::default();
    let mut idempotency = CloudBillingInvoiceGenerateIdempotencyLedger::default();
    let mut request = create_request(" ", "idem_billing_invoice_empty_header");
    assert_eq!(
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, request.clone()),
        Err(CloudBillingTaxApiError::EmptyRequestId)
    );

    request.boundary.request_id = "req-billing-invoice-tenant-drift".to_string();
    request.boundary.tenant_id = "ten_other".to_string();
    assert_eq!(
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, request),
        Err(CloudBillingTaxApiError::TenantMismatch {
            header_tenant_id: "ten_other".to_string(),
            principal_tenant_id: "ten_kr".to_string(),
            body_tenant_id: "ten_kr".to_string(),
            account_tenant_id: "ten_kr".to_string(),
        })
    );
    assert!(idempotency.is_empty());
    assert_eq!(ledger.invoices().count(), 0);
}

#[test]
fn invoice_generate_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut ledger = CloudBillingLedger::default();
    let mut idempotency = CloudBillingInvoiceGenerateIdempotencyLedger::default();
    let request = create_request("req-billing-invoice-idem", "idem_billing_invoice_idem");
    generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, request.clone())
        .expect("initial invoice generation succeeds");

    let mut drifted = request;
    drifted.body.tax = money(11_000);
    drifted.body.total = money(111_000);
    assert_eq!(
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, drifted),
        Err(CloudBillingTaxApiError::IdempotencyKeyReused {
            idempotency_key: "idem_billing_invoice_idem".to_string(),
        })
    );
    assert_eq!(idempotency.len(), 1);
    assert_eq!(ledger.invoices().count(), 1);
}

#[test]
fn invoice_generate_api_maps_duplicate_invoice_to_conflict() {
    let mut ledger = CloudBillingLedger::default();
    let mut idempotency = CloudBillingInvoiceGenerateIdempotencyLedger::default();
    generate_cloud_billing_invoice_from_api(
        &mut ledger,
        &mut idempotency,
        create_request("req-billing-invoice-dup-1", "idem_billing_invoice_dup_1"),
    )
    .expect("first invoice generation succeeds");

    let error = generate_cloud_billing_invoice_from_api(
        &mut ledger,
        &mut idempotency,
        create_request("req-billing-invoice-dup-2", "idem_billing_invoice_dup_2"),
    )
    .expect_err("same invoice id through a new idempotency key conflicts");
    assert_eq!(
        error,
        CloudBillingTaxApiError::Billing(CloudBillingError::DuplicateInvoice)
    );
    assert_eq!(error.invoice_generate_status_code(), 409);
    assert_eq!(ledger.invoices().count(), 1);
}

#[test]
fn invoice_generate_api_maps_tax_registration_total_and_inactive_account_invariants() {
    let mut ledger = CloudBillingLedger::default();
    let mut idempotency = CloudBillingInvoiceGenerateIdempotencyLedger::default();
    let mut bad_registration =
        create_request("req-billing-invoice-reg", "idem_billing_invoice_reg");
    bad_registration.body.tax_registration_id = "kr-bizreg/notdigits".to_string();
    let registration_error =
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, bad_registration)
            .expect_err("tax registration shape follows regional invoice format");
    assert_eq!(
        registration_error,
        CloudBillingTaxApiError::Billing(CloudBillingError::InvalidTaxRegistrationId)
    );
    assert_eq!(registration_error.invoice_generate_status_code(), 400);

    let mut bad_total = create_request("req-billing-invoice-total", "idem_billing_invoice_total");
    bad_total.body.total = money(109_999);
    let total_error =
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, bad_total)
            .expect_err("invoice totals must match subtotal plus tax");
    assert_eq!(
        total_error,
        CloudBillingTaxApiError::Billing(CloudBillingError::InvalidInvoiceTotal)
    );
    assert_eq!(total_error.invoice_generate_status_code(), 400);

    let mut inactive = create_request(
        "req-billing-invoice-inactive",
        "idem_billing_invoice_inactive",
    );
    inactive.body.account.state = "suspended".to_string();
    let inactive_error =
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, inactive)
            .expect_err("inactive billing accounts cannot issue invoices");
    assert_eq!(
        inactive_error,
        CloudBillingTaxApiError::Billing(CloudBillingError::BillingAccountInactive)
    );
    assert_eq!(inactive_error.invoice_generate_status_code(), 403);
    assert_eq!(ledger.invoices().count(), 0);
}

#[test]
fn invoice_generate_api_rejects_invalid_labels_before_ledger() {
    let mut ledger = CloudBillingLedger::default();
    let mut idempotency = CloudBillingInvoiceGenerateIdempotencyLedger::default();
    let mut invalid_format =
        create_request("req-billing-invoice-format", "idem_billing_invoice_format");
    invalid_format.body.tax_invoice_format = "paper_scroll".to_string();
    let format_error =
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, invalid_format)
            .expect_err("unknown tax invoice format is rejected before ledger mutation");
    assert_eq!(
        format_error,
        CloudBillingTaxApiError::InvalidTaxInvoiceFormatLabel {
            tax_invoice_format: "paper_scroll".to_string(),
        }
    );
    assert_eq!(format_error.invoice_generate_status_code(), 400);

    let mut invalid_state =
        create_request("req-billing-invoice-state", "idem_billing_invoice_state");
    invalid_state.body.account.state = "paused".to_string();
    let state_error =
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, invalid_state)
            .expect_err("unknown account state is rejected before ledger mutation");
    assert_eq!(
        state_error,
        CloudBillingTaxApiError::InvalidAccountStateLabel {
            account_state: "paused".to_string(),
        }
    );
    assert_eq!(state_error.invoice_generate_status_code(), 400);

    let mut invalid_unit = create_request("req-billing-invoice-unit", "idem_billing_invoice_unit");
    invalid_unit.body.line_items[0].units[0].kind = "parsec".to_string();
    let unit_error =
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, invalid_unit)
            .expect_err("unknown meter unit kind is rejected before ledger mutation");
    assert_eq!(
        unit_error,
        CloudBillingTaxApiError::InvalidMeterUnitKindLabel {
            unit_kind: "parsec".to_string(),
        }
    );
    assert_eq!(unit_error.invoice_generate_status_code(), 400);

    let mut invalid_class =
        create_request("req-billing-invoice-class", "idem_billing_invoice_class");
    invalid_class.body.data_class = "PUBLIC".to_string();
    let class_error =
        generate_cloud_billing_invoice_from_api(&mut ledger, &mut idempotency, invalid_class)
            .expect_err("non-financial invoice metadata is rejected by the kernel");
    assert_eq!(
        class_error,
        CloudBillingTaxApiError::Billing(CloudBillingError::InvalidDataClass)
    );
    assert_eq!(class_error.invoice_generate_status_code(), 400);
    assert_eq!(ledger.invoices().count(), 0);
}
