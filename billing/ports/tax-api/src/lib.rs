//! Cloud billing tax invoice API application surface.
//!
//! This crate is the runtime proof surface for
//! `contracts/openapi/cloud/cloud-billing-invoice-v1.yaml`: request/response
//! structs intentionally mirror the OpenAPI schema names, fields, and status
//! codes so documentation gates can detect contract/runtime drift.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub const CLOUD_BILLING_INVOICE_GENERATE_EVIDENCE_SURFACE: &str = "cloud.billing.invoice.generate";
pub const CLOUD_BILLING_INVOICE_SCHEMA_VERSION: u32 = 1;

pub struct CloudBillingTaxInvoiceFormatPolicy;

impl CloudBillingTaxInvoiceFormatPolicy {
    pub fn expected_for_regional_pack(regional_pack: &str) -> Option<&'static str> {
        match regional_pack {
            "pack-kr" => Some("kr_electronic_tax_invoice"),
            "pack-jp" => Some("jp_qualified_invoice"),
            "pack-eu" => Some("eu_country_e_invoice"),
            "pack-in" => Some("in_gst"),
            "pack-br" => Some("br_nfe"),
            "pack-ksa" => Some("ksa_fatoora"),
            "pack-uae" => Some("uae_vat"),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingInvoiceGenerateRequest {
    pub id: String,                                  // data_class: INTERNAL_ONLY
    pub account: CloudBillingAccountSnapshotRequest, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                           // data_class: INTERNAL_ONLY
    pub regional_pack: String,                       // data_class: INTERNAL_ONLY
    pub period: CloudBillingPeriodRequest,           // data_class: INTERNAL_ONLY
    pub line_items: Vec<CloudBillingInvoiceLineItemCreateRequest>, // data_class: INTERNAL_ONLY
    pub subtotal: CloudBillingMoneyRequest,          // data_class: INTERNAL_ONLY
    pub tax: CloudBillingMoneyRequest,               // data_class: INTERNAL_ONLY
    pub total: CloudBillingMoneyRequest,             // data_class: INTERNAL_ONLY
    pub tax_invoice_format: String,                  // data_class: INTERNAL_ONLY
    pub tax_registration_id: String,                 // data_class: FINANCIAL_REGULATED_CREDIT
    pub issued_at_epoch_seconds: u64,                // data_class: INTERNAL_ONLY
    pub due_at_epoch_seconds: u64,                   // data_class: INTERNAL_ONLY
    pub data_class: String,                          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingAccountSnapshotRequest {
    pub id: String,                               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub region: String,                           // data_class: PUBLIC
    pub regional_pack: String,                    // data_class: INTERNAL_ONLY
    pub payment_method: String,                   // data_class: INTERNAL_ONLY
    pub credit_balance: CloudBillingMoneyRequest, // data_class: INTERNAL_ONLY
    pub state: String,                            // data_class: INTERNAL_ONLY
    pub data_class: String,                       // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingMoneyRequest {
    pub currency: String, // data_class: INTERNAL_ONLY
    pub minor_units: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingPeriodRequest {
    pub start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingInvoiceLineItemCreateRequest {
    pub id: String,                                  // data_class: INTERNAL_ONLY
    pub resource_id: String,                         // data_class: INTERNAL_ONLY
    pub description: String,                         // data_class: INTERNAL_ONLY
    pub units: Vec<CloudBillingTaxMeterUnitRequest>, // data_class: INTERNAL_ONLY
    pub subtotal: CloudBillingMoneyRequest,          // data_class: INTERNAL_ONLY
    pub data_class: String,                          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingTaxMeterUnitRequest {
    pub kind: String,             // data_class: INTERNAL_ONLY
    pub quantity_microunits: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingInvoiceGenerateSuccessResponse {
    pub data: CloudBillingInvoiceRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudBillingTaxApiMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingTaxApiMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingInvoiceRecord {
    pub id: String,                      // data_class: INTERNAL_ONLY
    pub billing_account_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub regional_pack: String,           // data_class: INTERNAL_ONLY
    pub period_start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub period_end_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub line_item_count: u32,            // data_class: INTERNAL_ONLY
    pub currency: String,                // data_class: INTERNAL_ONLY
    pub subtotal_minor_units: u64,       // data_class: INTERNAL_ONLY
    pub tax_minor_units: u64,            // data_class: INTERNAL_ONLY
    pub total_minor_units: u64,          // data_class: INTERNAL_ONLY
    pub tax_invoice_format: String,      // data_class: INTERNAL_ONLY
    pub tax_registration_id: String,     // data_class: FINANCIAL_REGULATED_CREDIT
    pub state: String,                   // data_class: INTERNAL_ONLY
    pub issued_at_epoch_seconds: u64,    // data_class: INTERNAL_ONLY
    pub due_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub data_class: String,              // data_class: INTERNAL_ONLY
    pub schema_version: u32,             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingTaxApiErrorResponse {
    pub error: CloudBillingTaxApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingTaxApiErrorBody {
    pub code: String,                                // data_class: INTERNAL_ONLY
    pub message: String,                             // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,           // data_class: INTERNAL_ONLY
    pub request_id: String,                          // data_class: INTERNAL_ONLY
    pub details: Vec<CloudBillingTaxApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingTaxApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudBillingInvoiceGenerateApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl CloudBillingInvoiceGenerateApiStatus {
    pub fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingInvoiceGenerateApiResult {
    pub status: CloudBillingInvoiceGenerateApiStatus, // data_class: INTERNAL_ONLY
    pub response: CloudBillingInvoiceGenerateApiResponse, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudBillingInvoiceGenerateApiResponse {
    Created(CloudBillingInvoiceGenerateSuccessResponse),
    Error(CloudBillingTaxApiErrorResponse),
}

pub fn generate_cloud_billing_invoice_from_api(
    request_id: String,
    tenant_id: String,
    idempotency_key: String,
    request: CloudBillingInvoiceGenerateRequest,
) -> CloudBillingInvoiceGenerateApiResult {
    if request_id.is_empty() {
        return error_result(
            CloudBillingInvoiceGenerateApiStatus::Unauthorized,
            "missing_request_id",
            "authenticated request id evidence is required",
            request_id,
            None,
        );
    }
    if tenant_id.is_empty()
        || tenant_id != request.tenant_id
        || tenant_id != request.account.tenant_id
    {
        return error_result(
            CloudBillingInvoiceGenerateApiStatus::Forbidden,
            "tenant_mismatch",
            "tenant header must match invoice and account tenant",
            request_id,
            None,
        );
    }
    if idempotency_key.is_empty() {
        return error_result(
            CloudBillingInvoiceGenerateApiStatus::UnprocessableEntity,
            "missing_idempotency_key",
            "idempotency key is required",
            request_id,
            None,
        );
    }
    if request.id.is_empty()
        || request.account.id.is_empty()
        || request.line_items.is_empty()
        || request.subtotal.currency != request.total.currency
        || request.tax.currency != request.total.currency
        || request.subtotal.minor_units + request.tax.minor_units != request.total.minor_units
    {
        return error_result(
            CloudBillingInvoiceGenerateApiStatus::BadRequest,
            "invalid_invoice_request",
            "invoice request violates billing value contract",
            request_id,
            None,
        );
    }
    if request.account.regional_pack != request.regional_pack {
        return error_result(
            CloudBillingInvoiceGenerateApiStatus::BadRequest,
            "regional_pack_mismatch",
            "invoice regional pack must match billing account regional pack",
            request_id,
            None,
        );
    }
    let Some(expected_tax_invoice_format) =
        CloudBillingTaxInvoiceFormatPolicy::expected_for_regional_pack(&request.regional_pack)
    else {
        return error_result(
            CloudBillingInvoiceGenerateApiStatus::BadRequest,
            "unsupported_regional_pack",
            "regional pack does not declare a supported tax invoice format",
            request_id,
            None,
        );
    };
    if request.tax_invoice_format != expected_tax_invoice_format {
        return error_result(
            CloudBillingInvoiceGenerateApiStatus::BadRequest,
            "invalid_tax_invoice_format",
            "tax invoice format must match the billing regional pack",
            request_id,
            None,
        );
    }
    if request.account.state != "active" {
        return error_result(
            CloudBillingInvoiceGenerateApiStatus::Conflict,
            "billing_account_not_active",
            "invoice generation requires an active billing account",
            request_id,
            None,
        );
    }

    let record = CloudBillingInvoiceRecord {
        id: request.id,
        billing_account_id: request.account.id,
        tenant_id: request.tenant_id,
        regional_pack: request.regional_pack,
        period_start_epoch_seconds: request.period.start_epoch_seconds,
        period_end_epoch_seconds: request.period.end_epoch_seconds,
        line_item_count: request.line_items.len() as u32,
        currency: request.total.currency,
        subtotal_minor_units: request.subtotal.minor_units,
        tax_minor_units: request.tax.minor_units,
        total_minor_units: request.total.minor_units,
        tax_invoice_format: request.tax_invoice_format,
        tax_registration_id: request.tax_registration_id,
        state: "issued".to_owned(),
        issued_at_epoch_seconds: request.issued_at_epoch_seconds,
        due_at_epoch_seconds: request.due_at_epoch_seconds,
        data_class: request.data_class,
        schema_version: CLOUD_BILLING_INVOICE_SCHEMA_VERSION,
    };

    CloudBillingInvoiceGenerateApiResult {
        status: CloudBillingInvoiceGenerateApiStatus::Created,
        response: CloudBillingInvoiceGenerateApiResponse::Created(
            CloudBillingInvoiceGenerateSuccessResponse {
                data: record,
                metadata: CloudBillingTaxApiMetadata { request_id },
            },
        ),
    }
}

fn error_result(
    status: CloudBillingInvoiceGenerateApiStatus,
    code: &str,
    message: &str,
    request_id: String,
    retry_after_seconds: Option<u64>,
) -> CloudBillingInvoiceGenerateApiResult {
    CloudBillingInvoiceGenerateApiResult {
        status,
        response: CloudBillingInvoiceGenerateApiResponse::Error(CloudBillingTaxApiErrorResponse {
            error: CloudBillingTaxApiErrorBody {
                code: code.to_owned(),
                message: message.to_owned(),
                message_localized: None,
                request_id,
                details: Vec::new(),
                retry_after_seconds,
            },
        }),
    }
}
