use crate::*;

pub(crate) fn text(value: &str, path: &str) -> Result<(), ModelError> {
    if value.len() > MAX_MODEL_TEXT_BYTES {
        return Err(ModelError::Limit { path: path.into() });
    }
    Ok(())
}

fn display(value: &Option<ModelDisplay>, path: &str) -> Result<(), ModelError> {
    if let Some(row) = value {
        for field in [
            &row.display_name,
            &row.description,
            &row.icon,
            &row.color,
            &row.plural_name,
        ]
        .into_iter()
        .flatten()
        {
            text(field, path)?;
        }
    }
    Ok(())
}

fn value_bounds(
    value: &ModelValueType,
    depth: usize,
    remaining: &mut usize,
    path: &str,
) -> Result<(), ModelError> {
    if depth > data_ontology_kernel::MAX_VALUE_TYPE_DEPTH || *remaining == 0 {
        return Err(ModelError::Limit { path: path.into() });
    }
    *remaining -= 1;
    match value {
        ModelValueType::Scalar { .. } => {}
        ModelValueType::NamedString { type_id, .. } => text(type_id, path)?,
        ModelValueType::Array { element } => value_bounds(element, depth + 1, remaining, path)?,
        ModelValueType::Struct { fields } => {
            if fields.len() > MAX_MODEL_ITEMS {
                return Err(ModelError::Limit { path: path.into() });
            }
            for field in fields {
                text(&field.name, path)?;
                value_bounds(&field.value_type, depth + 1, remaining, path)?;
            }
        }
    }
    Ok(())
}

impl ModelDocument {
    /// Structural admission deliberately permits incomplete semantic drafts.
    pub fn check_bounds(&self) -> Result<(), ModelError> {
        if self.entities.len() > MAX_MODEL_ITEMS
            || self.links.len() > MAX_MODEL_ITEMS
            || self.named_string_types.len() > MAX_MODEL_ITEMS
        {
            return Err(ModelError::Limit {
                path: "document".into(),
            });
        }
        let mut remaining = MAX_MODEL_VALUE_NODES;
        for (index, row) in self.named_string_types.iter().enumerate() {
            let path = format!("named_string_types[{index}]");
            if row.members.len() > data_ontology_kernel::MAX_NAMED_STRING_MEMBERS {
                return Err(ModelError::Limit { path });
            }
            remaining = remaining
                .checked_sub(1 + row.members.len())
                .ok_or_else(|| ModelError::Limit { path: path.clone() })?;
            text(&row.id, &path)?;
            text(&row.display_name, &path)?;
            for member in &row.members {
                text(member, &path)?;
            }
        }
        for (index, entity) in self.entities.iter().enumerate() {
            entity.bounds(&format!("entities[{index}]"), &mut remaining)?;
        }
        for (index, link) in self.links.iter().enumerate() {
            let path = format!("links[{index}]");
            for field in [&link.id, &link.from_entity_type, &link.to_entity_type] {
                text(field, &path)?;
            }
            display(&link.display, &path)?;
        }
        let bytes = serde_json::to_vec(self).map_err(|_| ModelError::MalformedDocument)?;
        if bytes.len() > MAX_MODEL_BYTES {
            return Err(ModelError::Limit {
                path: "document".into(),
            });
        }
        Ok(())
    }
}

impl ModelEntity {
    pub(crate) fn bounds(&self, path: &str, remaining: &mut usize) -> Result<(), ModelError> {
        for field in [&self.id, &self.display_name] {
            text(field, path)?;
        }
        for field in [&self.primary_key_property, &self.title_property]
            .into_iter()
            .flatten()
        {
            text(field, path)?;
        }
        display(&self.display, path)?;
        if self.properties.len() > MAX_MODEL_ITEMS {
            return Err(ModelError::Limit { path: path.into() });
        }
        for property in &self.properties {
            text(&property.name, path)?;
            display(&property.display, path)?;
            if let Some(kind) = &property.value_type {
                value_bounds(kind, 1, remaining, path)?;
            }
        }
        Ok(())
    }
}
