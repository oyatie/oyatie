//! Status and error-code vocabularies of the upsert surface.

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
