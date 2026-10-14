use crate::native::native_error;
use crate::*;
use data_ontology_kernel::OntologyEngine;
use std::collections::BTreeSet;

/// Native metadata validation only: neither authorization nor object replay.
#[derive(Clone, Debug)]
pub struct PreparedPublication {
    pub(crate) draft: ModelDraft,
    pub(crate) base: ModelHistory,
    registry: OntologyEngine,
}

impl PreparedPublication {
    pub fn draft(&self) -> &ModelDraft {
        &self.draft
    }
    pub fn base(&self) -> &ModelHistory {
        &self.base
    }
    pub fn model_registry(&self) -> &OntologyEngine {
        &self.registry
    }
}

pub(crate) fn prepare_publication(
    draft: &ModelDraft,
    history: &ModelHistory,
) -> Result<PreparedPublication, ModelError> {
    if draft.tenant_id != history.binding.tenant_id {
        return Err(ModelError::TenantMismatch);
    }
    if draft.publication.is_some() {
        return Err(ModelError::DraftClosed);
    }
    if draft.id == 0
        || draft.version == 0
        || draft.version > MAX_MODEL_VERSION
        || draft.owner.trim().is_empty()
        || draft.owner.len() > MAX_MODEL_CONTEXT_BYTES
    {
        return Err(ModelError::CorruptState);
    }
    let head = history.head()?;
    if draft.base_generation != head.generation {
        return Err(ModelError::HeadConflict {
            current_generation: head.generation,
        });
    }
    if normalized(&draft.document) == normalized(&head.document) {
        return Err(ModelError::NoChanges);
    }
    let mut registry = history.registry()?;
    stage_document(&mut registry, &history.binding.tenant_id, &draft.document)?;
    Ok(PreparedPublication {
        draft: draft.clone(),
        base: history.clone(),
        registry,
    })
}

pub(crate) fn normalized(document: &ModelDocument) -> ModelDocument {
    let mut model = document.clone();
    model.entities.sort_by(|a, b| a.id.cmp(&b.id));
    model.links.sort_by(|a, b| a.id.cmp(&b.id));
    model
}

pub(crate) fn stage_document(
    registry: &mut OntologyEngine,
    tenant: &str,
    model: &ModelDocument,
) -> Result<(), ModelError> {
    model.check_bounds()?;
    let mut entities = BTreeSet::new();
    for row in &model.entities {
        if !entities.insert(&row.id) {
            return Err(ModelError::MalformedDocument);
        }
        for property in &row.properties {
            if property.data_class != ModelPrivacy::internal_only() {
                return Err(ModelError::UnsupportedPublicationDataClass {
                    path: format!("entities.{}.properties.{}", row.id, property.name),
                    data_class: property.data_class.native().label().into(),
                });
            }
        }
    }
    let mut links = BTreeSet::new();
    for row in &model.links {
        if !links.insert(&row.id) {
            return Err(ModelError::MalformedDocument);
        }
    }
    for old in registry.entity_types(tenant) {
        if !entities.contains(&old.id.value) {
            return Err(ModelError::UnsupportedRemoval {
                path: format!("entities.{}", old.id.value),
            });
        }
    }
    for old in registry.link_types(tenant) {
        if !links.contains(&old.id.value) {
            return Err(ModelError::UnsupportedRemoval {
                path: format!("links.{}", old.id.value),
            });
        }
    }
    for row in &model.entities {
        let definition = row.native(tenant)?;
        if let Some(prior) = registry.entity_type(tenant, &definition.id) {
            if &definition == prior {
                continue;
            }
            if prior.primary_key_property.is_none() && definition.primary_key_property.is_some() {
                return Err(ModelError::UnsupportedKeyAdoption {
                    entity_type: row.id.clone(),
                });
            }
        } else if row.revision != 1 {
            return Err(ModelError::NewDefinitionRevision {
                path: format!("entities.{}", row.id),
            });
        }
        registry
            .evolve_entity_type(definition)
            .map_err(|e| native_error(&format!("entities.{}", row.id), e))?;
    }
    for row in &model.links {
        let definition = row.native(tenant)?;
        if let Some(prior) = registry.link_type(tenant, &definition.id) {
            if &definition == prior {
                continue;
            }
            registry
                .evolve_link_type(definition)
                .map_err(|e| native_error(&format!("links.{}", row.id), e))?;
        } else {
            if row.allow_cross_tenant {
                return Err(ModelError::CrossTenantLink {
                    link_type: row.id.clone(),
                });
            }
            if row.revision != 1 {
                return Err(ModelError::NewDefinitionRevision {
                    path: format!("links.{}", row.id),
                });
            }
            registry
                .register_link_type(definition)
                .map_err(|e| native_error(&format!("links.{}", row.id), e))?;
        }
    }
    Ok(())
}

impl ModelHistory {
    pub fn head(&self) -> Result<&ModelPublication, ModelError> {
        self.publications.last().ok_or(ModelError::CorruptState)
    }

    pub fn registry(&self) -> Result<OntologyEngine, ModelError> {
        self.binding.validate()?;
        self.registry_with_runtime_baseline(&self.baseline.registry(&self.binding.tenant_id)?)
    }

    /// Restore metadata onto the independently trusted runtime's action vocabulary.
    pub fn registry_with_runtime_baseline(
        &self,
        baseline: &OntologyEngine,
    ) -> Result<OntologyEngine, ModelError> {
        self.binding.validate()?;
        if ModelBaseline::capture(baseline, &self.binding.tenant_id)? != self.baseline
            || baseline
                .link_instances(&self.binding.tenant_id)
                .next()
                .is_some()
        {
            return Err(ModelError::BindingMismatch);
        }
        if self.publications.is_empty() {
            return Err(ModelError::CorruptState);
        }
        let first = &self.publications[0];
        if first.generation != 1
            || first.receipt.is_some()
            || first.document != self.baseline.document
        {
            return Err(ModelError::CorruptState);
        }
        let mut registry = baseline.clone();
        let mut last = 1;
        for publication in self.publications.iter().skip(1) {
            if publication.generation != crate::store::next_model_version(last)? {
                return Err(ModelError::CorruptState);
            }
            let receipt = publication
                .receipt
                .as_ref()
                .ok_or(ModelError::CorruptState)?;
            if receipt.generation != publication.generation
                || receipt.prior_generation != last
                || receipt.draft_id == 0
                || receipt.draft_version == 0
                || receipt.draft_version > MAX_MODEL_VERSION
                || receipt.owner.trim().is_empty()
                || receipt.owner.len() > MAX_MODEL_CONTEXT_BYTES
            {
                return Err(ModelError::CorruptState);
            }
            stage_document(
                &mut registry,
                &self.binding.tenant_id,
                &publication.document,
            )?;
            last = publication.generation;
        }
        Ok(registry)
    }
}
