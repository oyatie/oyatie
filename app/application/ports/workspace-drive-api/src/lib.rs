//! Workspace Drive API boundary for object metadata PUT and GET.
//!
//! This crate owns HTTP-boundary normalization, idempotent object metadata
//! creation, coarse authorization proof checks, and per-object ACL projection
//! around the Workspace Drive kernel. Object bytes remain behind Cloud Storage;
//! this boundary records the metadata, KMS-shred binding, and permission graph
//! needed by Search, DSR, audit-chain, and Foundry consumers.

use std::collections::BTreeMap;

use oya_data_boundary_kernel::parse_data_class_label;
use storage_drive_domain::{
    DriveError, DriveObject, DriveObjectCreate, DriveRole, PermissionGrant, PermissionSet,
    workspace_drive_data_class_from_legacy,
};

pub const WORKSPACE_DRIVE_PUT_SURFACE: &str = "workspace.drive.put";
pub const WORKSPACE_DRIVE_GET_SURFACE: &str = "workspace.drive.get";
pub const WORKSPACE_DRIVE_OPENAPI_CONTRACT: &str =
    "contracts/openapi/workspace/workspace-drive-v1.yaml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceDriveObjectPutApiStatus {
    Created,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl WorkspaceDriveObjectPutApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceDriveObjectGetApiStatus {
    Ok,
    BadRequest,
    Forbidden,
    NotFound,
}

impl WorkspaceDriveObjectGetApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::NotFound => 404,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceDriveApiErrorCode {
    RequestIdEmpty,
    TenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    ObjectIdInvalid,
    ObjectIdMismatch,
    TenantMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    PermissionDenied,
    IdempotencyKeyReused,
    DataClassInvalid,
    RoleInvalid,
    ObjectNotFound,
    ObjectAlreadyExists,
    DriveInvalidRequest,
}

impl WorkspaceDriveApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "WORKSPACE_DRIVE_REQUEST_ID_EMPTY",
            Self::TenantHeaderEmpty => "WORKSPACE_DRIVE_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "WORKSPACE_DRIVE_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "WORKSPACE_DRIVE_PRINCIPAL_ID_EMPTY",
            Self::ObjectIdInvalid => "WORKSPACE_DRIVE_OBJECT_ID_INVALID",
            Self::ObjectIdMismatch => "WORKSPACE_DRIVE_OBJECT_ID_MISMATCH",
            Self::TenantMismatch => "WORKSPACE_DRIVE_TENANT_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "WORKSPACE_DRIVE_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "WORKSPACE_DRIVE_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "WORKSPACE_DRIVE_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "WORKSPACE_DRIVE_AUTHORIZATION_DENIED",
            Self::PermissionDenied => "WORKSPACE_DRIVE_PERMISSION_DENIED",
            Self::IdempotencyKeyReused => "WORKSPACE_DRIVE_IDEMPOTENCY_KEY_REUSED",
            Self::DataClassInvalid => "WORKSPACE_DRIVE_DATA_CLASS_INVALID",
            Self::RoleInvalid => "WORKSPACE_DRIVE_ROLE_INVALID",
            Self::ObjectNotFound => "WORKSPACE_DRIVE_OBJECT_NOT_FOUND",
            Self::ObjectAlreadyExists => "WORKSPACE_DRIVE_OBJECT_ALREADY_EXISTS",
            Self::DriveInvalidRequest => "WORKSPACE_DRIVE_INVALID_REQUEST",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveMutationBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveReadBoundaryContext {
    pub request_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: PII_IDENTIFYING
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: PII_IDENTIFYING
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDrivePermissionGrantRequest {
    pub subject_ref: String,           // data_class: PII_IDENTIFYING
    pub role: String,                  // data_class: INTERNAL_ONLY
    pub granted_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveObjectPutRequest {
    pub object_id: String,          // data_class: INTERNAL_ONLY
    pub folder_id: String,          // data_class: INTERNAL_ONLY
    pub path: String,               // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub region: String,             // data_class: INTERNAL_ONLY
    pub data_class: String,         // data_class: INTERNAL_ONLY
    pub object_storage_key: String, // data_class: INTERNAL_ONLY
    pub size_bytes: u64,            // data_class: INTERNAL_ONLY
    pub mime_type: String,          // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: String,   // data_class: INTERNAL_ONLY
    pub permissions: Vec<WorkspaceDrivePermissionGrantRequest>, // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveObjectPutApiRequest {
    pub path_object_id: String, // data_class: INTERNAL_ONLY
    pub boundary: WorkspaceDriveMutationBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: WorkspaceDriveApiPrincipal, // data_class: PII_IDENTIFYING
    pub authorization: WorkspaceDriveApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: WorkspaceDriveObjectPutRequest, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveObjectGetApiRequest {
    pub path_object_id: String, // data_class: INTERNAL_ONLY
    pub boundary: WorkspaceDriveReadBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: WorkspaceDriveApiPrincipal, // data_class: PII_IDENTIFYING
    pub authorization: WorkspaceDriveApiAuthorization, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceDriveObjectDirectory {
    objects: BTreeMap<WorkspaceDriveObjectKey, DriveObject>, // data_class: INTERNAL_ONLY
}

impl WorkspaceDriveObjectDirectory {
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn objects(&self) -> impl Iterator<Item = &DriveObject> {
        self.objects.values()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkspaceDriveObjectKey {
    tenant_id: String, // data_class: INTERNAL_ONLY
    object_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceDriveObjectPutIdempotencyLedger {
    entries: BTreeMap<WorkspaceDriveIdempotencyLedgerKey, WorkspaceDrivePutLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl WorkspaceDriveObjectPutIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct WorkspaceDriveIdempotencyLedgerKey {
    tenant_id: String,       // data_class: INTERNAL_ONLY
    principal_id: String,    // data_class: PII_IDENTIFYING
    surface: String,         // data_class: INTERNAL_ONLY
    idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceDrivePutLedgerEntry {
    fingerprint: WorkspaceDriveRequestFingerprint, // data_class: INTERNAL_ONLY
    result: WorkspaceDriveObjectPutSuccessResponse, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceDriveRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveObjectPutSuccessResponse {
    pub data: WorkspaceDriveObjectRecord, // data_class: INTERNAL_ONLY
    pub metadata: WorkspaceDriveObjectMetadata, // data_class: INTERNAL_ONLY
}

impl WorkspaceDriveObjectPutSuccessResponse {
    pub fn created(data: WorkspaceDriveObjectRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: WorkspaceDriveObjectMetadata {
                request_id: request_id.into(),
                surface: WORKSPACE_DRIVE_PUT_SURFACE.to_string(),
                openapi_contract: WORKSPACE_DRIVE_OPENAPI_CONTRACT.to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveObjectGetSuccessResponse {
    pub data: WorkspaceDriveObjectRecord, // data_class: INTERNAL_ONLY
    pub metadata: WorkspaceDriveObjectMetadata, // data_class: INTERNAL_ONLY
}

impl WorkspaceDriveObjectGetSuccessResponse {
    pub fn ok(data: WorkspaceDriveObjectRecord, request_id: impl Into<String>) -> Self {
        Self {
            data,
            metadata: WorkspaceDriveObjectMetadata {
                request_id: request_id.into(),
                surface: WORKSPACE_DRIVE_GET_SURFACE.to_string(),
                openapi_contract: WORKSPACE_DRIVE_OPENAPI_CONTRACT.to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveObjectMetadata {
    pub request_id: String,       // data_class: INTERNAL_ONLY
    pub surface: String,          // data_class: INTERNAL_ONLY
    pub openapi_contract: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveObjectRecord {
    pub object_id: String,          // data_class: INTERNAL_ONLY
    pub folder_id: String,          // data_class: INTERNAL_ONLY
    pub path: String,               // data_class: PII_QUASI_IDENTIFIER
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub region: String,             // data_class: INTERNAL_ONLY
    pub data_class: String,         // data_class: INTERNAL_ONLY
    pub object_storage_key: String, // data_class: INTERNAL_ONLY
    pub size_bytes: u64,            // data_class: INTERNAL_ONLY
    pub mime_type: String,          // data_class: INTERNAL_ONLY
    pub kms_shred_key_id: String,   // data_class: INTERNAL_ONLY
    pub permissions: Vec<WorkspaceDrivePermissionGrantRecord>, // data_class: PII_IDENTIFYING
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub schema_version: u32,        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDrivePermissionGrantRecord {
    pub subject_ref: String,           // data_class: PII_IDENTIFYING
    pub role: String,                  // data_class: INTERNAL_ONLY
    pub granted_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveApiErrorResponse {
    pub error: WorkspaceDriveApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveApiErrorBody {
    pub code: String,                               // data_class: INTERNAL_ONLY
    pub message: String,                            // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,          // data_class: INTERNAL_ONLY
    pub request_id: String,                         // data_class: INTERNAL_ONLY
    pub details: Vec<WorkspaceDriveApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceDriveApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceDriveApiError {
    EmptyRequestId,
    EmptyTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    InvalidObjectId {
        object_id: String,
    },
    ObjectIdMismatch {
        path_object_id: String,
        body_object_id: String,
    },
    TenantMismatch {
        header_tenant_id: String,
        principal_tenant_id: String,
        authorization_tenant_id: Option<String>,
        body_tenant_id: Option<String>,
        resource_tenant_id: Option<String>,
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
    PermissionDenied {
        principal_id: String,
        object_id: String,
        required_role: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    InvalidDataClassLabel {
        data_class: String,
    },
    InvalidDriveRole {
        role: String,
    },
    ObjectNotFound {
        tenant_id: String,
        object_id: String,
    },
    ObjectAlreadyExists {
        tenant_id: String,
        object_id: String,
    },
    Drive(DriveError),
}

impl WorkspaceDriveApiError {
    pub fn object_status_code(&self) -> u16 {
        match self.status_kind() {
            WorkspaceDriveApiStatusKind::BadRequest => 400,
            WorkspaceDriveApiStatusKind::Forbidden => 403,
            WorkspaceDriveApiStatusKind::NotFound => 404,
            WorkspaceDriveApiStatusKind::Conflict => 409,
            WorkspaceDriveApiStatusKind::UnprocessableEntity => 422,
        }
    }

    pub fn code(&self) -> WorkspaceDriveApiErrorCode {
        match self {
            Self::EmptyRequestId => WorkspaceDriveApiErrorCode::RequestIdEmpty,
            Self::EmptyTenantHeader => WorkspaceDriveApiErrorCode::TenantHeaderEmpty,
            Self::EmptyIdempotencyKey => WorkspaceDriveApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => WorkspaceDriveApiErrorCode::PrincipalIdEmpty,
            Self::InvalidObjectId { .. } => WorkspaceDriveApiErrorCode::ObjectIdInvalid,
            Self::ObjectIdMismatch { .. } => WorkspaceDriveApiErrorCode::ObjectIdMismatch,
            Self::TenantMismatch { .. } => WorkspaceDriveApiErrorCode::TenantMismatch,
            Self::EmptyAuthorizationDecisionId => {
                WorkspaceDriveApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                WorkspaceDriveApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                WorkspaceDriveApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => WorkspaceDriveApiErrorCode::AuthorizationDenied,
            Self::PermissionDenied { .. } => WorkspaceDriveApiErrorCode::PermissionDenied,
            Self::IdempotencyKeyReused { .. } => WorkspaceDriveApiErrorCode::IdempotencyKeyReused,
            Self::InvalidDataClassLabel { .. } => WorkspaceDriveApiErrorCode::DataClassInvalid,
            Self::InvalidDriveRole { .. } => WorkspaceDriveApiErrorCode::RoleInvalid,
            Self::ObjectNotFound { .. } => WorkspaceDriveApiErrorCode::ObjectNotFound,
            Self::ObjectAlreadyExists { .. } => WorkspaceDriveApiErrorCode::ObjectAlreadyExists,
            Self::Drive(_) => WorkspaceDriveApiErrorCode::DriveInvalidRequest,
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> WorkspaceDriveApiErrorResponse {
        WorkspaceDriveApiErrorResponse {
            error: WorkspaceDriveApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> WorkspaceDriveApiStatusKind {
        match self {
            Self::TenantMismatch { .. }
            | Self::EmptyPrincipalId
            | Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. }
            | Self::PermissionDenied { .. } => WorkspaceDriveApiStatusKind::Forbidden,
            Self::ObjectNotFound { .. } => WorkspaceDriveApiStatusKind::NotFound,
            Self::ObjectAlreadyExists { .. } => WorkspaceDriveApiStatusKind::Conflict,
            Self::IdempotencyKeyReused { .. } => WorkspaceDriveApiStatusKind::UnprocessableEntity,
            Self::EmptyRequestId
            | Self::EmptyTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::InvalidObjectId { .. }
            | Self::ObjectIdMismatch { .. }
            | Self::InvalidDataClassLabel { .. }
            | Self::InvalidDriveRole { .. }
            | Self::Drive(_) => WorkspaceDriveApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyTenantHeader => "X-Tenant-Id header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::InvalidObjectId { .. } => "Workspace Drive object id is required",
            Self::ObjectIdMismatch { .. } => "Path and body object ids must match",
            Self::TenantMismatch { .. } => {
                "Tenant header must match authenticated principal, authorization, body, and resource"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested Workspace Drive surface"
            }
            Self::PermissionDenied { .. } => {
                "Workspace Drive object ACL does not grant the required role"
            }
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::InvalidDataClassLabel { .. } => {
                "Request data_class must be a known privacy data class"
            }
            Self::InvalidDriveRole { .. } => "Permission role must be a known Drive role",
            Self::ObjectNotFound { .. } => "Workspace Drive object was not found",
            Self::ObjectAlreadyExists { .. } => "Workspace Drive object already exists",
            Self::Drive(error) => drive_error_message(error),
        }
    }

    fn details(&self) -> Vec<WorkspaceDriveApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyTenantHeader => vec![detail("header.X-Tenant-Id", "must be non-empty")],
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::InvalidObjectId { .. } => {
                vec![detail("path.object_id", "must be non-empty")]
            }
            Self::ObjectIdMismatch { .. } => vec![detail(
                "object_id",
                "path object_id and body object_id must match",
            )],
            Self::TenantMismatch { .. } => vec![detail(
                "tenant_id",
                "header tenant, principal tenant, authorization tenant, body tenant_id, and resource tenant must match",
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
                "must include the requested Workspace Drive surface",
            )],
            Self::PermissionDenied { .. } => vec![detail(
                "permissions",
                "principal must have an object ACL grant for the requested operation",
            )],
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::InvalidDataClassLabel { .. } => vec![detail(
                "body.data_class",
                "must be a canonical privacy data-class label",
            )],
            Self::InvalidDriveRole { .. } => vec![detail(
                "body.permissions.role",
                "must be one of viewer, commenter, editor, owner",
            )],
            Self::ObjectNotFound { .. } => vec![detail(
                "path.object_id",
                "object metadata was not found for the requested tenant",
            )],
            Self::ObjectAlreadyExists { .. } => vec![detail(
                "path.object_id",
                "object metadata already exists for the requested tenant",
            )],
            Self::Drive(error) => vec![detail("workspace_drive", drive_error_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkspaceDriveApiStatusKind {
    BadRequest,
    Forbidden,
    NotFound,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_workspace_drive_object_put_request(
    request: &WorkspaceDriveObjectPutApiRequest,
) -> Result<(), WorkspaceDriveApiError> {
    validate_mutation_boundary(&request.boundary)?;
    validate_path_object_id(&request.path_object_id)?;
    validate_path_body_binding(&request.path_object_id, &request.body.object_id)?;
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        &request.authorization,
        Some(&request.body.tenant_id),
        None,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        WORKSPACE_DRIVE_PUT_SURFACE,
    )?;
    Ok(())
}

pub fn put_workspace_drive_object_from_api(
    directory: &mut WorkspaceDriveObjectDirectory,
    idempotency_ledger: &mut WorkspaceDriveObjectPutIdempotencyLedger,
    request: WorkspaceDriveObjectPutApiRequest,
) -> Result<WorkspaceDriveObjectPutSuccessResponse, WorkspaceDriveApiError> {
    validate_workspace_drive_object_put_request(&request)?;
    let key = idempotency_key_for(
        &request.boundary,
        &request.principal,
        WORKSPACE_DRIVE_PUT_SURFACE,
    );
    let fingerprint = put_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(WorkspaceDriveApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }

    let request_id = request.boundary.request_id.clone();
    let object_id = request.body.object_id.clone();
    let object = drive_object_from_request(request.body)?;
    require_owner(&object, &request.principal.principal_id, &object_id)?;
    let directory_key = WorkspaceDriveObjectKey {
        tenant_id: object.tenant_id.value.clone(),
        object_id: object.id.value.clone(),
    };
    if directory.objects.contains_key(&directory_key) {
        return Err(WorkspaceDriveApiError::ObjectAlreadyExists {
            tenant_id: directory_key.tenant_id,
            object_id: directory_key.object_id,
        });
    }

    let response =
        WorkspaceDriveObjectPutSuccessResponse::created(object_record(&object), request_id);
    directory.objects.insert(directory_key, object);
    idempotency_ledger.entries.insert(
        key,
        WorkspaceDrivePutLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

pub fn validate_workspace_drive_object_get_request(
    request: &WorkspaceDriveObjectGetApiRequest,
) -> Result<(), WorkspaceDriveApiError> {
    validate_read_boundary(&request.boundary)?;
    validate_path_object_id(&request.path_object_id)?;
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        &request.authorization,
        None,
        None,
    )?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        WORKSPACE_DRIVE_GET_SURFACE,
    )
}

pub fn get_workspace_drive_object_from_api(
    directory: &WorkspaceDriveObjectDirectory,
    request: WorkspaceDriveObjectGetApiRequest,
) -> Result<WorkspaceDriveObjectGetSuccessResponse, WorkspaceDriveApiError> {
    validate_workspace_drive_object_get_request(&request)?;
    let request_id = request.boundary.request_id.clone();
    let directory_key = WorkspaceDriveObjectKey {
        tenant_id: request.boundary.tenant_id.clone(),
        object_id: request.path_object_id.clone(),
    };
    let object = directory.objects.get(&directory_key).ok_or_else(|| {
        WorkspaceDriveApiError::ObjectNotFound {
            tenant_id: request.boundary.tenant_id.clone(),
            object_id: request.path_object_id.clone(),
        }
    })?;
    validate_tenant_binding(
        &request.boundary.tenant_id,
        &request.principal,
        &request.authorization,
        None,
        Some(&object.tenant_id.value),
    )?;
    require_view(
        object,
        &request.principal.principal_id,
        &request.path_object_id,
    )?;
    Ok(WorkspaceDriveObjectGetSuccessResponse::ok(
        object_record(object),
        request_id,
    ))
}

fn validate_mutation_boundary(
    boundary: &WorkspaceDriveMutationBoundaryContext,
) -> Result<(), WorkspaceDriveApiError> {
    validate_request_tenant(&boundary.request_id, &boundary.tenant_id)?;
    if boundary.idempotency_key.trim().is_empty() {
        return Err(WorkspaceDriveApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_read_boundary(
    boundary: &WorkspaceDriveReadBoundaryContext,
) -> Result<(), WorkspaceDriveApiError> {
    validate_request_tenant(&boundary.request_id, &boundary.tenant_id)
}

fn validate_request_tenant(
    request_id: &str,
    tenant_id: &str,
) -> Result<(), WorkspaceDriveApiError> {
    if request_id.trim().is_empty() {
        return Err(WorkspaceDriveApiError::EmptyRequestId);
    }
    if tenant_id.trim().is_empty() {
        return Err(WorkspaceDriveApiError::EmptyTenantHeader);
    }
    Ok(())
}

fn validate_path_object_id(object_id: &str) -> Result<(), WorkspaceDriveApiError> {
    if object_id.trim().is_empty() {
        return Err(WorkspaceDriveApiError::InvalidObjectId {
            object_id: object_id.to_string(),
        });
    }
    Ok(())
}

fn validate_path_body_binding(
    path_object_id: &str,
    body_object_id: &str,
) -> Result<(), WorkspaceDriveApiError> {
    if path_object_id != body_object_id {
        return Err(WorkspaceDriveApiError::ObjectIdMismatch {
            path_object_id: path_object_id.to_string(),
            body_object_id: body_object_id.to_string(),
        });
    }
    Ok(())
}

fn validate_tenant_binding(
    header_tenant_id: &str,
    principal: &WorkspaceDriveApiPrincipal,
    authorization: &WorkspaceDriveApiAuthorization,
    body_tenant_id: Option<&str>,
    resource_tenant_id: Option<&str>,
) -> Result<(), WorkspaceDriveApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(WorkspaceDriveApiError::EmptyPrincipalId);
    }
    if header_tenant_id != principal.tenant_id {
        return Err(WorkspaceDriveApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            authorization_tenant_id: Some(authorization.tenant_id.clone()),
            body_tenant_id: body_tenant_id.map(str::to_string),
            resource_tenant_id: resource_tenant_id.map(str::to_string),
        });
    }
    if header_tenant_id != authorization.tenant_id {
        return Err(WorkspaceDriveApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            authorization_tenant_id: Some(authorization.tenant_id.clone()),
            body_tenant_id: body_tenant_id.map(str::to_string),
            resource_tenant_id: resource_tenant_id.map(str::to_string),
        });
    }
    if body_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id) {
        return Err(WorkspaceDriveApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            authorization_tenant_id: Some(authorization.tenant_id.clone()),
            body_tenant_id: body_tenant_id.map(str::to_string),
            resource_tenant_id: resource_tenant_id.map(str::to_string),
        });
    }
    if resource_tenant_id.is_some_and(|tenant_id| header_tenant_id != tenant_id) {
        return Err(WorkspaceDriveApiError::TenantMismatch {
            header_tenant_id: header_tenant_id.to_string(),
            principal_tenant_id: principal.tenant_id.clone(),
            authorization_tenant_id: Some(authorization.tenant_id.clone()),
            body_tenant_id: body_tenant_id.map(str::to_string),
            resource_tenant_id: resource_tenant_id.map(str::to_string),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &WorkspaceDriveApiPrincipal,
    authorization: &WorkspaceDriveApiAuthorization,
    surface: &str,
) -> Result<(), WorkspaceDriveApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(WorkspaceDriveApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(WorkspaceDriveApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(WorkspaceDriveApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed| allowed == surface)
    {
        return Err(WorkspaceDriveApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn drive_object_from_request(
    request: WorkspaceDriveObjectPutRequest,
) -> Result<DriveObject, WorkspaceDriveApiError> {
    let data_class = parse_data_class_label(&request.data_class).ok_or_else(|| {
        WorkspaceDriveApiError::InvalidDataClassLabel {
            data_class: request.data_class.clone(),
        }
    })?;
    let data_class = workspace_drive_data_class_from_legacy(data_class).map_err(|_| {
        WorkspaceDriveApiError::InvalidDataClassLabel {
            data_class: request.data_class.clone(),
        }
    })?;
    let permissions = permission_set_from_requests(request.permissions)?;
    DriveObject::new(DriveObjectCreate {
        id: request.object_id,
        folder_id: request.folder_id,
        path: request.path,
        tenant_id: request.tenant_id,
        region: request.region,
        data_class: Some(data_class),
        object_storage_key: request.object_storage_key,
        size_bytes: request.size_bytes,
        mime_type: request.mime_type,
        kms_shred_key_id: request.kms_shred_key_id,
        permissions,
        created_at_epoch_seconds: request.created_at_epoch_seconds,
    })
    .map_err(WorkspaceDriveApiError::Drive)
}

fn permission_set_from_requests(
    grants: Vec<WorkspaceDrivePermissionGrantRequest>,
) -> Result<PermissionSet, WorkspaceDriveApiError> {
    grants
        .into_iter()
        .map(permission_grant_from_request)
        .collect::<Result<Vec<_>, _>>()
        .and_then(|grants| PermissionSet::new(grants).map_err(WorkspaceDriveApiError::Drive))
}

fn permission_grant_from_request(
    grant: WorkspaceDrivePermissionGrantRequest,
) -> Result<PermissionGrant, WorkspaceDriveApiError> {
    PermissionGrant::new(
        grant.subject_ref,
        role_from_label(&grant.role)?,
        grant.granted_at_epoch_seconds,
    )
    .map_err(WorkspaceDriveApiError::Drive)
}

fn role_from_label(role: &str) -> Result<DriveRole, WorkspaceDriveApiError> {
    match role.trim() {
        "viewer" => Ok(DriveRole::Viewer),
        "commenter" => Ok(DriveRole::Commenter),
        "editor" => Ok(DriveRole::Editor),
        "owner" => Ok(DriveRole::Owner),
        _ => Err(WorkspaceDriveApiError::InvalidDriveRole {
            role: role.to_string(),
        }),
    }
}

fn role_label(role: DriveRole) -> &'static str {
    match role {
        DriveRole::Viewer => "viewer",
        DriveRole::Commenter => "commenter",
        DriveRole::Editor => "editor",
        DriveRole::Owner => "owner",
    }
}

fn require_owner(
    object: &DriveObject,
    principal_id: &str,
    object_id: &str,
) -> Result<(), WorkspaceDriveApiError> {
    if object.permissions.value.role_for_subject(principal_id) == Some(DriveRole::Owner) {
        return Ok(());
    }
    Err(WorkspaceDriveApiError::PermissionDenied {
        principal_id: principal_id.to_string(),
        object_id: object_id.to_string(),
        required_role: "owner".to_string(),
    })
}

fn require_view(
    object: &DriveObject,
    principal_id: &str,
    object_id: &str,
) -> Result<(), WorkspaceDriveApiError> {
    if object.permissions.value.can_view(principal_id) {
        return Ok(());
    }
    Err(WorkspaceDriveApiError::PermissionDenied {
        principal_id: principal_id.to_string(),
        object_id: object_id.to_string(),
        required_role: "viewer".to_string(),
    })
}

fn idempotency_key_for(
    boundary: &WorkspaceDriveMutationBoundaryContext,
    principal: &WorkspaceDriveApiPrincipal,
    surface: &str,
) -> WorkspaceDriveIdempotencyLedgerKey {
    WorkspaceDriveIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn put_fingerprint_for(
    request: &WorkspaceDriveObjectPutApiRequest,
) -> WorkspaceDriveRequestFingerprint {
    let permissions = request
        .body
        .permissions
        .iter()
        .map(|grant| {
            format!(
                "{}:{}:{}",
                grant.subject_ref, grant.role, grant.granted_at_epoch_seconds
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    WorkspaceDriveRequestFingerprint {
        canonical: [
            format!("path.object_id={}", request.path_object_id),
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
            format!("body.object_id={}", request.body.object_id),
            format!("body.folder_id={}", request.body.folder_id),
            format!("body.path={}", request.body.path),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.region={}", request.body.region),
            format!("body.data_class={}", request.body.data_class),
            format!(
                "body.object_storage_key={}",
                request.body.object_storage_key
            ),
            format!("body.size_bytes={}", request.body.size_bytes),
            format!("body.mime_type={}", request.body.mime_type),
            format!("body.kms_shred_key_id={}", request.body.kms_shred_key_id),
            format!("body.permissions={permissions}"),
            format!(
                "body.created_at_epoch_seconds={}",
                request.body.created_at_epoch_seconds
            ),
        ]
        .join("|"),
    }
}

fn object_record(object: &DriveObject) -> WorkspaceDriveObjectRecord {
    WorkspaceDriveObjectRecord {
        object_id: object.id.value.clone(),
        folder_id: object.folder_id.value.clone(),
        path: object.path.value.clone(),
        tenant_id: object.tenant_id.value.clone(),
        region: object.region.value.clone(),
        data_class: object.privacy_data_class().label().to_string(),
        object_storage_key: object.object_storage_key.value.clone(),
        size_bytes: object.size_bytes.value,
        mime_type: object.mime_type.value.clone(),
        kms_shred_key_id: object.kms_shred_key_id.value.clone(),
        permissions: object
            .permissions
            .value
            .grants
            .iter()
            .map(|grant| WorkspaceDrivePermissionGrantRecord {
                subject_ref: grant.subject_ref.value.clone(),
                role: role_label(grant.role.value).to_string(),
                granted_at_epoch_seconds: grant.granted_at_epoch_seconds.value,
            })
            .collect(),
        created_at_epoch_seconds: object.created_at_epoch_seconds.value,
        schema_version: object.schema_version.value,
    }
}

fn detail(field: &str, issue: &str) -> WorkspaceDriveApiErrorDetail {
    WorkspaceDriveApiErrorDetail {
        field: field.to_string(),
        issue: issue.to_string(),
    }
}

fn drive_error_message(error: &DriveError) -> &'static str {
    match error {
        DriveError::InvalidObjectId => "Workspace Drive object id is invalid",
        DriveError::InvalidFolderId => "Workspace Drive folder id is invalid",
        DriveError::InvalidTenantId => "Workspace Drive tenant id is invalid",
        DriveError::InvalidRegion => "Workspace Drive region is invalid",
        DriveError::InvalidPath => "Workspace Drive path is invalid",
        DriveError::InvalidObjectStorageKey => "Workspace Drive object storage key is invalid",
        DriveError::InvalidMimeType => "Workspace Drive MIME type is invalid",
        DriveError::InvalidKmsShredKeyId => "Workspace Drive KMS shred key id is invalid",
        DriveError::InvalidPermissionSubject => "Workspace Drive permission subject is invalid",
        DriveError::EmptyPermissionSet => "Workspace Drive permission set is required",
        DriveError::MissingOwnerGrant => "Workspace Drive permission set must include an owner",
        DriveError::InvalidDataClass => "Workspace Drive data class is invalid",
    }
}

fn drive_error_issue(error: &DriveError) -> &'static str {
    match error {
        DriveError::InvalidObjectId => "object_id must be non-empty",
        DriveError::InvalidFolderId => "folder_id must be non-empty",
        DriveError::InvalidTenantId => "tenant_id must be non-empty",
        DriveError::InvalidRegion => "region must be non-empty",
        DriveError::InvalidPath => {
            "path must start with /, cannot use parent traversal, and cannot end with /"
        }
        DriveError::InvalidObjectStorageKey => "object_storage_key must be non-empty",
        DriveError::InvalidMimeType => "mime_type must be non-empty",
        DriveError::InvalidKmsShredKeyId => "kms_shred_key_id must be non-empty",
        DriveError::InvalidPermissionSubject => "permission subject_ref must be non-empty",
        DriveError::EmptyPermissionSet => "permissions must contain at least one grant",
        DriveError::MissingOwnerGrant => "permissions must contain at least one owner grant",
        DriveError::InvalidDataClass => "data_class must be a privacy-program data class",
    }
}
