use crate::*;
use data_boundary_kernel::{DataClass, DataClassification};
use data_ontology_kernel::{
    DisplayMetadata, EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition,
    LinkTypeDefinition, LinkTypeId, OntologyEngine,
};

pub(crate) fn native_error(
    path: &str,
    cause: data_ontology_kernel::OntologyEngineError,
) -> ModelError {
    ModelError::Native {
        path: path.into(),
        cause,
    }
}

impl ModelDisplay {
    fn native(&self) -> DisplayMetadata {
        DisplayMetadata {
            display_name: self.display_name.clone(),
            description: self.description.clone(),
            icon: self.icon.clone(),
            color: self.color.clone(),
            plural_name: self.plural_name.clone(),
        }
    }
}

impl From<&DisplayMetadata> for ModelDisplay {
    fn from(value: &DisplayMetadata) -> Self {
        Self {
            display_name: value.display_name.clone(),
            description: value.description.clone(),
            icon: value.icon.clone(),
            color: value.color.clone(),
            plural_name: value.plural_name.clone(),
        }
    }
}

impl ModelEntity {
    pub(crate) fn native(
        &self,
        registry: &OntologyEngine,
        tenant: &str,
    ) -> Result<EntityTypeDefinition, ModelError> {
        let path = format!("entities.{}", self.id);
        let properties = self
            .properties
            .iter()
            .map(|row| {
                let mut property = EntityTypePropertyDefinition::new(
                    &row.name,
                    row.tier.into(),
                    row.data_class.native(),
                    row.required,
                )
                .map_err(|e| native_error(&path, e))?;
                property.value_type = row
                    .value_type
                    .as_ref()
                    .map(|kind| {
                        kind.native(
                            registry,
                            tenant,
                            &format!("{path}.properties.{}.value_type", row.name),
                        )
                    })
                    .transpose()?;
                property.display = row.display.as_ref().map(ModelDisplay::native);
                Ok(property)
            })
            .collect::<Result<Vec<_>, ModelError>>()?;
        let id = EntityTypeId::new(&self.id).map_err(|e| native_error(&path, e))?;
        let mut entity =
            EntityTypeDefinition::new(tenant, id, &self.display_name, properties, self.revision)
                .map_err(|e| native_error(&path, e))?;
        entity.pillar = self.pillar.map(Into::into);
        entity.primary_key_property = self.primary_key_property.clone();
        entity.title_property = self.title_property.clone();
        entity.display = self.display.as_ref().map(ModelDisplay::native);
        Ok(entity)
    }

    fn capture(row: &EntityTypeDefinition) -> Result<Self, ModelError> {
        if row.display_name.data_class != DataClassification::from(DataClass::InternalOnly) {
            return Err(ModelError::MalformedDocument);
        }
        let properties = row
            .properties
            .iter()
            .map(|property| {
                Ok(ModelProperty {
                    name: property.name.clone(),
                    tier: property.tier.into(),
                    data_class: ModelPrivacy::new(property.data_class),
                    required: property.required,
                    value_type: property.value_type.as_ref().map(Into::into),
                    display: property.display.as_ref().map(Into::into),
                })
            })
            .collect::<Result<Vec<_>, ModelError>>()?;
        Ok(Self {
            id: row.id.value.clone(),
            display_name: row.display_name.value.clone(),
            properties,
            revision: row.revision,
            pillar: row.pillar.map(Into::into),
            primary_key_property: row.primary_key_property.clone(),
            title_property: row.title_property.clone(),
            display: row.display.as_ref().map(Into::into),
        })
    }
}

impl ModelLink {
    pub(crate) fn native(&self, tenant: &str) -> Result<LinkTypeDefinition, ModelError> {
        let path = format!("links.{}", self.id);
        let mut link = LinkTypeDefinition::new(
            tenant,
            LinkTypeId::new(&self.id).map_err(|e| native_error(&path, e))?,
            EntityTypeId::new(&self.from_entity_type).map_err(|e| native_error(&path, e))?,
            EntityTypeId::new(&self.to_entity_type).map_err(|e| native_error(&path, e))?,
            self.cardinality.into(),
            self.allow_cross_tenant,
        )
        .map_err(|e| native_error(&path, e))?;
        link.revision = self.revision;
        link.display = self.display.as_ref().map(ModelDisplay::native);
        Ok(link)
    }
    fn capture(row: &LinkTypeDefinition) -> Self {
        Self {
            id: row.id.value.clone(),
            from_entity_type: row.from_entity_type.value.clone(),
            to_entity_type: row.to_entity_type.value.clone(),
            cardinality: row.cardinality.into(),
            allow_cross_tenant: row.allow_cross_tenant,
            revision: row.revision,
            display: row.display.as_ref().map(Into::into),
        }
    }
}

impl ModelDocument {
    pub fn capture(engine: &OntologyEngine, tenant: &str) -> Result<Self, ModelError> {
        let document = Self {
            entities: engine
                .entity_types(tenant)
                .map(ModelEntity::capture)
                .collect::<Result<Vec<_>, _>>()?,
            links: engine.link_types(tenant).map(ModelLink::capture).collect(),
            named_string_types: engine
                .named_string_types(tenant)
                .map(|row| ModelNamedStringType {
                    id: row.reference().id().as_str().into(),
                    revision: row.reference().revision(),
                    display_name: row.display_name().value.clone(),
                    members: row.members().iter().cloned().collect(),
                })
                .collect(),
        };
        document.check_bounds()?;
        Ok(document)
    }
}

impl ModelBaseline {
    pub fn capture(engine: &OntologyEngine, tenant: &str) -> Result<Self, ModelError> {
        let document = ModelDocument::capture(engine, tenant)?;
        let prior_entity_revisions = engine
            .entity_type_revisions(tenant)
            .filter(|row| {
                engine
                    .entity_type(tenant, &row.id)
                    .is_some_and(|current| current.revision != row.revision)
            })
            .map(ModelEntity::capture)
            .collect::<Result<Vec<_>, _>>()?;
        let baseline = Self {
            document,
            prior_entity_revisions,
        };
        baseline.registry(tenant)?;
        Ok(baseline)
    }

    pub fn registry(&self, tenant: &str) -> Result<OntologyEngine, ModelError> {
        self.document.check_bounds()?;
        let mut registry = OntologyEngine::default();
        crate::named_strings::stage_named_string_types(&mut registry, tenant, &self.document)?;
        let mut rows: Vec<&ModelEntity> = self
            .prior_entity_revisions
            .iter()
            .chain(&self.document.entities)
            .collect();
        rows.sort_by_key(|row| (&row.id, row.revision));
        for row in rows {
            let mut remaining = MAX_MODEL_VALUE_NODES;
            row.bounds("baseline.entity_revisions", &mut remaining)?;
            let current = self
                .document
                .entities
                .iter()
                .find(|current| current.id == row.id)
                .ok_or(ModelError::CorruptState)?;
            if row.revision > current.revision {
                return Err(ModelError::CorruptState);
            }
            registry
                .evolve_entity_type(row.native(&registry, tenant)?)
                .map_err(|e| native_error("baseline.entity_revisions", e))?;
        }
        for row in &self.document.links {
            registry
                .register_link_type(row.native(tenant)?)
                .map_err(|e| native_error("baseline.links", e))?;
        }
        crate::schema::schema_bytes(&registry, tenant)?;
        if ModelDocument::capture(&registry, tenant)? != self.document {
            return Err(ModelError::CorruptState);
        }
        Ok(registry)
    }
}
