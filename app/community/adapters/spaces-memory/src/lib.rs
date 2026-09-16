//! In-memory `SpaceCatalog`. Volatile process-local fake; not a durable store.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use community_spaces_api::{CommunitySpace, ListSpacesQuery, SpaceCatalog, SpaceError};

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
        if self.spaces.contains_key(&key) {
            return Err(SpaceError::DuplicateSpace);
        }
        self.spaces.insert(key, space);
        Ok(())
    }
}

impl SpaceCatalog for MemorySpaceCatalog {
    fn list_spaces(&self, query: &ListSpacesQuery) -> Result<Vec<CommunitySpace>, SpaceError> {
        query.validate()?;
        Ok(self
            .spaces
            .range((query.tenant_scope_ref.clone(), String::new())..)
            .take_while(|((tenant, _), _)| tenant == &query.tenant_scope_ref)
            .map(|(_, space)| space.clone())
            .collect())
    }
}
