use serde::{Deserialize, Serialize};

use crate::error::ContractShapeError;
use crate::identity::ResourceName;

/// Maximum page size any provider must accept.
pub const MAX_PAGE_SIZE: u32 = 1000;

/// An opaque pagination cursor (AIP-158: tokens are opaque to clients).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageToken(String);

impl PageToken {
    /// Build a non-empty page token.
    pub fn new(value: impl Into<String>) -> Result<Self, ContractShapeError> {
        let value = value.into();
        if value.is_empty() {
            Err(ContractShapeError::EmptyPageToken)
        } else {
            Ok(Self(value))
        }
    }

    /// The raw token text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A list-page request: bounded page size + optional cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageRequest {
    pub page_size: u32,                // data_class: INTERNAL_ONLY
    pub page_token: Option<PageToken>, // data_class: INTERNAL_ONLY
}

impl PageRequest {
    /// First page with the given size (`1..=MAX_PAGE_SIZE`).
    pub fn first(page_size: u32) -> Result<Self, ContractShapeError> {
        if page_size == 0 || page_size > MAX_PAGE_SIZE {
            return Err(ContractShapeError::PageSizeOutOfRange {
                requested: page_size,
            });
        }
        Ok(Self {
            page_size,
            page_token: None,
        })
    }

    /// The page after `token` with the same size.
    #[must_use]
    pub fn after(&self, token: PageToken) -> Self {
        Self {
            page_size: self.page_size,
            page_token: Some(token),
        }
    }
}

/// One listed entry: the resource plus its name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListEntry<R> {
    pub name: ResourceName, // data_class: TENANT_SCOPED
    pub resource: R,        // data_class: TENANT_SCOPED
}

/// One page of list results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page<T> {
    pub items: Vec<T>,                      // data_class: TENANT_SCOPED
    pub next_page_token: Option<PageToken>, // data_class: INTERNAL_ONLY
}
