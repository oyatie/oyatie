use application_ontology_read::{FilterValue, ObjectListQuery, OntologyReadError, PropertyFilter};
use reqwest::Url;

pub(super) fn listing(url: &mut Url, query: &ObjectListQuery) -> Result<(), OntologyReadError> {
    query.validate()?;
    let mut raw = format!(
        "type={}&revision={}&limit={}",
        literal(&query.entity_type)?,
        query.revision,
        query.limit
    );
    if let Some(cursor) = &query.cursor {
        raw.push_str(&format!("&cursor={}", literal(cursor)?));
    }
    if let Some(filter) = &query.filter {
        raw.push_str(&format!("&property={}", literal(filter.property())?));
        match filter {
            PropertyFilter::Equals { value, .. } => {
                raw.push_str(&format!("&equals={}", value_literal(value)?));
            }
            PropertyFilter::Range { from, to, .. } => {
                raw.push_str(&format!(
                    "&from={}&to={}",
                    value_literal(from)?,
                    value_literal(to)?
                ));
            }
        }
    }
    // Foundry does not percent-decode: refuse any transport rewrite of the literal query.
    url.set_query(Some(&raw));
    if url.query() != Some(raw.as_str()) {
        return Err(OntologyReadError::InvalidQuery);
    }
    Ok(())
}

fn literal(value: &str) -> Result<&str, OntologyReadError> {
    if value
        .bytes()
        .all(|b| b.is_ascii_graphic() && !matches!(b, b'&' | b'#'))
    {
        Ok(value)
    } else {
        Err(OntologyReadError::InvalidQuery)
    }
}

fn value_literal(value: &FilterValue) -> Result<String, OntologyReadError> {
    Ok(match value {
        FilterValue::Text(text) => format!("text:{}", literal(text)?),
        FilterValue::Integer(number) => format!("int:{number}"),
        FilterValue::Boolean(flag) => format!("bool:{flag}"),
    })
}
