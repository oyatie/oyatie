//! Cloud Observability audit read API boundary.
//!
//! This crate owns tenant/header/body normalization, authentication and
//! authorization evidence checks, fail-closed label parsing, and typed REST
//! projection for the `cloud.observability.audit.read` surface before handing
//! immutable reads to the Cloud observability kernel.

use oya_audit_chain_domain::Plane;
use oya_cloud_observability_domain::{
    AuditReadRequest, AuditReadScope, CloudAuditOperation, CloudAuditRecord, CloudAuditTopic,
    CloudObservabilityCatalog, CloudObservabilityError,
};
use oya_data_boundary_kernel::{OperationalDataClass, Purpose};

pub const CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE: &str = "cloud.observability.audit.read";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudObservabilityAuditReadApiStatus {
    Ok,
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

impl CloudObservabilityAuditReadApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudObservabilityApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    PrincipalMissing,
    PrincipalIdEmpty,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    AuditScopeInvalid,
    AuditTopicInvalid,
    ObservabilityInvalidRequest,
    ObservabilityForbidden,
    ObservabilityUnprocessable,
}

impl CloudObservabilityApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_OBSERVABILITY_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_OBSERVABILITY_TENANT_HEADER_EMPTY",
            Self::PrincipalMissing => "CLOUD_OBSERVABILITY_PRINCIPAL_MISSING",
            Self::PrincipalIdEmpty => "CLOUD_OBSERVABILITY_PRINCIPAL_ID_EMPTY",
            Self::TenantMismatch => "CLOUD_OBSERVABILITY_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => {
                "CLOUD_OBSERVABILITY_AUTHORIZATION_DECISION_ID_EMPTY"
            }
            Self::AuthorizationTenantMismatch => {
                "CLOUD_OBSERVABILITY_AUTHORIZATION_TENANT_MISMATCH"
            }
            Self::AuthorizationPrincipalMismatch => {
                "CLOUD_OBSERVABILITY_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "CLOUD_OBSERVABILITY_AUTHORIZATION_DENIED",
            Self::AuditScopeInvalid => "CLOUD_OBSERVABILITY_AUDIT_SCOPE_INVALID",
            Self::AuditTopicInvalid => "CLOUD_OBSERVABILITY_AUDIT_TOPIC_INVALID",
            Self::ObservabilityInvalidRequest => "CLOUD_OBSERVABILITY_INVALID_REQUEST",
            Self::ObservabilityForbidden => "CLOUD_OBSERVABILITY_FORBIDDEN",
            Self::ObservabilityUnprocessable => "CLOUD_OBSERVABILITY_UNPROCESSABLE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiBoundaryContext {
    pub request_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditReadTopicRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditReadRequest {
    pub tenant_id: String,        // data_class: INTERNAL_ONLY
    pub region: String,           // data_class: PUBLIC
    pub cell_id: Option<String>,  // data_class: PUBLIC
    pub scope: String,            // data_class: INTERNAL_ONLY
    pub start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub topics: Vec<CloudObservabilityAuditReadTopicRef>, // data_class: INTERNAL_ONLY
    pub actor: Option<String>,    // data_class: INTERNAL_ONLY
    pub resource_id: Option<String>, // data_class: INTERNAL_ONLY
    pub cursor: Option<String>,   // data_class: INTERNAL_ONLY
    pub page_size: Option<u16>,   // data_class: INTERNAL_ONLY
    pub require_complete_chain: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditReadApiRequest {
    pub boundary: CloudObservabilityApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: Option<CloudObservabilityApiPrincipal>, // data_class: INTERNAL_ONLY
    pub authorization: CloudObservabilityApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudObservabilityAuditReadRequest,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditReadSuccessResponse {
    pub data: Vec<CloudObservabilityAuditRecord>, // data_class: INTERNAL_ONLY
    pub metadata: CloudObservabilityAuditReadMetadata, // data_class: INTERNAL_ONLY
}

impl CloudObservabilityAuditReadSuccessResponse {
    pub fn ok(
        data: Vec<CloudObservabilityAuditRecord>,
        metadata: CloudObservabilityAuditReadMetadata,
    ) -> Self {
        Self { data, metadata }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditReadMetadata {
    pub request_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: PUBLIC
    pub record_count: u32,                    // data_class: INTERNAL_ONLY
    pub next_cursor: Option<String>,          // data_class: INTERNAL_ONLY
    pub chain_complete: bool,                 // data_class: INTERNAL_ONLY
    pub high_watermark_sequence: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityAuditRecord {
    pub id: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: PUBLIC
    pub cell_id: Option<String>,            // data_class: PUBLIC
    pub topic: String,                      // data_class: INTERNAL_ONLY
    pub operation: String,                  // data_class: INTERNAL_ONLY
    pub record_class: String,               // data_class: INTERNAL_ONLY
    pub source_resource_id: Option<String>, // data_class: INTERNAL_ONLY
    pub actor: String,                      // data_class: INTERNAL_ONLY
    pub iam_role: Option<String>,           // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub chain_sequence: u64,                // data_class: INTERNAL_ONLY
    pub previous_hash: String,              // data_class: INTERNAL_ONLY
    pub hash: String,                       // data_class: INTERNAL_ONLY
    pub payload_hash: String,               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,            // data_class: INTERNAL_ONLY
    pub decision: String,                   // data_class: INTERNAL_ONLY
    pub purpose: String,                    // data_class: INTERNAL_ONLY
    pub plane: String,                      // data_class: INTERNAL_ONLY
    pub data_classes_referenced: Vec<CloudObservabilityDataClassRef>, // data_class: INTERNAL_ONLY
    pub signed_export_uri: String,          // data_class: INTERNAL_ONLY
    pub audit_marker: String,               // data_class: INTERNAL_ONLY
    pub schema_version: u32,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityDataClassRef {
    pub label: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiErrorResponse {
    pub error: CloudObservabilityApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiErrorBody {
    pub code: String,                                   // data_class: INTERNAL_ONLY
    pub message: String,                                // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,              // data_class: INTERNAL_ONLY
    pub request_id: String,                             // data_class: INTERNAL_ONLY
    pub details: Vec<CloudObservabilityApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudObservabilityApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudObservabilityApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    MissingPrincipal,
    EmptyPrincipalId,
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: Option<String>,
        body_tenant_id: String,
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
    InvalidAuditScopeLabel {
        scope: String,
    },
    InvalidAuditTopicLabel {
        topic: String,
    },
    Observability(CloudObservabilityError),
}

impl CloudObservabilityApiError {
    pub fn audit_read_status(&self) -> CloudObservabilityAuditReadApiStatus {
        match self.status_kind() {
            CloudObservabilityApiStatusKind::BadRequest => {
                CloudObservabilityAuditReadApiStatus::BadRequest
            }
            CloudObservabilityApiStatusKind::Unauthorized => {
                CloudObservabilityAuditReadApiStatus::Unauthorized
            }
            CloudObservabilityApiStatusKind::Forbidden => {
                CloudObservabilityAuditReadApiStatus::Forbidden
            }
            CloudObservabilityApiStatusKind::UnprocessableEntity => {
                CloudObservabilityAuditReadApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn status_code(&self) -> u16 {
        self.audit_read_status().code()
    }

    pub fn code(&self) -> CloudObservabilityApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudObservabilityApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudObservabilityApiErrorCode::TenantHeaderEmpty,
            Self::MissingPrincipal => CloudObservabilityApiErrorCode::PrincipalMissing,
            Self::EmptyPrincipalId => CloudObservabilityApiErrorCode::PrincipalIdEmpty,
            Self::TenantMismatch { .. } => CloudObservabilityApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudObservabilityApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudObservabilityApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudObservabilityApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudObservabilityApiErrorCode::AuthorizationDenied,
            Self::InvalidAuditScopeLabel { .. } => {
                CloudObservabilityApiErrorCode::AuditScopeInvalid
            }
            Self::InvalidAuditTopicLabel { .. } => {
                CloudObservabilityApiErrorCode::AuditTopicInvalid
            }
            Self::Observability(error) => match observability_status_kind(error) {
                CloudObservabilityApiStatusKind::BadRequest => {
                    CloudObservabilityApiErrorCode::ObservabilityInvalidRequest
                }
                CloudObservabilityApiStatusKind::Forbidden => {
                    CloudObservabilityApiErrorCode::ObservabilityForbidden
                }
                CloudObservabilityApiStatusKind::UnprocessableEntity => {
                    CloudObservabilityApiErrorCode::ObservabilityUnprocessable
                }
                CloudObservabilityApiStatusKind::Unauthorized => {
                    CloudObservabilityApiErrorCode::ObservabilityInvalidRequest
                }
            },
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> CloudObservabilityApiErrorResponse {
        CloudObservabilityApiErrorResponse {
            error: CloudObservabilityApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudObservabilityApiStatusKind {
        match self {
            Self::MissingPrincipal | Self::EmptyPrincipalId => {
                CloudObservabilityApiStatusKind::Unauthorized
            }
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudObservabilityApiStatusKind::Forbidden,
            Self::Observability(error) => observability_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::InvalidAuditScopeLabel { .. }
            | Self::InvalidAuditTopicLabel { .. } => CloudObservabilityApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::MissingPrincipal => "Authenticated principal evidence is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal id"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud Observability surface"
            }
            Self::InvalidAuditScopeLabel { .. } => "Audit read scope label is not supported",
            Self::InvalidAuditTopicLabel { .. } => "Audit topic label is not supported",
            Self::Observability(error) => observability_message(error),
        }
    }

    fn details(&self) -> Vec<CloudObservabilityApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::MissingPrincipal => vec![detail("principal", "must be present")],
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and request body tenant must match",
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
                "must include the Cloud Observability audit read surface",
            )],
            Self::InvalidAuditScopeLabel { .. } => vec![detail(
                "body.scope",
                "must be control_plane_mutations or all_tenant_audit",
            )],
            Self::InvalidAuditTopicLabel { .. } => {
                vec![detail(
                    "body.topics",
                    "must use a supported audit topic label",
                )]
            }
            Self::Observability(error) => {
                vec![detail("cloud_observability", observability_issue(error))]
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudObservabilityApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

pub fn read_cloud_observability_audit_from_api(
    catalog: &CloudObservabilityCatalog,
    request: CloudObservabilityAuditReadApiRequest,
) -> Result<CloudObservabilityAuditReadSuccessResponse, CloudObservabilityApiError> {
    validate_boundary(&request.boundary)?;
    let principal = request
        .principal
        .as_ref()
        .ok_or(CloudObservabilityApiError::MissingPrincipal)?;
    validate_tenant_binding(&request.boundary, principal, &request.body.tenant_id)?;
    validate_authorization(
        principal,
        &request.authorization,
        CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE,
    )?;

    let request_id = request.boundary.request_id.clone();
    let tenant_id = request.body.tenant_id.clone();
    let region = request.body.region.clone();
    let kernel_request = audit_read_request(request.body)?;
    let result = catalog
        .read_audit(kernel_request)
        .map_err(CloudObservabilityApiError::Observability)?;
    let data = result.records.iter().map(audit_record).collect::<Vec<_>>();
    let record_count = data.len() as u32;
    Ok(CloudObservabilityAuditReadSuccessResponse::ok(
        data,
        CloudObservabilityAuditReadMetadata {
            request_id,
            tenant_id,
            region,
            record_count,
            next_cursor: result.next_cursor.map(|cursor| cursor.value),
            chain_complete: result.chain_complete,
            high_watermark_sequence: result.high_watermark_sequence,
        },
    ))
}

fn validate_boundary(
    boundary: &CloudObservabilityApiBoundaryContext,
) -> Result<(), CloudObservabilityApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudObservabilityApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudObservabilityApiError::EmptyTenantHeader);
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudObservabilityApiBoundaryContext,
    principal: &CloudObservabilityApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudObservabilityApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudObservabilityApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body_tenant_id {
        return Err(CloudObservabilityApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: Some(principal.tenant_id.clone()),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudObservabilityApiPrincipal,
    authorization: &CloudObservabilityApiAuthorization,
    surface: &str,
) -> Result<(), CloudObservabilityApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudObservabilityApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudObservabilityApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudObservabilityApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudObservabilityApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn audit_read_request(
    input: CloudObservabilityAuditReadRequest,
) -> Result<AuditReadRequest, CloudObservabilityApiError> {
    Ok(AuditReadRequest {
        tenant_id: input.tenant_id,
        region: input.region,
        cell_id: input.cell_id,
        scope: parse_audit_scope(&input.scope)?,
        start_epoch_seconds: input.start_epoch_seconds,
        end_epoch_seconds: input.end_epoch_seconds,
        topics: input
            .topics
            .into_iter()
            .map(|topic| parse_audit_topic(&topic.value))
            .collect::<Result<Vec<_>, _>>()?,
        actor: input.actor,
        resource_id: input.resource_id,
        cursor: input.cursor,
        page_size: input.page_size,
        require_complete_chain: input.require_complete_chain,
    })
}

fn parse_audit_scope(value: &str) -> Result<AuditReadScope, CloudObservabilityApiError> {
    match value.trim() {
        "control_plane_mutations" => Ok(AuditReadScope::ControlPlaneMutations),
        "all_tenant_audit" => Ok(AuditReadScope::AllTenantAudit),
        _ => Err(CloudObservabilityApiError::InvalidAuditScopeLabel {
            scope: value.to_string(),
        }),
    }
}

fn parse_audit_topic(value: &str) -> Result<CloudAuditTopic, CloudObservabilityApiError> {
    match value.trim() {
        "oya.audit.cloud_resource_created" => Ok(CloudAuditTopic::CloudResourceCreated),
        "oya.audit.cloud_resource_terminated" => Ok(CloudAuditTopic::CloudResourceTerminated),
        "oya.audit.cloud_iam_assume" => Ok(CloudAuditTopic::CloudIamAssume),
        "oya.audit.cloud_iam_policy" => Ok(CloudAuditTopic::CloudIamPolicy),
        "oya.audit.cloud_region_register" => Ok(CloudAuditTopic::CloudRegionRegister),
        "oya.audit.cloud_kms_use" => Ok(CloudAuditTopic::CloudKmsUse),
        "oya.audit.cloud_replication" => Ok(CloudAuditTopic::CloudReplication),
        "oya.audit.cloud_flow_anomaly" => Ok(CloudAuditTopic::CloudFlowAnomaly),
        "oya.audit.cloud_invoice" => Ok(CloudAuditTopic::CloudInvoice),
        "oya.audit.cloud_interconnect" => Ok(CloudAuditTopic::CloudInterconnect),
        "oya.audit.cloud_cell_rebalanced" => Ok(CloudAuditTopic::CloudCellRebalanced),
        _ => Err(CloudObservabilityApiError::InvalidAuditTopicLabel {
            topic: value.to_string(),
        }),
    }
}

fn audit_record(record: &CloudAuditRecord) -> CloudObservabilityAuditRecord {
    CloudObservabilityAuditRecord {
        id: record.id.value.value.clone(),
        tenant_id: record.tenant_id.value.clone(),
        region: record.region.value.value.clone(),
        cell_id: record
            .cell_id
            .value
            .as_ref()
            .map(|cell_id| cell_id.value.clone()),
        topic: record.topic.value.as_str().to_string(),
        operation: audit_operation_label(record.operation.value).to_string(),
        record_class: audit_record_class_label(record.record_class.value).to_string(),
        source_resource_id: record
            .source_resource_id
            .value
            .as_ref()
            .map(|resource_id| resource_id.value.clone()),
        actor: record.actor.value.value.clone(),
        iam_role: record
            .iam_role
            .value
            .as_ref()
            .map(|role| role.value.clone()),
        occurred_at_epoch_seconds: record.occurred_at_epoch_seconds.value,
        chain_sequence: record.chain_sequence.value,
        previous_hash: record.previous_hash.value.value.clone(),
        hash: record.hash.value.value.clone(),
        payload_hash: record.payload_hash.value.value.clone(),
        idempotency_key: record.idempotency_key.value.value.clone(),
        decision: record.decision.value.clone(),
        purpose: purpose_label(record.purpose.value).to_string(),
        plane: plane_label(record.plane.value).to_string(),
        data_classes_referenced: record
            .data_classes_referenced
            .value
            .iter()
            .map(|classification| CloudObservabilityDataClassRef {
                label: classification.label().to_string(),
            })
            .collect(),
        signed_export_uri: record.signed_export_uri.value.value.clone(),
        audit_marker: operational_data_class_label(record.audit_marker.value).to_string(),
        schema_version: record.schema_version.value,
    }
}

fn audit_operation_label(operation: CloudAuditOperation) -> &'static str {
    match operation {
        CloudAuditOperation::ResourceCreated => "resource_created",
        CloudAuditOperation::ResourceTerminated => "resource_terminated",
        CloudAuditOperation::IamRoleAssumed => "iam_role_assumed",
        CloudAuditOperation::IamPolicyChanged => "iam_policy_changed",
        CloudAuditOperation::RegionRegistered => "region_registered",
        CloudAuditOperation::KmsKeyUsed => "kms_key_used",
        CloudAuditOperation::CrossRegionReplication => "cross_region_replication",
        CloudAuditOperation::NetworkFlowAnomaly => "network_flow_anomaly",
        CloudAuditOperation::InvoiceIssued => "invoice_issued",
        CloudAuditOperation::DirectInterconnectProvisioned => "direct_interconnect_provisioned",
        CloudAuditOperation::CellRebalanced => "cell_rebalanced",
    }
}

fn audit_record_class_label(
    record_class: oya_cloud_observability_domain::AuditRecordClass,
) -> &'static str {
    match record_class {
        oya_cloud_observability_domain::AuditRecordClass::ControlPlaneMutation => {
            "control_plane_mutation"
        }
        oya_cloud_observability_domain::AuditRecordClass::DataPlaneSecurity => {
            "data_plane_security"
        }
        oya_cloud_observability_domain::AuditRecordClass::BillingAnalytics => "billing_analytics",
        oya_cloud_observability_domain::AuditRecordClass::Replication => "replication",
        oya_cloud_observability_domain::AuditRecordClass::CapacityOperations => {
            "capacity_operations"
        }
    }
}

fn plane_label(plane: Plane) -> &'static str {
    match plane {
        Plane::Control => "control",
        Plane::Data => "data",
        Plane::Audit => "audit",
        Plane::Analytics => "analytics",
    }
}

fn purpose_label(purpose: Purpose) -> &'static str {
    match purpose {
        Purpose::CoreService => "core_service",
        Purpose::CapabilityInvocation => "capability_invocation",
        Purpose::SearchIndex => "search_index",
        Purpose::AdsTargeting => "ads_targeting",
        Purpose::Analytics => "analytics",
        Purpose::Support => "support",
        Purpose::TenantAnalyticsFirstParty => "tenant_analytics_first_party",
        Purpose::CrossTenantAggregateAnonymous => "cross_tenant_aggregate_anonymous",
        Purpose::PersonalizationInProduct => "personalization_in_product",
        Purpose::SearchIndexPrivate => "search_index_private",
        Purpose::SearchIndexPublic => "search_index_public",
        Purpose::AdTargetingDeclared => "ad_targeting_declared",
        Purpose::AdTargetingBehavioral => "ad_targeting_behavioral",
        Purpose::ModelTrainingOya => "model_training_oya",
        Purpose::ModelTrainingThirdParty => "model_training_third_party",
    }
}

fn operational_data_class_label(data_class: OperationalDataClass) -> &'static str {
    match data_class {
        OperationalDataClass::Audit => "AUDIT",
        OperationalDataClass::Secret => "SECRET",
    }
}

fn observability_status_kind(error: &CloudObservabilityError) -> CloudObservabilityApiStatusKind {
    match error {
        CloudObservabilityError::ResourceTenantMismatch
        | CloudObservabilityError::ResourceRegionMismatch => {
            CloudObservabilityApiStatusKind::Forbidden
        }
        CloudObservabilityError::InvalidCursor
        | CloudObservabilityError::CursorTenantMismatch
        | CloudObservabilityError::CursorRegionMismatch
        | CloudObservabilityError::IncompleteAuditChain
        | CloudObservabilityError::UnverifiedAuditChain => {
            CloudObservabilityApiStatusKind::UnprocessableEntity
        }
        _ => CloudObservabilityApiStatusKind::BadRequest,
    }
}

fn observability_message(error: &CloudObservabilityError) -> &'static str {
    match observability_status_kind(error) {
        CloudObservabilityApiStatusKind::BadRequest => {
            "Cloud Observability rejected the audit read request shape"
        }
        CloudObservabilityApiStatusKind::Forbidden => {
            "Cloud Observability policy denied the audit read request"
        }
        CloudObservabilityApiStatusKind::UnprocessableEntity => {
            "Cloud Observability could not process the supplied audit cursor or chain state"
        }
        CloudObservabilityApiStatusKind::Unauthorized => {
            "Cloud Observability authentication is required"
        }
    }
}

fn observability_issue(error: &CloudObservabilityError) -> &'static str {
    match error {
        CloudObservabilityError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudObservabilityError::InvalidRegion => "region must be canonical and residency-valid",
        CloudObservabilityError::InvalidCellId => "cell_id must be canonical for the region",
        CloudObservabilityError::InvalidAuditTopic => {
            "control-plane scope may only include control-plane audit topics"
        }
        CloudObservabilityError::InvalidReadWindow => {
            "start_epoch_seconds must be before end_epoch_seconds within the maximum read window"
        }
        CloudObservabilityError::InvalidPageSize => {
            "page_size must be between 1 and the maximum audit read page size"
        }
        CloudObservabilityError::InvalidCursor => "cursor must use the audit read cursor format",
        CloudObservabilityError::CursorTenantMismatch => {
            "cursor tenant must match the request tenant"
        }
        CloudObservabilityError::CursorRegionMismatch => {
            "cursor region must match the request region"
        }
        CloudObservabilityError::InvalidActorRef => {
            "actor filter must use an accepted principal reference"
        }
        CloudObservabilityError::InvalidResourceId => {
            "resource_id filter must be a valid Cloud resource id"
        }
        CloudObservabilityError::ResourceTenantMismatch => {
            "resource_id filter must belong to the request tenant"
        }
        CloudObservabilityError::ResourceRegionMismatch => {
            "resource_id filter must belong to the request region"
        }
        CloudObservabilityError::IncompleteAuditChain => {
            "audit chain completeness is required but unavailable"
        }
        CloudObservabilityError::UnverifiedAuditChain => "audit chain verification failed",
        _ => "cloud observability invariant rejected the request",
    }
}

fn detail(field: &str, issue: &str) -> CloudObservabilityApiErrorDetail {
    CloudObservabilityApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
