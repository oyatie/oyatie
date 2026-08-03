//! Cloud Network load balancer API boundary for tenant LB creation.
//!
//! This crate owns tenant/header/path/body normalization, idempotency, and
//! authenticated API projection before handing typed load balancer creation
//! requests to the Cloud network kernel.

use std::collections::BTreeMap;

use oya_cloud_network_domain::{
    CloudNetworkCatalog, CloudNetworkError, LbKind, LbState, ListenerCreate, LoadBalancer,
    LoadBalancerCreate, MtlsClientPolicy, MtlsConfigCreate, NetworkRepo, TargetGroupCreate,
};
use oya_data_boundary_kernel::{DataClass, parse_data_class_label};

pub const CLOUD_NETWORK_LB_CREATE_SURFACE: &str = "cloud.network.lb.create";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudNetworkLbCreateApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudNetworkLbCreateApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudNetworkLbApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathLoadBalancerIdEmpty,
    LoadBalancerIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    LoadBalancerKindInvalid,
    MtlsClientPolicyInvalid,
    DataClassInvalid,
    NetworkInvalidRequest,
    NetworkForbidden,
    NetworkNotFound,
    NetworkConflict,
}

impl CloudNetworkLbApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_NETWORK_LB_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_NETWORK_LB_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_NETWORK_LB_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_NETWORK_LB_PRINCIPAL_ID_EMPTY",
            Self::PathLoadBalancerIdEmpty => "CLOUD_NETWORK_LB_PATH_LOAD_BALANCER_ID_EMPTY",
            Self::LoadBalancerIdMismatch => "CLOUD_NETWORK_LB_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_NETWORK_LB_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => {
                "CLOUD_NETWORK_LB_AUTHORIZATION_DECISION_ID_EMPTY"
            }
            Self::AuthorizationTenantMismatch => "CLOUD_NETWORK_LB_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "CLOUD_NETWORK_LB_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "CLOUD_NETWORK_LB_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_NETWORK_LB_IDEMPOTENCY_KEY_REUSED",
            Self::LoadBalancerKindInvalid => "CLOUD_NETWORK_LB_KIND_INVALID",
            Self::MtlsClientPolicyInvalid => "CLOUD_NETWORK_LB_MTLS_CLIENT_POLICY_INVALID",
            Self::DataClassInvalid => "CLOUD_NETWORK_LB_DATA_CLASS_INVALID",
            Self::NetworkInvalidRequest => "CLOUD_NETWORK_LB_INVALID_REQUEST",
            Self::NetworkForbidden => "CLOUD_NETWORK_LB_FORBIDDEN",
            Self::NetworkNotFound => "CLOUD_NETWORK_LB_NOT_FOUND",
            Self::NetworkConflict => "CLOUD_NETWORK_LB_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbListenerCreateRequest {
    pub port: u16,                       // data_class: PUBLIC
    pub target_group_id: String,         // data_class: INTERNAL_ONLY
    pub tls_certificate: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbSubnetRef {
    pub subnet_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbTargetGroupCreateRequest {
    pub id: String,                               // data_class: INTERNAL_ONLY
    pub subnet_ids: Vec<CloudNetworkLbSubnetRef>, // data_class: INTERNAL_ONLY
    pub health_check_path: Option<String>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbMtlsConfigCreateRequest {
    pub ca_bundle_ref: String, // data_class: INTERNAL_ONLY
    pub client_policy: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbCreateRequest {
    pub resource_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,   // data_class: INTERNAL_ONLY
    pub vpc_id: String,      // data_class: INTERNAL_ONLY
    pub region: String,      // data_class: PUBLIC
    pub kind: String,        // data_class: PUBLIC
    pub listeners: Vec<CloudNetworkLbListenerCreateRequest>, // data_class: INTERNAL_ONLY
    pub target_groups: Vec<CloudNetworkLbTargetGroupCreateRequest>, // data_class: INTERNAL_ONLY
    pub mtls: Option<CloudNetworkLbMtlsConfigCreateRequest>, // data_class: INTERNAL_ONLY
    pub waf_policy: Option<String>, // data_class: INTERNAL_ONLY
    pub data_class: String,  // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbCreateApiRequest {
    pub path_load_balancer_id: String, // data_class: INTERNAL_ONLY
    pub boundary: CloudNetworkLbApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudNetworkLbApiPrincipal, // data_class: INTERNAL_ONLY
    pub authorization: CloudNetworkLbApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudNetworkLbCreateRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudNetworkLbCreateIdempotencyLedger {
    entries:
        BTreeMap<CloudNetworkLbIdempotencyLedgerKey, CloudNetworkLbCreateIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CloudNetworkLbCreateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudNetworkLbIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudNetworkLbCreateIdempotencyLedgerEntry {
    fingerprint: CloudNetworkLbRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudNetworkLbCreateApiResult,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudNetworkLbRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudNetworkLbCreateApiResult =
    Result<CloudNetworkLbCreateSuccessResponse, CloudNetworkLbApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbCreateSuccessResponse {
    pub data: CloudNetworkLbRecord,          // data_class: INTERNAL_ONLY
    pub metadata: CloudNetworkLbApiMetadata, // data_class: INTERNAL_ONLY
}

impl CloudNetworkLbCreateSuccessResponse {
    pub fn created(data: CloudNetworkLbRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudNetworkLbApiMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbApiMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbRecord {
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub vpc_id: String,                // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub kind: String,                  // data_class: PUBLIC
    pub listener_count: u32,           // data_class: PUBLIC
    pub target_group_count: u32,       // data_class: PUBLIC
    pub mtls_enabled: bool,            // data_class: PUBLIC
    pub waf_policy: Option<String>,    // data_class: INTERNAL_ONLY
    pub data_class: String,            // data_class: PUBLIC
    pub state: String,                 // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbApiErrorResponse {
    pub error: CloudNetworkLbApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbApiErrorBody {
    pub code: String,                               // data_class: INTERNAL_ONLY
    pub message: String,                            // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,          // data_class: INTERNAL_ONLY
    pub request_id: String,                         // data_class: INTERNAL_ONLY
    pub details: Vec<CloudNetworkLbApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkLbApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudNetworkLbApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathLoadBalancerId,
    LoadBalancerIdMismatch {
        path_load_balancer_id: String,
        body_resource_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
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
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidLoadBalancerKindLabel {
        kind: String,
    },
    InvalidMtlsClientPolicyLabel {
        client_policy: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    Network(CloudNetworkError),
}

impl CloudNetworkLbApiError {
    pub fn lb_create_status(&self) -> CloudNetworkLbCreateApiStatus {
        match self.status_kind() {
            CloudNetworkLbApiStatusKind::BadRequest => CloudNetworkLbCreateApiStatus::BadRequest,
            CloudNetworkLbApiStatusKind::Unauthorized => {
                CloudNetworkLbCreateApiStatus::Unauthorized
            }
            CloudNetworkLbApiStatusKind::Forbidden => CloudNetworkLbCreateApiStatus::Forbidden,
            CloudNetworkLbApiStatusKind::NotFound => CloudNetworkLbCreateApiStatus::NotFound,
            CloudNetworkLbApiStatusKind::Conflict => CloudNetworkLbCreateApiStatus::Conflict,
            CloudNetworkLbApiStatusKind::UnprocessableEntity => {
                CloudNetworkLbCreateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn lb_create_status_code(&self) -> u16 {
        self.lb_create_status().code()
    }

    pub fn code(&self) -> CloudNetworkLbApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudNetworkLbApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudNetworkLbApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudNetworkLbApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudNetworkLbApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathLoadBalancerId => CloudNetworkLbApiErrorCode::PathLoadBalancerIdEmpty,
            Self::LoadBalancerIdMismatch { .. } => {
                CloudNetworkLbApiErrorCode::LoadBalancerIdMismatch
            }
            Self::TenantMismatch { .. } => CloudNetworkLbApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudNetworkLbApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudNetworkLbApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudNetworkLbApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudNetworkLbApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => CloudNetworkLbApiErrorCode::IdempotencyKeyReused,
            Self::InvalidLoadBalancerKindLabel { .. } => {
                CloudNetworkLbApiErrorCode::LoadBalancerKindInvalid
            }
            Self::InvalidMtlsClientPolicyLabel { .. } => {
                CloudNetworkLbApiErrorCode::MtlsClientPolicyInvalid
            }
            Self::InvalidDataClassLabel { .. } => CloudNetworkLbApiErrorCode::DataClassInvalid,
            Self::Network(error) => match cloud_network_status_kind(error) {
                CloudNetworkLbApiStatusKind::BadRequest => {
                    CloudNetworkLbApiErrorCode::NetworkInvalidRequest
                }
                CloudNetworkLbApiStatusKind::Forbidden => {
                    CloudNetworkLbApiErrorCode::NetworkForbidden
                }
                CloudNetworkLbApiStatusKind::NotFound => {
                    CloudNetworkLbApiErrorCode::NetworkNotFound
                }
                CloudNetworkLbApiStatusKind::Conflict => {
                    CloudNetworkLbApiErrorCode::NetworkConflict
                }
                CloudNetworkLbApiStatusKind::Unauthorized
                | CloudNetworkLbApiStatusKind::UnprocessableEntity => {
                    CloudNetworkLbApiErrorCode::NetworkInvalidRequest
                }
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudNetworkLbApiErrorResponse {
        CloudNetworkLbApiErrorResponse {
            error: CloudNetworkLbApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudNetworkLbApiStatusKind {
        match self {
            Self::EmptyPrincipalId => CloudNetworkLbApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudNetworkLbApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => CloudNetworkLbApiStatusKind::UnprocessableEntity,
            Self::Network(error) => cloud_network_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathLoadBalancerId
            | Self::LoadBalancerIdMismatch { .. }
            | Self::InvalidLoadBalancerKindLabel { .. }
            | Self::InvalidMtlsClientPolicyLabel { .. }
            | Self::InvalidDataClassLabel { .. } => CloudNetworkLbApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathLoadBalancerId => "Path load balancer id is required",
            Self::LoadBalancerIdMismatch { .. } => "Path and body load balancer ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal and request body"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Cloud Network load balancer surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidLoadBalancerKindLabel { .. } => {
                "Request load balancer kind must be l4_tcp, l4_udp, l7_http, or l7_grpc"
            }
            Self::InvalidMtlsClientPolicyLabel { .. } => {
                "Request mTLS client policy must be a known Cloud Network policy label"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::Network(error) => cloud_network_message(error),
        }
    }

    fn details(&self) -> Vec<CloudNetworkLbApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathLoadBalancerId => {
                vec![detail("path.load_balancer_id", "must be non-empty")]
            }
            Self::LoadBalancerIdMismatch { .. } => vec![detail(
                "resource_id",
                "path load_balancer_id and body resource_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, and body tenant_id must match",
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
                "must include the requested Cloud Network load balancer surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidLoadBalancerKindLabel { .. } => vec![detail(
                "body.kind",
                "must be l4_tcp, l4_udp, l7_http, or l7_grpc",
            )],
            Self::InvalidMtlsClientPolicyLabel { .. } => vec![detail(
                "body.mtls.client_policy",
                "must be require_verified_client_cert or forward_verified_identity",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::Network(error) => vec![detail("cloud_network", cloud_network_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudNetworkLbApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cloud_network_lb_create_request(
    request: &CloudNetworkLbCreateApiRequest,
) -> Result<(), CloudNetworkLbApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_load_balancer_id(&request.path_load_balancer_id, &request.body.resource_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_NETWORK_LB_CREATE_SURFACE,
    )
}

pub fn create_cloud_network_load_balancer_from_api(
    catalog: &mut CloudNetworkCatalog,
    idempotency_ledger: &mut CloudNetworkLbCreateIdempotencyLedger,
    request: CloudNetworkLbCreateApiRequest,
) -> Result<CloudNetworkLbCreateSuccessResponse, CloudNetworkLbApiError> {
    validate_cloud_network_lb_create_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_NETWORK_LB_CREATE_SURFACE,
    );
    let fingerprint = lb_create_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudNetworkLbApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = load_balancer_create_input(request.body)
        .and_then(|input| {
            catalog
                .create_load_balancer(input)
                .map_err(CloudNetworkLbApiError::Network)
        })
        .map(|load_balancer| {
            CloudNetworkLbCreateSuccessResponse::created(
                load_balancer_record(load_balancer),
                request_id,
            )
        });
    idempotency_ledger.entries.insert(
        key,
        CloudNetworkLbCreateIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(
    boundary: &CloudNetworkLbApiBoundaryContext,
) -> Result<(), CloudNetworkLbApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudNetworkLbApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudNetworkLbApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudNetworkLbApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_load_balancer_id(
    path_load_balancer_id: &str,
    body_resource_id: &str,
) -> Result<(), CloudNetworkLbApiError> {
    if path_load_balancer_id.trim().is_empty() {
        return Err(CloudNetworkLbApiError::EmptyPathLoadBalancerId);
    }
    if path_load_balancer_id != body_resource_id {
        return Err(CloudNetworkLbApiError::LoadBalancerIdMismatch {
            path_load_balancer_id: path_load_balancer_id.to_string(),
            body_resource_id: body_resource_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudNetworkLbApiBoundaryContext,
    principal: &CloudNetworkLbApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudNetworkLbApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudNetworkLbApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body_tenant_id {
        return Err(CloudNetworkLbApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudNetworkLbApiPrincipal,
    authorization: &CloudNetworkLbApiAuthorization,
    surface: &str,
) -> Result<(), CloudNetworkLbApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudNetworkLbApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudNetworkLbApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudNetworkLbApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudNetworkLbApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn load_balancer_create_input(
    body: CloudNetworkLbCreateRequest,
) -> Result<LoadBalancerCreate, CloudNetworkLbApiError> {
    Ok(LoadBalancerCreate {
        resource_id: body.resource_id,
        tenant_id: body.tenant_id,
        vpc_id: body.vpc_id,
        region: body.region,
        kind: parse_api_load_balancer_kind(body.kind)?,
        listeners: body
            .listeners
            .into_iter()
            .map(listener_create_input)
            .collect(),
        target_groups: body
            .target_groups
            .into_iter()
            .map(target_group_create_input)
            .collect(),
        mtls: body.mtls.map(mtls_create_input).transpose()?,
        waf_policy: body.waf_policy,
        state: LbState::Creating,
        data_class: parse_api_data_class(body.data_class)?,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    })
}

fn listener_create_input(input: CloudNetworkLbListenerCreateRequest) -> ListenerCreate {
    ListenerCreate {
        port: input.port,
        target_group_id: input.target_group_id,
        tls_certificate: input.tls_certificate,
    }
}

fn target_group_create_input(input: CloudNetworkLbTargetGroupCreateRequest) -> TargetGroupCreate {
    TargetGroupCreate {
        id: input.id,
        subnet_ids: input
            .subnet_ids
            .into_iter()
            .map(|subnet| subnet.subnet_id)
            .collect(),
        health_check_path: input.health_check_path,
    }
}

fn mtls_create_input(
    input: CloudNetworkLbMtlsConfigCreateRequest,
) -> Result<MtlsConfigCreate, CloudNetworkLbApiError> {
    Ok(MtlsConfigCreate {
        ca_bundle_ref: input.ca_bundle_ref,
        client_policy: parse_api_mtls_client_policy(input.client_policy)?,
    })
}

fn parse_api_load_balancer_kind(label: String) -> Result<LbKind, CloudNetworkLbApiError> {
    match label.as_str() {
        "l4_tcp" => Ok(LbKind::L4Tcp),
        "l4_udp" => Ok(LbKind::L4Udp),
        "l7_http" => Ok(LbKind::L7Http),
        "l7_grpc" => Ok(LbKind::L7Grpc),
        _ => Err(CloudNetworkLbApiError::InvalidLoadBalancerKindLabel { kind: label }),
    }
}

fn parse_api_mtls_client_policy(label: String) -> Result<MtlsClientPolicy, CloudNetworkLbApiError> {
    match label.as_str() {
        "require_verified_client_cert" => Ok(MtlsClientPolicy::RequireVerifiedClientCert),
        "forward_verified_identity" => Ok(MtlsClientPolicy::ForwardVerifiedIdentity),
        _ => Err(CloudNetworkLbApiError::InvalidMtlsClientPolicyLabel {
            client_policy: label,
        }),
    }
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudNetworkLbApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudNetworkLbApiError::InvalidDataClassLabel { data_class: label })
}

fn idempotency_key_for(
    boundary: &CloudNetworkLbApiBoundaryContext,
    principal: &CloudNetworkLbApiPrincipal,
    surface: &str,
) -> CloudNetworkLbIdempotencyLedgerKey {
    CloudNetworkLbIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn lb_create_fingerprint_for(
    request: &CloudNetworkLbCreateApiRequest,
) -> CloudNetworkLbRequestFingerprint {
    CloudNetworkLbRequestFingerprint {
        canonical: [
            format!("path.load_balancer_id={}", request.path_load_balancer_id),
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
            format!("body.resource_id={}", request.body.resource_id),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.vpc_id={}", request.body.vpc_id),
            format!("body.region={}", request.body.region),
            format!("body.kind={}", request.body.kind),
            format!("body.listeners={:?}", request.body.listeners),
            format!("body.target_groups={:?}", request.body.target_groups),
            format!("body.mtls={:?}", request.body.mtls),
            format!("body.waf_policy={:?}", request.body.waf_policy),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.created_at_epoch_seconds={}",
                request.body.created_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn load_balancer_record(load_balancer: LoadBalancer) -> CloudNetworkLbRecord {
    CloudNetworkLbRecord {
        resource_id: load_balancer.resource_id.value.value,
        tenant_id: load_balancer.tenant_id.value,
        vpc_id: load_balancer.vpc_id.value.value,
        region: load_balancer.region.value.value,
        kind: load_balancer_kind_label(load_balancer.kind.value).to_string(),
        listener_count: load_balancer.listeners.value.len() as u32,
        target_group_count: load_balancer.target_groups.value.len() as u32,
        mtls_enabled: load_balancer.mtls.value.is_some(),
        waf_policy: load_balancer.waf_policy.value.map(|policy| policy.value),
        data_class: load_balancer.data_class.value.label().to_string(),
        state: load_balancer_state_label(load_balancer.state.value).to_string(),
        created_at_epoch_seconds: load_balancer.created_at_epoch_seconds.value,
        schema_version: load_balancer.schema_version.value,
    }
}

fn load_balancer_kind_label(kind: LbKind) -> &'static str {
    match kind {
        LbKind::L4Tcp => "l4_tcp",
        LbKind::L4Udp => "l4_udp",
        LbKind::L7Http => "l7_http",
        LbKind::L7Grpc => "l7_grpc",
    }
}

fn load_balancer_state_label(state: LbState) -> &'static str {
    match state {
        LbState::Creating => "creating",
        LbState::Active => "active",
        LbState::Draining => "draining",
        LbState::Deleting => "deleting",
    }
}

fn cloud_network_status_kind(error: &CloudNetworkError) -> CloudNetworkLbApiStatusKind {
    match error {
        CloudNetworkError::DuplicateRoute
        | CloudNetworkError::DuplicateSecurityGroup
        | CloudNetworkError::DuplicateSubnet
        | CloudNetworkError::DuplicateTargetGroup
        | CloudNetworkError::DuplicateListenerPort
        | CloudNetworkError::DuplicateCdnOrigin
        | CloudNetworkError::DuplicateCdnHostname
        | CloudNetworkError::DuplicateBgpSession
        | CloudNetworkError::DuplicateInterconnectPartner
        | CloudNetworkError::DuplicateProtectedResource
        | CloudNetworkError::DuplicateServiceMesh
        | CloudNetworkError::DuplicateFlowAnomaly
        | CloudNetworkError::DuplicateVpc
        | CloudNetworkError::DuplicateLoadBalancer
        | CloudNetworkError::DuplicateDnsZone
        | CloudNetworkError::DuplicateCdnDistribution
        | CloudNetworkError::DuplicateDirectInterconnect
        | CloudNetworkError::DuplicateDdosProtection => CloudNetworkLbApiStatusKind::Conflict,
        CloudNetworkError::UnknownVpc
        | CloudNetworkError::UnknownSubnet
        | CloudNetworkError::UnknownLoadBalancer
        | CloudNetworkError::UnknownDnsZone
        | CloudNetworkError::UnknownInterconnectPartner
        | CloudNetworkError::UnknownProtectedResource => CloudNetworkLbApiStatusKind::NotFound,
        CloudNetworkError::ResourceTenantMismatch
        | CloudNetworkError::ResourceRegionMismatch
        | CloudNetworkError::AzRegionMismatch
        | CloudNetworkError::SubnetOutsideVpc
        | CloudNetworkError::ListenerTargetGroupMissing
        | CloudNetworkError::PrivateZoneRequiresVpc
        | CloudNetworkError::PublicZoneMustNotBindVpc
        | CloudNetworkError::ScrubbingRegionRequired => CloudNetworkLbApiStatusKind::Forbidden,
        CloudNetworkError::InvalidTenantId
        | CloudNetworkError::InvalidResourceId
        | CloudNetworkError::ResourceKindMismatch
        | CloudNetworkError::InvalidDataClass
        | CloudNetworkError::InvalidVpcState
        | CloudNetworkError::InvalidSubnetState
        | CloudNetworkError::InvalidLbState
        | CloudNetworkError::InvalidDnsZoneState
        | CloudNetworkError::InvalidIpv4Cidr
        | CloudNetworkError::InvalidIpv6Cidr
        | CloudNetworkError::InvalidCidrPrefix
        | CloudNetworkError::Ipv6Required
        | CloudNetworkError::InvalidRouteTableId
        | CloudNetworkError::InvalidRoute
        | CloudNetworkError::InvalidSecurityGroupId
        | CloudNetworkError::InvalidSecurityRule
        | CloudNetworkError::InvalidAzCode
        | CloudNetworkError::OverlappingSubnet
        | CloudNetworkError::InvalidTargetGroupId
        | CloudNetworkError::InvalidListener
        | CloudNetworkError::L7RequiresTls
        | CloudNetworkError::GrpcRequiresMtls
        | CloudNetworkError::InvalidCertificateRef
        | CloudNetworkError::InvalidWafPolicyId
        | CloudNetworkError::InvalidDnsName
        | CloudNetworkError::InvalidDnssecKeyRef
        | CloudNetworkError::DnssecRequired
        | CloudNetworkError::InvalidCdnState
        | CloudNetworkError::InvalidCdnOrigin
        | CloudNetworkError::CdnWafRequired
        | CloudNetworkError::CdnTlsRequired
        | CloudNetworkError::InvalidInterconnectPartnerId
        | CloudNetworkError::InvalidInterconnectPortId
        | CloudNetworkError::InvalidPeeringLocation
        | CloudNetworkError::InvalidInterconnectState
        | CloudNetworkError::InvalidBandwidth
        | CloudNetworkError::InvalidVlanTag
        | CloudNetworkError::InvalidBgpSessionId
        | CloudNetworkError::InvalidBgpSession
        | CloudNetworkError::InvalidAsn
        | CloudNetworkError::InterconnectRedundancyRequired
        | CloudNetworkError::InterconnectSlaRequired
        | CloudNetworkError::RegionalInterconnectDiversityRequired
        | CloudNetworkError::InvalidDdosState
        | CloudNetworkError::LineRateScrubbingRequired
        | CloudNetworkError::DdosAlwaysOnRequired
        | CloudNetworkError::InvalidRunbookRef
        | CloudNetworkError::InvalidOnCallGroupRef
        | CloudNetworkError::InvalidMeshId
        | CloudNetworkError::InvalidCellId
        | CloudNetworkError::InvalidMeshNamespace
        | CloudNetworkError::InvalidMeshState
        | CloudNetworkError::InvalidMeshMode
        | CloudNetworkError::InvalidMeshGateway
        | CloudNetworkError::MeshMtlsRequired
        | CloudNetworkError::MeshExtAuthzRequired
        | CloudNetworkError::InvalidCedarPolicyRef
        | CloudNetworkError::InvalidAuditStreamRef
        | CloudNetworkError::InvalidHealthAlarmRef
        | CloudNetworkError::MeshControlPlaneReplicasRequired
        | CloudNetworkError::MeshUpgradeDrillRequired
        | CloudNetworkError::DefaultDenyIngressRequired
        | CloudNetworkError::DefaultDenyEgressRequired
        | CloudNetworkError::DnsEgressExceptionRequired
        | CloudNetworkError::CrossCellDefaultTrafficForbidden
        | CloudNetworkError::EnvoyExtAuthzRequired
        | CloudNetworkError::EnvoyFailClosedRequired
        | CloudNetworkError::CoreDnsInsecurePodModeForbidden
        | CloudNetworkError::EvidenceRefMissing
        | CloudNetworkError::EvidenceRefLooksSecretLike
        | CloudNetworkError::InvalidFlowAnomalyId
        | CloudNetworkError::FlowLogsRequired
        | CloudNetworkError::InvalidResourceContractPolicyRef
        | CloudNetworkError::InvalidResourceContractQuota
        | CloudNetworkError::InvalidResourceContractBillingMeter
        | CloudNetworkError::InvalidResourceContractAuditEvent
        | CloudNetworkError::InvalidResourceContractObservabilityHook
        | CloudNetworkError::InvalidResourceContractRollbackPlan
        | CloudNetworkError::InvalidResourceContractReconciliationStatus
        | CloudNetworkError::InvalidResourceContractScope
        | CloudNetworkError::ResourceContractRuntimeClaimOutOfScope
        | CloudNetworkError::ResourceContractMeasuredSloClaimOutOfScope => {
            CloudNetworkLbApiStatusKind::BadRequest
        }
    }
}

fn cloud_network_message(error: &CloudNetworkError) -> &'static str {
    match cloud_network_status_kind(error) {
        CloudNetworkLbApiStatusKind::BadRequest => "Cloud Network rejected the request shape",
        CloudNetworkLbApiStatusKind::Unauthorized => "Cloud Network authentication is required",
        CloudNetworkLbApiStatusKind::Forbidden => "Cloud Network policy denied the request",
        CloudNetworkLbApiStatusKind::NotFound => "Cloud Network resource was not found",
        CloudNetworkLbApiStatusKind::Conflict => "Cloud Network resource already exists",
        CloudNetworkLbApiStatusKind::UnprocessableEntity => {
            "Cloud Network rejected request idempotency"
        }
    }
}

fn cloud_network_issue(error: &CloudNetworkError) -> &'static str {
    match error {
        CloudNetworkError::InvalidTenantId => "tenant_id must be a ten_ identifier",
        CloudNetworkError::InvalidResourceId => "resource_id must be canonical cloud resource id",
        CloudNetworkError::ResourceTenantMismatch => "resource tenant must match request tenant",
        CloudNetworkError::ResourceRegionMismatch => {
            "resource region must match request region and residency"
        }
        CloudNetworkError::ResourceKindMismatch => "resource kind must match network type",
        CloudNetworkError::InvalidDataClass => "data_class must be public metadata for LB create",
        CloudNetworkError::InvalidVpcState => "VPC create requests must start in Creating state",
        CloudNetworkError::InvalidSubnetState => {
            "subnet create requests must start in Creating state"
        }
        CloudNetworkError::InvalidLbState => {
            "load balancer create requests must start in Creating state"
        }
        CloudNetworkError::InvalidDnsZoneState => {
            "DNS zone create requests must start in Creating state"
        }
        CloudNetworkError::InvalidIpv4Cidr => "IPv4 CIDR must be canonical",
        CloudNetworkError::InvalidIpv6Cidr => "IPv6 CIDR must be canonical",
        CloudNetworkError::InvalidCidrPrefix => "CIDR prefix length is out of range",
        CloudNetworkError::Ipv6Required => "IPv6 CIDR is required",
        CloudNetworkError::InvalidRouteTableId => "route table id must use the rtb_ prefix",
        CloudNetworkError::InvalidRoute => "route next hop and target reference must be consistent",
        CloudNetworkError::DuplicateRoute => "route destinations must be unique",
        CloudNetworkError::InvalidSecurityGroupId => "security group id must use the sg_ prefix",
        CloudNetworkError::InvalidSecurityRule => {
            "security rule must use valid ports and description"
        }
        CloudNetworkError::DuplicateSecurityGroup => "security group ids must be unique",
        CloudNetworkError::InvalidAzCode => "AZ must be canonical lowercase ASCII",
        CloudNetworkError::AzRegionMismatch => "AZ code must sit under its region code",
        CloudNetworkError::SubnetOutsideVpc => "subnet CIDR must fit inside the VPC CIDR",
        CloudNetworkError::OverlappingSubnet => "subnet CIDRs must not overlap",
        CloudNetworkError::DuplicateSubnet => "subnet resource id is already present",
        CloudNetworkError::UnknownVpc => "referenced VPC must exist",
        CloudNetworkError::UnknownSubnet => "referenced subnet must exist",
        CloudNetworkError::InvalidTargetGroupId => "target group id must use the tg_ prefix",
        CloudNetworkError::DuplicateTargetGroup => "target group ids must be unique",
        CloudNetworkError::InvalidListener => "listener must bind a valid port and target group",
        CloudNetworkError::DuplicateListenerPort => "listener ports must be unique",
        CloudNetworkError::ListenerTargetGroupMissing => "listener target group must exist",
        CloudNetworkError::L7RequiresTls => "L7 load balancers require TLS",
        CloudNetworkError::GrpcRequiresMtls => "gRPC load balancers require mTLS",
        CloudNetworkError::InvalidCertificateRef => {
            "certificate reference must use the cert/ prefix"
        }
        CloudNetworkError::InvalidWafPolicyId => "WAF policy id must use the waf_ prefix",
        CloudNetworkError::InvalidDnsName => "DNS name must be a canonical DNS label",
        CloudNetworkError::InvalidDnssecKeyRef => {
            "DNSSEC key reference must use the dnssec/ prefix"
        }
        CloudNetworkError::DnssecRequired => "DNSSEC key reference is required",
        CloudNetworkError::PrivateZoneRequiresVpc => "private DNS zones require a VPC binding",
        CloudNetworkError::PublicZoneMustNotBindVpc => "public DNS zones must not bind to a VPC",
        CloudNetworkError::InvalidCdnState => "CDN create requests must start in Creating state",
        CloudNetworkError::InvalidCdnOrigin => "CDN origin must reference a supported origin type",
        CloudNetworkError::DuplicateCdnOrigin => "CDN origins must be unique",
        CloudNetworkError::UnknownLoadBalancer => "referenced load balancer must exist",
        CloudNetworkError::UnknownDnsZone => "referenced DNS zone must exist",
        CloudNetworkError::CdnWafRequired => "CDN distribution requires a WAF policy",
        CloudNetworkError::CdnTlsRequired => "CDN distribution requires a TLS certificate",
        CloudNetworkError::DuplicateCdnHostname => "CDN hostnames must be unique",
        CloudNetworkError::InvalidInterconnectPartnerId => {
            "interconnect partner id must use the ixp_ prefix"
        }
        CloudNetworkError::InvalidInterconnectPortId => {
            "interconnect port id must use the icp_ prefix"
        }
        CloudNetworkError::InvalidPeeringLocation => "peering location must be canonical",
        CloudNetworkError::InvalidInterconnectState => {
            "direct interconnect create requests must start in Creating state"
        }
        CloudNetworkError::InvalidBandwidth => "bandwidth must be greater than zero",
        CloudNetworkError::InvalidVlanTag => "VLAN tag must be in the valid customer range",
        CloudNetworkError::InvalidBgpSessionId => "BGP session id must use the bgp_ prefix",
        CloudNetworkError::InvalidBgpSession => "BGP session addresses must be valid",
        CloudNetworkError::DuplicateBgpSession => "BGP session ids must be unique",
        CloudNetworkError::InvalidAsn => "ASN must be non-zero",
        CloudNetworkError::InterconnectRedundancyRequired => {
            "interconnect redundancy must meet minimum policy"
        }
        CloudNetworkError::InterconnectSlaRequired => "interconnect SLA must meet policy",
        CloudNetworkError::RegionalInterconnectDiversityRequired => {
            "interconnect must include regional diversity"
        }
        CloudNetworkError::UnknownInterconnectPartner => "interconnect partner must exist",
        CloudNetworkError::DuplicateInterconnectPartner => {
            "interconnect partner id is already present"
        }
        CloudNetworkError::DuplicateDirectInterconnect => {
            "direct interconnect resource id is already present"
        }
        CloudNetworkError::InvalidDdosState => "DDoS create requests must start in Creating state",
        CloudNetworkError::UnknownProtectedResource => "protected resource must exist",
        CloudNetworkError::DuplicateProtectedResource => "protected resources must be unique",
        CloudNetworkError::ScrubbingRegionRequired => {
            "scrubbing region must satisfy residency policy"
        }
        CloudNetworkError::LineRateScrubbingRequired => "line-rate scrubbing is required",
        CloudNetworkError::DdosAlwaysOnRequired => "always-on DDoS protection is required",
        CloudNetworkError::InvalidRunbookRef => "runbook reference must use the runbook/ prefix",
        CloudNetworkError::InvalidOnCallGroupRef => {
            "on-call group reference must use the oncall/ prefix"
        }
        CloudNetworkError::DuplicateDdosProtection => {
            "DDoS protection resource id is already present"
        }
        CloudNetworkError::InvalidMeshId => "service mesh id must use the mesh_ prefix",
        CloudNetworkError::InvalidCellId => "cell id must be canonical",
        CloudNetworkError::InvalidMeshNamespace => "mesh namespace must be canonical",
        CloudNetworkError::InvalidMeshState => {
            "service mesh create requests must start in Creating state"
        }
        CloudNetworkError::InvalidMeshMode => "service mesh mode must be supported",
        CloudNetworkError::InvalidMeshGateway => "service mesh gateway must be supported",
        CloudNetworkError::MeshMtlsRequired => "service mesh requires mTLS everywhere",
        CloudNetworkError::MeshExtAuthzRequired => "service mesh requires external authorization",
        CloudNetworkError::InvalidCedarPolicyRef => {
            "Cedar policy reference must use the cedar/ prefix"
        }
        CloudNetworkError::InvalidAuditStreamRef => {
            "audit stream reference must use the audit/ prefix"
        }
        CloudNetworkError::InvalidHealthAlarmRef => {
            "health alarm reference must use the alarm/ prefix"
        }
        CloudNetworkError::MeshControlPlaneReplicasRequired => {
            "service mesh requires control-plane replicas"
        }
        CloudNetworkError::MeshUpgradeDrillRequired => {
            "service mesh requires quarterly upgrade drills"
        }
        CloudNetworkError::DefaultDenyIngressRequired => {
            "cell guardrail requires default-deny ingress"
        }
        CloudNetworkError::DefaultDenyEgressRequired => {
            "cell guardrail requires default-deny egress"
        }
        CloudNetworkError::DnsEgressExceptionRequired => {
            "cell guardrail requires an explicit DNS egress exception"
        }
        CloudNetworkError::CrossCellDefaultTrafficForbidden => {
            "cell guardrail forbids default cross-cell traffic"
        }
        CloudNetworkError::EnvoyExtAuthzRequired => {
            "Envoy guardrail requires external authorization"
        }
        CloudNetworkError::EnvoyFailClosedRequired => {
            "Envoy external authorization must fail closed"
        }
        CloudNetworkError::CoreDnsInsecurePodModeForbidden => {
            "CoreDNS pod mode must not be insecure"
        }
        CloudNetworkError::EvidenceRefMissing => "evidence ref must use evidence://",
        CloudNetworkError::EvidenceRefLooksSecretLike => {
            "evidence ref must not contain secret-like material"
        }
        CloudNetworkError::DuplicateServiceMesh => "service mesh id is already present",
        CloudNetworkError::InvalidFlowAnomalyId => "flow anomaly id must use the flowanom_ prefix",
        CloudNetworkError::FlowLogsRequired => "VPC flow logs must be enabled",
        CloudNetworkError::DuplicateFlowAnomaly => "flow anomaly id is already present",
        CloudNetworkError::DuplicateVpc => "VPC resource id is already present",
        CloudNetworkError::DuplicateLoadBalancer => "load balancer resource id is already present",
        CloudNetworkError::DuplicateDnsZone => "DNS zone resource id is already present",
        CloudNetworkError::DuplicateCdnDistribution => {
            "CDN distribution resource id is already present"
        }
        CloudNetworkError::InvalidResourceContractPolicyRef => {
            "resource contract policy reference is required"
        }
        CloudNetworkError::InvalidResourceContractQuota => {
            "resource contract quota reservation or refusal is required"
        }
        CloudNetworkError::InvalidResourceContractBillingMeter => {
            "resource contract billing meter intent is required"
        }
        CloudNetworkError::InvalidResourceContractAuditEvent => {
            "resource contract audit event envelope is required"
        }
        CloudNetworkError::InvalidResourceContractObservabilityHook => {
            "resource contract observability hook is required"
        }
        CloudNetworkError::InvalidResourceContractRollbackPlan => {
            "resource contract rollback/compensating action is required"
        }
        CloudNetworkError::InvalidResourceContractReconciliationStatus => {
            "resource contract desired-vs-actual reconciliation status is required"
        }
        CloudNetworkError::InvalidResourceContractScope => {
            "resource contract scope must match the resource type boundary"
        }
        CloudNetworkError::ResourceContractRuntimeClaimOutOfScope => {
            "resource contract must not claim live registry, ledger, reconciler, or provider apply"
        }
        CloudNetworkError::ResourceContractMeasuredSloClaimOutOfScope => {
            "resource contract must not claim measured SLO evidence"
        }
    }
}

fn detail(field: &str, issue: &str) -> CloudNetworkLbApiErrorDetail {
    CloudNetworkLbApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
