#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpaceError {
    Invalid,
    MissingTenantScope,
    DuplicateSpace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunitySpace {
    pub space_id: String,
    pub tenant_scope_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListSpacesQuery {
    pub tenant_scope_ref: String,
}

impl CommunitySpace {
    pub fn new(
        space_id: impl Into<String>,
        tenant_scope_ref: impl Into<String>,
    ) -> Result<Self, SpaceError> {
        let space = Self {
            space_id: space_id.into(),
            tenant_scope_ref: tenant_scope_ref.into(),
        };
        space.validate()?;
        Ok(space)
    }

    pub fn validate(&self) -> Result<(), SpaceError> {
        require_identity(&self.space_id)?;
        require_tenant(&self.tenant_scope_ref)
    }
}

impl ListSpacesQuery {
    pub fn new(tenant_scope_ref: impl Into<String>) -> Result<Self, SpaceError> {
        let query = Self {
            tenant_scope_ref: tenant_scope_ref.into(),
        };
        query.validate()?;
        Ok(query)
    }

    pub fn validate(&self) -> Result<(), SpaceError> {
        require_tenant(&self.tenant_scope_ref)
    }
}

pub trait SpaceCatalog: Send + Sync {
    fn list_spaces(&self, query: &ListSpacesQuery) -> Result<Vec<CommunitySpace>, SpaceError>;
}

fn require_identity(value: &str) -> Result<(), SpaceError> {
    if value.is_empty()
        || value.trim() != value
        || value.chars().any(|c| c.is_control() || c.is_whitespace())
    {
        Err(SpaceError::Invalid)
    } else {
        Ok(())
    }
}

fn require_tenant(value: &str) -> Result<(), SpaceError> {
    require_identity(value)?;
    if value.starts_with("tenant:") {
        Ok(())
    } else {
        Err(SpaceError::MissingTenantScope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_requires_identity_and_tenant_scope() {
        assert_eq!(
            CommunitySpace::new("", "tenant:t"),
            Err(SpaceError::Invalid)
        );
        assert_eq!(
            CommunitySpace::new(" space:s", "tenant:t"),
            Err(SpaceError::Invalid)
        );
        assert_eq!(
            CommunitySpace::new("space:s", "person:u"),
            Err(SpaceError::MissingTenantScope)
        );
        assert_eq!(
            CommunitySpace::new("space:s", "tenant:t").unwrap(),
            CommunitySpace {
                space_id: "space:s".into(),
                tenant_scope_ref: "tenant:t".into(),
            }
        );
    }

    #[test]
    fn list_query_requires_tenant_scope() {
        assert_eq!(ListSpacesQuery::new(""), Err(SpaceError::Invalid));
        assert_eq!(
            ListSpacesQuery::new("person:u"),
            Err(SpaceError::MissingTenantScope)
        );
        assert_eq!(
            ListSpacesQuery::new("tenant:t").unwrap().tenant_scope_ref,
            "tenant:t"
        );
    }
}
