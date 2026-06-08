#![forbid(unsafe_code)]
//! Provider-neutral search/indexing port contracts for Drive and office content.
//!
//! This crate keeps search contracts tenant-scoped and redacted before any OpenSearch-compatible
//! adapter is adopted.

use oya_office_kernel::{DataClass, ObjectId, TenantId};

/// Stable crate identifier used by workspace and Buck2 scaffold verification.
pub const CRATE_NAME: &str = "oya-office-search-kernel";

/// Product vertical slice owned by this crate.
pub const VERTICAL_SLICE: &str = "platform";

/// Canonical ADR-0056 architectural layer represented by this crate. Port-trait
/// declarations + pure typed contracts live in the `kernel` layer.
pub const ARCHITECTURE_LAYER: &str = "kernel";

/// Search validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchPortError {
    message: String,
}

impl SearchPortError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the validation message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

impl core::fmt::Display for SearchPortError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for SearchPortError {}

/// Search index name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SearchIndexName(String);

impl SearchIndexName {
    /// Creates a search index name.
    pub fn new(value: impl Into<String>) -> Result<Self, SearchPortError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(SearchPortError::new("search index name must not be empty"));
        }
        if !trimmed
            .chars()
            .all(|character| character.is_ascii_lowercase() || character == '-')
        {
            return Err(SearchPortError::new(
                "search index name must use lowercase letters and hyphens",
            ));
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns index name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Search response projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SearchProjection {
    /// Metadata only, no preview snippets.
    MetadataOnly,
    /// Metadata plus redacted preview snippets.
    MetadataAndPreview,
}

/// Redacted Drive search document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveSearchDocument {
    tenant_id: TenantId,
    object_id: ObjectId,
    data_class: DataClass,
    title: String,
    preview_available: bool,
}

impl DriveSearchDocument {
    /// Creates a search document without storage pointer or KMS reference leakage.
    pub fn new(
        tenant_id: TenantId,
        object_id: ObjectId,
        data_class: DataClass,
        title: impl Into<String>,
        preview_available: bool,
    ) -> Result<Self, SearchPortError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(SearchPortError::new(
                "search document title must not be empty",
            ));
        }
        Ok(Self {
            tenant_id,
            object_id,
            data_class,
            title: title.trim().to_owned(),
            preview_available,
        })
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns object id.
    #[must_use]
    pub const fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    /// Returns data class.
    #[must_use]
    pub const fn data_class(&self) -> DataClass {
        self.data_class
    }

    /// Returns title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns true when a redacted preview is available.
    #[must_use]
    pub const fn preview_available(&self) -> bool {
        self.preview_available
    }
}

/// Bounded Drive search query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriveSearchQuery {
    index_name: SearchIndexName,
    tenant_id: TenantId,
    query: String,
    limit: u16,
    projection: SearchProjection,
}

impl DriveSearchQuery {
    /// Maximum query result limit.
    pub const MAX_LIMIT: u16 = 200;

    /// Creates a bounded search query.
    pub fn new(
        index_name: SearchIndexName,
        tenant_id: TenantId,
        query: impl Into<String>,
        limit: u16,
        projection: SearchProjection,
    ) -> Result<Self, SearchPortError> {
        let query = query.into();
        if query.trim().is_empty() {
            return Err(SearchPortError::new("drive search query must not be empty"));
        }
        let limit = if limit == 0 {
            50
        } else if limit > Self::MAX_LIMIT {
            Self::MAX_LIMIT
        } else {
            limit
        };
        Ok(Self {
            index_name,
            tenant_id,
            query: query.trim().to_owned(),
            limit,
            projection,
        })
    }

    /// Returns index name.
    #[must_use]
    pub const fn index_name(&self) -> &SearchIndexName {
        &self.index_name
    }

    /// Returns tenant id.
    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Returns query string.
    #[must_use]
    pub fn query(&self) -> &str {
        self.query.as_str()
    }

    /// Returns bounded limit.
    #[must_use]
    pub const fn limit(&self) -> u16 {
        self.limit
    }

    /// Returns projection.
    #[must_use]
    pub const fn projection(&self) -> SearchProjection {
        self.projection
    }
}

#[cfg(test)]
mod tests {
    use oya_office_kernel::{DataClass, ObjectId, TenantId};

    use super::{
        ARCHITECTURE_LAYER, CRATE_NAME, DriveSearchDocument, DriveSearchQuery, SearchIndexName,
        SearchProjection, VERTICAL_SLICE,
    };

    #[test]
    fn scaffold_identity_is_declared() {
        assert!(!ARCHITECTURE_LAYER.is_empty());
        assert!(!CRATE_NAME.is_empty());
        assert!(!VERTICAL_SLICE.is_empty());
    }

    #[test]
    fn search_documents_and_queries_are_tenant_scoped_and_bounded() {
        let document = DriveSearchDocument::new(
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            ObjectId::new("object-1").expect("valid object id"),
            DataClass::Confidential,
            "Quarterly Plan",
            true,
        )
        .expect("valid document");
        assert!(document.preview_available());

        let query = DriveSearchQuery::new(
            SearchIndexName::new("drive-metadata").expect("valid index"),
            TenantId::new("tenant-alpha").expect("valid tenant id"),
            "plan",
            999,
            SearchProjection::MetadataOnly,
        )
        .expect("valid query");
        assert_eq!(query.limit(), DriveSearchQuery::MAX_LIMIT);
    }
}
