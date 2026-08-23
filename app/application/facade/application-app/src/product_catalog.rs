//! Application product-catalog value objects (P19-application merge-variant delta-1).
//!
//! `ProductEntry` and `ProductMetadata` are the kernel-layer value objects described
//! in the P19 impl-plan for the product-enablement bounded context.  They carry no
//! framework dependencies and impose no new workspace crate requirements.

/// Stable product identifier used as a routing key across bounded contexts.
///
/// Examples: `"workflow"`, `"ontology"`, `"search"`, `"connect"`.
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ProductId(String);

impl ProductId {
    /// Constructs a new `ProductId`, trimming whitespace and rejecting empty strings.
    pub fn new(id: impl Into<String>) -> Result<Self, ProductCatalogError> {
        let id = id.into();
        let trimmed = id.trim();
        if trimmed.is_empty() {
            return Err(ProductCatalogError::EmptyProductId);
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProductId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Human-readable metadata about a product shown in the tenant launchpad.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductMetadata {
    /// Display name shown in the UI, e.g. `"Workflow Studio"`.
    display_name: String,
    /// Short one-sentence description of the product.
    description: String,
    /// Relative icon slug, e.g. `"workflow-icon"`.
    icon_slug: String,
    /// Subdomain path suffix, e.g. `"workflow"` → `app.oyatie.com/workflow/…`.
    /// Stored as a single path segment (no leading/trailing slashes).
    subdomain: String,
}

impl ProductMetadata {
    /// Constructs `ProductMetadata`, rejecting empty display_name or subdomain,
    /// and rejecting subdomain values that contain `/` (must be a single slug).
    pub fn new(
        display_name: impl Into<String>,
        description: impl Into<String>,
        icon_slug: impl Into<String>,
        subdomain: impl Into<String>,
    ) -> Result<Self, ProductCatalogError> {
        let display_name = display_name.into();
        let subdomain = subdomain.into().trim().to_owned();
        let subdomain = subdomain.trim_matches('/').to_owned();
        if display_name.trim().is_empty() {
            return Err(ProductCatalogError::EmptyDisplayName);
        }
        if subdomain.is_empty() {
            return Err(ProductCatalogError::EmptySubdomain);
        }
        // Reject any character that is not a URL-safe slug character.
        // Allowed: ASCII alphanumeric, hyphen, underscore, dot.
        if subdomain
            .chars()
            .any(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.'))
        {
            return Err(ProductCatalogError::InvalidSubdomainSlug);
        }
        // Reject dot-segment slugs (".", "..") which resolve as path traversal
        // components when embedded in a URL path (e.g. `app.oyatie.com/../`).
        if subdomain == "." || subdomain == ".." {
            return Err(ProductCatalogError::InvalidSubdomainSlug);
        }
        Ok(Self {
            display_name,
            description: description.into(),
            icon_slug: icon_slug.into(),
            subdomain,
        })
    }

    /// Returns the display name.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the icon slug.
    pub fn icon_slug(&self) -> &str {
        &self.icon_slug
    }

    /// Returns the subdomain slug (single path segment, no slashes).
    pub fn subdomain(&self) -> &str {
        &self.subdomain
    }
}

/// A single row in a tenant's enabled-product catalog.
///
/// `ProductEntry` combines a stable `ProductId` with human-readable `ProductMetadata`
/// and a flag indicating whether the product is currently active for this tenant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductEntry {
    /// Stable routing key.
    pub id: ProductId,
    /// Display metadata for the launchpad UI.
    pub metadata: ProductMetadata,
    /// `true` if the tenant has an active subscription to this product.
    pub enabled: bool,
}

impl ProductEntry {
    /// Constructs a new `ProductEntry`.
    pub fn new(id: ProductId, metadata: ProductMetadata, enabled: bool) -> Self {
        Self {
            id,
            metadata,
            enabled,
        }
    }

    /// Returns `true` when this entry should appear in the launchpad (enabled).
    pub fn is_active(&self) -> bool {
        self.enabled
    }

    /// Returns the deep-link path for this product under `app.oyatie.com`.
    ///
    /// Example: `ProductId("workflow")` → `"/workflow"`.
    pub fn deep_link_path(&self) -> String {
        format!("/{}", self.metadata.subdomain())
    }
}

/// Errors produced by the product-catalog value objects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductCatalogError {
    EmptyProductId,
    EmptyDisplayName,
    EmptySubdomain,
    /// Subdomain contained a `/` after trimming leading/trailing slashes,
    /// meaning it is a multi-segment path rather than a single slug.
    InvalidSubdomainSlug,
}

impl std::fmt::Display for ProductCatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyProductId => f.write_str("product_id must not be empty"),
            Self::EmptyDisplayName => f.write_str("display_name must not be empty"),
            Self::EmptySubdomain => f.write_str("subdomain must not be empty"),
            Self::InvalidSubdomainSlug => {
                f.write_str("subdomain must be a URL-safe single-segment slug (ASCII alphanumeric, '-', '_', '.' only)")
            }
        }
    }
}

impl std::error::Error for ProductCatalogError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metadata(subdomain: &str) -> ProductMetadata {
        ProductMetadata::new(
            "Workflow Studio",
            "Automate business processes",
            "workflow-icon",
            subdomain,
        )
        .unwrap()
    }

    #[test]
    fn test_product_id_rejects_empty() {
        assert_eq!(
            ProductId::new("").unwrap_err(),
            ProductCatalogError::EmptyProductId
        );
        assert_eq!(
            ProductId::new("   ").unwrap_err(),
            ProductCatalogError::EmptyProductId
        );
    }

    #[test]
    fn test_product_id_roundtrip() {
        let id = ProductId::new("workflow").unwrap();
        assert_eq!(id.as_str(), "workflow");
        assert_eq!(id.to_string(), "workflow");
    }

    #[test]
    fn test_product_metadata_rejects_empty_display_name() {
        let err = ProductMetadata::new("", "desc", "icon", "workflow").unwrap_err();
        assert_eq!(err, ProductCatalogError::EmptyDisplayName);
    }

    #[test]
    fn test_product_metadata_rejects_empty_subdomain() {
        let err = ProductMetadata::new("Workflow Studio", "desc", "icon", "").unwrap_err();
        assert_eq!(err, ProductCatalogError::EmptySubdomain);
    }

    #[test]
    fn test_product_entry_is_active() {
        let id = ProductId::new("workflow").unwrap();
        let meta = make_metadata("workflow");
        let enabled = ProductEntry::new(id.clone(), meta.clone(), true);
        let disabled = ProductEntry::new(id, meta, false);
        assert!(enabled.is_active());
        assert!(!disabled.is_active());
    }

    #[test]
    fn test_product_entry_deep_link_path() {
        let id = ProductId::new("search").unwrap();
        let meta = make_metadata("search");
        let entry = ProductEntry::new(id, meta, true);
        assert_eq!(entry.deep_link_path(), "/search");
    }

    #[test]
    fn test_product_id_trims_whitespace() {
        let id = ProductId::new("  workflow  ").unwrap();
        assert_eq!(id.as_str(), "workflow");
        assert_eq!(id, ProductId::new("workflow").unwrap());
    }

    #[test]
    fn test_product_metadata_rejects_slash_subdomain() {
        let err = ProductMetadata::new("Workflow Studio", "desc", "icon", "workflow/settings")
            .unwrap_err();
        assert_eq!(err, ProductCatalogError::InvalidSubdomainSlug);
    }

    #[test]
    fn test_product_metadata_normalizes_leading_trailing_slash() {
        // leading/trailing slashes are stripped; result is a valid single slug
        let meta = ProductMetadata::new("Workflow Studio", "desc", "icon", "/workflow/").unwrap();
        assert_eq!(meta.subdomain(), "workflow");
    }

    #[test]
    fn test_product_metadata_accessors() {
        let meta =
            ProductMetadata::new("Workflow Studio", "Automate", "wf-icon", "workflow").unwrap();
        assert_eq!(meta.display_name(), "Workflow Studio");
        assert_eq!(meta.description(), "Automate");
        assert_eq!(meta.icon_slug(), "wf-icon");
        assert_eq!(meta.subdomain(), "workflow");
    }

    #[test]
    fn test_product_entry_only_enabled_filtered() {
        let entries = [
            ProductEntry::new(
                ProductId::new("workflow").unwrap(),
                make_metadata("workflow"),
                true,
            ),
            ProductEntry::new(
                ProductId::new("search").unwrap(),
                make_metadata("search"),
                false,
            ),
            ProductEntry::new(
                ProductId::new("ontology").unwrap(),
                make_metadata("ontology"),
                true,
            ),
        ];
        let active: Vec<_> = entries.iter().filter(|e| e.is_active()).collect();
        assert_eq!(active.len(), 2);
        assert_eq!(active[0].id.as_str(), "workflow");
        assert_eq!(active[1].id.as_str(), "ontology");
    }

    #[test]
    fn test_product_id_ordering() {
        let mut ids = [
            ProductId::new("workflow").unwrap(),
            ProductId::new("ontology").unwrap(),
            ProductId::new("search").unwrap(),
        ];
        ids.sort();
        assert_eq!(ids[0].as_str(), "ontology");
        assert_eq!(ids[1].as_str(), "search");
        assert_eq!(ids[2].as_str(), "workflow");
    }
}
