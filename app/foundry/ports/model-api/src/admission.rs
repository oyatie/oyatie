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

    pub fn registry_with_runtime_baseline(
        &self,
        baseline: &OntologyEngine,
    ) -> Result<OntologyEngine, ModelError> {
        let mut registry = self.base.registry_with_runtime_baseline(baseline)?;
        stage_document(
            &mut registry,
            &self.base.binding.tenant_id,
            &self.draft.document,
        )?;
        Ok(registry)
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
        .named_string_types
        .sort_by(|a, b| (&a.id, a.revision).cmp(&(&b.id, b.revision)));
    for row in &mut model.named_string_types {
        row.members.sort();
    }
    model
}

pub(crate) fn stage_document(
    registry: &mut OntologyEngine,
    tenant: &str,
    model: &ModelDocument,
) -> Result<(), ModelError> {
    model.check_bounds()?;
    stage_named_string_types(registry, tenant, model)?;
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
        let definition = row.native(registry, tenant)?;
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
    crate::schema::schema_bytes(registry, tenant)?;
    Ok(())
}

impl ModelHistory {
    pub fn head(&self) -> Result<&ModelPublication, ModelError> {
        self.publications.last().ok_or(ModelError::CorruptState)
    }

    pub fn registry(&self) -> Result<OntologyEngine, ModelError> {
        self.restore_registry(None)
    }

    /// Restore metadata onto the independently trusted runtime's action vocabulary.
    pub fn registry_with_runtime_baseline(
        &self,
        baseline: &OntologyEngine,
    ) -> Result<OntologyEngine, ModelError> {
        self.restore_registry(Some(baseline))
    }

    fn restore_registry(
        &self,
        runtime_baseline: Option<&OntologyEngine>,
    ) -> Result<OntologyEngine, ModelError> {
        self.binding.validate()?;
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
        let mut registry = match runtime_baseline {
            Some(baseline) => {
                if ModelBaseline::capture(baseline, &self.binding.tenant_id)? != self.baseline
                    || baseline
                        .link_instances(&self.binding.tenant_id)
                        .next()
                        .is_some()
                {
                    return Err(ModelError::BindingMismatch);
                }
                baseline.clone()
            }
            None => self.baseline.registry(&self.binding.tenant_id)?,
        };
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

pub(crate) fn stage_named_string_types(
    registry: &mut OntologyEngine,
    tenant: &str,
    model: &ModelDocument,
) -> Result<(), ModelError> {
    use data_ontology_kernel::{
        NamedStringTypeDefinition, NamedStringTypeId, NamedStringTypeRef, OntologyEngineError,
    };
    let mut seen = BTreeSet::new();
    for (index, row) in model.named_string_types.iter().enumerate() {
        let path = format!("named_string_types[{index}]");
        if !seen.insert((row.id.clone(), row.revision)) {
            return Err(native_error(
                &path,
                OntologyEngineError::DuplicateNamedStringType,
            ));
        }
        let id =
            NamedStringTypeId::new(&row.id).map_err(|e| native_error(&format!("{path}.id"), e))?;
        let reference = NamedStringTypeRef::new(tenant, id, row.revision)
            .map_err(|e| native_error(&format!("{path}.revision"), e))?;
        if row.revision != 1 {
            return Err(ModelError::NewDefinitionRevision {
                path: format!("{path}.revision"),
            });
        }
        let definition = NamedStringTypeDefinition::new(
            reference.clone(),
            &row.display_name,
            row.members.clone(),
        )
        .map_err(|e| {
            let failure_path = if e == OntologyEngineError::EmptyDisplayName {
                path.clone()
            } else {
                format!("{path}.members")
            };
            native_error(&failure_path, e)
        })?;
        match registry.named_string_type(&reference) {
            Some(old) if old == &definition => {}
            Some(_) => {
                return Err(native_error(
                    &path,
                    OntologyEngineError::NamedStringDefinitionMismatch,
                ));
            }
            None => {
                registry
                    .register_named_string_type(definition)
                    .map_err(|e| native_error(&path, e))?;
            }
        }
    }
    for old in registry.named_string_types(tenant) {
        if !seen.contains(&(
            old.reference().id().as_str().to_string(),
            old.reference().revision(),
        )) {
            return Err(ModelError::UnsupportedRemoval {
                path: format!("named_string_types.{}", old.reference().id().as_str()),
            });
        }
    }
    Ok(())
}
