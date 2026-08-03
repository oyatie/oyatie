//! Cloud Network VPC API boundary for tenant VPC creation.
//!
//! This crate owns tenant/header/path/body normalization, idempotency, and
//! authenticated API projection before handing typed VPC creation requests to
//! the Cloud network kernel.

use std::collections::BTreeMap;

use oya_cloud_network_domain::{
    CloudNetworkCatalog, CloudNetworkError, IpProtocol, NetworkRepo, RouteCreate, RouteNextHopKind,
    RouteTableCreate, RuleDirection, SecurityGroupCreate, SecurityRule, Vpc, VpcCreate, VpcState,
};
use oya_data_boundary_kernel::{DataClass, parse_data_class_label};
use oya_residency_domain::{ResidencyClass, parse_residency_class_label};

pub const CLOUD_NETWORK_VPC_CREATE_SURFACE: &str = "cloud.network.vpc.create";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudNetworkVpcCreateApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

impl CloudNetworkVpcCreateApiStatus {
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
pub enum CloudNetworkVpcApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathVpcIdEmpty,
    VpcIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    IdempotencyKeyReused,
    ResidencyInvalid,
    DataClassInvalid,
    RouteNextHopInvalid,
    RuleDirectionInvalid,
    ProtocolInvalid,
    PortRangeInvalid,
    NetworkInvalidRequest,
    NetworkForbidden,
    NetworkNotFound,
    NetworkConflict,
}

impl CloudNetworkVpcApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "CLOUD_NETWORK_VPC_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "CLOUD_NETWORK_VPC_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "CLOUD_NETWORK_VPC_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "CLOUD_NETWORK_VPC_PRINCIPAL_ID_EMPTY",
            Self::PathVpcIdEmpty => "CLOUD_NETWORK_VPC_PATH_VPC_ID_EMPTY",
            Self::VpcIdMismatch => "CLOUD_NETWORK_VPC_ID_MISMATCH",
            Self::TenantMismatch => "CLOUD_NETWORK_VPC_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => {
                "CLOUD_NETWORK_VPC_AUTHORIZATION_DECISION_ID_EMPTY"
            }
            Self::AuthorizationTenantMismatch => "CLOUD_NETWORK_VPC_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "CLOUD_NETWORK_VPC_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "CLOUD_NETWORK_VPC_AUTHORIZATION_DENIED",
            Self::IdempotencyKeyReused => "CLOUD_NETWORK_VPC_IDEMPOTENCY_KEY_REUSED",
            Self::ResidencyInvalid => "CLOUD_NETWORK_VPC_RESIDENCY_INVALID",
            Self::DataClassInvalid => "CLOUD_NETWORK_VPC_DATA_CLASS_INVALID",
            Self::RouteNextHopInvalid => "CLOUD_NETWORK_VPC_ROUTE_NEXT_HOP_INVALID",
            Self::RuleDirectionInvalid => "CLOUD_NETWORK_VPC_RULE_DIRECTION_INVALID",
            Self::ProtocolInvalid => "CLOUD_NETWORK_VPC_PROTOCOL_INVALID",
            Self::PortRangeInvalid => "CLOUD_NETWORK_VPC_PORT_RANGE_INVALID",
            Self::NetworkInvalidRequest => "CLOUD_NETWORK_VPC_INVALID_REQUEST",
            Self::NetworkForbidden => "CLOUD_NETWORK_VPC_FORBIDDEN",
            Self::NetworkNotFound => "CLOUD_NETWORK_VPC_NOT_FOUND",
            Self::NetworkConflict => "CLOUD_NETWORK_VPC_CONFLICT",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcRouteCreateRequest {
    pub destination: String,        // data_class: PUBLIC
    pub next_hop: String,           // data_class: PUBLIC
    pub target_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcRouteTableCreateRequest {
    pub id: String,                                     // data_class: INTERNAL_ONLY
    pub routes: Vec<CloudNetworkVpcRouteCreateRequest>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcSecurityRuleCreateRequest {
    pub direction: String,       // data_class: PUBLIC
    pub protocol: String,        // data_class: PUBLIC
    pub port_start: Option<u16>, // data_class: PUBLIC
    pub port_end: Option<u16>,   // data_class: PUBLIC
    pub cidr: String,            // data_class: PUBLIC
    pub description: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcSecurityGroupCreateRequest {
    pub id: String,                                           // data_class: INTERNAL_ONLY
    pub rules: Vec<CloudNetworkVpcSecurityRuleCreateRequest>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcCreateRequest {
    pub resource_id: String,     // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub region: String,          // data_class: PUBLIC
    pub cidr_v4: String,         // data_class: PUBLIC
    pub cidr_v6: String,         // data_class: PUBLIC
    pub flow_logs_enabled: bool, // data_class: PUBLIC
    pub route_table: CloudNetworkVpcRouteTableCreateRequest, // data_class: INTERNAL_ONLY
    pub security_groups: Vec<CloudNetworkVpcSecurityGroupCreateRequest>, // data_class: INTERNAL_ONLY
    pub residency: String,             // data_class: INTERNAL_ONLY
    pub data_class: String,            // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcCreateApiRequest {
    pub path_vpc_id: String,                         // data_class: INTERNAL_ONLY
    pub boundary: CloudNetworkVpcApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: CloudNetworkVpcApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: CloudNetworkVpcApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: CloudNetworkVpcCreateRequest,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudNetworkVpcCreateIdempotencyLedger {
    entries:
        BTreeMap<CloudNetworkVpcIdempotencyLedgerKey, CloudNetworkVpcCreateIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl CloudNetworkVpcCreateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CloudNetworkVpcIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudNetworkVpcCreateIdempotencyLedgerEntry {
    fingerprint: CloudNetworkVpcRequestFingerprint, // data_class: INTERNAL_ONLY
    result: CloudNetworkVpcCreateApiResult,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CloudNetworkVpcRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

type CloudNetworkVpcCreateApiResult =
    Result<CloudNetworkVpcCreateSuccessResponse, CloudNetworkVpcApiError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcCreateSuccessResponse {
    pub data: CloudNetworkVpcRecord,          // data_class: INTERNAL_ONLY
    pub metadata: CloudNetworkVpcApiMetadata, // data_class: INTERNAL_ONLY
}

impl CloudNetworkVpcCreateSuccessResponse {
    pub fn created(data: CloudNetworkVpcRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: CloudNetworkVpcApiMetadata {
                request_id: request_id.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcApiMetadata {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcRecord {
    pub resource_id: String,           // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: PUBLIC
    pub cidr_v4: String,               // data_class: PUBLIC
    pub cidr_v6: String,               // data_class: PUBLIC
    pub flow_logs_enabled: bool,       // data_class: PUBLIC
    pub route_count: u32,              // data_class: PUBLIC
    pub security_group_count: u32,     // data_class: PUBLIC
    pub residency: String,             // data_class: INTERNAL_ONLY
    pub data_class: String,            // data_class: PUBLIC
    pub state: String,                 // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcApiErrorResponse {
    pub error: CloudNetworkVpcApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcApiErrorBody {
    pub code: String,                                // data_class: INTERNAL_ONLY
    pub message: String,                             // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,           // data_class: INTERNAL_ONLY
    pub request_id: String,                          // data_class: INTERNAL_ONLY
    pub details: Vec<CloudNetworkVpcApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudNetworkVpcApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudNetworkVpcApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathVpcId,
    VpcIdMismatch {
        path_vpc_id: String,
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
    InvalidResidencyLabel {
        residency: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    InvalidRouteNextHopLabel {
        next_hop: String,
    },
    InvalidRuleDirectionLabel {
        direction: String,
    },
    InvalidProtocolLabel {
        protocol: String,
    },
    InvalidPortRange,
    Network(CloudNetworkError),
}

impl CloudNetworkVpcApiError {
    pub fn vpc_create_status(&self) -> CloudNetworkVpcCreateApiStatus {
        match self.status_kind() {
            CloudNetworkVpcApiStatusKind::BadRequest => CloudNetworkVpcCreateApiStatus::BadRequest,
            CloudNetworkVpcApiStatusKind::Unauthorized => {
                CloudNetworkVpcCreateApiStatus::Unauthorized
            }
            CloudNetworkVpcApiStatusKind::Forbidden => CloudNetworkVpcCreateApiStatus::Forbidden,
            CloudNetworkVpcApiStatusKind::NotFound => CloudNetworkVpcCreateApiStatus::NotFound,
            CloudNetworkVpcApiStatusKind::Conflict => CloudNetworkVpcCreateApiStatus::Conflict,
            CloudNetworkVpcApiStatusKind::UnprocessableEntity => {
                CloudNetworkVpcCreateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn vpc_create_status_code(&self) -> u16 {
        self.vpc_create_status().code()
    }

    pub fn code(&self) -> CloudNetworkVpcApiErrorCode {
        match self {
            Self::EmptyRequestId => CloudNetworkVpcApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => CloudNetworkVpcApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => CloudNetworkVpcApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => CloudNetworkVpcApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathVpcId => CloudNetworkVpcApiErrorCode::PathVpcIdEmpty,
            Self::VpcIdMismatch { .. } => CloudNetworkVpcApiErrorCode::VpcIdMismatch,
            Self::TenantMismatch { .. } => CloudNetworkVpcApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                CloudNetworkVpcApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                CloudNetworkVpcApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                CloudNetworkVpcApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => CloudNetworkVpcApiErrorCode::AuthorizationDenied,
            Self::IdempotencyKeyReused { .. } => CloudNetworkVpcApiErrorCode::IdempotencyKeyReused,
            Self::InvalidResidencyLabel { .. } => CloudNetworkVpcApiErrorCode::ResidencyInvalid,
            Self::InvalidDataClassLabel { .. } => CloudNetworkVpcApiErrorCode::DataClassInvalid,
            Self::InvalidRouteNextHopLabel { .. } => {
                CloudNetworkVpcApiErrorCode::RouteNextHopInvalid
            }
            Self::InvalidRuleDirectionLabel { .. } => {
                CloudNetworkVpcApiErrorCode::RuleDirectionInvalid
            }
            Self::InvalidProtocolLabel { .. } => CloudNetworkVpcApiErrorCode::ProtocolInvalid,
            Self::InvalidPortRange => CloudNetworkVpcApiErrorCode::PortRangeInvalid,
            Self::Network(error) => match cloud_network_status_kind(error) {
                CloudNetworkVpcApiStatusKind::BadRequest => {
                    CloudNetworkVpcApiErrorCode::NetworkInvalidRequest
                }
                CloudNetworkVpcApiStatusKind::Forbidden => {
                    CloudNetworkVpcApiErrorCode::NetworkForbidden
                }
                CloudNetworkVpcApiStatusKind::NotFound => {
                    CloudNetworkVpcApiErrorCode::NetworkNotFound
                }
                CloudNetworkVpcApiStatusKind::Conflict => {
                    CloudNetworkVpcApiErrorCode::NetworkConflict
                }
                CloudNetworkVpcApiStatusKind::Unauthorized
                | CloudNetworkVpcApiStatusKind::UnprocessableEntity => {
                    CloudNetworkVpcApiErrorCode::NetworkInvalidRequest
                }
            },
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> CloudNetworkVpcApiErrorResponse {
        CloudNetworkVpcApiErrorResponse {
            error: CloudNetworkVpcApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> CloudNetworkVpcApiStatusKind {
        match self {
            Self::EmptyPrincipalId => CloudNetworkVpcApiStatusKind::Unauthorized,
            Self::TenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => CloudNetworkVpcApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => CloudNetworkVpcApiStatusKind::UnprocessableEntity,
            Self::Network(error) => cloud_network_status_kind(error),
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathVpcId
            | Self::VpcIdMismatch { .. }
            | Self::InvalidResidencyLabel { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::InvalidRouteNextHopLabel { .. }
            | Self::InvalidRuleDirectionLabel { .. }
            | Self::InvalidProtocolLabel { .. }
            | Self::InvalidPortRange => CloudNetworkVpcApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathVpcId => "Path VPC id is required",
            Self::VpcIdMismatch { .. } => "Path and body VPC ids must match",
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
                "Authorization decision does not allow the requested Cloud Network VPC surface"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidResidencyLabel { .. } => {
                "Request residency must be a known residency label"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::InvalidRouteNextHopLabel { .. } => {
                "Request route next_hop must be a known Cloud Network VPC next-hop label"
            }
            Self::InvalidRuleDirectionLabel { .. } => {
                "Request security rule direction must be ingress or egress"
            }
            Self::InvalidProtocolLabel { .. } => {
                "Request security rule protocol must be tcp, udp, icmp, or any"
            }
            Self::InvalidPortRange => "Request security rule port interval is invalid",
            Self::Network(error) => cloud_network_message(error),
        }
    }

    fn details(&self) -> Vec<CloudNetworkVpcApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathVpcId => vec![detail("path.vpc_id", "must be non-empty")],
            Self::VpcIdMismatch { .. } => vec![detail(
                "resource_id",
                "path vpc_id and body resource_id must match",
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
                "must include the requested Cloud Network VPC surface",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidResidencyLabel { .. } => vec![detail(
                "body.residency",
                "must be a canonical residency label",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::InvalidRouteNextHopLabel { .. } => vec![detail(
                "body.route_table.routes.next_hop",
                "must be local, internet_gateway, nat_gateway, vpc_peering, or transit_gateway",
            )],
            Self::InvalidRuleDirectionLabel { .. } => vec![detail(
                "body.security_groups.rules.direction",
                "must be ingress or egress",
            )],
            Self::InvalidProtocolLabel { .. } => vec![detail(
                "body.security_groups.rules.protocol",
                "must be tcp, udp, icmp, or any",
            )],
            Self::InvalidPortRange => vec![detail(
                "body.security_groups.rules.port_range",
                "port_start and port_end must be both absent or ordered non-zero ports",
            )],
            Self::Network(error) => vec![detail("cloud_network", cloud_network_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloudNetworkVpcApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_cloud_network_vpc_create_request(
    request: &CloudNetworkVpcCreateApiRequest,
) -> Result<(), CloudNetworkVpcApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_vpc_id(&request.path_vpc_id, &request.body.resource_id)?;
    validate_tenant_binding(
        &request.boundary,
        &request.principal,
        &request.body.tenant_id,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        CLOUD_NETWORK_VPC_CREATE_SURFACE,
    )
}

pub fn create_cloud_network_vpc_from_api(
    catalog: &mut CloudNetworkCatalog,
    idempotency_ledger: &mut CloudNetworkVpcCreateIdempotencyLedger,
    request: CloudNetworkVpcCreateApiRequest,
) -> Result<CloudNetworkVpcCreateSuccessResponse, CloudNetworkVpcApiError> {
    validate_cloud_network_vpc_create_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        CLOUD_NETWORK_VPC_CREATE_SURFACE,
    );
    let fingerprint = vpc_create_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return entry.result.clone();
        }
        return Err(CloudNetworkVpcApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let result = vpc_create_input(request.body)
        .and_then(|input| {
            catalog
                .create_vpc(input)
                .map_err(CloudNetworkVpcApiError::Network)
        })
        .map(|vpc| CloudNetworkVpcCreateSuccessResponse::created(vpc_record(vpc), request_id));
    idempotency_ledger.entries.insert(
        key,
        CloudNetworkVpcCreateIdempotencyLedgerEntry {
            fingerprint,
            result: result.clone(),
        },
    );
    result
}

fn validate_boundary(
    boundary: &CloudNetworkVpcApiBoundaryContext,
) -> Result<(), CloudNetworkVpcApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(CloudNetworkVpcApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(CloudNetworkVpcApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(CloudNetworkVpcApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_vpc_id(
    path_vpc_id: &str,
    body_resource_id: &str,
) -> Result<(), CloudNetworkVpcApiError> {
    if path_vpc_id.trim().is_empty() {
        return Err(CloudNetworkVpcApiError::EmptyPathVpcId);
    }
    if path_vpc_id != body_resource_id {
        return Err(CloudNetworkVpcApiError::VpcIdMismatch {
            path_vpc_id: path_vpc_id.to_string(),
            body_resource_id: body_resource_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    boundary: &CloudNetworkVpcApiBoundaryContext,
    principal: &CloudNetworkVpcApiPrincipal,
    body_tenant_id: &str,
) -> Result<(), CloudNetworkVpcApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(CloudNetworkVpcApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id || boundary.tenant_id != body_tenant_id {
        return Err(CloudNetworkVpcApiError::TenantMismatch {
            header_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &CloudNetworkVpcApiPrincipal,
    authorization: &CloudNetworkVpcApiAuthorization,
    surface: &str,
) -> Result<(), CloudNetworkVpcApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(CloudNetworkVpcApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(CloudNetworkVpcApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(CloudNetworkVpcApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(CloudNetworkVpcApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn vpc_create_input(
    body: CloudNetworkVpcCreateRequest,
) -> Result<VpcCreate, CloudNetworkVpcApiError> {
    Ok(VpcCreate {
        resource_id: body.resource_id,
        tenant_id: body.tenant_id,
        region: body.region,
        cidr_v4: body.cidr_v4,
        cidr_v6: body.cidr_v6,
        flow_logs_enabled: body.flow_logs_enabled,
        route_table: route_table_create_input(body.route_table)?,
        security_groups: body
            .security_groups
            .into_iter()
            .map(security_group_create_input)
            .collect::<Result<Vec<_>, _>>()?,
        residency: parse_api_residency(body.residency)?,
        state: VpcState::Creating,
        data_class: parse_api_data_class(body.data_class)?,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    })
}

fn route_table_create_input(
    request: CloudNetworkVpcRouteTableCreateRequest,
) -> Result<RouteTableCreate, CloudNetworkVpcApiError> {
    Ok(RouteTableCreate {
        id: request.id,
        routes: request
            .routes
            .into_iter()
            .map(route_create_input)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn route_create_input(
    request: CloudNetworkVpcRouteCreateRequest,
) -> Result<RouteCreate, CloudNetworkVpcApiError> {
    Ok(RouteCreate {
        destination: request.destination,
        next_hop: parse_api_route_next_hop(request.next_hop)?,
        target_ref: request.target_ref,
    })
}

fn security_group_create_input(
    request: CloudNetworkVpcSecurityGroupCreateRequest,
) -> Result<SecurityGroupCreate, CloudNetworkVpcApiError> {
    Ok(SecurityGroupCreate {
        id: request.id,
        rules: request
            .rules
            .into_iter()
            .map(security_rule_create_input)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn security_rule_create_input(
    request: CloudNetworkVpcSecurityRuleCreateRequest,
) -> Result<SecurityRule, CloudNetworkVpcApiError> {
    Ok(SecurityRule {
        direction: parse_api_rule_direction(request.direction)?,
        protocol: parse_api_protocol(request.protocol)?,
        port_range: parse_api_port_range(request.port_start, request.port_end)?,
        cidr: oya_cloud_network_domain::RouteDestination::new(request.cidr)
            .map_err(CloudNetworkVpcApiError::Network)?,
        description: request.description,
    })
}

fn parse_api_residency(label: String) -> Result<ResidencyClass, CloudNetworkVpcApiError> {
    parse_residency_class_label(&label)
        .ok_or(CloudNetworkVpcApiError::InvalidResidencyLabel { residency: label })
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudNetworkVpcApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudNetworkVpcApiError::InvalidDataClassLabel { data_class: label })
}

fn parse_api_route_next_hop(label: String) -> Result<RouteNextHopKind, CloudNetworkVpcApiError> {
    match label.as_str() {
        "local" => Ok(RouteNextHopKind::Local),
        "internet_gateway" => Ok(RouteNextHopKind::InternetGateway),
        "nat_gateway" => Ok(RouteNextHopKind::NatGateway),
        "vpc_peering" => Ok(RouteNextHopKind::VpcPeering),
        "transit_gateway" => Ok(RouteNextHopKind::TransitGateway),
        _ => Err(CloudNetworkVpcApiError::InvalidRouteNextHopLabel { next_hop: label }),
    }
}

fn parse_api_rule_direction(label: String) -> Result<RuleDirection, CloudNetworkVpcApiError> {
    match label.as_str() {
        "ingress" => Ok(RuleDirection::Ingress),
        "egress" => Ok(RuleDirection::Egress),
        _ => Err(CloudNetworkVpcApiError::InvalidRuleDirectionLabel { direction: label }),
    }
}

fn parse_api_protocol(label: String) -> Result<IpProtocol, CloudNetworkVpcApiError> {
    match label.as_str() {
        "tcp" => Ok(IpProtocol::Tcp),
        "udp" => Ok(IpProtocol::Udp),
        "icmp" => Ok(IpProtocol::Icmp),
        "any" => Ok(IpProtocol::Any),
        _ => Err(CloudNetworkVpcApiError::InvalidProtocolLabel { protocol: label }),
    }
}

fn parse_api_port_range(
    port_start: Option<u16>,
    port_end: Option<u16>,
) -> Result<Option<(u16, u16)>, CloudNetworkVpcApiError> {
    match (port_start, port_end) {
        (Some(start), Some(end)) if start != 0 && end != 0 && start <= end => {
            Ok(Some((start, end)))
        }
        (None, None) => Ok(None),
        (Some(_), Some(_)) | (Some(_), None) | (None, Some(_)) => {
            Err(CloudNetworkVpcApiError::InvalidPortRange)
        }
    }
}

fn idempotency_key_for(
    boundary: &CloudNetworkVpcApiBoundaryContext,
    principal: &CloudNetworkVpcApiPrincipal,
    surface: &str,
) -> CloudNetworkVpcIdempotencyLedgerKey {
    CloudNetworkVpcIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn vpc_create_fingerprint_for(
    request: &CloudNetworkVpcCreateApiRequest,
) -> CloudNetworkVpcRequestFingerprint {
    CloudNetworkVpcRequestFingerprint {
        canonical: [
            format!("path.vpc_id={}", request.path_vpc_id),
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
            format!("body.region={}", request.body.region),
            format!("body.cidr_v4={}", request.body.cidr_v4),
            format!("body.cidr_v6={}", request.body.cidr_v6),
            format!("body.flow_logs_enabled={}", request.body.flow_logs_enabled),
            format!("body.route_table={:?}", request.body.route_table),
            format!("body.security_groups={:?}", request.body.security_groups),
            format!("body.residency={}", request.body.residency),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.created_at_epoch_seconds={}",
                request.body.created_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn vpc_record(vpc: Vpc) -> CloudNetworkVpcRecord {
    CloudNetworkVpcRecord {
        resource_id: vpc.resource_id.value.value,
        tenant_id: vpc.tenant_id.value,
        region: vpc.region.value.value,
        cidr_v4: vpc.cidr_v4.value.value,
        cidr_v6: vpc.cidr_v6.value.value,
        flow_logs_enabled: vpc.flow_logs_enabled.value,
        route_count: vpc.route_table.value.routes.len() as u32,
        security_group_count: vpc.security_groups.value.len() as u32,
        residency: vpc
            .residency
            .value
            .label()
            .unwrap_or("per_pack")
            .to_string(),
        data_class: vpc.data_class.value.label().to_string(),
        state: vpc_state_label(vpc.state.value).to_string(),
        created_at_epoch_seconds: vpc.created_at_epoch_seconds.value,
        schema_version: vpc.schema_version.value,
    }
}

fn vpc_state_label(state: VpcState) -> &'static str {
    match state {
        VpcState::Creating => "creating",
        VpcState::Active => "active",
        VpcState::Suspended => "suspended",
        VpcState::Deleting => "deleting",
    }
}

fn cloud_network_status_kind(error: &CloudNetworkError) -> CloudNetworkVpcApiStatusKind {
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
        | CloudNetworkError::DuplicateDdosProtection => CloudNetworkVpcApiStatusKind::Conflict,
        CloudNetworkError::UnknownVpc
        | CloudNetworkError::UnknownSubnet
        | CloudNetworkError::UnknownLoadBalancer
        | CloudNetworkError::UnknownDnsZone
        | CloudNetworkError::UnknownInterconnectPartner
        | CloudNetworkError::UnknownProtectedResource => CloudNetworkVpcApiStatusKind::NotFound,
        CloudNetworkError::ResourceTenantMismatch
        | CloudNetworkError::ResourceRegionMismatch
        | CloudNetworkError::AzRegionMismatch
        | CloudNetworkError::SubnetOutsideVpc
        | CloudNetworkError::ListenerTargetGroupMissing
        | CloudNetworkError::PrivateZoneRequiresVpc
        | CloudNetworkError::PublicZoneMustNotBindVpc
        | CloudNetworkError::ScrubbingRegionRequired => CloudNetworkVpcApiStatusKind::Forbidden,
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
            CloudNetworkVpcApiStatusKind::BadRequest
        }
    }
}

fn cloud_network_message(error: &CloudNetworkError) -> &'static str {
    match cloud_network_status_kind(error) {
        CloudNetworkVpcApiStatusKind::BadRequest => "Cloud Network rejected the request shape",
        CloudNetworkVpcApiStatusKind::Unauthorized => "Cloud Network authentication is required",
        CloudNetworkVpcApiStatusKind::Forbidden => "Cloud Network policy denied the request",
        CloudNetworkVpcApiStatusKind::NotFound => "Cloud Network resource was not found",
        CloudNetworkVpcApiStatusKind::Conflict => "Cloud Network resource already exists",
        CloudNetworkVpcApiStatusKind::UnprocessableEntity => {
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
        CloudNetworkError::InvalidDataClass => "data_class must be public metadata for VPC create",
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

fn detail(field: &str, issue: &str) -> CloudNetworkVpcApiErrorDetail {
    CloudNetworkVpcApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
