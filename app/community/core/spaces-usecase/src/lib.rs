#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use community_post_store_api::{AuthorizedCommunityContext, CommunityApiError};
use community_spaces_api::{CommunitySpace, SpaceCatalog, SpaceError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpacesUsecaseError {
    Api(CommunityApiError),
    TenantMismatch,
}

impl From<SpaceError> for SpacesUsecaseError {
    fn from(error: SpaceError) -> Self {
        match error {
            SpaceError::Invalid => Self::Api(CommunityApiError::Invalid),
            SpaceError::MissingTenantScope => Self::Api(CommunityApiError::MissingTenantScope),
        }
    }
}

pub fn list_spaces(
    ctx: &AuthorizedCommunityContext,
    catalog: &impl SpaceCatalog,
) -> Result<Vec<CommunitySpace>, SpacesUsecaseError> {
    ctx.validate().map_err(SpacesUsecaseError::Api)?;
    let spaces = catalog.list_spaces(&ctx.tenant_scope_ref)?;
    if spaces
        .iter()
        .any(|space| space.tenant_scope_ref != ctx.tenant_scope_ref)
    {
        return Err(SpacesUsecaseError::TenantMismatch);
    }
    Ok(spaces)
}

#[cfg(test)]
mod tests {
    use super::*;
    use community_spaces_api::require_tenant_scope;

    fn ctx() -> AuthorizedCommunityContext {
        AuthorizedCommunityContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    struct OwnedCatalog {
        spaces: Vec<CommunitySpace>,
    }

    impl SpaceCatalog for OwnedCatalog {
        fn list_spaces(&self, tenant_scope_ref: &str) -> Result<Vec<CommunitySpace>, SpaceError> {
            require_tenant_scope(tenant_scope_ref)?;
            Ok(self
                .spaces
                .iter()
                .filter(|space| space.tenant_scope_ref == tenant_scope_ref)
                .cloned()
                .collect())
        }
    }

    struct ForeignCatalog;

    impl SpaceCatalog for ForeignCatalog {
        fn list_spaces(&self, tenant_scope_ref: &str) -> Result<Vec<CommunitySpace>, SpaceError> {
            require_tenant_scope(tenant_scope_ref)?;
            CommunitySpace::new("space:s", "tenant:other").map(|space| vec![space])
        }
    }

    struct PanicCatalog;

    impl SpaceCatalog for PanicCatalog {
        fn list_spaces(&self, _: &str) -> Result<Vec<CommunitySpace>, SpaceError> {
            panic!("catalog must not be contacted");
        }
    }

    #[test]
    fn list_spaces_returns_the_caller_tenant_only() {
        let catalog = OwnedCatalog {
            spaces: vec![
                CommunitySpace::new("space:keep", "tenant:t").unwrap(),
                CommunitySpace::new("space:drop", "tenant:other").unwrap(),
            ],
        };
        assert_eq!(
            list_spaces(&ctx(), &catalog).unwrap(),
            vec![CommunitySpace::new("space:keep", "tenant:t").unwrap()]
        );
    }

    #[test]
    fn list_spaces_refuses_a_catalog_that_leaks_another_tenant() {
        assert_eq!(
            list_spaces(&ctx(), &ForeignCatalog),
            Err(SpacesUsecaseError::TenantMismatch)
        );
    }

    #[test]
    fn list_spaces_rejects_invalid_context_before_the_catalog() {
        let mut invalid = ctx();
        invalid.policy_decision_ref.clear();
        assert_eq!(
            list_spaces(&invalid, &PanicCatalog),
            Err(SpacesUsecaseError::Api(
                CommunityApiError::MissingPolicyDecision
            ))
        );
    }
}
