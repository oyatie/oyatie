use application_ontology_read::{
    FilterValue, ObjectListQuery, OntologyReadError, PinnedObject, Property, PropertyFilter,
};

pub(super) fn apply(
    objects: &mut Vec<PinnedObject>,
    query: &ObjectListQuery,
) -> Result<(), OntologyReadError> {
    let Some(filter) = &query.filter else {
        return Ok(());
    };
    if !super::fixture::properties(query.revision).contains(&filter.property().to_owned()) {
        return Err(OntologyReadError::InvalidQuery);
    }
    // Native range kind drift is refused across the type, before cursor/limit.
    if let PropertyFilter::Range { property, from, .. } = filter {
        for object in objects.iter() {
            if let Some(stored) = object.properties.get(property)
                && literal(stored).is_none_or(|value| {
                    std::mem::discriminant(&value) != std::mem::discriminant(from)
                })
            {
                return Err(OntologyReadError::FilterConflict);
            }
        }
    }
    objects.retain(|object| {
        let Some(stored) = object.properties.get(filter.property()).and_then(literal) else {
            return false;
        };
        match filter {
            PropertyFilter::Equals { value, .. } => &stored == value,
            PropertyFilter::Range { from, to, .. } => &stored >= from && &stored <= to,
        }
    });
    Ok(())
}

fn literal(property: &Property) -> Option<FilterValue> {
    match property.value_type.as_str() {
        "string" => property
            .value
            .as_str()
            .map(|value| FilterValue::Text(value.to_owned())),
        "integer" => property.value.as_i64().map(FilterValue::Integer),
        "boolean" => property.value.as_bool().map(FilterValue::Boolean),
        _ => None,
    }
}
