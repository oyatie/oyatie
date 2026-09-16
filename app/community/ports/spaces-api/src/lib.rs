#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpaceError {
    Invalid,
    MissingTenantScope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommunitySpace {
    pub space_id: String,
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
        require_tenant_scope(&self.tenant_scope_ref)
    }
}

pub trait SpaceCatalog: Send + Sync {
    fn list_spaces(&self, tenant_scope_ref: &str) -> Result<Vec<CommunitySpace>, SpaceError>;
}

pub fn require_tenant_scope(value: &str) -> Result<(), SpaceError> {
    require_identity(value)?;
    if value.starts_with("tenant:") {
        Ok(())
    } else {
        Err(SpaceError::MissingTenantScope)
    }
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
    fn tenant_scope_rejects_blank_and_unscoped_values() {
        assert_eq!(require_tenant_scope(""), Err(SpaceError::Invalid));
        assert_eq!(
            require_tenant_scope("person:u"),
            Err(SpaceError::MissingTenantScope)
        );
        assert_eq!(require_tenant_scope("tenant:t"), Ok(()));
    }
}
