//! Link-instance registration: idempotency and cardinality enforcement
//! against the declared [`LinkCardinality`](crate::LinkCardinality).

use crate::definitions::{LinkCardinality, LinkTypeId};
use crate::error::OntologyEngineError;

use super::{OntologyEngine, ontology_scoped_key};

/// Outcome returned by [`OntologyEngine::register_link_instance`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkInstanceOutcome {
    /// The instance was freshly inserted into the registry.
    Registered,
    /// The identical `(link_type_id, from_entity_id, to_entity_id)` tuple
    /// already existed; no state change occurred.
    AlreadyExists,
}

impl OntologyEngine {
    /// Register a directed link instance from `from_entity_id` to `to_entity_id`
    /// under `link_type_id` for `tenant_id`.
    ///
    /// # Behaviour
    ///
    /// 1. Rejects an unknown `link_type_id` with [`OntologyEngineError::UnknownLinkType`].
    /// 2. Is **idempotent** for the identical `(link_type_id, from_entity_id, to_entity_id)`
    ///    tuple — returns `Ok(LinkInstanceOutcome::AlreadyExists)` without mutation.
    /// 3. Enforces [`LinkCardinality`]:
    ///    - `OneToOne`: rejects a second outbound from `from_entity_id` **and** a second
    ///      inbound into `to_entity_id`.
    ///    - `OneToMany`: rejects a second inbound into `to_entity_id`; fan-out is permitted.
    ///    - `ManyToMany`: no restriction.
    pub fn register_link_instance(
        &mut self,
        tenant_id: &str,
        link_type_id: &LinkTypeId,
        from_entity_id: &str,
        to_entity_id: &str,
    ) -> Result<LinkInstanceOutcome, OntologyEngineError> {
        // Step 1: look up the link type definition.
        let link_def = self
            .link_types
            .get(&ontology_scoped_key(tenant_id, &link_type_id.value))
            .ok_or(OntologyEngineError::UnknownLinkType)?;
        let cardinality = link_def.cardinality;

        // Step 2: idempotency — identical 4-tuple already registered.
        let instance_key = (
            tenant_id.to_string(),
            link_type_id.value.clone(),
            from_entity_id.to_string(),
            to_entity_id.to_string(),
        );
        if self.link_instances.contains_key(&instance_key) {
            return Ok(LinkInstanceOutcome::AlreadyExists);
        }

        // Step 3: cardinality enforcement.
        let outbound_key = (
            tenant_id.to_string(),
            link_type_id.value.clone(),
            from_entity_id.to_string(),
        );
        let inbound_key = (
            tenant_id.to_string(),
            link_type_id.value.clone(),
            to_entity_id.to_string(),
        );
        match cardinality {
            LinkCardinality::OneToOne => {
                if self.link_outbound.contains_key(&outbound_key) {
                    return Err(OntologyEngineError::CardinalityViolation { cardinality });
                }
                if self.link_inbound.contains_key(&inbound_key) {
                    return Err(OntologyEngineError::CardinalityViolation { cardinality });
                }
            }
            LinkCardinality::OneToMany => {
                if self.link_inbound.contains_key(&inbound_key) {
                    return Err(OntologyEngineError::CardinalityViolation { cardinality });
                }
            }
            LinkCardinality::ManyToMany => {}
        }

        // Step 4: insert into all three indices.
        self.link_instances.insert(instance_key, ());
        self.link_outbound.insert(outbound_key, ());
        self.link_inbound.insert(inbound_key, ());

        Ok(LinkInstanceOutcome::Registered)
    }
}
