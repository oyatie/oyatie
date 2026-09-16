//! In-memory `SpaceCatalog`. Volatile process-local fake; not a durable store.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use community_spaces_api::{CommunitySpace, SpaceCatalog, SpaceError, require_tenant_scope};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MemorySpaceCatalog {
    spaces: BTreeMap<(String, String), CommunitySpace>,
}

impl MemorySpaceCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, space: CommunitySpace) -> Result<(), SpaceError> {
        space.validate()?;
        let key = (space.tenant_scope_ref.clone(), space.space_id.clone());
        self.spaces.insert(key, space);
        Ok(())
    }
}

impl SpaceCatalog for MemorySpaceCatalog {
    fn list_spaces(&self, tenant_scope_ref: &str) -> Result<Vec<CommunitySpace>, SpaceError> {
        require_tenant_scope(tenant_scope_ref)?;
        Ok(self
            .spaces
            .range((tenant_scope_ref.to_owned(), String::new())..)
            .take_while(|((tenant, _), _)| tenant == tenant_scope_ref)
            .map(|(_, space)| space.clone())
            .collect())
    }
}
