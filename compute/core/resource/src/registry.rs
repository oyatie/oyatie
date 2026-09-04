use std::collections::BTreeMap;

use crate::{
    aggregate::{Resource, ResourceCreate},
    error::CloudResourceError,
    identity::ResourceId,
    lifecycle::ResourceState,
};

pub trait ResourceRepo {
    fn create(&mut self, input: ResourceCreate) -> Result<Resource, CloudResourceError>;
    fn get(&self, id: &ResourceId) -> Option<&Resource>;
    fn transition_state(
        &mut self,
        id: &ResourceId,
        next_state: ResourceState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Resource, CloudResourceError>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResourceRegistry {
    resources: BTreeMap<ResourceId, Resource>,
}

impl ResourceRepo for ResourceRegistry {
    fn create(&mut self, input: ResourceCreate) -> Result<Resource, CloudResourceError> {
        let resource = Resource::new(input)?;
        if self.resources.contains_key(&resource.id.value) {
            return Err(CloudResourceError::DuplicateResource);
        }
        self.resources
            .insert(resource.id.value.clone(), resource.clone());
        Ok(resource)
    }

    fn get(&self, id: &ResourceId) -> Option<&Resource> {
        self.resources.get(id)
    }

    fn transition_state(
        &mut self,
        id: &ResourceId,
        next_state: ResourceState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Resource, CloudResourceError> {
        let current = self
            .resources
            .get(id)
            .ok_or(CloudResourceError::UnknownResource)?;
        let updated = current.transition_state(next_state, updated_at_epoch_seconds)?;
        self.resources.insert(id.clone(), updated.clone());
        Ok(updated)
    }
}

impl ResourceRegistry {
    pub fn resources(&self) -> impl Iterator<Item = &Resource> {
        self.resources.values()
    }
}
