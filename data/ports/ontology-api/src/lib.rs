//! Platform Object Graph entity upsert API boundary.
//!
//! This crate owns authenticated REST-boundary normalization, path/body tenant
//! and entity binding, request fingerprint idempotency, property tier/data-class
//! parsing, in-memory row-isolated entity projection, and stable public error
//! projection for `object-graph.entity.upsert` before handing typed entity
//! construction to the Object Graph kernel.

use std::collections::BTreeMap;

use data_ontology_domain::{ObjectEntity, ObjectGraphError, ObjectProperty, PropertyTier};
use data_boundary_kernel::{PrivacyDataClass, parse_data_class_label};

pub const OBJECT_GRAPH_ENTITY_UPSERT_SURFACE: &str = "object-graph.entity.upsert";
pub const OBJECT_GRAPH_ENTITY_UPSERT_OPENAPI_CONTRACT: &str =
    "contracts/openapi/platform/platform-object-graph-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectGraphEntityUpsertApiStatus {
    Ok,
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

impl ObjectGraphEntityUpsertApiStatus {
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
pub enum ObjectGraphEntityUpsertApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathTenantIdEmpty,
    PathEntityIdEmpty,
    TenantPathBodyMismatch,
    EntityPathBodyMismatch,
    PrincipalTenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    PropertyTierInvalid,
    PropertyDataClassInvalid,
    IdempotencyKeyReused,
    KernelInvalidEntityId,
    KernelEmptyEntityType,
    KernelMissingProperties,
    KernelEmptyPropertyName,
    KernelInvalidDataClass,
}

impl ObjectGraphEntityUpsertApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "OBJECT_GRAPH_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "OBJECT_GRAPH_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "OBJECT_GRAPH_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "OBJECT_GRAPH_PRINCIPAL_ID_EMPTY",
            Self::PathTenantIdEmpty => "OBJECT_GRAPH_PATH_TENANT_ID_EMPTY",
            Self::PathEntityIdEmpty => "OBJECT_GRAPH_PATH_ENTITY_ID_EMPTY",
            Self::TenantPathBodyMismatch => "OBJECT_GRAPH_TENANT_PATH_BODY_MISMATCH",
            Self::EntityPathBodyMismatch => "OBJECT_GRAPH_ENTITY_PATH_BODY_MISMATCH",
            Self::PrincipalTenantMismatch => "OBJECT_GRAPH_PRINCIPAL_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "OBJECT_GRAPH_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "OBJECT_GRAPH_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => "OBJECT_GRAPH_AUTHORIZATION_PRINCIPAL_MISMATCH",
            Self::AuthorizationDenied => "OBJECT_GRAPH_AUTHORIZATION_DENIED",
            Self::PropertyTierInvalid => "OBJECT_GRAPH_PROPERTY_TIER_INVALID",
            Self::PropertyDataClassInvalid => "OBJECT_GRAPH_PROPERTY_DATA_CLASS_INVALID",
            Self::IdempotencyKeyReused => "OBJECT_GRAPH_IDEMPOTENCY_KEY_REUSED",
            Self::KernelInvalidEntityId => "OBJECT_GRAPH_KERNEL_INVALID_ENTITY_ID",
            Self::KernelEmptyEntityType => "OBJECT_GRAPH_KERNEL_EMPTY_ENTITY_TYPE",
            Self::KernelMissingProperties => "OBJECT_GRAPH_KERNEL_MISSING_PROPERTIES",
            Self::KernelEmptyPropertyName => "OBJECT_GRAPH_KERNEL_EMPTY_PROPERTY_NAME",
            Self::KernelInvalidDataClass => "OBJECT_GRAPH_KERNEL_INVALID_DATA_CLASS",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityPropertyRef {
    pub name: String,       // data_class: INTERNAL_ONLY
    pub value: String,      // data_class: INTERNAL_ONLY
    pub tier: String,       // data_class: INTERNAL_ONLY
    pub data_class: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertRequest {
    pub tenant_id: String,   // data_class: INTERNAL_ONLY
    pub entity_id: String,   // data_class: INTERNAL_ONLY
    pub entity_type: String, // data_class: INTERNAL_ONLY
    pub property_refs: Vec<ObjectGraphEntityPropertyRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertApiRequest {
    pub path_tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub path_entity_id: String,                  // data_class: INTERNAL_ONLY
    pub boundary: ObjectGraphApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: ObjectGraphApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: ObjectGraphApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: ObjectGraphEntityUpsertRequest,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectGraphEntityDirectory {
    entities: BTreeMap<ObjectGraphEntityKey, ObjectEntity>, // data_class: INTERNAL_ONLY
    events: Vec<ObjectGraphEntityMutationEvent>,            // data_class: INTERNAL_ONLY
}

impl ObjectGraphEntityDirectory {
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ObjectGraphEntityKey {
    tenant_id: String, // data_class: INTERNAL_ONLY
    entity_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObjectGraphEntityMutationEvent {
    event_id: String,   // data_class: INTERNAL_ONLY
    tenant_id: String,  // data_class: INTERNAL_ONLY
    entity_id: String,  // data_class: INTERNAL_ONLY
    request_id: String, // data_class: INTERNAL_ONLY
    result: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertIdempotencyLedger {
    entries: BTreeMap<
        ObjectGraphEntityUpsertIdempotencyLedgerKey,
        ObjectGraphEntityUpsertIdempotencyLedgerEntry,
    >, // data_class: INTERNAL_ONLY
}

impl ObjectGraphEntityUpsertIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ObjectGraphEntityUpsertIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: INTERNAL_ONLY
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObjectGraphEntityUpsertIdempotencyLedgerEntry {
    fingerprint: ObjectGraphEntityUpsertRequestFingerprint, // data_class: INTERNAL_ONLY
    result: ObjectGraphEntityUpsertSuccessResponse,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObjectGraphEntityUpsertRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertSuccessResponse {
    pub data: ObjectGraphEntityRecord, // data_class: INTERNAL_ONLY
    pub metadata: ObjectGraphEntityUpsertMetadata, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertMetadata {
    pub request_id: String,   // data_class: INTERNAL_ONLY
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
    pub result: String,       // data_class: INTERNAL_ONLY
    pub event_id: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityRecord {
    pub tenant_id: String,   // data_class: INTERNAL_ONLY
    pub entity_id: String,   // data_class: INTERNAL_ONLY
    pub entity_type: String, // data_class: INTERNAL_ONLY
    pub property_refs: Vec<ObjectGraphEntityPropertyRef>, // data_class: INTERNAL_ONLY
    pub schema_version: u32, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertApiErrorResponse {
    pub error: ObjectGraphEntityUpsertApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertApiErrorBody {
    pub code: String,                                        // data_class: INTERNAL_ONLY
    pub message: String,                                     // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,                   // data_class: INTERNAL_ONLY
    pub request_id: String,                                  // data_class: INTERNAL_ONLY
    pub details: Vec<ObjectGraphEntityUpsertApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGraphEntityUpsertApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObjectGraphEntityUpsertApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathTenantId,
    EmptyPathEntityId,
    TenantPathBodyMismatch {
        path_tenant_id: String,
        body_tenant_id: String,
    },
    EntityPathBodyMismatch {
        path_entity_id: String,
        body_entity_id: String,
    },
    PrincipalTenantMismatch {
        principal_tenant_id: String,
        boundary_tenant_id: String,
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
    InvalidPropertyTier {
        tier: String,
    },
    InvalidPropertyDataClass {
        data_class: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    Kernel(ObjectGraphError),
}

impl ObjectGraphEntityUpsertApiError {
    pub fn object_graph_entity_upsert_status(&self) -> ObjectGraphEntityUpsertApiStatus {
        match self.status_kind() {
            ObjectGraphEntityUpsertApiStatusKind::BadRequest => {
                ObjectGraphEntityUpsertApiStatus::BadRequest
            }
            ObjectGraphEntityUpsertApiStatusKind::Unauthorized => {
                ObjectGraphEntityUpsertApiStatus::Unauthorized
            }
            ObjectGraphEntityUpsertApiStatusKind::Forbidden => {
                ObjectGraphEntityUpsertApiStatus::Forbidden
            }
            ObjectGraphEntityUpsertApiStatusKind::UnprocessableEntity => {
                ObjectGraphEntityUpsertApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn object_graph_entity_upsert_status_code(&self) -> u16 {
        self.object_graph_entity_upsert_status().code()
    }

    pub fn code(&self) -> ObjectGraphEntityUpsertApiErrorCode {
        match self {
            Self::EmptyRequestId => ObjectGraphEntityUpsertApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => ObjectGraphEntityUpsertApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => ObjectGraphEntityUpsertApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => ObjectGraphEntityUpsertApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathTenantId => ObjectGraphEntityUpsertApiErrorCode::PathTenantIdEmpty,
            Self::EmptyPathEntityId => ObjectGraphEntityUpsertApiErrorCode::PathEntityIdEmpty,
            Self::TenantPathBodyMismatch { .. } => {
                ObjectGraphEntityUpsertApiErrorCode::TenantPathBodyMismatch
            }
            Self::EntityPathBodyMismatch { .. } => {
                ObjectGraphEntityUpsertApiErrorCode::EntityPathBodyMismatch
            }
            Self::PrincipalTenantMismatch { .. } => {
                ObjectGraphEntityUpsertApiErrorCode::PrincipalTenantMismatch
            }
            Self::EmptyAuthorizationDecisionId => {
                ObjectGraphEntityUpsertApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                ObjectGraphEntityUpsertApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                ObjectGraphEntityUpsertApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => {
                ObjectGraphEntityUpsertApiErrorCode::AuthorizationDenied
            }
            Self::InvalidPropertyTier { .. } => {
                ObjectGraphEntityUpsertApiErrorCode::PropertyTierInvalid
            }
            Self::InvalidPropertyDataClass { .. } => {
                ObjectGraphEntityUpsertApiErrorCode::PropertyDataClassInvalid
            }
            Self::IdempotencyKeyReused { .. } => {
                ObjectGraphEntityUpsertApiErrorCode::IdempotencyKeyReused
            }
            Self::Kernel(error) => object_graph_kernel_error_code(error),
        }
    }

    pub fn error_response(
        &self,
        request_id: impl Into<String>,
    ) -> ObjectGraphEntityUpsertApiErrorResponse {
        ObjectGraphEntityUpsertApiErrorResponse {
            error: ObjectGraphEntityUpsertApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> ObjectGraphEntityUpsertApiStatusKind {
        match self {
            Self::EmptyPrincipalId => ObjectGraphEntityUpsertApiStatusKind::Unauthorized,
            Self::PrincipalTenantMismatch { .. }
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => ObjectGraphEntityUpsertApiStatusKind::Forbidden,
            Self::IdempotencyKeyReused { .. } => {
                ObjectGraphEntityUpsertApiStatusKind::UnprocessableEntity
            }
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathTenantId
            | Self::EmptyPathEntityId
            | Self::TenantPathBodyMismatch { .. }
            | Self::EntityPathBodyMismatch { .. }
            | Self::InvalidPropertyTier { .. }
            | Self::InvalidPropertyDataClass { .. }
            | Self::Kernel(_) => ObjectGraphEntityUpsertApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathTenantId => "Path tenant id is required",
            Self::EmptyPathEntityId => "Path entity id is required",
            Self::TenantPathBodyMismatch { .. } => {
                "Path tenant id must match request body tenant_id"
            }
            Self::EntityPathBodyMismatch { .. } => {
                "Path entity id must match request body entity_id"
            }
            Self::PrincipalTenantMismatch { .. } => {
                "Authenticated principal tenant must match the tenant header"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal id"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Object Graph entity upsert surface"
            }
            Self::InvalidPropertyTier { .. } => {
                "Object Graph property tier must be scalar, vector, timeseries, geo, ciphertext, or struct"
            }
            Self::InvalidPropertyDataClass { .. } => {
                "Object Graph property data_class must be a privacy-program data class"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::Kernel(error) => object_graph_kernel_error_message(error),
        }
    }

    fn details(&self) -> Vec<ObjectGraphEntityUpsertApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathTenantId => vec![detail("path.tenant_id", "must be non-empty")],
            Self::EmptyPathEntityId => vec![detail("path.entity_id", "must be non-empty")],
            Self::TenantPathBodyMismatch { .. } => vec![detail(
                "body.tenant_id",
                "must match the tenant_id path parameter",
            )],
            Self::EntityPathBodyMismatch { .. } => vec![detail(
                "body.entity_id",
                "must match the entity_id path parameter",
            )],
            Self::PrincipalTenantMismatch { .. } => vec![detail(
                "principal.tenant_id",
                "must match the authenticated tenant header",
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
                "must include the requested object-graph.entity.upsert surface",
            )],
            Self::InvalidPropertyTier { .. } => vec![detail(
                "body.property_refs[].tier",
                "must be a closed Object Graph property tier label",
            )],
            Self::InvalidPropertyDataClass { .. } => vec![detail(
                "body.property_refs[].data_class",
                "must be a privacy-program data-class label",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::Kernel(error) => vec![detail(
                "object_graph_kernel",
                object_graph_kernel_issue(error),
            )],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ObjectGraphEntityUpsertApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    UnprocessableEntity,
}

pub fn validate_object_graph_entity_upsert_request(
    request: &ObjectGraphEntityUpsertApiRequest,
) -> Result<(), ObjectGraphEntityUpsertApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_body_binding(request)?;
    validate_principal_binding(&request.boundary, &request.principal)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        OBJECT_GRAPH_ENTITY_UPSERT_SURFACE,
    )?;
    for property in &request.body.property_refs {
        parse_property_tier(&property.tier)?;
        parse_property_data_class(&property.data_class)?;
    }
    Ok(())
}

pub fn upsert_object_graph_entity_from_api(
    directory: &mut ObjectGraphEntityDirectory,
    idempotency_ledger: &mut ObjectGraphEntityUpsertIdempotencyLedger,
    request: ObjectGraphEntityUpsertApiRequest,
) -> Result<ObjectGraphEntityUpsertSuccessResponse, ObjectGraphEntityUpsertApiError> {
    validate_object_graph_entity_upsert_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        OBJECT_GRAPH_ENTITY_UPSERT_SURFACE,
    );
    let fingerprint = object_graph_entity_upsert_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(ObjectGraphEntityUpsertApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let entity_key = ObjectGraphEntityKey {
        tenant_id: request.body.tenant_id.clone(),
        entity_id: request.body.entity_id.clone(),
    };
    let result = if directory.entities.contains_key(&entity_key) {
        "updated"
    } else {
        "created"
    };
    let entity = object_entity_from_request(&request.body)?;
    directory.entities.insert(entity_key, entity.clone());
    let event = object_graph_entity_mutation_event(&request, result);
    directory.events.push(event.clone());
    let response = ObjectGraphEntityUpsertSuccessResponse {
        data: object_graph_entity_record(&entity),
        metadata: ObjectGraphEntityUpsertMetadata {
            request_id: request.boundary.request_id.clone(),
            tenant_id: request.boundary.tenant_id.clone(),
            principal_id: request.principal.principal_id.clone(),
            result: result.to_string(),
            event_id: event.event_id,
        },
    };
    idempotency_ledger.entries.insert(
        key,
        ObjectGraphEntityUpsertIdempotencyLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(
    boundary: &ObjectGraphApiBoundaryContext,
) -> Result<(), ObjectGraphEntityUpsertApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_body_binding(
    request: &ObjectGraphEntityUpsertApiRequest,
) -> Result<(), ObjectGraphEntityUpsertApiError> {
    if request.path_tenant_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyPathTenantId);
    }
    if request.path_entity_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyPathEntityId);
    }
    if request.path_tenant_id != request.body.tenant_id {
        return Err(ObjectGraphEntityUpsertApiError::TenantPathBodyMismatch {
            path_tenant_id: request.path_tenant_id.clone(),
            body_tenant_id: request.body.tenant_id.clone(),
        });
    }
    if request.path_entity_id != request.body.entity_id {
        return Err(ObjectGraphEntityUpsertApiError::EntityPathBodyMismatch {
            path_entity_id: request.path_entity_id.clone(),
            body_entity_id: request.body.entity_id.clone(),
        });
    }
    Ok(())
}

fn validate_principal_binding(
    boundary: &ObjectGraphApiBoundaryContext,
    principal: &ObjectGraphApiPrincipal,
) -> Result<(), ObjectGraphEntityUpsertApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyPrincipalId);
    }
    if principal.tenant_id != boundary.tenant_id {
        return Err(ObjectGraphEntityUpsertApiError::PrincipalTenantMismatch {
            principal_tenant_id: principal.tenant_id.clone(),
            boundary_tenant_id: boundary.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &ObjectGraphApiPrincipal,
    authorization: &ObjectGraphApiAuthorization,
    surface: &str,
) -> Result<(), ObjectGraphEntityUpsertApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(ObjectGraphEntityUpsertApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(
            ObjectGraphEntityUpsertApiError::AuthorizationTenantMismatch {
                authorization_tenant_id: authorization.tenant_id.clone(),
                principal_tenant_id: principal.tenant_id.clone(),
            },
        );
    }
    if authorization.principal_id != principal.principal_id {
        return Err(
            ObjectGraphEntityUpsertApiError::AuthorizationPrincipalMismatch {
                authorization_principal_id: authorization.principal_id.clone(),
                principal_id: principal.principal_id.clone(),
            },
        );
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed| allowed == surface)
    {
        return Err(ObjectGraphEntityUpsertApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn object_entity_from_request(
    request: &ObjectGraphEntityUpsertRequest,
) -> Result<ObjectEntity, ObjectGraphEntityUpsertApiError> {
    let properties = request
        .property_refs
        .iter()
        .map(object_property_from_ref)
        .collect::<Result<Vec<_>, _>>()?;
    ObjectEntity::new(
        request.tenant_id.clone(),
        request.entity_id.clone(),
        request.entity_type.clone(),
        properties,
    )
    .map_err(ObjectGraphEntityUpsertApiError::Kernel)
}

fn object_property_from_ref(
    property: &ObjectGraphEntityPropertyRef,
) -> Result<ObjectProperty, ObjectGraphEntityUpsertApiError> {
    Ok(ObjectProperty::new(
        property.name.clone(),
        property.value.clone(),
        parse_property_tier(&property.tier)?,
        parse_property_data_class(&property.data_class)?,
    ))
}

fn parse_property_tier(tier: &str) -> Result<PropertyTier, ObjectGraphEntityUpsertApiError> {
    match tier.trim() {
        "scalar" => Ok(PropertyTier::Scalar),
        "vector" => Ok(PropertyTier::Vector),
        "timeseries" => Ok(PropertyTier::Timeseries),
        "geo" => Ok(PropertyTier::Geo),
        "ciphertext" => Ok(PropertyTier::Ciphertext),
        "struct" => Ok(PropertyTier::Struct),
        _ => Err(ObjectGraphEntityUpsertApiError::InvalidPropertyTier {
            tier: tier.to_string(),
        }),
    }
}

fn parse_property_data_class(
    label: &str,
) -> Result<PrivacyDataClass, ObjectGraphEntityUpsertApiError> {
    let data_class = parse_data_class_label(label).ok_or_else(|| {
        ObjectGraphEntityUpsertApiError::InvalidPropertyDataClass {
            data_class: label.to_string(),
        }
    })?;
    PrivacyDataClass::try_from(data_class).map_err(|_| {
        ObjectGraphEntityUpsertApiError::InvalidPropertyDataClass {
            data_class: label.to_string(),
        }
    })
}

fn object_graph_entity_record(entity: &ObjectEntity) -> ObjectGraphEntityRecord {
    ObjectGraphEntityRecord {
        tenant_id: entity.tenant_id.clone(),
        entity_id: entity.id.clone(),
        entity_type: entity.entity_type.value.clone(),
        property_refs: entity
            .properties
            .values()
            .map(object_graph_property_ref)
            .collect(),
        schema_version: 1,
    }
}

fn object_graph_property_ref(property: &ObjectProperty) -> ObjectGraphEntityPropertyRef {
    ObjectGraphEntityPropertyRef {
        name: property.name.clone(),
        value: property.value.value.clone(),
        tier: property_tier_label(property.tier).to_string(),
        data_class: property.value.data_class.label().to_string(),
    }
}

fn object_graph_entity_mutation_event(
    request: &ObjectGraphEntityUpsertApiRequest,
    result: &str,
) -> ObjectGraphEntityMutationEvent {
    ObjectGraphEntityMutationEvent {
        event_id: format!("evt_og_{}", request.boundary.request_id),
        tenant_id: request.body.tenant_id.clone(),
        entity_id: request.body.entity_id.clone(),
        request_id: request.boundary.request_id.clone(),
        result: result.to_string(),
    }
}

fn property_tier_label(tier: PropertyTier) -> &'static str {
    match tier {
        PropertyTier::Scalar => "scalar",
        PropertyTier::Vector => "vector",
        PropertyTier::Timeseries => "timeseries",
        PropertyTier::Geo => "geo",
        PropertyTier::Ciphertext => "ciphertext",
        PropertyTier::Struct => "struct",
    }
}

fn idempotency_key_for(
    boundary: &ObjectGraphApiBoundaryContext,
    principal: &ObjectGraphApiPrincipal,
    surface: &str,
) -> ObjectGraphEntityUpsertIdempotencyLedgerKey {
    ObjectGraphEntityUpsertIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn object_graph_entity_upsert_fingerprint_for(
    request: &ObjectGraphEntityUpsertApiRequest,
) -> ObjectGraphEntityUpsertRequestFingerprint {
    let mut canonical = format!(
        "path_tenant_id={};path_entity_id={};tenant_id={};entity_id={};entity_type={};",
        request.path_tenant_id,
        request.path_entity_id,
        request.body.tenant_id,
        request.body.entity_id,
        request.body.entity_type
    );
    for property in &request.body.property_refs {
        canonical.push_str(&format!(
            "property[name={},value={},tier={},data_class={}];",
            property.name, property.value, property.tier, property.data_class
        ));
    }
    ObjectGraphEntityUpsertRequestFingerprint { canonical }
}

fn object_graph_kernel_error_code(error: &ObjectGraphError) -> ObjectGraphEntityUpsertApiErrorCode {
    match error {
        ObjectGraphError::InvalidEntityId => {
            ObjectGraphEntityUpsertApiErrorCode::KernelInvalidEntityId
        }
        ObjectGraphError::EmptyEntityType => {
            ObjectGraphEntityUpsertApiErrorCode::KernelEmptyEntityType
        }
        ObjectGraphError::MissingProperties => {
            ObjectGraphEntityUpsertApiErrorCode::KernelMissingProperties
        }
        ObjectGraphError::EmptyPropertyName => {
            ObjectGraphEntityUpsertApiErrorCode::KernelEmptyPropertyName
        }
        ObjectGraphError::InvalidDataClass => {
            ObjectGraphEntityUpsertApiErrorCode::KernelInvalidDataClass
        }
    }
}

fn object_graph_kernel_error_message(error: &ObjectGraphError) -> &'static str {
    match error {
        ObjectGraphError::InvalidEntityId => "Object Graph entity id must use the ent_ shape",
        ObjectGraphError::EmptyEntityType => "Object Graph entity type is required",
        ObjectGraphError::MissingProperties => "Object Graph entity requires at least one property",
        ObjectGraphError::EmptyPropertyName => "Object Graph property names must be non-empty",
        ObjectGraphError::InvalidDataClass => "Object Graph property data class is invalid",
    }
}

fn object_graph_kernel_issue(error: &ObjectGraphError) -> &'static str {
    match error {
        ObjectGraphError::InvalidEntityId => "entity_id must start with ent_",
        ObjectGraphError::EmptyEntityType => "entity_type must be non-empty",
        ObjectGraphError::MissingProperties => "properties must contain at least one property",
        ObjectGraphError::EmptyPropertyName => "property name must be non-empty",
        ObjectGraphError::InvalidDataClass => "property data_class must be a privacy class",
    }
}

fn detail(field: &str, issue: &str) -> ObjectGraphEntityUpsertApiErrorDetail {
    ObjectGraphEntityUpsertApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}
