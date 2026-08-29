//! Product-catalog regression coverage.

use super::*;

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
    let err =
        ProductMetadata::new("Workflow Studio", "desc", "icon", "workflow/settings").unwrap_err();
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
    let meta = ProductMetadata::new("Workflow Studio", "Automate", "wf-icon", "workflow").unwrap();
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
