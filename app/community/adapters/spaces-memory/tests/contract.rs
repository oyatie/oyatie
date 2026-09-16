#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use community_post_store_api::{AuthorizedCommunityContext, CommunityApiError};
use community_spaces_api::{CommunitySpace, SpaceCatalog, SpaceError};
use community_spaces_memory::MemorySpaceCatalog;
use community_spaces_usecase::{SpacesUsecaseError, list_spaces};

fn ctx(tenant: &str) -> AuthorizedCommunityContext {
    AuthorizedCommunityContext {
        tenant_scope_ref: tenant.into(),
        principal_ref: "user:u".into(),
        idempotency_key: "idem".into(),
        policy_decision_ref: "policy".into(),
        audit_correlation_id: "audit".into(),
    }
}

fn space(tenant: &str, id: &str) -> CommunitySpace {
    CommunitySpace::new(id, tenant).unwrap()
}

#[test]
fn empty_catalog_lists_no_spaces() {
    let catalog = MemorySpaceCatalog::new();
    let listed = catalog.list_spaces("tenant:t").unwrap();
    assert!(listed.is_empty());
}

#[test]
fn list_returns_only_the_query_tenant_in_space_id_order() {
    let mut catalog = MemorySpaceCatalog::new();
    catalog.insert(space("tenant:b", "space:z")).unwrap();
    catalog.insert(space("tenant:a", "space:m")).unwrap();
    catalog.insert(space("tenant:a", "space:a")).unwrap();

    assert_eq!(
        catalog.list_spaces("tenant:a").unwrap(),
        vec![space("tenant:a", "space:a"), space("tenant:a", "space:m")]
    );
}

#[test]
fn same_space_id_may_exist_in_two_tenants() {
    let mut catalog = MemorySpaceCatalog::new();
    catalog.insert(space("tenant:a", "space:s")).unwrap();
    catalog.insert(space("tenant:b", "space:s")).unwrap();
    assert_eq!(
        catalog.list_spaces("tenant:a").unwrap(),
        vec![space("tenant:a", "space:s")]
    );
}

#[test]
fn list_rejects_an_unscoped_tenant() {
    let catalog = MemorySpaceCatalog::new();
    assert_eq!(
        catalog.list_spaces("person:u"),
        Err(SpaceError::MissingTenantScope)
    );
}

#[test]
fn usecase_lists_through_the_memory_catalog() {
    let mut catalog = MemorySpaceCatalog::new();
    catalog.insert(space("tenant:t", "space:s")).unwrap();
    assert_eq!(
        list_spaces(&ctx("tenant:t"), &catalog).unwrap(),
        vec![space("tenant:t", "space:s")]
    );
}

#[test]
fn usecase_does_not_list_when_context_is_invalid() {
    let mut catalog = MemorySpaceCatalog::new();
    catalog.insert(space("tenant:t", "space:s")).unwrap();
    let mut invalid = ctx("tenant:t");
    invalid.idempotency_key.clear();
    assert_eq!(
        list_spaces(&invalid, &catalog),
        Err(SpacesUsecaseError::Api(
            CommunityApiError::MissingIdempotencyKey
        ))
    );
}
