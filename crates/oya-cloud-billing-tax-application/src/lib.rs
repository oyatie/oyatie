//! Cloud Billing tax app API boundary for invoice generation.
//!
//! This crate owns tenant/header/path/body normalization, idempotency, and
//! authenticated API projection before handing typed invoice generation requests
//! to the Cloud billing kernel.

use std::collections::BTreeMap;

use oya_cloud_billing_domain::{
    BillingAccount, BillingAccountCreate, BillingAccountState, BillingPeriod, CloudBillingError,
    CloudBillingLedger, Invoice, InvoiceGenerate, InvoiceLineItemCreate, InvoiceState, Money,
    TaxInvoiceFormat,
};
use oya_data_boundary_kernel::{DataClass, parse_data_class_label};
use oya_metering_domain::{MeterUnit, MeterUnitKind, MeteringError};

pub const CLOUD_BILLING_INVOICE_GENERATE_SURFACE: &str = "cloud.billing.invoice.generate";

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
    pub const fn code(self) -> u16 {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudBillingTaxApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathInvoiceIdEmpty,
    InvoiceIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    AccountStateInvalid,
    TaxInvoiceFormatInvalid,
    MeterUnitKindInvalid,
    DataClassInvalid,
    BillingInvalidRequest,
    BillingForbidden,
    BillingConflict,
}

impl CloudBillingTaxApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_BILLING_TAX_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_BILLING_TAX_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_BILLING_TAX_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_BILLING_TAX_PRINCIPAL_ID_EMPTY",
            Self::PathInvoiceIdEmpty => "CLOUD_BILLING_TAX_PATH_INVOICE_ID_EMPTY",
            Self::InvoiceIdMismatch => "CLOUD_BILLING_TAX_INVOICE_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_BILLING_TAX_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => {
                "CLOUD_BILLING_TAX_AUTHORIZATION_DECISION_ID_EMPTY"
            }
            Self::AuthorizationTenantMismatch => "CLOUD_BILLING_TAX_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "CLOUD_BILLING_TAX_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "CLOUD_BILLING_TAX_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_BILLING_TAX_IDEMPOTENCY_KEY_REUSED",
            Self::AccountStateInvalid => "CLOUD_BILLING_TAX_ACCOUNT_STATE_INVALID",
            Self::TaxInvoiceFormatInvalid => "CLOUD_BILLING_TAX_INVOICE_FORMAT_INVALID",
            Self::MeterUnitKindInvalid => "CLOUD_BILLING_TAX_METER_UNIT_KIND_INVALID",
            Self::DataClassInvalid => "CLOUD_BILLING_TAX_DATA_CLASS_INVALID",
            Self::BillingInvalidRequest => "CLOUD_BILLING_TAX_INVALID_REQUEST",
            Self::BillingForbidden => "CLOUD_BILLING_TAX_FORBIDDEN",
            Self::BillingConflict => "CLOUD_BILLING_TAX_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingTaxApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingTaxApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingTaxApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
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
pub struct CloudBillingTaxMeterUnitRequest {
    pub kind: String,             // data_class: INTERNAL_ONLY
    pub quantity_microunits: u64, // data_class: INTERNAL_ONLY
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
    pub tax_registration_id: String,                 // data_class: FINANCIAL_KR_신용정보
    pub issued_at_epoch_seconds: u64,                // data_class: INTERNAL_ONLY
    pub due_at_epoch_seconds: u64,                   // data_class: INTERNAL_ONLY
    pub data_class: String,                          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingInvoiceGenerateApiRequest {
    pub path_invoice_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudBillingTaxApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudBillingTaxApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudBillingTaxApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudBillingInvoiceGenerateRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudBillingInvoiceGenerateIdempotencyLedger {
    entries: BTreeMap<
        CloudBillingInvoiceGenerateIdempotencyLedgerKey,
        CloudBillingInvoiceGenerateIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl CloudBillingInvoiceGenerateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudBillingInvoiceGenerateIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudBillingInvoiceGenerateIdempotencyLedgerEntry {
    fingerprint: CloudBillingInvoiceGenerateRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudBillingInvoiceGenerateApiResult,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudBillingInvoiceGenerateRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudBillingInvoiceGenerateApiResult =
    Result<CloudBillingInvoiceGenerateSuccessResponse, CloudBillingTaxApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudBillingInvoiceGenerateSuccessResponse {
    pub data: CloudBillingInvoiceRecord, // data_class: INTERNAL_ONLY
    pub metadata: CloudBillingTaxApiMetadata, // data_class: INTERNAL_ONLY
}

impl CloudBillingInvoiceGenerateSuccessResponse {
    pub fn created(data: CloudBillingInvoiceRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudBillingTaxApiMetadata {
                request_id: request_id.into(),
            },
        }
    }
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
    pub tax_registration_id: String,     // data_class: FINANCIAL_KR_신용정보
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudBillingTaxApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathInvoiceId,
    InvoiceIdMismatch {
        path_invoice_id: String,
        body_invoice_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        body_tenant_id: String,
        account_tenant_id: String,
    },
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        principal_tenant_id: String,
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String,
        principal_id: String,
    },
    AuthorizationDenied {
        surface: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidAccountStateLabel {
        account_state: String,
    },
    InvalidTaxInvoiceFormatLabel {
        tax_invoice_format: String,
    },
    InvalidMeterUnitKindLabel {
        unit_kind: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    Billing(CloudBillingError),
}

impl CloudBillingTaxApiError {
    pub fn invoice_generate_status(&self) -> CloudBillingInvoiceGenerateApiStatus {
        match self.status_kind() {
            CloudBillingTaxApiStatusKind::BadRequest => {
                CloudBillingInvoiceGenerateApiStatus::BadRequest
            }
            CloudBillingTaxApiStatusKind::Unauthorized => {
                CloudBillingInvoiceGenerateApiStatus::Unauthorized
            }
            CloudBillingTaxApiStatusKind::Forbidden => {
                CloudBillingInvoiceGenerateApiStatus::Forbidden
            }
            CloudBillingTaxApiStatusKind::Conflict => {
                CloudBillingInvoiceGenerateApiStatus::Conflict
            }
            CloudBillingTaxApiStatusKind::UnprocessableEntity => {
                CloudBillingInvoiceGenerateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn invoice_generate_status_code(&self) -> u16 {
        self.invoice_generate_status().code()
    }

    pub fn code(&self) -> CloudBillingTaxApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudBillingTaxApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudBillingTaxApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudBillingTaxApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudBillingTaxApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathInvoiceId => CloudBillingTaxApiErrorCode::PathInvoiceIdEmpty,
            Self::InvoiceIdMismatch { .. } => CloudBillingTaxApiErrorCode::InvoiceIdMismatch,
            Self::TenantMismatch { .. } => CloudBillingTaxApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudBillingTaxApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudBillingTaxApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudBillingTaxApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudBillingTaxApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => CloudBillingTaxApiErrorCode::IdempotencyKeyReused,
            Self::InvalidAccountStateLabel { .. } => {
                CloudBillingTaxApiErrorCode::AccountStateInvalid
            }
            Self::InvalidTaxInvoiceFormatLabel { .. } => {
                CloudBillingTaxApiErrorCode::TaxInvoiceFormatInvalid
            }
            Self::InvalidMeterUnitKindLabel { .. } => {
                CloudBillingTaxApiErrorCode::MeterUnitKindInvalid
            }
            Self::InvalidDataClassLabel { .. } => CloudBillingTaxApiErrorCode::DataClassInvalid,
            Self::Billing(error) => match cloud_billing_status_kind(error) {
                CloudBillingTaxApiStatusKind::BadRequest => {
                    CloudBillingTaxApiErrorCode::BillingInvalidRequest
                }
                CloudBillingTaxApiStatusKind::Forbidden => {
                    CloudBillingTaxApiErrorCode::BillingForbidden
                }
                CloudBillingTaxApiStatusKind::Conflict => {
                    CloudBillingTaxApiErrorCode::BillingConflict
                }
                CloudBillingTaxApiStatusKind::Unauthorized
                | CloudBillingTaxApiStatusKind::UnprocessableEntity => {
                    CloudBillingTaxApiErrorCode::BillingInvalidRequest
                }
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudBillingTaxApiErrorResponse {
        CloudBillingTaxApiErrorResponse {
            error: CloudBillingTaxApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudBillingTaxApiStatusKind {
        match self {
            Self::EmptyPrincipalId => CloudBillingTaxApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudBillingTaxApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => CloudBillingTaxApiStatusKind::UnprocessableEntity,
            Self::Billing(error) => cloud_billing_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathInvoiceId
            | Self::InvoiceIdMismatch { .. }
            | Self::InvalidAccountStateLabel { .. }
            | Self::InvalidTaxInvoiceFormatLabel { .. }
            | Self::InvalidMeterUnitKindLabel { .. }
            | Self::InvalidDataClassLabel { .. } => CloudBillingTaxApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathInvoiceId => "Path invoice id is required",
            Self::InvoiceIdMismatch { .. } => "Path and body invoice ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, request body, and account snapshot"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud Billing invoice surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidAccountStateLabel { .. } => {
                "Account state must be active, suspended, or delinquent"
            }
            Self::InvalidTaxInvoiceFormatLabel { .. } => {
                "Tax invoice format must be a known Cloud Billing tax format"
            }
            Self::InvalidMeterUnitKindLabel { .. } => {
                "Meter unit kind must be a known platform metering unit"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::Billing(error) => cloud_billing_message(error),
        }
    }

    fn details(&self) -> Vec<CloudBillingTaxApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathInvoiceId => vec![detail("path.invoice_id", "must be non-empty")],
            Self::InvoiceIdMismatch { .. } => {
                vec![detail("id", "path invoice_id and body id must match")]
            }
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, body tenant_id, and account tenant_id must match",
            )],
            Self::EmptyAuthorizationDecisionId => vec![detail(
                "authorization.decision_id",
                "must be non-empty authorization evidence",
            )],
            Self::AuthorizationTenantMismatch { .. } => vec![detail(
                "authorization.tenant_id",
                "must match the authenticated principal tenant",
            )],
            Self::AuthorizationPrincipalMismatch { .. } => vec![detail(
                "authorization.principal_id",
                "must match the authenticated principal id",
            )],
            Self::AuthorizationDenied { .. } => vec![detail(
                "authorization.allowed_surfaces",
                "must include the requested Cloud Billing invoice surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidAccountStateLabel { .. } => vec![detail(
                "body.account.state",
                "must be active, suspended, or delinquent",
            )],
            Self::InvalidTaxInvoiceFormatLabel { .. } => vec![detail(
                "body.tax_invoice_format",
                "must be a supported regional tax invoice format label",
            )],
            Self::InvalidMeterUnitKindLabel { .. } => vec![detail(
                "body.line_items.units.kind",
                "must be a supported platform metering unit kind label",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::Billing(error) => vec![detail("cloud_billing", cloud_billing_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudBillingTaxApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cloud_billing_invoice_generate_request(
    request: &CloudBillingInvoiceGenerateApiRequest,
) -> Result<(), CloudBillingTaxApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_invoice_id(&request.path_invoice_id, &request.body.id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
        &request.body.account.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_BILLING_INVOICE_GENERATE_SURFACE,
    )
}

pub fn generate_cloud_billing_invoice_from_api(
    ledger: &mut CloudBillingLedger,
    idempotency_ledger: &mut CloudBillingInvoiceGenerateIdempotencyLedger,
    request: CloudBillingInvoiceGenerateApiRequest,
) -> Result<CloudBillingInvoiceGenerateSuccessResponse, CloudBillingTaxApiError> {
    validate_cloud_billing_invoice_generate_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_BILLING_INVOICE_GENERATE_SURFACE,
    );
    let fingerprint = invoice_generate_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudBillingTaxApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let body = request.body;
    let account_snapshot = body.account.clone();
    let result = account_create_input(account_snapshot)
        .and_then(|account| BillingAccount::new(account).map_err(CloudBillingTaxApiError::Billing))
        .and_then(|account| {
            invoice_generate_input(body).and_then(|input| {
                ledger
                    .generate_invoice(&account, input)
                    .map_err(CloudBillingTaxApiError::Billing)
            })
        })
        .map(|invoice| {
            CloudBillingInvoiceGenerateSuccessResponse::created(invoice_record(invoice), request_id)
        });
    idempotency_ledger.entries.insert(
        key,
        CloudBillingInvoiceGenerateIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(
    boundary: &CloudBillingTaxApiBoundaryContext,
) -> Result<(), CloudBillingTaxApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudBillingTaxApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudBillingTaxApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudBillingTaxApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_invoice_id(
    path_invoice_id: &str,
    body_invoice_id: &str,
) -> Result<(), CloudBillingTaxApiError> {
    if path_invoice_id.trim().is_empty() {
        return Err(CloudBillingTaxApiError::EmptyPathInvoiceId);
    }
    if path_invoice_id != body_invoice_id {
        return Err(CloudBillingTaxApiError::InvoiceIdMismatch {
            path_invoice_id: path_invoice_id.to_string(),
            body_invoice_id: body_invoice_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudBillingTaxApiBoundaryContext,
    principal: &CloudBillingTaxApiPrincipal,
    body_tenant_id: &str,
    account_tenant_id: &str,
) -> Result<(), CloudBillingTaxApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudBillingTaxApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id
        || boundary.tenant_id != body_tenant_id
        || boundary.tenant_id != account_tenant_id
    {
        return Err(CloudBillingTaxApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
            account_tenant_id: account_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudBillingTaxApiPrincipal,
    authorization: &CloudBillingTaxApiAuthorization,
    surface: &str,
) -> Result<(), CloudBillingTaxApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudBillingTaxApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudBillingTaxApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudBillingTaxApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudBillingTaxApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn account_create_input(
    input: CloudBillingAccountSnapshotRequest,
) -> Result<BillingAccountCreate, CloudBillingTaxApiError> {
    Ok(BillingAccountCreate {
        id: input.id,
        tenant_id: input.tenant_id,
        region: input.region,
        regional_pack: input.regional_pack,
        payment_method: input.payment_method,
        credit_balance: money_input(input.credit_balance)?,
        state: parse_account_state(input.state)?,
        data_class: parse_api_data_class(input.data_class)?,
        created_at_epoch_seconds: input.created_at_epoch_seconds,
    })
}

fn invoice_generate_input(
    input: CloudBillingInvoiceGenerateRequest,
) -> Result<InvoiceGenerate, CloudBillingTaxApiError> {
    Ok(InvoiceGenerate {
        id: input.id,
        billing_account_id: input.account.id,
        tenant_id: input.tenant_id,
        regional_pack: input.regional_pack,
        period: billing_period_input(input.period)?,
        line_items: input
            .line_items
            .into_iter()
            .map(invoice_line_item_input)
            .collect::<Result<Vec<_>, _>>()?,
        subtotal: money_input(input.subtotal)?,
        tax: money_input(input.tax)?,
        total: money_input(input.total)?,
        tax_invoice_format: parse_tax_invoice_format(input.tax_invoice_format)?,
        tax_registration_id: input.tax_registration_id,
        issued_at_epoch_seconds: input.issued_at_epoch_seconds,
        due_at_epoch_seconds: input.due_at_epoch_seconds,
        data_class: parse_api_data_class(input.data_class)?,
    })
}

fn invoice_line_item_input(
    input: CloudBillingInvoiceLineItemCreateRequest,
) -> Result<InvoiceLineItemCreate, CloudBillingTaxApiError> {
    Ok(InvoiceLineItemCreate {
        id: input.id,
        resource_id: input.resource_id,
        description: input.description,
        units: input
            .units
            .into_iter()
            .map(meter_unit_input)
            .collect::<Result<Vec<_>, _>>()?,
        subtotal: money_input(input.subtotal)?,
        data_class: parse_api_data_class(input.data_class)?,
    })
}

fn money_input(input: CloudBillingMoneyRequest) -> Result<Money, CloudBillingTaxApiError> {
    Money::new(input.currency, input.minor_units).map_err(CloudBillingTaxApiError::Billing)
}

fn billing_period_input(
    input: CloudBillingPeriodRequest,
) -> Result<BillingPeriod, CloudBillingTaxApiError> {
    BillingPeriod::new(input.start_epoch_seconds, input.end_epoch_seconds)
        .map_err(CloudBillingTaxApiError::Billing)
}

fn meter_unit_input(
    input: CloudBillingTaxMeterUnitRequest,
) -> Result<MeterUnit, CloudBillingTaxApiError> {
    MeterUnit::new(
        parse_meter_unit_kind(input.kind)?,
        input.quantity_microunits,
    )
    .map_err(|error| CloudBillingTaxApiError::Billing(CloudBillingError::MeteringRejected(error)))
}

fn parse_account_state(label: String) -> Result<BillingAccountState, CloudBillingTaxApiError> {
    match label.as_str() {
        "active" => Ok(BillingAccountState::Active),
        "suspended" => Ok(BillingAccountState::Suspended),
        "delinquent" => Ok(BillingAccountState::Delinquent),
        _ => Err(CloudBillingTaxApiError::InvalidAccountStateLabel {
            account_state: label,
        }),
    }
}

fn parse_tax_invoice_format(label: String) -> Result<TaxInvoiceFormat, CloudBillingTaxApiError> {
    match label.as_str() {
        "kr_electronic_tax_invoice" => Ok(TaxInvoiceFormat::KrElectronicTaxInvoice),
        "jp_qualified_invoice" => Ok(TaxInvoiceFormat::JpQualifiedInvoice),
        "eu_country_e_invoice" => Ok(TaxInvoiceFormat::EuCountryEInvoice),
        "in_gst" => Ok(TaxInvoiceFormat::InGst),
        "br_nfe" => Ok(TaxInvoiceFormat::BrNfe),
        "ksa_fatoora" => Ok(TaxInvoiceFormat::KsaFatoora),
        "uae_vat" => Ok(TaxInvoiceFormat::UaeVat),
        _ => Err(CloudBillingTaxApiError::InvalidTaxInvoiceFormatLabel {
            tax_invoice_format: label,
        }),
    }
}

fn parse_meter_unit_kind(label: String) -> Result<MeterUnitKind, CloudBillingTaxApiError> {
    match label.as_str() {
        "request" => Ok(MeterUnitKind::Request),
        "byte_in" => Ok(MeterUnitKind::ByteIn),
        "byte_out" => Ok(MeterUnitKind::ByteOut),
        "millisecond" => Ok(MeterUnitKind::Millisecond),
        "gpu_second" => Ok(MeterUnitKind::GpuSecond),
        "llm_token" => Ok(MeterUnitKind::LlmToken),
        "resource_second" => Ok(MeterUnitKind::ResourceSecond),
        "storage_gb_second" => Ok(MeterUnitKind::StorageGbSecond),
        "egress_gb" => Ok(MeterUnitKind::EgressGb),
        _ => Err(CloudBillingTaxApiError::InvalidMeterUnitKindLabel { unit_kind: label }),
    }
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudBillingTaxApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudBillingTaxApiError::InvalidDataClassLabel { data_class: label })
}

fn idempotency_key_for(
    boundary: &CloudBillingTaxApiBoundaryContext,
    principal: &CloudBillingTaxApiPrincipal,
    surface: &str,
) -> CloudBillingInvoiceGenerateIdempotencyLedgerKey {
    CloudBillingInvoiceGenerateIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn invoice_generate_fingerprint_for(
    request: &CloudBillingInvoiceGenerateApiRequest,
) -> CloudBillingInvoiceGenerateRequestFingerprint {
    CloudBillingInvoiceGenerateRequestFingerprint {
        canonical: [
            format!("path.invoice_id={}", request.path_invoice_id),
            format!("header.tenant_id={}", request.boundary.tenant_id),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!(
                "authorization.tenant_id={}",
                request.authorization.tenant_id
            ),
            format!(
                "authorization.principal_id={}",
                request.authorization.principal_id
            ),
            format!(
                "authorization.decision_id={}",
                request.authorization.decision_id
            ),
            format!(
                "authorization.allowed_surfaces={}",
                request.authorization.allowed_surfaces.join(",")
            ),
            format!("body.id={}", request.body.id),
            format!("body.account={:?}", request.body.account),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.regional_pack={}", request.body.regional_pack),
            format!("body.period={:?}", request.body.period),
            format!("body.line_items={:?}", request.body.line_items),
            format!("body.subtotal={:?}", request.body.subtotal),
            format!("body.tax={:?}", request.body.tax),
            format!("body.total={:?}", request.body.total),
            format!(
                "body.tax_invoice_format={}",
                request.body.tax_invoice_format
            ),
            format!(
                "body.tax_registration_id={}",
                request.body.tax_registration_id
            ),
            format!(
                "body.issued_at_epoch_seconds={}",
                request.body.issued_at_epoch_seconds
            ),
            format!(
                "body.due_at_epoch_seconds={}",
                request.body.due_at_epoch_seconds
            ),
            format!("body.data_class={}", request.body.data_class),
        ]
        .join("|"),
    }
}

fn invoice_record(invoice: Invoice) -> CloudBillingInvoiceRecord {
    CloudBillingInvoiceRecord {
        id: invoice.id.value.value,
        billing_account_id: invoice.billing_account_id.value.value,
        tenant_id: invoice.tenant_id.value,
        regional_pack: invoice.regional_pack.value,
        period_start_epoch_seconds: invoice.period.value.start_epoch_seconds,
        period_end_epoch_seconds: invoice.period.value.end_epoch_seconds,
        line_item_count: invoice.line_items.value.len() as u32,
        currency: invoice.total.value.currency.value,
        subtotal_minor_units: invoice.subtotal.value.minor_units,
        tax_minor_units: invoice.tax.value.minor_units,
        total_minor_units: invoice.total.value.minor_units,
        tax_invoice_format: tax_invoice_format_label(invoice.tax_invoice_format.value).to_string(),
        tax_registration_id: invoice.tax_registration_id.value.value,
        state: invoice_state_label(invoice.state.value).to_string(),
        issued_at_epoch_seconds: invoice.issued_at_epoch_seconds.value,
        due_at_epoch_seconds: invoice.due_at_epoch_seconds.value,
        data_class: invoice.data_class.value.label().to_string(),
        schema_version: invoice.schema_version.value,
    }
}

fn tax_invoice_format_label(format: TaxInvoiceFormat) -> &'static str {
    match format {
        TaxInvoiceFormat::KrElectronicTaxInvoice => "kr_electronic_tax_invoice",
        TaxInvoiceFormat::JpQualifiedInvoice => "jp_qualified_invoice",
        TaxInvoiceFormat::EuCountryEInvoice => "eu_country_e_invoice",
        TaxInvoiceFormat::InGst => "in_gst",
        TaxInvoiceFormat::BrNfe => "br_nfe",
        TaxInvoiceFormat::KsaFatoora => "ksa_fatoora",
        TaxInvoiceFormat::UaeVat => "uae_vat",
    }
}

fn invoice_state_label(state: InvoiceState) -> &'static str {
    match state {
        InvoiceState::Issued => "issued",
        InvoiceState::Paid => "paid",
        InvoiceState::Overdue => "overdue",
        InvoiceState::Void => "void",
    }
}

fn cloud_billing_status_kind(error: &CloudBillingError) -> CloudBillingTaxApiStatusKind {
    match error {
        CloudBillingError::DuplicateBillingEvent | CloudBillingError::DuplicateInvoice => {
            CloudBillingTaxApiStatusKind::Conflict
        }
        CloudBillingError::BillingAccountInactive
        | CloudBillingError::TenantMismatch
        | CloudBillingError::RegionMismatch => CloudBillingTaxApiStatusKind::Forbidden,
        CloudBillingError::MeteringRejected(MeteringError::DuplicateMeterEvent) => {
            CloudBillingTaxApiStatusKind::Conflict
        }
        CloudBillingError::InvalidBillingAccountId
        | CloudBillingError::InvalidCloudBillingEventId
        | CloudBillingError::InvalidInvoiceId
        | CloudBillingError::InvalidInvoiceLineItemId
        | CloudBillingError::InvalidTaxRegistrationId
        | CloudBillingError::InvalidTenantId
        | CloudBillingError::InvalidPaymentMethodRef
        | CloudBillingError::InvalidRateCardRef
        | CloudBillingError::InvalidRegionalPack
        | CloudBillingError::InvalidCurrencyCode
        | CloudBillingError::InvalidResourceId
        | CloudBillingError::InvalidMeteringTag
        | CloudBillingError::InvalidOccurredAt
        | CloudBillingError::InvalidBillingPeriod
        | CloudBillingError::InvalidInvoiceLineItem
        | CloudBillingError::InvalidInvoiceTotal
        | CloudBillingError::InvalidTaxInvoiceFormat
        | CloudBillingError::InvalidDataClass
        | CloudBillingError::MeteringRejected(_) => CloudBillingTaxApiStatusKind::BadRequest,
    }
}

fn cloud_billing_message(error: &CloudBillingError) -> &'static str {
    match cloud_billing_status_kind(error) {
        CloudBillingTaxApiStatusKind::BadRequest => "Cloud Billing rejected the request shape",
        CloudBillingTaxApiStatusKind::Unauthorized => "Cloud Billing authentication is required",
        CloudBillingTaxApiStatusKind::Forbidden => "Cloud Billing policy denied the request",
        CloudBillingTaxApiStatusKind::Conflict => "Cloud Billing invoice already exists",
        CloudBillingTaxApiStatusKind::UnprocessableEntity => {
            "Cloud Billing rejected request idempotency"
        }
    }
}

fn cloud_billing_issue(error: &CloudBillingError) -> &'static str {
    match error {
        CloudBillingError::InvalidBillingAccountId => "billing account id must use the ba_ prefix",
        CloudBillingError::InvalidCloudBillingEventId => {
            "billing event id must use the cbill_ prefix"
        }
        CloudBillingError::InvalidInvoiceId => "invoice id must use the inv_ prefix",
        CloudBillingError::InvalidInvoiceLineItemId => {
            "invoice line item id must use the ili_ prefix"
        }
        CloudBillingError::InvalidTaxRegistrationId => {
            "tax registration id must match the selected regional tax format"
        }
        CloudBillingError::InvalidTenantId => "tenant_id must use the ten_ prefix",
        CloudBillingError::InvalidPaymentMethodRef => "payment method must use the pm_ prefix",
        CloudBillingError::InvalidRateCardRef => "rate card reference must use the rate/ prefix",
        CloudBillingError::InvalidRegionalPack => "regional pack must be supported for tax format",
        CloudBillingError::InvalidCurrencyCode => "currency must be a three-letter uppercase code",
        CloudBillingError::InvalidResourceId => "resource id must be canonical cloud resource id",
        CloudBillingError::InvalidMeteringTag => "metering tag must match tenant and resource kind",
        CloudBillingError::InvalidOccurredAt => "occurred_at_epoch_seconds must be non-zero",
        CloudBillingError::InvalidBillingPeriod => "billing period and due dates must be ordered",
        CloudBillingError::InvalidInvoiceLineItem => {
            "invoice line items must be non-empty and well-formed"
        }
        CloudBillingError::InvalidInvoiceTotal => "invoice subtotal plus tax must equal total",
        CloudBillingError::InvalidTaxInvoiceFormat => {
            "tax invoice format must match the regional pack"
        }
        CloudBillingError::InvalidDataClass => "invoice data must use a financial data class",
        CloudBillingError::BillingAccountInactive => "billing account must be active",
        CloudBillingError::TenantMismatch => "tenant values must match across account and invoice",
        CloudBillingError::RegionMismatch => "region values must match regional identifiers",
        CloudBillingError::DuplicateBillingEvent => "billing event is already present",
        CloudBillingError::DuplicateInvoice => "invoice id is already present",
        CloudBillingError::MeteringRejected(error) => match error {
            MeteringError::InvalidMeterEventId => "meter event id must use the mtr_ prefix",
            MeteringError::InvalidTenantId => "meter tenant id must use the ten_ prefix",
            MeteringError::InvalidCapabilityId => "meter capability id must use the cap. prefix",
            MeteringError::EmptyUnits => "meter units must be non-empty",
            MeteringError::InvalidUnitQuantity => "meter unit quantity must be non-zero",
            MeteringError::DuplicateUnitKind => "meter units must not repeat unit kinds",
            MeteringError::InvalidRecordedAt => "meter recorded_at must be non-zero",
            MeteringError::InvalidIdempotencyKey => {
                "meter idempotency key must use the idem_ prefix"
            }
            MeteringError::InvalidDataClass => "meter data_class must be public metadata",
            MeteringError::DuplicateMeterEvent => "meter event is already present",
        },
    }
}

fn detail(field: &str, issue: &str) -> CloudBillingTaxApiErrorDetail {
    CloudBillingTaxApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
