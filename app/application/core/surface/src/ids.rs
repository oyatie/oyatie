//! Opaque surface, SKU, and provider identifiers.

use crate::error::CloudSurfaceError;
use crate::validate::prefixed_token;
use crate::{PROVIDER_REF_PREFIX, SKU_ID_PREFIX, SURFACE_ID_PREFIX};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudSurfaceId {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudSkuId {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProviderRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

impl CloudSurfaceId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudSurfaceError> {
        prefixed_token(
            value.into(),
            SURFACE_ID_PREFIX,
            CloudSurfaceError::InvalidSurfaceId,
        )
        .map(|value| Self { value })
    }
}

impl CloudSkuId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudSurfaceError> {
        prefixed_token(value.into(), SKU_ID_PREFIX, CloudSurfaceError::InvalidSkuId)
            .map(|value| Self { value })
    }
}

impl ProviderRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudSurfaceError> {
        prefixed_token(
            value.into(),
            PROVIDER_REF_PREFIX,
            CloudSurfaceError::InvalidProviderRef,
        )
        .map(|value| Self { value })
    }
}
