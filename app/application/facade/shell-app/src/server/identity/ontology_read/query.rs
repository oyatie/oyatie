use application_ontology_read::{
    DEFAULT_PAGE_LIMIT, FilterValue, ObjectListQuery, OntologyReadError, PropertyFilter,
};
use std::collections::BTreeMap;

fn parameters(
    raw: Option<&str>,
    admitted: &[&str],
) -> Result<BTreeMap<String, String>, OntologyReadError> {
    let url = reqwest::Url::parse(&format!(
        "https://query.invalid/?{}",
        raw.unwrap_or_default()
    ))
    .map_err(|_| OntologyReadError::InvalidQuery)?;
    let mut values = BTreeMap::new();
    for (key, value) in url.query_pairs() {
        if !admitted.contains(&key.as_ref())
            || values
                .insert(key.into_owned(), value.into_owned())
                .is_some()
        {
            return Err(OntologyReadError::InvalidQuery);
        }
    }
    Ok(values)
}
fn required(values: &BTreeMap<String, String>, key: &str) -> Result<String, OntologyReadError> {
    values
        .get(key)
        .cloned()
        .ok_or(OntologyReadError::InvalidQuery)
}
fn number<T: std::str::FromStr>(value: &str) -> Result<T, OntologyReadError> {
    value.parse().map_err(|_| OntologyReadError::InvalidQuery)
}
fn literal(raw: &str) -> Result<FilterValue, OntologyReadError> {
    let (kind, value) = raw.split_once(':').ok_or(OntologyReadError::InvalidQuery)?;
    match kind {
        "text" => Ok(FilterValue::Text(value.into())),
        "int" => number(value).map(FilterValue::Integer),
        "bool" => match value {
            "true" => Ok(FilterValue::Boolean(true)),
            "false" => Ok(FilterValue::Boolean(false)),
            _ => Err(OntologyReadError::InvalidQuery),
        },
        _ => Err(OntologyReadError::InvalidQuery),
    }
}
pub(super) fn revision(raw: Option<&str>) -> Result<u32, OntologyReadError> {
    let values = parameters(raw, &["revision"])?;
    number(&required(&values, "revision")?)
}
pub(super) fn listing(raw: Option<&str>) -> Result<ObjectListQuery, OntologyReadError> {
    let values = parameters(
        raw,
        &[
            "type", "revision", "limit", "cursor", "property", "equals", "from", "to",
        ],
    )?;
    let filter = match (
        values.get("property"),
        values.get("equals"),
        values.get("from"),
        values.get("to"),
    ) {
        (None, None, None, None) => None,
        (Some(property), Some(value), None, None) => Some(PropertyFilter::Equals {
            property: property.clone(),
            value: literal(value)?,
        }),
        (Some(property), None, Some(from), Some(to)) => Some(PropertyFilter::Range {
            property: property.clone(),
            from: literal(from)?,
            to: literal(to)?,
        }),
        _ => return Err(OntologyReadError::InvalidQuery),
    };
    let query = ObjectListQuery {
        entity_type: required(&values, "type")?,
        revision: number(&required(&values, "revision")?)?,
        limit: values
            .get("limit")
            .map_or(Ok(DEFAULT_PAGE_LIMIT), |v| number(v))?,
        cursor: values.get("cursor").cloned(),
        filter,
    };
    query.validate()?;
    Ok(query)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_form_values_decode_once_before_the_native_literal_transport() {
        let q = listing(Some(
            "type=ety_record&revision=2&property=name&equals=text%3AAda%2B%2520%3D%3A%3F",
        ))
        .unwrap();
        assert_eq!(
            q.filter,
            Some(PropertyFilter::Equals {
                property: "name".into(),
                value: FilterValue::Text("Ada+%20=:?".into())
            })
        );
        assert_eq!(q.limit, 100);
        assert_eq!(revision(Some("revision=2")), Ok(2));
    }
    #[test]
    fn duplicate_unknown_incomplete_and_invalid_queries_refuse() {
        for raw in [
            "type=ety_record&revision=1&revision=2",
            "type=x&revision=1&unknown=x",
            "type=x&revision=1&property=name",
            "type=x&revision=1&from=int:1&to=int:2",
            "type=x&revision=1&property=x&equals=int:1&from=int:1&to=int:2",
            "type=x&revision=1&property=x&from=int:2&to=int:1",
            "type=x&revision=1&limit=0",
            "type=x&revision=abc",
        ] {
            assert!(listing(Some(raw)).is_err(), "{raw}");
        }
        assert!(revision(Some("revision=1&unknown=x")).is_err());
        assert!(revision(None).is_err());
    }
}
